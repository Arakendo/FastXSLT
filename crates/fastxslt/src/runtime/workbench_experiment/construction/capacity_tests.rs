//! Complete engine construction with a private prepared-capacity selection.

use super::{
    ConstructionOptions, ExperimentalEngine, PreparationCapacity, WorkbenchLimits,
    WorkbenchStylesheetResources,
};
use crate::runtime::workbench_experiment::{WorkbenchCancellation, WorkbenchFailure};
use std::{fmt::Write, sync::Arc};

const CAPACITIES: [PreparationCapacity; 3] = [
    PreparationCapacity::Growth,
    PreparationCapacity::Frozen,
    PreparationCapacity::Presized,
];
const STYLE: &[u8] = br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:copy-of select="/root"/></xsl:template></xsl:stylesheet>"#;

#[cfg(feature = "allocation-observation")]
mod allocation_tests;

mod timing_tests;

fn source(items: usize, job: usize) -> String {
    let mut source = format!("<root job=\"{job}\">");
    for index in 0..items {
        write!(
            source,
            "<item n=\"{index}\">{}&amp;&lt;</item>",
            "x".repeat(512)
        )
        .unwrap();
    }
    source.push_str("</root>");
    source
}

fn create(
    items: usize,
    job: usize,
    capacity: PreparationCapacity,
    limits: WorkbenchLimits,
    source: Vec<u8>,
    stylesheet: Vec<u8>,
) -> Result<ExperimentalEngine, WorkbenchFailure> {
    ExperimentalEngine::new_with_construction_options(
        format!("urn:ar0027:engine:{items}:{job}"),
        source,
        "urn:ar0027:engine:style",
        stylesheet,
        WorkbenchStylesheetResources::default(),
        limits,
        &ConstructionOptions {
            capacity,
            ..ConstructionOptions::default()
        },
    )
}

fn engine(items: usize, job: usize, capacity: PreparationCapacity) -> ExperimentalEngine {
    create(
        items,
        job,
        capacity,
        WorkbenchLimits::default(),
        source(items, job).into_bytes(),
        STYLE.to_vec(),
    )
    .unwrap()
}

#[test]
fn complete_candidate_engines_preserve_retention_components_and_generation_leases() {
    for items in [5, 50, 500] {
        let reference = engine(items, 8, PreparationCapacity::Growth).retention_estimate();
        for capacity in CAPACITIES {
            let old = Arc::new(engine(items, 8, capacity));
            let new = Arc::new(engine(items, 9, capacity));
            let estimate = old.retention_estimate();
            assert_eq!(
                estimate.prepared_xdm_node_count,
                reference.prepared_xdm_node_count
            );
            assert_eq!(estimate.prepared_document_count, 1);
            assert!(estimate.prepared_xdm_capacity_bytes <= reference.prepared_xdm_capacity_bytes);
            assert_eq!(
                estimate.snapshot_known_capacity_bytes,
                reference.snapshot_known_capacity_bytes
            );
            assert_eq!(
                estimate.prepared_map_known_capacity_bytes,
                reference.prepared_map_known_capacity_bytes
            );
            assert_eq!(
                estimate.known_retained_capacity_bytes - estimate.prepared_xdm_capacity_bytes,
                reference.known_retained_capacity_bytes - reference.prepared_xdm_capacity_bytes
            );
            let leases = [old.clone(), new.clone()];
            let weak = [Arc::downgrade(&old), Arc::downgrade(&new)];
            drop(old);
            drop(new);
            let mut retained = Vec::new();
            for (index, lease) in leases.iter().enumerate() {
                for _ in 0..8 {
                    let output = lease.transform_bytes("engine-capacity").unwrap();
                    assert_eq!(output, source(items, 8 + index).as_bytes());
                    retained.push((index, output));
                }
            }
            drop(leases);
            assert!(weak.iter().all(|owner| owner.upgrade().is_none()));
            for (index, output) in retained {
                assert_eq!(output, source(items, 8 + index).as_bytes());
            }
        }
    }
}

#[test]
fn candidate_creation_failures_preserve_projection_and_existing_generation() {
    for items in [5, 50, 500] {
        let old = Arc::new(engine(items, 8, PreparationCapacity::Growth));
        let nodes = old.retention_estimate().prepared_xdm_node_count;
        for case in 0..3 {
            let mut limits = WorkbenchLimits::default();
            if case == 2 {
                limits.max_xdm_nodes = nodes - 1;
            }
            let source = if case == 0 {
                b"<root>".to_vec()
            } else {
                source(items, 9).into_bytes()
            };
            let style = if case == 1 {
                b"<xsl:stylesheet".to_vec()
            } else {
                STYLE.to_vec()
            };
            let expected = create(
                items,
                9,
                PreparationCapacity::Growth,
                limits,
                source.clone(),
                style.clone(),
            )
            .err()
            .unwrap();
            for capacity in CAPACITIES {
                let failure = create(items, 9, capacity, limits, source.clone(), style.clone())
                    .err()
                    .unwrap();
                assert_eq!(failure, expected);
                assert_eq!(
                    old.transform_bytes("old-survives").unwrap(),
                    self::source(items, 8).as_bytes()
                );
                let recovered = engine(items, 9, capacity);
                assert_eq!(
                    recovered.transform_bytes("creation-recovery").unwrap(),
                    self::source(items, 9).as_bytes()
                );
            }
        }
    }
}

#[test]
fn complete_candidate_engines_preserve_cancel_and_budget_recovery() {
    for items in [5, 50, 500] {
        let reference = engine(items, 8, PreparationCapacity::Growth);
        for capacity in CAPACITIES {
            let candidate = engine(items, 8, capacity);
            for cancelled in [false, true] {
                let run = |engine: &ExperimentalEngine| {
                    let token = WorkbenchCancellation::new();
                    if cancelled {
                        token.cancel();
                    }
                    engine.transform_with_invocation_policy(
                        "engine-controls",
                        token,
                        if cancelled { usize::MAX } else { 0 },
                    )
                };
                let expected = run(&reference).unwrap_err();
                assert_eq!(run(&candidate).unwrap_err(), expected);
                assert_eq!(
                    expected.code,
                    if cancelled { "FXCT0001" } else { "FXCT0002" }
                );
                assert_eq!(
                    expected.category,
                    if cancelled { "cancelled" } else { "limit" }
                );
                assert_eq!(
                    candidate.transform_bytes("same-engine-recovery").unwrap(),
                    source(items, 8).as_bytes()
                );
            }
        }
    }
}
