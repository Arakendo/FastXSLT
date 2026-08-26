//! Phase-specific cancellation tests for the private golden runtime.

use std::collections::HashSet;

use super::{
    ExecutionPolicy, FailureCategory, ResultNode, TransformRequest, TransformSetBuilder,
    append_text, compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document_controlled};

const SOURCE_ID: &str = "urn:fastxslt:golden:hello:input";
const STYLESHEET_ID: &str = "urn:fastxslt:golden:hello:stylesheet";

fn snapshot() -> crate::resources::ResourceSnapshot {
    let source = include_bytes!("../../../../corpus/golden/hello/input.xml").to_vec();
    let stylesheet = include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl").to_vec();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE_ID, source)
        .expect("admit golden source");
    resources
        .admit(STYLESHEET_ID, stylesheet)
        .expect("admit golden stylesheet");
    resources.seal()
}

fn request(
    identity: &str,
    result_identity: &str,
    cancellation_fault: Option<(WorkDomain, usize)>,
) -> TransformRequest {
    TransformRequest {
        identity: identity.to_owned(),
        result_identity: result_identity.to_owned(),
        source_resource: SOURCE_ID.to_owned(),
        cancellation: CancellationToken::new(),
        cancellation_fault,
    }
}

fn policy() -> ExecutionPolicy {
    ExecutionPolicy {
        denied_sources: HashSet::new(),
        serialized_byte_limit: 4_096,
        work_limits: WorkLimits::unbounded(),
    }
}

#[test]
fn cancellation_after_partial_work_retains_phase_and_request_identity() {
    let cases = [
        (WorkDomain::XmlEvent, 2),
        (WorkDomain::XdmNode, 2),
        (WorkDomain::XsltInstruction, 1),
        (WorkDomain::XPathNodeVisit, 1),
        (WorkDomain::XdmStringValueNode, 1),
        (WorkDomain::ResultNode, 1),
        (WorkDomain::ResultTextByte, 0),
        (WorkDomain::SerializedByte, 2),
    ];

    for (domain, accepted_charges_before_signal) in cases {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile golden program");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy());
        let request_id = format!("cancel-at-{}", domain.name());
        builder
            .add(request(
                &request_id,
                "cancelled-result",
                Some((domain, accepted_charges_before_signal)),
            ))
            .expect("admit fault-injected request");

        let failure = execute_transform_set(builder.seal())
            .expect_err("phase-specific cancellation should stop execution");

        assert_eq!(failure.code, "FXCT0001");
        assert_eq!(failure.category, FailureCategory::Cancelled);
        assert_eq!(failure.request_id.as_deref(), Some(request_id.as_str()));
        assert_eq!(failure.work_domain, Some(domain));
    }
}

#[test]
fn a_completed_sibling_is_not_exposed_when_the_set_returns_cancellation() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile golden program");
    let mut builder = TransformSetBuilder::new(snapshot, program, 2, policy());
    builder
        .add(request(
            "cancelled-after-sibling",
            "cancelled-result",
            Some((WorkDomain::SerializedByte, 2)),
        ))
        .expect("admit request that will cancel");
    builder
        .add(request("completed-first", "completed-result", None))
        .expect("admit healthy sibling");

    // The private reference executor runs in reverse admission order, so the
    // healthy sibling completes before cancellation is injected. The operation
    // still returns only the structured failure and no partial ResultSet.
    let failure = execute_transform_set(builder.seal())
        .expect_err("the private set operation is all-result or failure");

    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.category, FailureCategory::Cancelled);
    assert_eq!(
        failure.request_id.as_deref(),
        Some("cancelled-after-sibling")
    );
    assert_eq!(failure.work_domain, Some(WorkDomain::SerializedByte));
}

#[test]
fn result_nodes_and_utf8_text_bytes_are_bounded_before_serialization() {
    let mut limits = WorkLimits::unbounded();
    limits.result_nodes = 1;
    limits.result_text_bytes = 4;
    limits.serialized_bytes = 0;
    let mut control = InvocationControl::new(CancellationToken::new(), limits);
    let mut nodes = Vec::new();

    append_text(&mut nodes, "🚀", "result-growth", &mut control)
        .expect("one node and four UTF-8 bytes should fit exactly");
    assert_eq!(nodes, [ResultNode::Text("🚀".to_owned())]);

    let failure = append_text(&mut nodes, "!", "result-growth", &mut control)
        .expect_err("semantic result text should be bounded before serialization");
    assert_eq!(failure.code, "FXCT0002");
    assert_eq!(failure.category, FailureCategory::Limit);
    assert_eq!(failure.request_id.as_deref(), Some("result-growth"));
    assert_eq!(failure.work_domain, Some(WorkDomain::ResultTextByte));
}

#[test]
fn golden_path_has_an_exact_attributable_charge_profile() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile golden program");
    let mut control = InvocationControl::unbounded();
    let parsed = parse_document_controlled(
        SOURCE_ID,
        snapshot.get(SOURCE_ID).expect("golden source"),
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
        &mut control,
    )
    .expect("parse controlled golden source");
    let source = Document::from_parsed_controlled(parsed, &mut control)
        .expect("construct controlled golden XDM");
    let semantic = super::execute_program(&program, &source, "charge-profile", &mut control)
        .expect("execute controlled golden program");
    let serialized = super::serialize_xml(
        &semantic,
        &program.output,
        "charge-profile",
        4_096,
        &mut control,
    )
    .expect("serialize controlled golden result");
    let profile: Vec<_> = [
        WorkDomain::XmlEvent,
        WorkDomain::XdmNode,
        WorkDomain::XsltInstruction,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XdmStringValueNode,
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::SerializedByte,
    ]
    .into_iter()
    .map(|domain| (domain.name(), control.consumed(domain)))
    .collect();

    assert_eq!(serialized, "<message>Hello, FastXSLT!</message>");
    assert_eq!(
        profile,
        [
            ("xml-event", 10),
            ("xdm-node", 6),
            ("xslt-instruction", 4),
            ("xpath-node-visit", 4),
            ("xdm-string-value-node", 2),
            ("result-node", 2),
            ("result-text-byte", 16),
            ("serialized-byte", 35),
        ]
    );
}
