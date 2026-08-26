//! Phase-specific cancellation tests for the private golden runtime.

use std::collections::HashSet;

use super::{
    ExecutionPolicy, FailureCategory, TransformRequest, TransformSetBuilder, compile_resource,
    execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};

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
