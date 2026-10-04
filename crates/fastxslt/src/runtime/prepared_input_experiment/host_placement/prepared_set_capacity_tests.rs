//! Capacity candidates through the real sealed prepared-set owner.

use super::{Input, PreparedInputBuilder, input, program};
use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
use crate::runtime::prepared_input_experiment::PreparationCapacity;
use crate::runtime::prepared_input_experiment::PreparationFailure;
use crate::xml::quick_xml_experiment::ParseLimits;
use std::sync::Arc;

const CAPACITIES: [PreparationCapacity; 3] = [
    PreparationCapacity::Growth,
    PreparationCapacity::Frozen,
    PreparationCapacity::Presized,
];

#[test]
fn structural_xml_limits_preserve_spans_publish_nothing_and_allow_recovery() {
    for (limits, span, detail) in [
        (
            ParseLimits {
                max_events: 3,
                max_depth: 1,
            },
            3..7,
            "XML depth limit is 1",
        ),
        (
            ParseLimits {
                max_events: 2,
                max_depth: 2,
            },
            7..7,
            "XML event limit is 2",
        ),
    ] {
        for capacity in CAPACITIES {
            let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 1024, 2048));
            resources
                .admit("urn:limited", b"<r><a/></r>".to_vec())
                .unwrap();
            resources.admit("urn:recovery", b"<r/>".to_vec()).unwrap();
            let mut builder = PreparedInputBuilder::with_parse_limits(resources.seal(), limits)
                .with_capacity(capacity);
            let failure = builder
                .prepare("urn:limited", &mut InvocationControl::unbounded())
                .unwrap_err();
            assert_eq!(
                failure,
                PreparationFailure::XmlLimit {
                    identity: "urn:limited".to_owned(),
                    location: crate::xdm::owned_tree_experiment::SourceLocation {
                        resource: "urn:limited".to_owned(),
                        span: span.clone(),
                    },
                    detail: detail.to_owned(),
                }
            );
            assert!(builder.documents.is_empty());
            assert!(builder.parsed_phase_capacity_bytes.is_empty());
            let token = CancellationToken::new();
            token.cancel();
            assert!(matches!(
                builder.prepare(
                    "urn:limited",
                    &mut InvocationControl::new(token, WorkLimits::unbounded())
                ),
                Err(PreparationFailure::Control(
                    crate::execution_control_experiment::ControlFailure::Cancelled { .. }
                ))
            ));
            assert!(builder.documents.is_empty());
            builder
                .prepare("urn:recovery", &mut InvocationControl::unbounded())
                .unwrap();
            let prepared = builder.seal();
            assert!(prepared.get("urn:limited").is_none());
            assert!(prepared.get("urn:recovery").is_some());
        }
    }
    for capacity in CAPACITIES {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1024, 1024));
        resources
            .admit("urn:exact", b"<r><a/></r>".to_vec())
            .unwrap();
        let mut builder = PreparedInputBuilder::with_parse_limits(
            resources.seal(),
            ParseLimits {
                max_events: 3,
                max_depth: 2,
            },
        )
        .with_capacity(capacity);
        builder
            .prepare("urn:exact", &mut InvocationControl::unbounded())
            .unwrap();
        assert!(builder.seal().get("urn:exact").is_some());
    }
}

fn builder(input: &Input, capacity: PreparationCapacity) -> PreparedInputBuilder {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
    resources
        .admit(&input.identity, input.source.clone())
        .unwrap();
    PreparedInputBuilder::with_parse_limits(
        resources.seal(),
        ParseLimits {
            max_events: 10_000,
            max_depth: 64,
        },
    )
    .with_capacity(capacity)
}

#[test]
fn prepared_capacity_candidates_keep_snapshot_and_retired_map_ownership() {
    let program = program();
    for items in [5, 50, 500] {
        let inputs = [input(items, 8), input(items, 9)];
        let mut growth = builder(&inputs[0], PreparationCapacity::Growth);
        growth
            .prepare(&inputs[0].identity, &mut InvocationControl::unbounded())
            .unwrap();
        let growth = growth.seal();
        let reference = growth.observe(&inputs[0].identity).unwrap();
        for capacity in CAPACITIES {
            let sets = inputs.each_ref().map(|input| {
                let mut builder = builder(input, capacity);
                builder
                    .prepare(&input.identity, &mut InvocationControl::unbounded())
                    .unwrap();
                builder.seal()
            });
            let observed = sets[0].observe(&inputs[0].identity).unwrap();
            assert_eq!(observed.raw_bytes, reference.raw_bytes);
            assert_eq!(
                observed.parsed_phase_owned_capacity_bytes,
                reference.parsed_phase_owned_capacity_bytes
            );
            assert_eq!(observed.xdm_nodes, reference.xdm_nodes);
            assert!(observed.xdm_owned_capacity_bytes <= reference.xdm_owned_capacity_bytes);
            let documents = inputs
                .each_ref()
                .into_iter()
                .enumerate()
                .map(|(index, input)| sets[index].get(&input.identity).unwrap())
                .collect::<Vec<_>>();
            assert!(!documents[0].same_origin(&documents[1]));
            let weak = documents.iter().map(Arc::downgrade).collect::<Vec<_>>();
            let snapshots = sets.each_ref().map(|set| set.snapshot().clone());
            drop(sets);
            for (index, document) in documents.iter().enumerate() {
                assert_eq!(
                    snapshots[index].get(&inputs[index].identity).unwrap(),
                    inputs[index].source
                );
                let mut control = InvocationControl::unbounded();
                let result =
                    execute_program(&program, document, "prepared-capacity", &mut control).unwrap();
                assert_eq!(
                    serialize_xml(
                        &result,
                        &program.output,
                        "prepared-capacity",
                        inputs[index].expected.len(),
                        &mut control
                    )
                    .unwrap(),
                    inputs[index].expected
                );
            }
            drop(documents);
            assert!(weak.iter().all(|owner| owner.upgrade().is_none()));
        }
    }
}

#[test]
fn prepared_capacity_failures_publish_nothing_and_allow_same_builder_recovery() {
    for items in [5, 50, 500] {
        let input = input(items, 8);
        let mut measured = InvocationControl::unbounded();
        builder(&input, PreparationCapacity::Growth)
            .prepare(&input.identity, &mut measured)
            .unwrap();
        let used = measured.consumed(WorkDomain::XdmNode);
        for case in 0..3 {
            let make_control = || {
                let mut limits = WorkLimits::unbounded();
                limits.xdm_nodes = used - usize::from(case != 0);
                let control = InvocationControl::new(CancellationToken::new(), limits);
                if case == 2 {
                    control.cancelling_on_charge(WorkDomain::XdmNode, 1)
                } else {
                    control
                }
            };
            let mut reference_control = make_control();
            let reference = builder(&input, PreparationCapacity::Growth)
                .prepare(&input.identity, &mut reference_control);
            assert_eq!(reference.is_ok(), case == 0);
            for capacity in CAPACITIES {
                let mut candidate = builder(&input, capacity);
                let mut control = make_control();
                assert_eq!(candidate.prepare(&input.identity, &mut control), reference);
                assert_eq!(
                    control.consumed(WorkDomain::XmlEvent),
                    reference_control.consumed(WorkDomain::XmlEvent)
                );
                assert_eq!(
                    control.consumed(WorkDomain::XdmNode),
                    reference_control.consumed(WorkDomain::XdmNode)
                );
                if case != 0 {
                    assert!(candidate.documents.is_empty());
                    assert!(candidate.parsed_phase_capacity_bytes.is_empty());
                    candidate
                        .prepare(&input.identity, &mut InvocationControl::unbounded())
                        .unwrap();
                }
                let set = candidate.seal();
                assert!(set.get(&input.identity).is_some());
                assert_eq!(set.observe_totals().xdm_nodes, used);
            }
        }
    }
}
