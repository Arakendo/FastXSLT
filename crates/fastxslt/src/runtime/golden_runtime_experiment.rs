use std::collections::{BTreeMap, HashSet};

use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::execution_control_experiment::{
    CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::{
    BuildFailure, Document, NodeId, NodeKind, StringValueVisitFailure,
};
use crate::xml::quick_xml_experiment::{
    ExpandedName, ParseLimits, parse_document, parse_document_controlled,
};
use crate::xpath::path_experiment::evaluate_child_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, Instruction, MatchPattern, StylesheetProgram,
};

mod serialization;

pub(super) use serialization::serialize_xml;

const XML_LIMITS: ParseLimits = ParseLimits {
    max_events: 1_024,
    max_depth: 64,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResultNode {
    Element {
        name: ExpandedName,
        children: Vec<ResultNode>,
    },
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticResult {
    children: Vec<ResultNode>,
}

#[derive(Debug)]
struct TransformRequest {
    identity: String,
    result_identity: String,
    source_resource: String,
    cancellation: CancellationToken,
    cancellation_fault: Option<(WorkDomain, usize)>,
}

#[derive(Debug)]
struct TransformSetBuilder {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    request_ids: HashSet<String>,
    result_ids: HashSet<String>,
    request_limit: usize,
    policy: ExecutionPolicy,
}

#[derive(Debug)]
struct TransformSet {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    policy: ExecutionPolicy,
}

#[derive(Debug, Clone)]
struct ExecutionPolicy {
    denied_sources: HashSet<String>,
    serialized_byte_limit: usize,
    work_limits: WorkLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResultEntry {
    result_id: String,
    semantic: SemanticResult,
    serialized: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ResultSet {
    by_request: BTreeMap<String, ResultEntry>,
    completion_order: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureCategory {
    Invalid,
    Unsupported,
    MissingResource,
    Denied,
    Limit,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExecutionFailure {
    code: &'static str,
    category: FailureCategory,
    request_id: Option<String>,
    work_domain: Option<WorkDomain>,
    detail: String,
}

impl TransformSetBuilder {
    fn new(
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

    fn add(&mut self, request: TransformRequest) -> Result<(), ExecutionFailure> {
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
        if self
            .policy
            .denied_sources
            .contains(&request.source_resource)
        {
            self.request_ids.remove(&request.identity);
            self.result_ids.remove(&request.result_identity);
            return Err(failure(
                "FXRS0003",
                FailureCategory::Denied,
                Some(&request.identity),
                format!("source authority is denied: {}", request.source_resource),
            ));
        }
        if self.snapshot.get(&request.source_resource).is_none() {
            self.request_ids.remove(&request.identity);
            self.result_ids.remove(&request.result_identity);
            return Err(failure(
                "FXRS0001",
                FailureCategory::MissingResource,
                Some(&request.identity),
                format!("source is not admitted: {}", request.source_resource),
            ));
        }
        self.requests.push(request);
        Ok(())
    }

    fn seal(self) -> TransformSet {
        TransformSet {
            snapshot: self.snapshot,
            stylesheet: self.stylesheet,
            requests: self.requests,
            policy: self.policy,
        }
    }
}

pub(super) fn compile_resource(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let bytes = snapshot.get(stylesheet_id).ok_or_else(|| {
        failure(
            "FXRS0002",
            FailureCategory::MissingResource,
            None,
            format!("stylesheet is not admitted: {stylesheet_id}"),
        )
    })?;
    let parsed = parse_document(stylesheet_id, bytes, XML_LIMITS).map_err(|error| {
        failure(
            "FXXM0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XML is invalid: {error:?}"),
        )
    })?;
    let document = Document::from_parsed(parsed).map_err(|error| {
        failure(
            "FXXD0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XDM construction failed: {error:?}"),
        )
    })?;
    compile_stylesheet(&document).map_err(|error| {
        failure(
            error.code,
            match error.category {
                crate::compile::golden_stylesheet_experiment::CompileCategory::Invalid => {
                    FailureCategory::Invalid
                }
                crate::compile::golden_stylesheet_experiment::CompileCategory::Unsupported => {
                    FailureCategory::Unsupported
                }
            },
            None,
            format!(
                "{} at {}:{}..{}",
                error.detail,
                error.location.resource,
                error.location.span.start,
                error.location.span.end
            ),
        )
    })
}

fn execute_transform_set(set: TransformSet) -> Result<ResultSet, ExecutionFailure> {
    let mut by_request = BTreeMap::new();
    let mut completion_order = Vec::new();

    for request in set.requests.into_iter().rev() {
        let mut control =
            InvocationControl::new(request.cancellation.clone(), set.policy.work_limits);
        if let Some((domain, accepted_charges_before_signal)) = request.cancellation_fault {
            control = control.cancelling_on_charge(domain, accepted_charges_before_signal);
        }
        let bytes = set
            .snapshot
            .get(&request.source_resource)
            .expect("sealed transform sets contain admitted sources");
        let parsed =
            parse_document_controlled(&request.source_resource, bytes, XML_LIMITS, &mut control)
                .map_err(|error| {
                    error.control_failure().map_or_else(
                        || {
                            failure(
                                "FXXM0002",
                                FailureCategory::Invalid,
                                Some(&request.identity),
                                format!("source XML is invalid: {error:?}"),
                            )
                        },
                        |failure| control_failure(*failure, &request.identity),
                    )
                })?;
        let source =
            Document::from_parsed_controlled(parsed, &mut control).map_err(
                |error| match error {
                    BuildFailure::Control(failure) => control_failure(failure, &request.identity),
                    _ => failure(
                        "FXXD0002",
                        FailureCategory::Invalid,
                        Some(&request.identity),
                        format!("source XDM construction failed: {error:?}"),
                    ),
                },
            )?;
        let semantic = execute_program(&set.stylesheet, &source, &request.identity, &mut control)?;
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

pub(super) fn execute_program(
    program: &StylesheetProgram,
    source: &Document,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let children = if let Some(root_template) = &program.root_template {
        execute_sequence(
            program,
            &root_template.body,
            source,
            source.document_node(),
            request_id,
            control,
        )?
    } else {
        apply_template(
            program,
            source,
            source.document_node(),
            None,
            request_id,
            control,
        )?
    };
    Ok(SemanticResult { children })
}

fn execute_sequence(
    program: &StylesheetProgram,
    instructions: &[Instruction],
    source: &Document,
    context: crate::xdm::owned_tree_experiment::NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let mut result = Vec::new();
    for instruction in instructions {
        control
            .charge(WorkDomain::XsltInstruction, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        match instruction {
            Instruction::LiteralElement { name, body, .. } => {
                control
                    .charge(WorkDomain::ResultNode, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                result.push(ResultNode::Element {
                    name: name.clone(),
                    children: execute_sequence(
                        program, body, source, context, request_id, control,
                    )?,
                });
            }
            Instruction::Text { value, .. } => {
                append_text(&mut result, value, request_id, control)?;
            }
            Instruction::ValueOf { select, .. } => {
                let selected = evaluate_child_path_controlled(source, context, select, control)
                    .map_err(|failure| control_failure(failure, request_id))?;
                if selected.len() > 1 {
                    return Err(failure(
                        "FXRT1001",
                        FailureCategory::Unsupported,
                        Some(request_id),
                        "the private value-of slice does not define multi-node conversion",
                    ));
                }
                if let Some(node) = selected.first() {
                    source
                        .visit_string_value_controlled(*node, control, &mut |part, control| {
                            append_text(&mut result, part, request_id, control)
                        })
                        .map_err(|failure| match failure {
                            StringValueVisitFailure::Control(failure) => {
                                control_failure(failure, request_id)
                            }
                            StringValueVisitFailure::Sink(failure) => failure,
                        })?;
                }
            }
            Instruction::ApplyTemplates { select, mode, .. } => {
                let selected = if let Some(select) = select {
                    match select {
                        ApplySelection::ChildPath(path) => {
                            evaluate_child_path_controlled(source, context, path, control)
                                .map_err(|failure| control_failure(failure, request_id))?
                        }
                        ApplySelection::Comments => {
                            let mut comments = Vec::new();
                            for child in source.children(context).iter().copied() {
                                control
                                    .charge(WorkDomain::XPathNodeVisit, 1)
                                    .map_err(|failure| control_failure(failure, request_id))?;
                                if source.kind(child) == NodeKind::Comment {
                                    comments.push(child);
                                }
                            }
                            comments
                        }
                    }
                } else {
                    source.children(context).to_vec()
                };
                for selected_node in selected {
                    result.extend(apply_template(
                        program,
                        source,
                        selected_node,
                        mode.as_deref(),
                        request_id,
                        control,
                    )?);
                }
            }
        }
    }
    Ok(result)
}

fn apply_template(
    program: &StylesheetProgram,
    source: &Document,
    node: NodeId,
    mode: Option<&str>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    if let Some(template) = program
        .matched_templates
        .iter()
        .filter(|template| template.mode.as_deref() == mode)
        .find(|template| match &template.pattern {
            MatchPattern::Element(name) => source.name(node) == Some(name),
            MatchPattern::Comment => source.kind(node) == NodeKind::Comment,
        })
    {
        return execute_sequence(
            program,
            &template.template.body,
            source,
            node,
            request_id,
            control,
        );
    }

    match source.kind(node) {
        NodeKind::Document | NodeKind::Element => {
            let mut result = Vec::new();
            for child in source.children(node) {
                result.extend(apply_template(
                    program, source, *child, mode, request_id, control,
                )?);
            }
            Ok(result)
        }
        NodeKind::Text => {
            let mut result = Vec::new();
            append_text(
                &mut result,
                source.value(node).unwrap_or_default(),
                request_id,
                control,
            )?;
            Ok(result)
        }
        NodeKind::Attribute | NodeKind::Comment | NodeKind::ProcessingInstruction => Ok(Vec::new()),
    }
}

fn append_text(
    nodes: &mut Vec<ResultNode>,
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if value.is_empty() {
        return Ok(());
    }
    if !matches!(nodes.last(), Some(ResultNode::Text(_))) {
        control
            .charge(WorkDomain::ResultNode, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
    }
    control
        .charge(WorkDomain::ResultTextByte, value.len())
        .map_err(|failure| control_failure(failure, request_id))?;
    if let Some(ResultNode::Text(existing)) = nodes.last_mut() {
        existing.push_str(value);
    } else {
        nodes.push(ResultNode::Text(value.to_owned()));
    }
    Ok(())
}

fn failure(
    code: &'static str,
    category: FailureCategory,
    request_id: Option<&str>,
    detail: impl Into<String>,
) -> ExecutionFailure {
    ExecutionFailure {
        code,
        category,
        request_id: request_id.map(str::to_owned),
        work_domain: None,
        detail: detail.into(),
    }
}

fn control_failure(failure: ControlFailure, request_id: &str) -> ExecutionFailure {
    let work_domain = failure.domain();
    match failure {
        ControlFailure::Cancelled { .. } => ExecutionFailure {
            code: "FXCT0001",
            category: FailureCategory::Cancelled,
            request_id: Some(request_id.to_owned()),
            work_domain: Some(work_domain),
            detail: format!(
                "host cancellation observed while charging {} work",
                work_domain.name()
            ),
        },
        ControlFailure::BudgetExhausted {
            limit,
            consumed,
            attempted,
            ..
        } => ExecutionFailure {
            code: "FXCT0002",
            category: FailureCategory::Limit,
            request_id: Some(request_id.to_owned()),
            work_domain: Some(work_domain),
            detail: format!(
                "{} work budget exhausted: limit {limit}, consumed {consumed}, next charge {attempted}",
                work_domain.name()
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::execution_control_experiment::{
        CancellationToken, InvocationControl, WorkDomain, WorkLimits,
    };
    use crate::resources::{ResourceLimits, ResourceSetBuilder};

    use super::{
        ExecutionPolicy, FailureCategory, ResultNode, SemanticResult, TransformRequest,
        TransformSetBuilder, compile_resource, execute_transform_set, serialize_xml,
    };

    const SOURCE_ID: &str = "urn:fastxslt:golden:hello:source";
    const STYLESHEET_ID: &str = "urn:fastxslt:golden:hello:stylesheet";
    type ConfigureWorkLimits = fn(&mut WorkLimits);

    fn snapshot() -> crate::resources::ResourceSnapshot {
        let mut builder = ResourceSetBuilder::new(ResourceLimits::new(8, 4_096, 8_192));
        builder
            .admit(
                SOURCE_ID,
                include_bytes!("../../../../corpus/golden/hello/input.xml").to_vec(),
            )
            .expect("admit source");
        builder
            .admit(
                STYLESHEET_ID,
                include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl").to_vec(),
            )
            .expect("admit stylesheet");
        builder.seal()
    }

    fn request(request_id: &str, result_id: &str, source_id: &str) -> TransformRequest {
        TransformRequest {
            identity: request_id.to_owned(),
            result_identity: result_id.to_owned(),
            source_resource: source_id.to_owned(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        }
    }

    fn policy(serialized_byte_limit: usize) -> ExecutionPolicy {
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit,
            work_limits: WorkLimits::unbounded(),
        }
    }

    fn execute_with_work_limits(
        request_id: &str,
        work_limits: WorkLimits,
    ) -> super::ExecutionFailure {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(
            snapshot,
            program,
            1,
            ExecutionPolicy {
                denied_sources: HashSet::new(),
                serialized_byte_limit: 4_096,
                work_limits,
            },
        );
        builder
            .add(request(request_id, "controlled-result", SOURCE_ID))
            .expect("admit controlled request");
        execute_transform_set(builder.seal()).expect_err("work limit should stop execution")
    }

    #[test]
    fn golden_transform_executes_through_an_unordered_identified_set() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 4, policy(4_096));
        builder
            .add(request("request-a", "result-a.html", SOURCE_ID))
            .expect("add first request");
        builder
            .add(request("request-b", "result-b.html", SOURCE_ID))
            .expect("add second request");

        let results = execute_transform_set(builder.seal()).expect("execute set");

        assert_eq!(results.completion_order, ["request-b", "request-a"]);
        let first = &results.by_request["request-a"];
        assert_eq!(first.result_id, "result-a.html");
        assert_eq!(
            first.semantic.children,
            [ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "message".to_owned(),
                },
                children: vec![ResultNode::Text("Hello, FastXSLT!".to_owned())],
            }]
        );
        assert_eq!(first.serialized, "<message>Hello, FastXSLT!</message>");
        assert_eq!(
            format!("{}\n", first.serialized),
            include_str!("../../../../corpus/golden/hello/expected.xml")
        );
        assert_eq!(results.by_request["request-b"].semantic, first.semantic);
    }

    #[test]
    fn exact_element_templates_dispatch_repeated_nodes_in_document_order() {
        const DISPATCH_SOURCE: &str = "urn:fastxslt:golden:template-dispatch:source";
        const DISPATCH_STYLESHEET: &str = "urn:fastxslt:golden:template-dispatch:stylesheet";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 4_096, 8_192));
        resources
            .admit(
                DISPATCH_SOURCE,
                include_bytes!("../../../../corpus/golden/template-dispatch/input.xml").to_vec(),
            )
            .expect("admit dispatch source");
        resources
            .admit(
                DISPATCH_STYLESHEET,
                include_bytes!("../../../../corpus/golden/template-dispatch/stylesheet.xsl")
                    .to_vec(),
            )
            .expect("admit dispatch stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, DISPATCH_STYLESHEET)
            .expect("compile dispatch stylesheet once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request(
                "dispatch-request",
                "dispatch-result",
                DISPATCH_SOURCE,
            ))
            .expect("add dispatch request");

        let results = execute_transform_set(builder.seal()).expect("execute dispatch set");

        assert_eq!(
            results.by_request["dispatch-request"].serialized,
            include_str!("../../../../corpus/golden/template-dispatch/expected.xml").trim()
        );
    }

    #[test]
    fn default_selection_uses_built_in_element_and_text_rules() {
        const BUILT_IN_SOURCE: &str = "urn:fastxslt:golden:built-in-rules:source";
        const BUILT_IN_STYLESHEET: &str = "urn:fastxslt:golden:built-in-rules:stylesheet";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 4_096, 8_192));
        resources
            .admit(
                BUILT_IN_SOURCE,
                include_bytes!("../../../../corpus/golden/built-in-template-rules/input.xml")
                    .to_vec(),
            )
            .expect("admit built-in-rule source");
        resources
            .admit(
                BUILT_IN_STYLESHEET,
                include_bytes!("../../../../corpus/golden/built-in-template-rules/stylesheet.xsl")
                    .to_vec(),
            )
            .expect("admit built-in-rule stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, BUILT_IN_STYLESHEET)
            .expect("compile built-in-rule stylesheet once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request(
                "built-in-request",
                "built-in-result",
                BUILT_IN_SOURCE,
            ))
            .expect("add built-in-rule request");

        let results = execute_transform_set(builder.seal()).expect("execute built-in-rule set");

        assert_eq!(
            results.by_request["built-in-request"].serialized,
            include_str!("../../../../corpus/golden/built-in-template-rules/expected.xml").trim()
        );
    }

    #[test]
    fn batch_of_one_matches_the_same_semantic_and_serialization_path() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request("only", "only-result", SOURCE_ID))
            .expect("add request");

        let results = execute_transform_set(builder.seal()).expect("execute one");

        assert_eq!(results.completion_order, ["only"]);
        assert_eq!(
            results.by_request["only"].serialized,
            "<message>Hello, FastXSLT!</message>"
        );
    }

    #[test]
    fn absent_output_declaration_does_not_silently_apply_html_serialization() {
        let result = SemanticResult {
            children: vec![ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "html".to_owned(),
                },
                children: Vec::new(),
            }],
        };
        let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
            method: None,
            omit_xml_declaration: false,
        };

        let mut control = InvocationControl::unbounded();
        let failure = serialize_xml(&result, &settings, "html-result", 4_096, &mut control)
            .expect_err("adaptive HTML output remains unsupported");

        assert_eq!(failure.code, "FXSR1001");
        assert_eq!(failure.category, FailureCategory::Unsupported);
        assert_eq!(failure.request_id.as_deref(), Some("html-result"));
    }

    #[test]
    fn builder_rejects_duplicates_limits_and_unadmitted_sibling_results() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 2, policy(4_096));
        builder
            .add(request("first", "future.xml", SOURCE_ID))
            .expect("add first request");

        let failure = builder
            .add(request("first", "other.xml", SOURCE_ID))
            .expect_err("duplicate request should fail");
        assert_eq!(failure.code, "FXBT0002");
        assert_eq!(failure.category, FailureCategory::Invalid);

        let failure = builder
            .add(request("second", "future.xml", SOURCE_ID))
            .expect_err("duplicate result should fail");
        assert_eq!(failure.code, "FXBT0003");

        let failure = builder
            .add(request("second", "second-result", "future.xml"))
            .expect_err("a sibling result is not an admitted source");
        assert_eq!(failure.code, "FXRS0001");
        assert_eq!(failure.category, FailureCategory::MissingResource);

        builder
            .add(request("second", "second-result", SOURCE_ID))
            .expect("failed additions do not mutate the builder");
        let failure = builder
            .add(request("third", "third-result", SOURCE_ID))
            .expect_err("request limit should fail");
        assert_eq!(failure.code, "FXBT0001");
        assert_eq!(failure.category, FailureCategory::Limit);
    }

    #[test]
    fn explicit_source_denial_is_distinct_from_missing_resource() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut denied_sources = HashSet::new();
        denied_sources.insert(SOURCE_ID.to_owned());
        let mut builder = TransformSetBuilder::new(
            snapshot,
            program,
            1,
            ExecutionPolicy {
                denied_sources,
                serialized_byte_limit: 4_096,
                work_limits: WorkLimits::unbounded(),
            },
        );

        let failure = builder
            .add(request("denied", "denied-result", SOURCE_ID))
            .expect_err("admitted source should still be deniable");

        assert_eq!(failure.code, "FXRS0003");
        assert_eq!(failure.category, FailureCategory::Denied);
        assert_eq!(failure.request_id.as_deref(), Some("denied"));
    }

    #[test]
    fn serialization_stops_before_exceeding_the_host_byte_limit() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16));
        builder
            .add(request("limited", "limited-result", SOURCE_ID))
            .expect("add limited request");

        let failure = execute_transform_set(builder.seal()).expect_err("output should be limited");

        assert_eq!(failure.code, "FXSR0002");
        assert_eq!(failure.category, FailureCategory::Limit);
        assert_eq!(failure.request_id.as_deref(), Some("limited"));
        assert_eq!(failure.work_domain, None);
    }

    #[test]
    fn host_cancellation_is_observed_as_cooperative_control_not_a_budget_failure() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let token = CancellationToken::new();
        let mut controlled_request = request("cancelled", "cancelled-result", SOURCE_ID);
        controlled_request.cancellation = token.clone();
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(controlled_request)
            .expect("admit cancellable request");
        token.cancel();

        let failure =
            execute_transform_set(builder.seal()).expect_err("cancelled work should stop");

        assert_eq!(failure.code, "FXCT0001");
        assert_eq!(failure.category, FailureCategory::Cancelled);
        assert_eq!(failure.request_id.as_deref(), Some("cancelled"));
        assert_eq!(failure.work_domain, Some(WorkDomain::XmlEvent));
    }

    #[test]
    fn each_implemented_layer_charges_its_own_work_domain() {
        let cases: [(WorkDomain, ConfigureWorkLimits); 8] = [
            (WorkDomain::XmlEvent, |limits: &mut WorkLimits| {
                limits.xml_events = 0;
            }),
            (WorkDomain::XdmNode, |limits: &mut WorkLimits| {
                limits.xdm_nodes = 1;
            }),
            (WorkDomain::XPathNodeVisit, |limits: &mut WorkLimits| {
                limits.xpath_node_visits = 0;
            }),
            (WorkDomain::XdmStringValueNode, |limits: &mut WorkLimits| {
                limits.xdm_string_value_nodes = 0;
            }),
            (WorkDomain::XsltInstruction, |limits: &mut WorkLimits| {
                limits.xslt_instructions = 0;
            }),
            (WorkDomain::ResultNode, |limits: &mut WorkLimits| {
                limits.result_nodes = 0;
            }),
            (WorkDomain::ResultTextByte, |limits: &mut WorkLimits| {
                limits.result_text_bytes = 0;
            }),
            (WorkDomain::SerializedByte, |limits: &mut WorkLimits| {
                limits.serialized_bytes = 0;
            }),
        ];

        for (domain, configure) in cases {
            let mut limits = WorkLimits::unbounded();
            configure(&mut limits);
            let failure = execute_with_work_limits(domain.name(), limits);

            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(failure.category, FailureCategory::Limit);
            assert_eq!(failure.request_id.as_deref(), Some(domain.name()));
            assert_eq!(failure.work_domain, Some(domain));
        }
    }
}

#[cfg(test)]
#[path = "golden_runtime_control_tests.rs"]
mod control_phase_tests;

#[cfg(test)]
#[path = "golden_runtime_workflow_tests.rs"]
mod workflow_tests;

#[cfg(test)]
#[path = "golden_runtime_xslt30_tests.rs"]
mod xslt30_tests;
