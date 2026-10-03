//! AR-0027 allocation boundary observations; hooks do not add control checks.

use std::time::Instant;

use crate::execution_control_experiment::{
    CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
};
use crate::xdm::owned_tree_experiment::{BuildFailure, CapacityCheckpoint, Document};
use crate::xml::quick_xml_experiment::parse_document_controlled;

use super::{IDENTITY, LIMITS, fixtures};

#[test]
fn capacity_gap_hooks_preserve_unobserved_charges_and_expose_cancellation_boundaries() {
    for fixture in fixtures() {
        for presize in [false, true] {
            let parsed = parse_document_controlled(
                IDENTITY,
                fixture.xml.as_bytes(),
                LIMITS,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let events = parsed.events.len();
            let token = CancellationToken::new();
            let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
            let mut scanned = 0;
            let result = Document::from_parsed_with_capacity_observer(
                parsed,
                &mut control,
                presize,
                &mut |stage| match stage {
                    CapacityCheckpoint::BeforeSpanScan { .. } => token.cancel(),
                    CapacityCheckpoint::AfterSpanScan { events } => scanned = events,
                    _ => (),
                },
            );
            assert_eq!(
                result.unwrap_err(),
                BuildFailure::Control(ControlFailure::Cancelled {
                    domain: WorkDomain::XdmNode
                })
            );
            assert_eq!(scanned, events);
            assert_eq!(control.consumed(WorkDomain::XdmNode), 0);
        }
        let parsed = parse_document_controlled(
            IDENTITY,
            fixture.xml.as_bytes(),
            LIMITS,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let token = CancellationToken::new();
        let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
        let mut reserved = 0;
        let result = Document::from_parsed_with_capacity_observer(
            parsed,
            &mut control,
            true,
            &mut |stage| match stage {
                CapacityCheckpoint::BeforeReserve { slots } => {
                    assert_eq!(slots, fixture.nodes);
                    token.cancel();
                }
                CapacityCheckpoint::AfterReserve { slots } => reserved = slots,
                _ => (),
            },
        );
        assert_eq!(
            result.unwrap_err(),
            BuildFailure::Control(ControlFailure::Cancelled {
                domain: WorkDomain::XdmNode
            })
        );
        assert!(reserved >= fixture.nodes);
        assert_eq!(control.consumed(WorkDomain::XdmNode), 1);

        let parsed = parse_document_controlled(
            IDENTITY,
            fixture.xml.as_bytes(),
            LIMITS,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let token = CancellationToken::new();
        let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
        let mut frozen = 0;
        let document = Document::from_parsed_with_frozen_capacity_observer(
            parsed,
            &mut control,
            &mut |stage| match stage {
                CapacityCheckpoint::BeforeFreeze { slots } => {
                    assert_eq!(slots, fixture.nodes);
                    token.cancel();
                }
                CapacityCheckpoint::AfterFreeze { slots } => frozen = slots,
                _ => (),
            },
        )
        .unwrap();
        assert!(frozen >= fixture.nodes);
        assert_eq!(document.node_count(), fixture.nodes);
        assert_eq!(control.consumed(WorkDomain::XdmNode), fixture.nodes);
        // Cancellation was signalled, but no further check occurs inside freezing.
        assert_eq!(
            control.charge(WorkDomain::XdmNode, 0),
            Err(ControlFailure::Cancelled {
                domain: WorkDomain::XdmNode
            })
        );
    }
}

#[test]
fn capacity_gap_observation_is_inert_without_a_fault() {
    for fixture in fixtures() {
        for presize in [false, true] {
            let parse = || {
                parse_document_controlled(
                    IDENTITY,
                    fixture.xml.as_bytes(),
                    LIMITS,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            };
            let mut reference_control = InvocationControl::unbounded();
            let reference = if presize {
                Document::from_parsed_with_presized_node_capacity(parse(), &mut reference_control)
            } else {
                Document::from_parsed_controlled(parse(), &mut reference_control)
            }
            .unwrap();
            let mut control = InvocationControl::unbounded();
            let observed = Document::from_parsed_with_capacity_observer(
                parse(),
                &mut control,
                presize,
                &mut |_| (),
            )
            .unwrap();
            assert_eq!(
                reference.capacity_anatomy().total_capacity_bytes(),
                observed.capacity_anatomy().total_capacity_bytes()
            );
            assert_eq!(reference.node_count(), observed.node_count());
            assert_eq!(
                reference.string_value(reference.document_node()),
                observed.string_value(observed.document_node())
            );
            assert_eq!(
                reference_control.consumed(WorkDomain::XdmNode),
                control.consumed(WorkDomain::XdmNode)
            );
        }
    }
}

#[test]
#[ignore = "manual release span-scan/reserve/freeze interval observations; not hard latency bounds"]
fn measure_capacity_unchecked_intervals() {
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
        for lane in ["growth", "presized", "frozen"] {
            let mut scans = Vec::new();
            let mut resizes = Vec::new();
            let mut observed_events = 0;
            for index in 0..19 {
                let parsed = parse_document_controlled(
                    IDENTITY,
                    xml.as_bytes(),
                    LIMITS,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                if lane == "frozen" {
                    observed_events = parsed.events.len();
                }
                let mut start = Instant::now();
                let mut scan = 0;
                let mut resize = 0;
                let mut observer = |stage| match stage {
                    CapacityCheckpoint::BeforeSpanScan { events } => {
                        observed_events = events;
                        start = Instant::now();
                    }
                    CapacityCheckpoint::AfterSpanScan { .. } => scan = start.elapsed().as_nanos(),
                    CapacityCheckpoint::BeforeReserve { .. }
                    | CapacityCheckpoint::BeforeFreeze { .. } => start = Instant::now(),
                    CapacityCheckpoint::AfterReserve { .. }
                    | CapacityCheckpoint::AfterFreeze { .. } => resize = start.elapsed().as_nanos(),
                    CapacityCheckpoint::SpanScanProgress { .. } => (),
                };
                let mut control = InvocationControl::unbounded();
                let document = if lane == "frozen" {
                    Document::from_parsed_with_frozen_capacity_observer(
                        parsed,
                        &mut control,
                        &mut observer,
                    )
                } else {
                    Document::from_parsed_with_capacity_observer(
                        parsed,
                        &mut control,
                        lane == "presized",
                        &mut observer,
                    )
                }
                .unwrap();
                assert_eq!(document.node_count(), nodes);
                if index >= 3 {
                    scans.push(scan);
                    resizes.push(resize);
                }
            }
            scans.sort_unstable();
            resizes.sort_unstable();
            println!(
                "ar0027 gap_shape={name} lane={lane} nodes={nodes} events={observed_events} scan_observed={} resize_observed={} scan_median_ns={} scan_max_ns={} resize_median_ns={} resize_max_ns={}",
                lane != "frozen",
                lane != "growth",
                scans[8],
                scans[15],
                resizes[8],
                resizes[15]
            );
        }
    }
}
