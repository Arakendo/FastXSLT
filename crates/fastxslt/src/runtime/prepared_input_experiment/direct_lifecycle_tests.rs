//! AR-0027 direct Rust caller control, without adapters or layout changes.

use std::{hint::black_box, time::Instant};

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{PreparedInputBuilder, PreparedInputSet};

const SOURCE_ID: &str = "urn:ar0027:source";
const STYLE_ID: &str = "urn:ar0027:stylesheet";
const SOURCE: &[u8] = include_bytes!("../../../../../vendor/xslt30-test/tests/expr/for/for03.xml");
const STYLE: &[u8] = include_bytes!("../../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl");
const EXPECTED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>";

#[cfg(feature = "workbench")]
#[path = "host_placement/direct_creation_tests.rs"]
mod direct_creation_tests;

#[derive(Debug, Default)]
struct Phases {
    admission: u128,
    compile: u128,
    prepare: u128,
    execute: u128,
    serialize: u128,
    total: u128,
}

fn snapshot(source_id: &str) -> ResourceSnapshot {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    resources.admit(source_id, SOURCE.to_vec()).unwrap();
    resources.admit(STYLE_ID, STYLE.to_vec()).unwrap();
    resources.seal()
}

fn prepared(snapshot: ResourceSnapshot, source_id: &str) -> PreparedInputSet {
    let mut builder = PreparedInputBuilder::new(snapshot);
    builder
        .prepare(source_id, &mut InvocationControl::unbounded())
        .unwrap();
    builder.seal()
}

fn execute(
    program: &StylesheetProgram,
    prepared: &PreparedInputSet,
    source_id: &str,
    request_id: &str,
    phases: &mut Phases,
) {
    let mut control = InvocationControl::unbounded();
    let start = Instant::now();
    let document = prepared.get(source_id).unwrap();
    let result = execute_program(program, &document, request_id, &mut control).unwrap();
    phases.execute = start.elapsed().as_nanos();
    let start = Instant::now();
    let serialized =
        serialize_xml(&result, &program.output, request_id, 8192, &mut control).unwrap();
    phases.serialize = start.elapsed().as_nanos();
    // Validation is inside total, but outside execution/serialization phase clocks.
    assert_eq!(serialized, EXPECTED);
    black_box(&serialized);
}

fn one_shot(source_id: &str, request_id: &str) -> Phases {
    let total_start = Instant::now();
    let mut phases = Phases::default();
    {
        let start = Instant::now();
        let snapshot = snapshot(source_id);
        phases.admission = start.elapsed().as_nanos();
        let start = Instant::now();
        let program = compile_resource(&snapshot, STYLE_ID).unwrap();
        phases.compile = start.elapsed().as_nanos();
        let start = Instant::now();
        let prepared = prepared(snapshot, source_id);
        phases.prepare = start.elapsed().as_nanos();
        execute(&program, &prepared, source_id, request_id, &mut phases);
    }
    // Includes validation and release of owned result, prepared state and snapshot.
    phases.total = total_start.elapsed().as_nanos();
    phases
}

fn reused(program: &StylesheetProgram, prepared: &PreparedInputSet) -> Phases {
    let start = Instant::now();
    let mut phases = Phases::default();
    execute(program, prepared, SOURCE_ID, "ar0027-reuse", &mut phases);
    phases.total = start.elapsed().as_nanos();
    phases
}

// This pinned stylesheet has no runtime resource dependencies. A source-only
// snapshot is a control for this case, not a general resource-graph contract.
fn source_snapshot(source_id: &str) -> ResourceSnapshot {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit(source_id, SOURCE.to_vec()).unwrap();
    resources.seal()
}

fn fresh_source(program: &StylesheetProgram, source_id: &str, request_id: &str) -> Phases {
    let total_start = Instant::now();
    let mut phases = Phases::default();
    {
        let start = Instant::now();
        let snapshot = source_snapshot(source_id);
        phases.admission = start.elapsed().as_nanos();
        let start = Instant::now();
        let prepared = prepared(snapshot, source_id);
        phases.prepare = start.elapsed().as_nanos();
        execute(program, &prepared, source_id, request_id, &mut phases);
    }
    phases.total = total_start.elapsed().as_nanos();
    phases
}

#[test]
fn compiled_program_accepts_fresh_sources_without_merging_origins() {
    let program = compile_resource(&snapshot(SOURCE_ID), STYLE_ID).unwrap();
    let first_id = "urn:ar0027:fresh:00000";
    let second_id = "urn:ar0027:fresh:00001";
    let first = prepared(source_snapshot(first_id), first_id);
    let second = prepared(source_snapshot(second_id), second_id);
    let first_document = first.get(first_id).unwrap();
    let second_document = second.get(second_id).unwrap();
    assert!(!first_document.same_origin(&second_document));
    assert_eq!(
        first_document
            .location(first_document.document_node())
            .resource,
        first_id
    );
    assert_eq!(
        second_document
            .location(second_document.document_node())
            .resource,
        second_id
    );
    // Retiring one independent snapshot/prepared owner leaves the other usable.
    drop(first_document);
    drop(first);
    let mut phases = Phases::default();
    execute(&program, &second, second_id, "ar0027-second", &mut phases);
    fresh_source(&program, first_id, "ar0027-recreated");
}

#[test]
fn direct_lifecycle_preserves_exact_result_and_independent_equal_byte_sources() {
    for identity in [SOURCE_ID, "urn:ar0027:equal-bytes-other-origin"] {
        let phases = one_shot(identity, "ar0027-single-use");
        assert!(phases.total >= phases.execute + phases.serialize);
    }
    let snapshot = snapshot(SOURCE_ID);
    let program = compile_resource(&snapshot, STYLE_ID).unwrap();
    let prepared = prepared(snapshot, SOURCE_ID);
    for _ in 0..3 {
        reused(&program, &prepared);
    }
    assert_eq!(prepared.observe(SOURCE_ID).unwrap().raw_bytes, SOURCE.len());
}

#[test]
fn direct_lifecycle_cancelled_preparation_leaves_no_entry_and_allows_retry() {
    let snapshot = snapshot(SOURCE_ID);
    let program = compile_resource(&snapshot, STYLE_ID).unwrap();
    let mut builder = PreparedInputBuilder::new(snapshot);
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XmlEvent, 2);
    assert!(matches!(
        builder.prepare(SOURCE_ID, &mut cancelled),
        Err(super::PreparationFailure::Control(
            ControlFailure::Cancelled {
                domain: WorkDomain::XmlEvent,
            }
        ))
    ));
    assert!(builder.documents.is_empty());
    builder
        .prepare(SOURCE_ID, &mut InvocationControl::unbounded())
        .unwrap();
    let prepared = builder.seal();
    assert_eq!(
        prepared.observe_totals().xdm_nodes,
        prepared.get(SOURCE_ID).unwrap().node_count()
    );
    reused(&program, &prepared);
}

fn report(lane: &str, samples: &[Phases]) {
    fn median(samples: &[Phases], field: fn(&Phases) -> u128) -> u128 {
        let mut values: Vec<_> = samples.iter().map(field).collect();
        values.sort_unstable();
        values[values.len() / 2]
    }
    println!(
        "ar0027 lane={lane} observations={} source_bytes={} admission_median_ns={} compile_median_ns={} prepare_median_ns={} execute_median_ns={} serialize_median_ns={} total_median_ns={}",
        samples.len(),
        SOURCE.len(),
        median(samples, |p| p.admission),
        median(samples, |p| p.compile),
        median(samples, |p| p.prepare),
        median(samples, |p| p.execute),
        median(samples, |p| p.serialize),
        median(samples, |p| p.total)
    );
}

#[test]
#[ignore = "manual release direct-Rust lifecycle control; phase clocks add overhead"]
fn measure_direct_lifecycle_baseline() {
    const OBSERVATIONS: usize = 1001;
    let snapshot = snapshot(SOURCE_ID);
    let program = compile_resource(&snapshot, STYLE_ID).unwrap();
    let prepared = prepared(snapshot, SOURCE_ID);
    for _ in 0..32 {
        one_shot(SOURCE_ID, "ar0027-warmup");
        reused(&program, &prepared);
    }
    let mut cold = Vec::with_capacity(OBSERVATIONS);
    let mut warm = Vec::with_capacity(OBSERVATIONS);
    for index in 0..OBSERVATIONS {
        // Alternate order to avoid placing every cold call before every warm call.
        if index % 2 == 0 {
            cold.push(one_shot(SOURCE_ID, "ar0027-cold"));
            warm.push(reused(&program, &prepared));
        } else {
            warm.push(reused(&program, &prepared));
            cold.push(one_shot(SOURCE_ID, "ar0027-cold"));
        }
    }
    report("admit-compile-prepare-execute-result-release", &cold);
    report("compiled-prepared-reuse-execute-result-release", &warm);
    println!(
        "ar0027 prepared_observation={:?}",
        prepared.observe(SOURCE_ID).unwrap()
    );
}

#[test]
#[ignore = "manual release compile-once fresh-source control; phase clocks add overhead"]
fn measure_direct_lifecycle_fresh_sources() {
    const JOBS: usize = 5000;
    let program = compile_resource(&snapshot(SOURCE_ID), STYLE_ID).unwrap();
    // Identity creation and the report collection are outside per-job phase clocks.
    // Equal input bytes remain distinct document origins; only one is prepared at once.
    let identities: Vec<_> = (0..JOBS)
        .map(|index| {
            (
                format!("urn:ar0027:fresh:{index:05}"),
                format!("ar0027-job-{index:05}"),
            )
        })
        .collect();
    for (source_id, request_id) in identities.iter().take(32) {
        fresh_source(&program, source_id, request_id);
    }
    let mut samples = Vec::with_capacity(JOBS);
    let start = Instant::now();
    for (source_id, request_id) in &identities {
        samples.push(fresh_source(&program, source_id, request_id));
    }
    let elapsed = start.elapsed();
    report("compile-once-fresh-source-result-release", &samples);
    println!(
        "ar0027 jobs={JOBS} workers=1 source_bytes_per_job={} elapsed_ns={} jobs_per_second={:.0} prepared_documents_high_water=1",
        SOURCE.len(),
        elapsed.as_nanos(),
        f64::from(u32::try_from(JOBS).unwrap()) / elapsed.as_secs_f64()
    );
}
