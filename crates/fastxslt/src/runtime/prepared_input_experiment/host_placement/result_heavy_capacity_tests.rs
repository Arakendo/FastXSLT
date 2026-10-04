//! Capacity candidates with the same copied-result lifecycle, not adapter timing.

use super::{Input, StylesheetProgram, input, program};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document_controlled};
use std::{hint::black_box, time::Instant};

#[derive(Clone, Copy, Debug)]
enum Capacity {
    Growth,
    Frozen,
    Presized,
}

const CANDIDATES: [Capacity; 3] = [Capacity::Growth, Capacity::Frozen, Capacity::Presized];

#[path = "result_heavy_capacity_control_tests.rs"]
mod control_tests;

fn prepare(
    input: &Input,
    candidate: Capacity,
    control: &mut InvocationControl,
) -> (crate::resources::ResourceSnapshot, Document) {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
    resources
        .admit(&input.identity, input.source.clone())
        .unwrap();
    let snapshot = resources.seal();
    let parsed = parse_document_controlled(
        &input.identity,
        snapshot.get(&input.identity).unwrap(),
        ParseLimits {
            max_events: 10_000,
            max_depth: 64,
        },
        control,
    )
    .unwrap();
    let document = match candidate {
        Capacity::Growth => Document::from_parsed_controlled(parsed, control),
        Capacity::Frozen => Document::from_parsed_with_frozen_node_capacity(parsed, control),
        Capacity::Presized => Document::from_parsed_with_presized_node_capacity(parsed, control),
    }
    .unwrap();
    (snapshot, document)
}

fn sample(
    program: &StylesheetProgram,
    input: &Input,
    candidate: Capacity,
    uses: usize,
) -> [u128; 3] {
    let total = Instant::now();
    let mut control = InvocationControl::unbounded();
    let phase = Instant::now();
    let (snapshot, document) = prepare(input, candidate, &mut control);
    let preparation = phase.elapsed().as_nanos();
    let phase = Instant::now();
    for _ in 0..uses {
        let result = execute_program(program, &document, "capacity-copy", &mut control).unwrap();
        let output = serialize_xml(
            &result,
            &program.output,
            "capacity-copy",
            input.expected.len(),
            &mut control,
        )
        .unwrap();
        assert_eq!(output, input.expected);
        black_box(output);
    }
    let execution_output_release = phase.elapsed().as_nanos();
    drop(document);
    drop(snapshot);
    [
        preparation,
        execution_output_release,
        total.elapsed().as_nanos(),
    ]
}

#[test]
fn result_heavy_capacity_candidates_preserve_output_charges_and_retired_source() {
    let program = program();
    for items in [5, 50, 500] {
        let input = input(items, 8);
        let mut reference = None;
        for candidate in CANDIDATES {
            let mut control = InvocationControl::unbounded();
            let (snapshot, document) = prepare(&input, candidate, &mut control);
            let result =
                execute_program(&program, &document, "capacity-copy", &mut control).unwrap();
            drop(document);
            drop(snapshot);
            let output = serialize_xml(
                &result,
                &program.output,
                "capacity-copy",
                input.expected.len(),
                &mut control,
            )
            .unwrap();
            assert_eq!(output, input.expected);
            let charges = [
                WorkDomain::XmlEvent,
                WorkDomain::XdmNode,
                WorkDomain::XdmStringValueNode,
                WorkDomain::XPathNodeVisit,
                WorkDomain::XPathOperation,
                WorkDomain::XsltInstruction,
                WorkDomain::XsltTemplateCandidate,
                WorkDomain::ResultNode,
                WorkDomain::ResultTextByte,
                WorkDomain::SerializedByte,
            ]
            .map(|domain| control.consumed(domain));
            if let Some(expected) = reference {
                assert_eq!(charges, expected);
            } else {
                reference = Some(charges);
            }
            sample(&program, &input, candidate, 8);
        }
    }
}

#[test]
#[ignore = "manual release result-heavy capacity comparison; no publication gate"]
fn measure_result_heavy_capacity_lifecycle() {
    let order = std::env::var("AR0027_RESULT_ORDER").map_or(0, |v| v.parse::<usize>().unwrap());
    assert!(order < 3);
    let program = program();
    for items in [5, 50, 500] {
        let input = input(items, 8);
        for uses in [1, 8] {
            for offset in 0..3 {
                let candidate = CANDIDATES[(offset + order) % 3];
                for _ in 0..8 {
                    sample(&program, &input, candidate, uses);
                }
                let samples: Vec<_> = (0..32)
                    .map(|_| sample(&program, &input, candidate, uses))
                    .collect();
                for phase in 0..3 {
                    let mut values: Vec<_> = samples.iter().map(|s| s[phase]).collect();
                    values.sort_unstable();
                    println!(
                        "ar0027-copy-capacity order={order} items={items} uses={uses} candidate={candidate:?} phase={phase} median_ns={} p95_ns={} publication_eligible=false",
                        u128::midpoint(values[15], values[16]),
                        values[30]
                    );
                }
            }
        }
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release result-heavy capacity allocation comparison; not RSS"]
fn measure_result_heavy_capacity_allocations() {
    let program = program();
    for items in [5, 50, 500] {
        let input = input(items, 8);
        for candidate in CANDIDATES {
            for release in [false, true] {
                let mut retained = None;
                let info = allocation_counter::measure(|| {
                    let mut control = InvocationControl::unbounded();
                    let (snapshot, document) = prepare(&input, candidate, &mut control);
                    let result =
                        execute_program(&program, &document, "capacity-copy", &mut control)
                            .unwrap();
                    let output = serialize_xml(
                        &result,
                        &program.output,
                        "capacity-copy",
                        input.expected.len(),
                        &mut control,
                    )
                    .unwrap();
                    assert_eq!(output, input.expected);
                    if !release {
                        retained = Some((snapshot, document, result, output));
                    }
                });
                if release {
                    assert_eq!((info.count_current, info.bytes_current), (0, 0));
                }
                println!(
                    "ar0027-copy-allocation items={items} candidate={candidate:?} released={release} allocations={info:?}"
                );
                drop(retained);
            }
        }
    }
}
