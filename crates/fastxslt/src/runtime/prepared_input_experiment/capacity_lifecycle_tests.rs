//! AR-0027 capacity candidates through a direct Rust admitted-source lifecycle.

use std::{hint::black_box, sync::Arc, time::Instant};

use crate::execution_control_experiment::InvocationControl;
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document_controlled;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{Fixture, IDENTITY, LIMITS, fixtures};

#[derive(Clone, Copy, Debug)]
enum Capacity {
    Growth,
    Frozen,
    Presized,
}

impl Capacity {
    fn name(self) -> &'static str {
        match self {
            Self::Growth => "growth",
            Self::Frozen => "frozen",
            Self::Presized => "presized",
        }
    }
}

#[derive(Default)]
struct Phases {
    admission: u128,
    preparation: u128,
    execution: u128,
    serialization: u128,
    total: u128,
}

fn compiled() -> StylesheetProgram {
    const ID: &str = "urn:ar0027:capacity-count-style";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit(ID, br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="count(//*)"/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    compile_resource(&resources.seal(), ID).unwrap()
}

fn expected(fixture: &Fixture) -> &'static str {
    match fixture.name {
        "wide" => "1001",
        "deep" => "256",
        "attribute-heavy" | "text-heavy" => "65",
        "namespace-heavy" => "129",
        "low-repetition" => "513",
        _ => panic!("fixture requires an independently specified element count"),
    }
}

fn sample(
    program: &StylesheetProgram,
    fixture: &Fixture,
    capacity: Capacity,
    reuses: usize,
) -> Phases {
    let start = Instant::now();
    let mut phases = Phases::default();
    {
        let phase = Instant::now();
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
        resources
            .admit(IDENTITY, fixture.xml.as_bytes().to_vec())
            .unwrap();
        let snapshot = resources.seal();
        phases.admission = phase.elapsed().as_nanos();
        let phase = Instant::now();
        let mut preparation = InvocationControl::unbounded();
        let parsed = parse_document_controlled(
            IDENTITY,
            snapshot.get(IDENTITY).unwrap(),
            LIMITS,
            &mut preparation,
        )
        .unwrap();
        let document = Arc::new(
            match capacity {
                Capacity::Growth => Document::from_parsed_controlled(parsed, &mut preparation),
                Capacity::Frozen => {
                    Document::from_parsed_with_frozen_node_capacity(parsed, &mut preparation)
                }
                Capacity::Presized => {
                    Document::from_parsed_with_presized_node_capacity(parsed, &mut preparation)
                }
            }
            .unwrap(),
        );
        phases.preparation = phase.elapsed().as_nanos();
        assert_eq!(document.node_count(), fixture.nodes);
        for _ in 0..reuses {
            let mut control = InvocationControl::unbounded();
            let phase = Instant::now();
            let result = execute_program(
                program,
                &document,
                "ar0027-capacity-lifecycle",
                &mut control,
            )
            .unwrap();
            phases.execution += phase.elapsed().as_nanos();
            let phase = Instant::now();
            let serialized = serialize_xml(
                &result,
                &program.output,
                "ar0027-capacity-lifecycle",
                8192,
                &mut control,
            )
            .unwrap();
            phases.serialization += phase.elapsed().as_nanos();
            assert_eq!(serialized, expected(fixture));
            black_box(&serialized);
        }
    }
    // Includes exact validation and release; compilation and fixture creation are outside.
    phases.total = start.elapsed().as_nanos();
    phases
}

#[test]
fn capacity_lifecycle_candidates_return_exact_results_on_all_shapes() {
    let program = compiled();
    for fixture in fixtures() {
        for capacity in [Capacity::Growth, Capacity::Frozen, Capacity::Presized] {
            for reuses in [1, 8] {
                sample(&program, &fixture, capacity, reuses);
            }
        }
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
fn capacity_lifecycle_construction_releases_successful_and_failed_allocations() {
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    for fixture in fixtures() {
        for capacity in [Capacity::Growth, Capacity::Frozen, Capacity::Presized] {
            for limit in [fixture.nodes, fixture.nodes - 1] {
                let allocations = allocation_counter::measure(|| {
                    let mut resources =
                        ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
                    resources
                        .admit(IDENTITY, fixture.xml.as_bytes().to_vec())
                        .unwrap();
                    let snapshot = resources.seal();
                    let mut limits = WorkLimits::unbounded();
                    limits.xdm_nodes = limit;
                    let mut control = InvocationControl::new(CancellationToken::new(), limits);
                    let parsed = parse_document_controlled(
                        IDENTITY,
                        snapshot.get(IDENTITY).unwrap(),
                        LIMITS,
                        &mut control,
                    )
                    .unwrap();
                    let result = match capacity {
                        Capacity::Growth => Document::from_parsed_controlled(parsed, &mut control),
                        Capacity::Frozen => {
                            Document::from_parsed_with_frozen_node_capacity(parsed, &mut control)
                        }
                        Capacity::Presized => {
                            Document::from_parsed_with_presized_node_capacity(parsed, &mut control)
                        }
                    };
                    assert_eq!(result.is_ok(), limit == fixture.nodes);
                    drop(result);
                });
                assert!(allocations.count_total > 0);
                assert_eq!(allocations.count_current, 0);
                assert_eq!(allocations.bytes_current, 0);
            }
        }
    }
}

#[test]
#[ignore = "manual release admitted-source capacity lifecycle comparison; omit allocator feature"]
fn measure_capacity_single_use_and_reuse_lifecycle() {
    fn median(samples: &[Phases], field: fn(&Phases) -> u128) -> u128 {
        let mut values: Vec<_> = samples.iter().map(field).collect();
        values.sort_unstable();
        values[100]
    }
    let program = compiled();
    let candidates = [Capacity::Growth, Capacity::Frozen, Capacity::Presized];
    for fixture in fixtures() {
        for reuses in [1, 8] {
            let mut samples: [Vec<Phases>; 3] = std::array::from_fn(|_| Vec::with_capacity(201));
            for index in 0..217 {
                // Rotate the first lane, rather than letting one always receive the freshest state.
                for offset in 0..3 {
                    let lane = (index + offset) % 3;
                    let phases = sample(&program, &fixture, candidates[lane], reuses);
                    if index >= 16 {
                        samples[lane].push(phases);
                    }
                }
            }
            for lane in 0..3 {
                println!(
                    "ar0027 lifecycle_shape={} candidate={} reuses={} admission_ns={} preparation_ns={} execution_ns={} serialization_ns={} total_release_ns={}",
                    fixture.name,
                    candidates[lane].name(),
                    reuses,
                    median(&samples[lane], |p| p.admission),
                    median(&samples[lane], |p| p.preparation),
                    median(&samples[lane], |p| p.execution),
                    median(&samples[lane], |p| p.serialization),
                    median(&samples[lane], |p| p.total)
                );
            }
        }
    }
}
