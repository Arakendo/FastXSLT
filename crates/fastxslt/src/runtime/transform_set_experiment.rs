//! Private unordered transform-set admission and execution composition.

use std::collections::{BTreeMap, HashSet};

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::{BuildFailure, Document};
use crate::xml::quick_xml_experiment::{ExpandedName, parse_document_controlled};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{
    ExecutionFailure, FailureCategory, InvocationParameter, MultipleMatchPolicy, SemanticResult,
    XML_LIMITS, control_failure, execute_initial_mode, execute_initial_template, failure,
    failure_at, program_has_mode, serialize_xml,
};

#[derive(Debug)]
pub(super) enum InvocationEntry {
    PrincipalSource {
        resource: String,
    },
    InitialMode {
        resource: String,
        name: String,
    },
    InitialModeElement {
        resource: String,
        name: String,
        element: ExpandedName,
    },
    InitialTemplate {
        name: String,
    },
    InitialTemplateWithSource {
        resource: String,
        name: String,
    },
}

#[derive(Debug)]
pub(super) struct TransformRequest {
    pub(super) identity: String,
    pub(super) result_identity: String,
    pub(super) entry: InvocationEntry,
    pub(super) parameters: BTreeMap<String, InvocationParameter>,
    pub(super) cancellation: CancellationToken,
    pub(super) cancellation_fault: Option<(WorkDomain, usize)>,
}

#[derive(Debug)]
pub(super) struct TransformSetBuilder {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    request_ids: HashSet<String>,
    result_ids: HashSet<String>,
    request_limit: usize,
    policy: ExecutionPolicy,
    multiple_match_policy: MultipleMatchPolicy,
}

#[derive(Debug)]
pub(super) struct TransformSet {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    policy: ExecutionPolicy,
    multiple_match_policy: MultipleMatchPolicy,
}

#[derive(Debug, Clone)]
pub(super) struct ExecutionPolicy {
    pub(super) denied_sources: HashSet<String>,
    pub(super) serialized_byte_limit: usize,
    pub(super) work_limits: WorkLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResultEntry {
    pub(super) result_id: String,
    pub(super) semantic: SemanticResult,
    pub(super) serialized: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ResultSet {
    pub(super) by_request: BTreeMap<String, ResultEntry>,
    pub(super) completion_order: Vec<String>,
}

impl TransformSetBuilder {
    pub(super) fn new(
        snapshot: ResourceSnapshot,
        stylesheet: StylesheetProgram,
        request_limit: usize,
        policy: ExecutionPolicy,
    ) -> Self {
        Self {
            snapshot,
            stylesheet,
            requests: Vec::new(),
            request_ids: HashSet::new(),
            result_ids: HashSet::new(),
            request_limit,
            policy,
            multiple_match_policy: MultipleMatchPolicy::UseLast,
        }
    }

    pub(super) fn with_multiple_match_policy(mut self, policy: MultipleMatchPolicy) -> Self {
        self.multiple_match_policy = policy;
        self
    }

    pub(super) fn add(&mut self, request: TransformRequest) -> Result<(), ExecutionFailure> {
        if self.requests.len() >= self.request_limit {
            return Err(failure(
                "FXBT0001",
                FailureCategory::Limit,
                Some(&request.identity),
                format!("transform-set request limit is {}", self.request_limit),
            ));
        }
        if !self.request_ids.insert(request.identity.clone()) {
            return Err(failure(
                "FXBT0002",
                FailureCategory::Invalid,
                Some(&request.identity),
                "duplicate request identity",
            ));
        }
        if !self.result_ids.insert(request.result_identity.clone()) {
            self.request_ids.remove(&request.identity);
            return Err(failure(
                "FXBT0003",
                FailureCategory::Invalid,
                Some(&request.identity),
                "duplicate result identity",
            ));
        }
        if let Some(failure) = self.entry_failure(&request) {
            self.request_ids.remove(&request.identity);
            self.result_ids.remove(&request.result_identity);
            return Err(failure);
        }
        self.requests.push(request);
        Ok(())
    }

    fn entry_failure(&self, request: &TransformRequest) -> Option<ExecutionFailure> {
        match &request.entry {
            InvocationEntry::PrincipalSource { resource } => {
                if self.policy.denied_sources.contains(resource) {
                    return Some(failure(
                        "FXRS0003",
                        FailureCategory::Denied,
                        Some(&request.identity),
                        format!("source authority is denied: {resource}"),
                    ));
                }
                if self.snapshot.get(resource).is_none() {
                    return Some(failure(
                        "FXRS0001",
                        FailureCategory::MissingResource,
                        Some(&request.identity),
                        format!("source is not admitted: {resource}"),
                    ));
                }
            }
            InvocationEntry::InitialMode { resource, name }
            | InvocationEntry::InitialModeElement { resource, name, .. } => {
                if self.policy.denied_sources.contains(resource) {
                    return Some(failure(
                        "FXRS0003",
                        FailureCategory::Denied,
                        Some(&request.identity),
                        format!("source authority is denied: {resource}"),
                    ));
                }
                if self.snapshot.get(resource).is_none() {
                    return Some(failure(
                        "FXRS0001",
                        FailureCategory::MissingResource,
                        Some(&request.identity),
                        format!("source is not admitted: {resource}"),
                    ));
                }
                if !program_has_mode(&self.stylesheet, name) {
                    return Some(failure(
                        "XTDE0045",
                        FailureCategory::Invalid,
                        Some(&request.identity),
                        format!("unknown initial mode: {name}"),
                    ));
                }
                if let Some(private) = self
                    .stylesheet
                    .private_initial_modes
                    .iter()
                    .find(|mode| mode.name == *name)
                {
                    return Some(failure_at(
                        "XTDE0045",
                        FailureCategory::Invalid,
                        Some(&request.identity),
                        private.location.clone(),
                        format!("initial mode is private: {name}"),
                    ));
                }
            }
            InvocationEntry::InitialTemplate { name }
            | InvocationEntry::InitialTemplateWithSource { name, .. } => {
                if !self
                    .stylesheet
                    .named_templates
                    .iter()
                    .any(|template| template.name == *name)
                {
                    return Some(failure(
                        "XTDE0040",
                        FailureCategory::Invalid,
                        Some(&request.identity),
                        format!("unknown initial template: {name}"),
                    ));
                }
            }
        }
        if let InvocationEntry::InitialTemplateWithSource { resource, .. } = &request.entry {
            if self.policy.denied_sources.contains(resource) {
                return Some(failure(
                    "FXRS0003",
                    FailureCategory::Denied,
                    Some(&request.identity),
                    format!("source authority is denied: {resource}"),
                ));
            }
            if self.snapshot.get(resource).is_none() {
                return Some(failure(
                    "FXRS0001",
                    FailureCategory::MissingResource,
                    Some(&request.identity),
                    format!("source is not admitted: {resource}"),
                ));
            }
        }
        None
    }

    pub(super) fn seal(self) -> TransformSet {
        TransformSet {
            snapshot: self.snapshot,
            stylesheet: self.stylesheet,
            requests: self.requests,
            policy: self.policy,
            multiple_match_policy: self.multiple_match_policy,
        }
    }
}

pub(super) fn execute_transform_set(set: TransformSet) -> Result<ResultSet, ExecutionFailure> {
    let mut by_request = BTreeMap::new();
    let mut completion_order = Vec::new();

    for request in set.requests.into_iter().rev() {
        let result = execute_request(
            &set.snapshot,
            &set.stylesheet,
            &set.policy,
            set.multiple_match_policy,
            &request,
        )?;
        completion_order.push(request.identity.clone());
        by_request.insert(request.identity, result);
    }
    Ok(ResultSet {
        by_request,
        completion_order,
    })
}

fn execute_request(
    snapshot: &ResourceSnapshot,
    stylesheet: &StylesheetProgram,
    policy: &ExecutionPolicy,
    multiple_match_policy: MultipleMatchPolicy,
    request: &TransformRequest,
) -> Result<ResultEntry, ExecutionFailure> {
    let mut control = request_control(request, policy);
    let source = request_source_identity(&request.entry)
        .map(|resource| prepare_request_source(snapshot, resource, &request.identity, &mut control))
        .transpose()?;
    execute_prepared_request(
        snapshot,
        stylesheet,
        policy,
        multiple_match_policy,
        request,
        source.as_ref(),
        &mut control,
    )
}

pub(super) fn request_control(
    request: &TransformRequest,
    policy: &ExecutionPolicy,
) -> InvocationControl {
    let mut control = InvocationControl::new(request.cancellation.clone(), policy.work_limits);
    if let Some((domain, accepted_charges_before_signal)) = request.cancellation_fault {
        control = control.cancelling_on_charge(domain, accepted_charges_before_signal);
    }
    control
}

pub(super) fn execute_prepared_request(
    snapshot: &ResourceSnapshot,
    stylesheet: &StylesheetProgram,
    policy: &ExecutionPolicy,
    multiple_match_policy: MultipleMatchPolicy,
    request: &TransformRequest,
    source: Option<&Document>,
    control: &mut InvocationControl,
) -> Result<ResultEntry, ExecutionFailure> {
    let semantic = execute_request_semantics(
        snapshot,
        stylesheet,
        policy,
        multiple_match_policy,
        request,
        source,
        control,
    )?;
    let serialized = serialize_xml(
        &semantic,
        &stylesheet.output,
        &request.identity,
        policy.serialized_byte_limit,
        control,
    )?;
    Ok(ResultEntry {
        result_id: request.result_identity.clone(),
        semantic,
        serialized,
    })
}

fn execute_request_semantics(
    snapshot: &ResourceSnapshot,
    stylesheet: &StylesheetProgram,
    policy: &ExecutionPolicy,
    multiple_match_policy: MultipleMatchPolicy,
    request: &TransformRequest,
    source: Option<&Document>,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    match &request.entry {
        InvocationEntry::PrincipalSource { resource } => {
            let source = prepared_source(source, resource, &request.identity)?;
            super::execute_program_with_parameters_and_resources(
                stylesheet,
                source,
                &request.parameters,
                multiple_match_policy,
                &request.identity,
                Some(snapshot),
                Some(&policy.denied_sources),
                control,
            )
        }
        InvocationEntry::InitialMode { resource, name } => {
            let source = prepared_source(source, resource, &request.identity)?;
            execute_initial_mode(
                super::InitialModeInvocation {
                    program: stylesheet,
                    source,
                    initial_node: source.document_node(),
                    name,
                    parameters: &request.parameters,
                    multiple_match_policy,
                    request_id: &request.identity,
                },
                control,
            )
        }
        InvocationEntry::InitialModeElement {
            resource,
            name,
            element,
        } => {
            let source = prepared_source(source, resource, &request.identity)?;
            let node = source
                .children(source.document_node())
                .iter()
                .copied()
                .find(|node| source.name(*node) == Some(element))
                .ok_or_else(|| {
                    failure(
                        "FXRT0005",
                        FailureCategory::Invalid,
                        Some(&request.identity),
                        format!("initial context element is absent: {}", element.local),
                    )
                })?;
            execute_initial_mode(
                super::InitialModeInvocation {
                    program: stylesheet,
                    source,
                    initial_node: node,
                    name,
                    parameters: &request.parameters,
                    multiple_match_policy,
                    request_id: &request.identity,
                },
                control,
            )
        }
        InvocationEntry::InitialTemplate { name } => execute_initial_template(
            stylesheet,
            name,
            &request.parameters,
            multiple_match_policy,
            &request.identity,
            control,
        ),
        InvocationEntry::InitialTemplateWithSource { resource, name } => {
            let source = prepared_source(source, resource, &request.identity)?;
            super::execute_initial_template_with_source(
                stylesheet,
                name,
                source,
                &request.parameters,
                multiple_match_policy,
                &request.identity,
                control,
            )
        }
    }
}

pub(super) fn request_source_identity(entry: &InvocationEntry) -> Option<&str> {
    match entry {
        InvocationEntry::PrincipalSource { resource }
        | InvocationEntry::InitialMode { resource, .. }
        | InvocationEntry::InitialModeElement { resource, .. }
        | InvocationEntry::InitialTemplateWithSource { resource, .. } => Some(resource),
        InvocationEntry::InitialTemplate { .. } => None,
    }
}

fn prepared_source<'a>(
    source: Option<&'a Document>,
    resource: &str,
    request_id: &str,
) -> Result<&'a Document, ExecutionFailure> {
    source.ok_or_else(|| {
        failure(
            "FXRT0006",
            FailureCategory::Invalid,
            Some(request_id),
            format!("prepared execution packet is missing source: {resource}"),
        )
    })
}

fn prepare_request_source(
    snapshot: &ResourceSnapshot,
    resource: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Document, ExecutionFailure> {
    let bytes = snapshot
        .get(resource)
        .expect("sealed transform sets contain admitted sources");
    let parsed =
        parse_document_controlled(resource, bytes, XML_LIMITS, control).map_err(|error| {
            error.control_failure().map_or_else(
                || {
                    failure(
                        "FXXM0002",
                        FailureCategory::Invalid,
                        Some(request_id),
                        format!("source XML is invalid: {error:?}"),
                    )
                },
                |failure| control_failure(*failure, request_id),
            )
        })?;
    Document::from_parsed_controlled(parsed, control).map_err(|error| match error {
        BuildFailure::Control(failure) => control_failure(failure, request_id),
        _ => failure(
            "FXXD0002",
            FailureCategory::Invalid,
            Some(request_id),
            format!("source XDM construction failed: {error:?}"),
        ),
    })
}
