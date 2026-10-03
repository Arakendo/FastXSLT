//! AR-0027 uncharged cancellation polls; ordinary construction remains the reference.

use super::{IDENTITY, LIMITS, fixtures};
use crate::execution_control_experiment::{
    CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
};
use crate::xdm::owned_tree_experiment::{BuildFailure, CapacityCheckpoint, Document};
use crate::xml::quick_xml_experiment::{ParsedDocument, parse_document_controlled};
use std::time::Instant;

fn parsed(xml: &str) -> ParsedDocument {
    parse_document_controlled(
        IDENTITY,
        xml.as_bytes(),
        LIMITS,
        &mut InvocationControl::unbounded(),
    )
    .unwrap()
}

#[test]
fn capacity_polls_bound_scan_visits_after_signal_without_consuming_work() {
    let xml = format!("<r>{}</r>", "<item code='x'>text</item>".repeat(1024));
    for presize in [false, true] {
        for signal_at in [1, 255, 256, 257, 3074] {
            let token = CancellationToken::new();
            let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
            let mut visited = 0;
            let mut observer = |stage| {
                if let CapacityCheckpoint::SpanScanProgress { visited: count } = stage {
                    visited = count;
                    if count == signal_at {
                        token.cancel();
                    }
                }
            };
            let failure = Document::from_parsed_with_capacity_polls(
                parsed(&xml),
                &mut control,
                presize,
                false,
                Some(&mut observer),
            )
            .unwrap_err();
            assert_eq!(
                failure,
                BuildFailure::Control(ControlFailure::Cancelled {
                    domain: WorkDomain::XdmNode
                })
            );
            assert!(visited >= signal_at && visited - signal_at < 256);
            assert_eq!(control.consumed(WorkDomain::XdmNode), 0);
        }
    }
}

#[test]
fn capacity_polls_reject_before_and_after_resize_without_publishing_a_document() {
    for fixture in fixtures() {
        for freeze in [false, true] {
            for after in [false, true] {
                let token = CancellationToken::new();
                let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
                let mut signalled = false;
                let mut resized = false;
                let mut observer = |stage| {
                    let is_before = matches!(
                        stage,
                        CapacityCheckpoint::BeforeReserve { .. }
                            | CapacityCheckpoint::BeforeFreeze { .. }
                    );
                    let is_after = matches!(
                        stage,
                        CapacityCheckpoint::AfterReserve { .. }
                            | CapacityCheckpoint::AfterFreeze { .. }
                    );
                    resized |= is_after;
                    if (after && is_after) || (!after && is_before) {
                        token.cancel();
                        signalled = true;
                    }
                };
                let failure = Document::from_parsed_with_capacity_polls(
                    parsed(&fixture.xml),
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
                assert_eq!(
                    control.consumed(WorkDomain::XdmNode),
                    if freeze { fixture.nodes } else { 1 }
                );
            }
        }
    }
}

#[test]
fn capacity_polls_preserve_success_budget_failure_and_charge_fault_ordering() {
    for fixture in fixtures() {
        for lane in 0..3 {
            for limit in [fixture.nodes, fixture.nodes - 1, 0] {
                for fault in [None, Some(0), Some(1), Some(fixture.nodes - 1)] {
                    let make_control = || {
                        let mut limits = WorkLimits::unbounded();
                        limits.xdm_nodes = limit;
                        let control = InvocationControl::new(CancellationToken::new(), limits);
                        if let Some(index) = fault {
                            control.cancelling_on_charge(WorkDomain::XdmNode, index)
                        } else {
                            control
                        }
                    };
                    let mut reference_control = make_control();
                    let reference = match lane {
                        0 => Document::from_parsed_controlled(
                            parsed(&fixture.xml),
                            &mut reference_control,
                        ),
                        1 => Document::from_parsed_with_presized_node_capacity(
                            parsed(&fixture.xml),
                            &mut reference_control,
                        ),
                        _ => Document::from_parsed_with_frozen_node_capacity(
                            parsed(&fixture.xml),
                            &mut reference_control,
                        ),
                    };
                    let mut control = make_control();
                    let candidate = Document::from_parsed_with_capacity_polls(
                        parsed(&fixture.xml),
                        &mut control,
                        lane == 1,
                        lane == 2,
                        None,
                    );
                    match (reference, candidate) {
                        (Ok(reference), Ok(candidate)) => {
                            assert_eq!(reference.capacity_anatomy(), candidate.capacity_anatomy());
                            assert_eq!(
                                reference.string_value(reference.document_node()),
                                candidate.string_value(candidate.document_node())
                            );
                        }
                        (Err(reference), Err(candidate)) => assert_eq!(reference, candidate),
                        _ => panic!("checkpoint candidate changed disposition"),
                    }
                    assert_eq!(
                        reference_control.consumed(WorkDomain::XdmNode),
                        control.consumed(WorkDomain::XdmNode)
                    );
                }
            }
        }
    }
}

#[test]
#[ignore = "manual release checkpoint construction overhead; not host performance"]
fn measure_capacity_checkpoint_overhead() {
    let mut cases: Vec<_> = fixtures()
        .into_iter()
        .map(|fixture| (fixture.name, fixture.xml, fixture.nodes))
        .collect();
    cases.push((
        "wide-32000",
        format!(
            "<r>{}</r>",
            "<item code='same'>shared text</item>".repeat(32000)
        ),
        96002,
    ));
    for (name, xml, nodes) in cases {
        let mut samples: [Vec<u128>; 6] = std::array::from_fn(|_| Vec::new());
        for round in 0..34 {
            for offset in 0..6 {
                let lane = (round + offset) % 6;
                let source = parsed(&xml);
                let mut control = InvocationControl::unbounded();
                let start = Instant::now();
                let document = if lane >= 3 {
                    Document::from_parsed_with_capacity_polls(
                        source,
                        &mut control,
                        lane == 4,
                        lane == 5,
                        None,
                    )
                } else {
                    match lane {
                        0 => Document::from_parsed_controlled(source, &mut control),
                        1 => {
                            Document::from_parsed_with_presized_node_capacity(source, &mut control)
                        }
                        _ => Document::from_parsed_with_frozen_node_capacity(source, &mut control),
                    }
                }
                .unwrap();
                let elapsed = start.elapsed().as_nanos();
                assert_eq!(document.node_count(), nodes);
                assert_eq!(control.consumed(WorkDomain::XdmNode), nodes);
                if round >= 3 {
                    samples[lane].push(elapsed);
                }
            }
        }
        for (lane, samples) in samples.iter_mut().enumerate() {
            samples.sort_unstable();
            println!(
                "ar0027 checkpoint_shape={name} lane={lane} nodes={nodes} median_ns={} p95_ns={}",
                samples[15], samples[29]
            );
        }
    }
}
