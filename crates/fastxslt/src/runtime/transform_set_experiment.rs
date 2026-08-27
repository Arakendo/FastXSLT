//! Private unordered transform-set admission and execution composition.

use std::collections::{BTreeMap, HashSet};

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::{BuildFailure, Document};
use crate::xml::quick_xml_experiment::parse_document_controlled;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{
    ExecutionFailure, FailureCategory, InvocationParameter, SemanticResult, XML_LIMITS,
    control_failure, execute_initial_mode, execute_initial_template,
    execute_program_with_parameters, failure, program_has_mode, serialize_xml,
};

#[derive(Debug)]
pub(super) enum InvocationEntry {
    PrincipalSource { resource: String },
    InitialMode { resource: String, name: String },
    InitialTemplate { name: String },
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
}

#[derive(Debug)]
pub(super) struct TransformSet {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    policy: ExecutionPolicy,
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
        }
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
            InvocationEntry::InitialMode { resource, name } => {
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
            InvocationEntry::InitialTemplate { name } => {
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
        None
    }

    pub(super) fn seal(self) -> TransformSet {
        TransformSet {
            snapshot: self.snapshot,
            stylesheet: self.stylesheet,
            requests: self.requests,
            policy: self.policy,
        }
    }
}

pub(super) fn execute_transform_set(set: TransformSet) -> Result<ResultSet, ExecutionFailure> {
    let mut by_request = BTreeMap::new();
    let mut completion_order = Vec::new();

    for request in set.requests.into_iter().rev() {
        let mut control =
            InvocationControl::new(request.cancellation.clone(), set.policy.work_limits);
        if let Some((domain, accepted_charges_before_signal)) = request.cancellation_fault {
            control = control.cancelling_on_charge(domain, accepted_charges_before_signal);
        }
        let semantic = match &request.entry {
            InvocationEntry::PrincipalSource { resource } => {
                let source = prepare_request_source(
                    &set.snapshot,
                    resource,
                    &request.identity,
                    &mut control,
                )?;
                execute_program_with_parameters(
                    &set.stylesheet,
                    &source,
                    &request.parameters,
                    &request.identity,
                    &mut control,
                )?
            }
            InvocationEntry::InitialMode { resource, name } => {
                let source = prepare_request_source(
                    &set.snapshot,
                    resource,
                    &request.identity,
                    &mut control,
                )?;
                execute_initial_mode(
                    &set.stylesheet,
                    &source,
                    name,
                    &request.parameters,
                    &request.identity,
                    &mut control,
                )?
            }
            InvocationEntry::InitialTemplate { name } => {
                execute_initial_template(&set.stylesheet, name, &request.identity, &mut control)?
            }
        };
        let serialized = serialize_xml(
            &semantic,
            &set.stylesheet.output,
            &request.identity,
            set.policy.serialized_byte_limit,
            &mut control,
        )?;
        completion_order.push(request.identity.clone());
        by_request.insert(
            request.identity,
            ResultEntry {
                result_id: request.result_identity,
                semantic,
                serialized,
            },
        );
    }
    Ok(ResultSet {
        by_request,
        completion_order,
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
