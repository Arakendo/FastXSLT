//! Manual lifecycle measurements over executable pinned standards workloads.

use std::{hint::black_box, time::Instant};

use crate::execution_control_experiment::InvocationControl;
use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document;

use super::{PREPARATION_XML_LIMITS, PreparedInputBuilder, PreparedInputObservation};

const ITERATIONS: usize = 1_000;
const SAMPLES: usize = 7;

struct StandardsWorkload {
    name: &'static str,
    source_id: &'static str,
    stylesheet_id: &'static str,
    source: &'static [u8],
    stylesheet: &'static [u8],
}

const WORKLOADS: [StandardsWorkload; 2] = [
    StandardsWorkload {
        name: "xslt30-for-004",
        source_id: "urn:w3c:xslt30:for-004:source",
        stylesheet_id: "urn:w3c:xslt30:for-004:stylesheet",
        source: include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for03.xml"),
        stylesheet: include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl"),
    },
    StandardsWorkload {
        name: "xslt30-castable-004",
        source_id: "urn:w3c:xslt30:castable-004:source",
        stylesheet_id: "urn:w3c:xslt30:castable-004:stylesheet",
        source: include_bytes!("../../../../vendor/xslt30-test/tests/expr/castable/castbl01.xml"),
        stylesheet: include_bytes!(
            "../../../../vendor/xslt30-test/tests/expr/castable/castable-004.xsl"
        ),
    },
];

#[test]
#[ignore = "manual release-mode representative standards lifecycle probe"]
#[allow(
    clippy::too_many_lines,
    reason = "keeping phase setup, semantic conservation, timing, and reporting together makes the manual probe auditable"
)]
fn measures_representative_standards_lifecycle() {
    for workload in &WORKLOADS {
        let snapshot = workload_snapshot(workload);
        let program = compile_resource(&snapshot, workload.stylesheet_id)
            .expect("compile representative stylesheet");
        let mut prepared_builder = PreparedInputBuilder::new(snapshot.clone());
        prepared_builder
            .prepare(workload.source_id, &mut InvocationControl::unbounded())
            .expect("prepare representative source");
        let prepared = prepared_builder.seal();
        let observation = prepared
            .observe(workload.source_id)
            .expect("observe representative prepared source");
        let prepared_document = prepared
            .get(workload.source_id)
            .expect("get representative prepared source");

        let reference_document = build_document(&snapshot, workload.source_id);
        let direct_result = run(&program, &reference_document, "direct-reference");
        let prepared_result = run(&program, &prepared_document, "prepared-reference");
        assert_eq!(direct_result, prepared_result);

        let parse_ns = median_ns(|| {
            black_box(
                parse_document(
                    workload.source_id,
                    black_box(
                        snapshot
                            .get(workload.source_id)
                            .expect("representative source bytes"),
                    ),
                    PREPARATION_XML_LIMITS,
                )
                .expect("parse representative source"),
            );
        });

        let xdm_ns = median_xdm_ns(&snapshot, workload.source_id);

        let compile_ns = median_ns(|| {
            black_box(
                compile_resource(black_box(&snapshot), workload.stylesheet_id)
                    .expect("compile measured representative stylesheet"),
            );
        });

        let compiled_direct_ns = median_ns(|| {
            let document = build_document(&snapshot, workload.source_id);
            black_box(run(&program, &document, "compiled-direct"));
        });

        let compiled_prepared_ns = median_ns(|| {
            let document = prepared
                .get(workload.source_id)
                .expect("lookup representative prepared source");
            black_box(run(&program, &document, "compiled-prepared"));
        });

        let compile_each_ns = median_ns(|| {
            let iteration_program = compile_resource(&snapshot, workload.stylesheet_id)
                .expect("compile representative stylesheet per invocation");
            let document = build_document(&snapshot, workload.source_id);
            black_box(run(&iteration_program, &document, "compile-each"));
        });

        report(
            workload,
            observation,
            parse_ns,
            xdm_ns,
            compile_ns,
            compiled_direct_ns,
            compiled_prepared_ns,
            compile_each_ns,
        );
    }
}

fn workload_snapshot(workload: &StandardsWorkload) -> ResourceSnapshot {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(workload.source_id, workload.source.to_vec())
        .expect("admit representative source");
    resources
        .admit(workload.stylesheet_id, workload.stylesheet.to_vec())
        .expect("admit representative stylesheet");
    resources.seal()
}

fn build_document(snapshot: &ResourceSnapshot, identity: &str) -> Document {
    let parsed = parse_document(
        identity,
        snapshot.get(identity).expect("representative source bytes"),
        PREPARATION_XML_LIMITS,
    )
    .expect("parse representative source");
    Document::from_parsed(parsed).expect("build representative XDM")
}

fn run(
    program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
    document: &Document,
    request_id: &str,
) -> String {
    let mut control = InvocationControl::unbounded();
    let semantic = execute_program(program, document, request_id, &mut control)
        .expect("execute representative transform");
    serialize_xml(&semantic, &program.output, request_id, 8_192, &mut control)
        .expect("serialize representative transform")
}

fn median_ns(mut operation: impl FnMut()) -> f64 {
    let mut observations = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        for iteration in 0..ITERATIONS {
            operation();
            black_box(iteration);
        }
        observations.push(
            start.elapsed().as_secs_f64() * 1_000_000_000.0
                / f64::from(u32::try_from(ITERATIONS).expect("iteration count fits u32")),
        );
    }
    observations.sort_by(f64::total_cmp);
    observations[SAMPLES / 2]
}

fn median_xdm_ns(snapshot: &ResourceSnapshot, identity: &str) -> f64 {
    let bytes = snapshot.get(identity).expect("representative source bytes");
    let mut observations = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let parsed_documents: Vec<_> = (0..ITERATIONS)
            .map(|_| {
                parse_document(identity, bytes, PREPARATION_XML_LIMITS)
                    .expect("prepare representative XDM timing input")
            })
            .collect();
        let start = Instant::now();
        for parsed in parsed_documents {
            black_box(Document::from_parsed(parsed).expect("build representative XDM"));
        }
        observations.push(
            start.elapsed().as_secs_f64() * 1_000_000_000.0
                / f64::from(u32::try_from(ITERATIONS).expect("iteration count fits u32")),
        );
    }
    observations.sort_by(f64::total_cmp);
    observations[SAMPLES / 2]
}

#[allow(
    clippy::too_many_arguments,
    reason = "one flat probe record is easier to capture"
)]
fn report(
    workload: &StandardsWorkload,
    observation: PreparedInputObservation,
    parse_ns: f64,
    xdm_ns: f64,
    compile_ns: f64,
    compiled_direct_ns: f64,
    compiled_prepared_ns: f64,
    compile_each_ns: f64,
) {
    println!(
        "workload={} iterations={ITERATIONS} samples={SAMPLES} source_bytes={} stylesheet_bytes={} parsed_capacity_bytes={} xdm_nodes={} xdm_capacity_bytes={} parse_ns={parse_ns:.1} xdm_construct_drop_ns={xdm_ns:.1} compile_ns={compile_ns:.1} compiled_direct_ns={compiled_direct_ns:.1} compiled_prepared_ns={compiled_prepared_ns:.1} prepared_ratio={:.2} compile_each_ns={compile_each_ns:.1} compiled_ratio={:.2}",
        workload.name,
        observation.raw_bytes,
        workload.stylesheet.len(),
        observation.parsed_phase_owned_capacity_bytes,
        observation.xdm_nodes,
        observation.xdm_owned_capacity_bytes,
        compiled_direct_ns / compiled_prepared_ns,
        compile_each_ns / compiled_direct_ns,
    );
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual allocator-requested representative preparation peak probe"]
fn measures_representative_standards_preparation_allocations() {
    for workload in &WORKLOADS {
        let snapshot = workload_snapshot(workload);
        let mut builder = PreparedInputBuilder::new(snapshot);
        let mut control = InvocationControl::unbounded();
        let allocations = allocation_counter::measure(|| {
            builder
                .prepare(workload.source_id, &mut control)
                .expect("prepare representative allocation workload");
        });
        let prepared = builder.seal();
        assert!(allocations.bytes_current > 0);
        let retained = u64::try_from(allocations.bytes_current)
            .expect("positive retained bytes fit the unsigned observation");
        assert!(allocations.bytes_max >= retained);
        println!(
            "workload={} representation={:?} allocator_requested={allocations:?}",
            workload.name,
            prepared
                .observe(workload.source_id)
                .expect("observe representative allocation workload")
        );
    }
}
