//! Compile-once ingestion with distinct payloads, not repeated prepared reuse.

use std::{fmt::Write, hint::black_box, time::Instant};

use crate::execution_control_experiment::InvocationControl;
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::runtime::prepared_input_experiment::{PreparedInputBuilder, PreparedInputSet};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

struct Job {
    identity: String,
    request: String,
    source: Vec<u8>,
    expected: String,
    items: usize,
}

fn job(index: usize, order: usize) -> Job {
    let items = [5, 50, 500][(index + order) % 3];
    let mut xml = format!("<?xml version=\"1.0\"?><order job=\"{index}\">");
    let mut sum = 0;
    for item in 0..items {
        let qty = (index + item) % 5 + 1;
        write!(xml, "<order-item price=\"1.00\" qty=\"{qty}\"/>").unwrap();
        sum += qty;
    }
    xml.push_str("</order>");
    Job {
        identity: format!("urn:ar0027:distinct:{index}"),
        request: format!("ar0027-distinct-{index}"),
        source: xml.into_bytes(),
        expected: format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>{sum}.00</out>"),
        items,
    }
}

fn prepare(job: &Job) -> PreparedInputSet {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
    resources.admit(&job.identity, job.source.clone()).unwrap();
    let mut builder = PreparedInputBuilder::new(resources.seal());
    builder
        .prepare(&job.identity, &mut InvocationControl::unbounded())
        .unwrap();
    builder.seal()
}

fn transform(program: &StylesheetProgram, prepared: &PreparedInputSet, job: &Job) -> String {
    let document = prepared.get(&job.identity).unwrap();
    let mut control = InvocationControl::unbounded();
    let result = execute_program(program, &document, &job.request, &mut control).unwrap();
    serialize_xml(
        &result,
        &program.output,
        &job.request,
        1_048_576,
        &mut control,
    )
    .unwrap()
}

fn sample(program: &StylesheetProgram, job: &Job) -> [u128; 4] {
    let total = Instant::now();
    let phase = Instant::now();
    let prepared = prepare(job);
    let preparation = phase.elapsed().as_nanos();
    let phase = Instant::now();
    let output = transform(program, &prepared, job);
    assert_eq!(output, job.expected);
    black_box(&output);
    let execution = phase.elapsed().as_nanos();
    let phase = Instant::now();
    drop(output);
    drop(prepared);
    let release = phase.elapsed().as_nanos();
    [preparation, execution, release, total.elapsed().as_nanos()]
}

#[test]
fn distinct_payloads_match_fresh_compilation_and_preserve_origin_and_retirement() {
    let program = compile_resource(&super::snapshot(super::SOURCE_ID), super::STYLE_ID).unwrap();
    for index in 0..12 {
        let job = job(index, 0);
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 1_048_576, 2_097_152));
        resources
            .admit(super::STYLE_ID, super::STYLE.to_vec())
            .unwrap();
        resources.admit(&job.identity, job.source.clone()).unwrap();
        let fresh = compile_resource(&resources.seal(), super::STYLE_ID).unwrap();
        let prepared = prepare(&job);
        assert_eq!(transform(&fresh, &prepared, &job), job.expected);
        assert_eq!(transform(&program, &prepared, &job), job.expected);
        let next = job_for_overlap(index + 12);
        let other = prepare(&next);
        let first_document = prepared.get(&job.identity).unwrap();
        let other_document = other.get(&next.identity).unwrap();
        assert!(!first_document.same_origin(&other_document));
        assert_eq!(
            first_document
                .location(first_document.document_node())
                .resource,
            job.identity
        );
        assert_eq!(
            other_document
                .location(other_document.document_node())
                .resource,
            next.identity
        );
        drop(first_document);
        drop(prepared);
        assert_eq!(transform(&program, &other, &next), next.expected);
        sample(&program, &job);
    }
}

// Avoid shadowing the job constructor within the control's lexical scope.
fn job_for_overlap(index: usize) -> Job {
    job(index, 0)
}

#[test]
#[ignore = "manual release compile-once distinct-payload ingestion; phase clocks add overhead"]
fn measure_distinct_payload_ingestion() {
    let order = std::env::var("AR0027_INGESTION_ORDER").map_or(0, |v| v.parse::<usize>().unwrap());
    assert!(order <= 2);
    let program = compile_resource(&super::snapshot(super::SOURCE_ID), super::STYLE_ID).unwrap();
    for index in 0..32 {
        sample(&program, &job(index, order));
    }
    let mut groups: [Vec<[u128; 4]>; 3] = std::array::from_fn(|_| Vec::new());
    let mut bytes = 0;
    for index in 0..5000 {
        // Host fixture/identity construction is outside each job's phase clocks.
        let job = job(index, order);
        bytes += job.source.len();
        let group = [5, 50, 500]
            .iter()
            .position(|items| *items == job.items)
            .unwrap();
        groups[group].push(sample(&program, &job));
    }
    for (group, samples) in groups.iter().enumerate() {
        for (phase, name) in [
            "admit-prepare",
            "execute-serialize-validate",
            "release",
            "total",
        ]
        .into_iter()
        .enumerate()
        {
            let mut values: Vec<_> = samples.iter().map(|sample| sample[phase]).collect();
            values.sort_unstable();
            let n = values.len();
            println!(
                "ar0027-distinct order={order} items={} samples={n} phase={name} median_ns={} p95_ns={} min_ns={} max_ns={}",
                [5, 50, 500][group],
                u128::midpoint(values[(n - 1) / 2], values[n / 2]),
                values[(95 * n).div_ceil(100) - 1],
                values[0],
                values[n - 1]
            );
        }
    }
    println!(
        "ar0027-distinct order={order} jobs=5000 exact_outputs=5000 admitted_source_bytes={bytes} compiled_programs=1 prepared_documents_at_once=1 publication_eligible=false"
    );
}
