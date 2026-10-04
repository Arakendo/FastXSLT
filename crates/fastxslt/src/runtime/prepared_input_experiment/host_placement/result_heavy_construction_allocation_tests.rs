//! Fresh XML/XDM construction scopes; no live allocation may escape failure.

use super::{
    BuildFailure, CANDIDATES, CancellationToken, CapacityCheckpoint, Document, InvocationControl,
    ParseLimits, WorkDomain, WorkLimits, construct, input, parse_document_controlled,
};
use crate::execution_control_experiment::ControlFailure;
use allocation_counter::{AllocationInfo, measure};

fn control(limit: usize, cancel: Option<usize>) -> InvocationControl {
    let mut limits = WorkLimits::unbounded();
    limits.xdm_nodes = limit;
    let control = InvocationControl::new(CancellationToken::new(), limits);
    if let Some(index) = cancel {
        control.cancelling_on_charge(WorkDomain::XdmNode, index)
    } else {
        control
    }
}

fn assert_released(info: AllocationInfo) {
    assert_eq!((info.count_current, info.bytes_current), (0, 0));
    assert!(info.count_total > 0);
}

fn construction_scopes(print: bool) {
    for items in [5, 50, 500] {
        let input = input(items, 8);
        let nodes = construct(
            &input,
            super::Capacity::Growth,
            false,
            &mut InvocationControl::unbounded(),
        )
        .unwrap()
        .node_count();
        for candidate in CANDIDATES {
            for polls in [false, true] {
                let mut max_peak = 0;
                let mut failures = 0;
                for limit in [0, nodes - 1, nodes] {
                    for cancel in [None, Some(0), Some(1), Some(nodes - 1)] {
                        let mut reference_control = control(limit, cancel);
                        let expected = construct(&input, candidate, false, &mut reference_control)
                            .map(|document| document.node_count());
                        assert_eq!(expected.is_ok(), limit == nodes && cancel.is_none());
                        failures += usize::from(expected.is_err());
                        let info = measure(|| {
                            let mut observed_control = control(limit, cancel);
                            let observed =
                                construct(&input, candidate, polls, &mut observed_control);
                            assert_eq!(
                                observed.as_ref().map(Document::node_count),
                                expected.as_ref().copied()
                            );
                            assert_eq!(
                                observed_control.consumed(WorkDomain::XdmNode),
                                reference_control.consumed(WorkDomain::XdmNode)
                            );
                            // Both a complete success and a partial failed construction drop here.
                            drop(observed);
                        });
                        assert_released(info);
                        max_peak = max_peak.max(info.bytes_max);
                    }
                }
                assert_eq!(failures, 11);
                if print {
                    println!(
                        "ar0027-construction-allocation items={items} candidate={candidate:?} polls={polls} scopes=12 failures={failures} current_bytes=0 current_allocations=0 max_scope_peak_bytes={max_peak}"
                    );
                }
            }
        }
    }
}

fn resize_scopes(print: bool) {
    for items in [5, 50, 500] {
        let input = input(items, 8);
        for freeze in [false, true] {
            for after in [false, true] {
                let info = measure(|| {
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
                    let mut control =
                        InvocationControl::new(token.clone(), WorkLimits::unbounded());
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
                        BuildFailure::Control(ControlFailure::Cancelled {
                            domain: WorkDomain::XdmNode
                        })
                    );
                });
                assert_released(info);
                if print {
                    println!(
                        "ar0027-resize-allocation items={items} freeze={freeze} after={after} allocations={info:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn result_heavy_construction_and_resize_failures_release_all_scoped_allocations() {
    construction_scopes(false);
    resize_scopes(false);
}

#[test]
#[ignore = "manual release fresh construction/failure requested-allocation scopes; not RSS"]
fn measure_result_heavy_construction_allocations() {
    construction_scopes(true);
    resize_scopes(true);
}
