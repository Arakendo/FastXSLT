//! Host-owned workflow tests for the private golden runtime.

use std::collections::{BTreeMap, HashSet};

use super::{
    ExecutionPolicy, FailureCategory, InvocationEntry, TransformRequest, TransformSetBuilder,
    compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};

const STAGE_SOURCE: &str = "urn:fastxslt:workflow:stage-1-source";
const STAGE_ONE_STYLE: &str = "urn:fastxslt:workflow:stage-1-stylesheet";
const INTERMEDIATE: &str = "urn:fastxslt:workflow:intermediate";
const STAGE_TWO_STYLE: &str = "urn:fastxslt:workflow:stage-2-stylesheet";

fn request(request_id: &str, result_id: &str, source_id: &str) -> TransformRequest {
    TransformRequest {
        identity: request_id.to_owned(),
        result_identity: result_id.to_owned(),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id.to_owned(),
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
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
fn host_explicitly_admits_a_stage_one_result_into_a_later_snapshot() {
    let mut first_resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    first_resources
        .admit(
            STAGE_SOURCE,
            include_bytes!("../../../../corpus/golden/host-owned-two-stage/input.xml").to_vec(),
        )
        .expect("admit stage-one source");
    first_resources
        .admit(
            STAGE_ONE_STYLE,
            include_bytes!("../../../../corpus/golden/host-owned-two-stage/stage1-stylesheet.xsl")
                .to_vec(),
        )
        .expect("admit stage-one stylesheet");
    let first_snapshot = first_resources.seal();
    let first_program =
        compile_resource(&first_snapshot, STAGE_ONE_STYLE).expect("compile stage one");
    let mut first_set = TransformSetBuilder::new(first_snapshot, first_program, 1, policy());
    first_set
        .add(request("stage-one", INTERMEDIATE, STAGE_SOURCE))
        .expect("add stage-one request");
    let first_results = execute_transform_set(first_set.seal()).expect("execute stage one");
    let intermediate = &first_results.by_request["stage-one"];
    assert_eq!(intermediate.result_id, INTERMEDIATE);
    assert_eq!(
        intermediate.serialized,
        include_str!("../../../../corpus/golden/host-owned-two-stage/stage1-expected.xml").trim()
    );

    let stage_two_bytes =
        include_bytes!("../../../../corpus/golden/host-owned-two-stage/stage2-stylesheet.xsl");
    let mut sealed_before_result = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    sealed_before_result
        .admit(STAGE_TWO_STYLE, stage_two_bytes.to_vec())
        .expect("admit stage-two stylesheet before result exists");
    let sealed_before_result = sealed_before_result.seal();
    let program = compile_resource(&sealed_before_result, STAGE_TWO_STYLE)
        .expect("compile stage two against earlier snapshot");
    let mut unavailable = TransformSetBuilder::new(sealed_before_result, program, 1, policy());
    let missing = unavailable
        .add(request("stage-two-missing", "unused", INTERMEDIATE))
        .expect_err("a produced result does not mutate an earlier snapshot");
    assert_eq!(missing.code, "FXRS0001");
    assert_eq!(missing.category, FailureCategory::MissingResource);

    let mut second_resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    second_resources
        .admit(STAGE_TWO_STYLE, stage_two_bytes.to_vec())
        .expect("admit stage-two stylesheet");
    second_resources
        .admit(INTERMEDIATE, intermediate.serialized.as_bytes().to_vec())
        .expect("host explicitly admits identified intermediate result");
    let second_snapshot = second_resources.seal();
    let second_program =
        compile_resource(&second_snapshot, STAGE_TWO_STYLE).expect("compile stage two");
    let mut second_set = TransformSetBuilder::new(second_snapshot, second_program, 1, policy());
    second_set
        .add(request("stage-two", "final-result", INTERMEDIATE))
        .expect("add stage-two request after explicit admission");
    let second_results = execute_transform_set(second_set.seal()).expect("execute stage two");

    assert_eq!(
        second_results.by_request["stage-two"].serialized,
        include_str!("../../../../corpus/golden/host-owned-two-stage/expected.xml").trim()
    );
}
