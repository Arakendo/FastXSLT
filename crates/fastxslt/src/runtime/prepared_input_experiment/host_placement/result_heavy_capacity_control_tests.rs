//! Result-heavy checkpoint/control parity with recovery on the same document.

use super::{CANDIDATES, Capacity, input, prepare, program};
use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
use crate::xdm::owned_tree_experiment::{BuildFailure, CapacityCheckpoint, Document};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document_controlled};

#[cfg(feature = "allocation-observation")]
#[path = "result_heavy_construction_allocation_tests.rs"]
mod allocation_tests;

#[path = "result_heavy_publication_tests.rs"]
mod publication_tests;

#[path = "result_heavy_concurrent_tests.rs"]
mod concurrent_tests;

fn construct(
    input: &super::Input,
    candidate: Capacity,
    polls: bool,
    control: &mut InvocationControl,
) -> Result<Document, crate::xdm::owned_tree_experiment::BuildFailure> {
    let parsed = parse_document_controlled(
        &input.identity,
        &input.source,
        ParseLimits {
            max_events: 10_000,
            max_depth: 64,
        },
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    if polls {
        Document::from_parsed_with_capacity_polls(
            parsed,
            control,
            matches!(candidate, Capacity::Presized),
            matches!(candidate, Capacity::Frozen),
            None,
        )
    } else {
        match candidate {
            Capacity::Growth => Document::from_parsed_controlled(parsed, control),
            Capacity::Frozen => Document::from_parsed_with_frozen_node_capacity(parsed, control),
            Capacity::Presized => {
                Document::from_parsed_with_presized_node_capacity(parsed, control)
            }
        }
    }
}

#[test]
fn result_heavy_checkpoint_construction_preserves_limits_and_charge_cancellation() {
    for items in [5, 50, 500] {
        let input = input(items, 8);
        let nodes = construct(
            &input,
            Capacity::Growth,
            false,
            &mut InvocationControl::unbounded(),
        )
        .unwrap()
        .node_count();
        for candidate in CANDIDATES {
            for limit in [0, nodes - 1, nodes] {
                for cancel in [None, Some(0), Some(1), Some(nodes - 1)] {
                    let make_control = || {
                        let mut limits = WorkLimits::unbounded();
                        limits.xdm_nodes = limit;
                        let control = InvocationControl::new(CancellationToken::new(), limits);
                        if let Some(index) = cancel {
                            control.cancelling_on_charge(WorkDomain::XdmNode, index)
                        } else {
                            control
                        }
                    };
                    let mut reference_control = make_control();
                    let reference = construct(&input, candidate, false, &mut reference_control);
                    assert_eq!(reference.is_ok(), limit == nodes && cancel.is_none());
                    let mut polled_control = make_control();
                    let polled = construct(&input, candidate, true, &mut polled_control);
                    assert_eq!(
                        reference_control.consumed(WorkDomain::XdmNode),
                        polled_control.consumed(WorkDomain::XdmNode)
                    );
                    match (reference, polled) {
                        (Ok(reference), Ok(polled)) => {
                            assert_eq!(reference.capacity_anatomy(), polled.capacity_anatomy());
                            assert_eq!(
                                reference.string_value(reference.document_node()),
                                polled.string_value(polled.document_node())
                            );
                        }
                        (Err(reference), Err(polled)) => assert_eq!(reference, polled),
                        _ => panic!("checkpoint polling changed construction disposition"),
                    }
                }
            }
        }
    }
}

#[test]
fn result_heavy_capacity_resize_cancellation_never_publishes_partial_state() {
    for items in [5, 50, 500] {
        let input = input(items, 8);
        let nodes = construct(
            &input,
            Capacity::Growth,
            false,
            &mut InvocationControl::unbounded(),
        )
        .unwrap()
        .node_count();
        for freeze in [false, true] {
            for after in [false, true] {
                let parsed = parse_document_controlled(
                    &input.identity,
                    &input.source,
                    ParseLimits {
                        max_events: 10_000,
                        max_depth: 64,
                    },
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                let token = CancellationToken::new();
                let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
                let mut signalled = false;
                let mut resized = false;
                let mut observer = |checkpoint| {
                    let before_resize = matches!(
                        checkpoint,
                        CapacityCheckpoint::BeforeReserve { .. }
                            | CapacityCheckpoint::BeforeFreeze { .. }
                    );
                    let after_resize = matches!(
                        checkpoint,
                        CapacityCheckpoint::AfterReserve { .. }
                            | CapacityCheckpoint::AfterFreeze { .. }
                    );
                    resized |= after_resize;
                    if (!after && before_resize) || (after && after_resize) {
                        signalled = true;
                        token.cancel();
                    }
                };
                let failure = Document::from_parsed_with_capacity_polls(
                    parsed,
                    &mut control,
                    !freeze,
                    freeze,
                    Some(&mut observer),
                )
                .unwrap_err();
                assert!(signalled);
                assert_eq!(resized, after);
                assert_eq!(
                    failure,
                    BuildFailure::Control(
                        crate::execution_control_experiment::ControlFailure::Cancelled {
                            domain: WorkDomain::XdmNode
                        }
                    )
                );
                assert_eq!(
                    control.consumed(WorkDomain::XdmNode),
                    if freeze { nodes } else { 1 }
                );
                // The failed attempt did not mutate the caller's bytes.
                let recovered = construct(
                    &input,
                    if freeze {
                        Capacity::Frozen
                    } else {
                        Capacity::Presized
                    },
                    true,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                assert_eq!(recovered.node_count(), nodes);
            }
        }
    }
}

#[test]
fn result_heavy_failures_preserve_diagnostics_and_same_source_recovery() {
    let program = program();
    for items in [5, 50, 500] {
        let input = input(items, 8);
        let (_, reference) = prepare(
            &input,
            Capacity::Growth,
            &mut InvocationControl::unbounded(),
        );
        let mut measured = InvocationControl::unbounded();
        let result =
            execute_program(&program, &reference, "capacity-controls", &mut measured).unwrap();
        serialize_xml(
            &result,
            &program.output,
            "capacity-controls",
            input.expected.len(),
            &mut measured,
        )
        .unwrap();
        for domain in [
            WorkDomain::ResultNode,
            WorkDomain::ResultTextByte,
            WorkDomain::SerializedByte,
        ] {
            let used = measured.consumed(domain);
            assert!(used > 0);
            for case in 0..3 {
                let make_control = || runtime_case_control(domain, used, case);
                let run = |document: &Document, control: &mut InvocationControl| {
                    execute_program(&program, document, "capacity-controls", control).and_then(
                        |result| {
                            serialize_xml(
                                &result,
                                &program.output,
                                "capacity-controls",
                                input.expected.len(),
                                control,
                            )
                        },
                    )
                };
                let mut reference_control = make_control();
                let expected = run(&reference, &mut reference_control);
                assert_eq!(expected.is_ok(), case == 0);
                #[cfg(feature = "workbench")]
                if let Err(failure) = &expected {
                    let (code, category, request, _, _) = failure.workbench_parts();
                    assert_eq!(request, Some("capacity-controls"));
                    assert_eq!(
                        (code, category),
                        if case == 2 {
                            ("FXCT0001", "cancelled")
                        } else {
                            ("FXCT0002", "limit")
                        }
                    );
                }
                for candidate in CANDIDATES {
                    for polls in [false, true] {
                        let document = construct(
                            &input,
                            candidate,
                            polls,
                            &mut InvocationControl::unbounded(),
                        )
                        .unwrap();
                        let mut control = make_control();
                        #[cfg(feature = "allocation-observation")]
                        {
                            let info = allocation_counter::measure(|| {
                                assert_eq!(run(&document, &mut control), expected);
                            });
                            assert_eq!((info.count_current, info.bytes_current), (0, 0));
                        }
                        #[cfg(not(feature = "allocation-observation"))]
                        assert_eq!(run(&document, &mut control), expected);
                        assert_eq!(control.consumed(domain), reference_control.consumed(domain));
                        // Failed invocation and serializer state must not poison this source.
                        assert_eq!(
                            run(&document, &mut InvocationControl::unbounded()).unwrap(),
                            input.expected
                        );
                    }
                }
            }
        }
    }
}

fn runtime_case_control(domain: WorkDomain, used: usize, case: usize) -> InvocationControl {
    let mut limits = WorkLimits::unbounded();
    let limit = if case == 0 { used } else { used - 1 };
    match domain {
        WorkDomain::ResultNode => limits.result_nodes = limit,
        WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
        WorkDomain::SerializedByte => limits.serialized_bytes = limit,
        _ => unreachable!(),
    }
    let control = InvocationControl::new(CancellationToken::new(), limits);
    if case == 2 {
        control.cancelling_on_charge(domain, 1)
    } else {
        control
    }
}
