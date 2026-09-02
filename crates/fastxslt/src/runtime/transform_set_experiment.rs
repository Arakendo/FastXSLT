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
    XML_LIMITS, control_failure, execute_initial_mode, execute_initial_template,
    execute_program_with_parameters, failure, program_has_mode, serialize_xml,
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
                        "FXRT0004",
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
    let mut control = InvocationControl::new(request.cancellation.clone(), policy.work_limits);
    if let Some((domain, accepted_charges_before_signal)) = request.cancellation_fault {
        control = control.cancelling_on_charge(domain, accepted_charges_before_signal);
    }
    let semantic = match &request.entry {
        InvocationEntry::PrincipalSource { resource } => {
            let source =
                prepare_request_source(snapshot, resource, &request.identity, &mut control)?;
            execute_program_with_parameters(
                stylesheet,
                &source,
                &request.parameters,
                multiple_match_policy,
                &request.identity,
                &mut control,
            )?
        }
        InvocationEntry::InitialMode { resource, name } => {
            let source =
                prepare_request_source(snapshot, resource, &request.identity, &mut control)?;
            execute_initial_mode(
                super::InitialModeInvocation {
                    program: stylesheet,
                    source: &source,
                    initial_node: source.document_node(),
                    name,
                    parameters: &request.parameters,
                    multiple_match_policy,
                    request_id: &request.identity,
                },
                &mut control,
            )?
        }
        InvocationEntry::InitialModeElement {
            resource,
            name,
            element,
        } => {
            let source =
                prepare_request_source(snapshot, resource, &request.identity, &mut control)?;
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
                    source: &source,
                    initial_node: node,
                    name,
                    parameters: &request.parameters,
                    multiple_match_policy,
                    request_id: &request.identity,
                },
                &mut control,
            )?
        }
        InvocationEntry::InitialTemplate { name } => execute_initial_template(
            stylesheet,
            name,
            multiple_match_policy,
            &request.identity,
            &mut control,
        )?,
        InvocationEntry::InitialTemplateWithSource { resource, name } => {
            let source =
                prepare_request_source(snapshot, resource, &request.identity, &mut control)?;
            super::execute_initial_template_with_source(
                stylesheet,
                name,
                &source,
                multiple_match_policy,
                &request.identity,
                &mut control,
            )?
        }
    };
    let serialized = serialize_xml(
        &semantic,
        &stylesheet.output,
        &request.identity,
        policy.serialized_byte_limit,
        &mut control,
    )?;
    Ok(ResultEntry {
        result_id: request.result_identity.clone(),
        semantic,
        serialized,
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
