//! Result-heavy compile-once ingestion and prepared reuse control.

use std::{fmt::Write, hint::black_box, time::Instant};

use crate::execution_control_experiment::InvocationControl;
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{compile_resource, execute_program, serialize_xml};
use crate::runtime::prepared_input_experiment::{PreparedInputBuilder, PreparedInputSet};
use crate::xml::quick_xml_experiment::ParseLimits;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

const STYLE_ID: &str = "urn:ar0027:copy-style";

#[cfg(feature = "allocation-observation")]
#[path = "result_heavy_allocation_tests.rs"]
mod allocation_tests;

#[path = "result_heavy_capacity_tests.rs"]
mod capacity_tests;

#[path = "prepared_set_capacity_tests.rs"]
mod prepared_set_capacity_tests;

fn program() -> StylesheetProgram {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit(STYLE_ID, br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:copy-of select="/root"/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    compile_resource(&resources.seal(), STYLE_ID).unwrap()
}

struct Input {
    identity: String,
    source: Vec<u8>,
    expected: String,
}

fn input(items: usize, job: usize) -> Input {
    let mut body = format!("<root job=\"{job}\">");
    for index in 0..items {
        write!(
            body,
            "<item n=\"{index}\">{}&amp;&lt;</item>",
            "x".repeat(512)
        )
        .unwrap();
    }
    body.push_str("</root>");
    Input {
        identity: format!("urn:ar0027:copy:{items}:{job}"),
        source: format!("<?xml version=\"1.0\"?>{body}").into_bytes(),
        expected: format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>{body}"),
    }
}

fn prepare(input: &Input) -> PreparedInputSet {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
    resources
        .admit(&input.identity, input.source.clone())
        .unwrap();
    let mut builder = PreparedInputBuilder::with_parse_limits(
        resources.seal(),
        ParseLimits {
            max_events: 10_000,
            max_depth: 64,
        },
    );
    builder
        .prepare(&input.identity, &mut InvocationControl::unbounded())
        .unwrap();
    builder.seal()
}

fn sample(program: &StylesheetProgram, input: &Input, reuses: usize) -> [u128; 5] {
    let total = Instant::now();
    let phase = Instant::now();
    let prepared = prepare(input);
    let preparation = phase.elapsed().as_nanos();
    let mut execution = 0;
    let mut serialization = 0;
    let mut release = 0;
    for _ in 0..reuses {
        let document = prepared.get(&input.identity).unwrap();
        let mut control = InvocationControl::unbounded();
        let phase = Instant::now();
        let result = execute_program(program, &document, "ar0027-copy", &mut control).unwrap();
        execution += phase.elapsed().as_nanos();
        let phase = Instant::now();
        let output = serialize_xml(
            &result,
            &program.output,
            "ar0027-copy",
            1_048_576,
            &mut control,
        )
        .unwrap();
        assert_eq!(output, input.expected);
        black_box(&output);
        serialization += phase.elapsed().as_nanos();
        let phase = Instant::now();
        drop(output);
        drop(result);
        drop(document);
        release += phase.elapsed().as_nanos();
    }
    let phase = Instant::now();
    drop(prepared);
    release += phase.elapsed().as_nanos();
    [
        preparation,
        execution,
        serialization,
        release,
        total.elapsed().as_nanos(),
    ]
}

#[test]
fn copied_result_outlives_distinct_source_owners_and_reuse_matches() {
    let program = program();
    for items in [5, 50, 500] {
        for job in [0, 1] {
            let input = input(items, job);
            for reuses in [1, 8] {
                sample(&program, &input, reuses);
            }
            let prepared = prepare(&input);
            let document = prepared.get(&input.identity).unwrap();
            let mut control = InvocationControl::unbounded();
            let result =
                execute_program(&program, &document, "retire-source", &mut control).unwrap();
            drop(document);
            drop(prepared);
            assert_eq!(
                serialize_xml(
                    &result,
                    &program.output,
                    "retire-source",
                    1_048_576,
                    &mut control
                )
                .unwrap(),
                input.expected
            );
        }
    }
}

#[test]
#[ignore = "manual release result-heavy fresh-source and prepared-reuse phase control"]
fn measure_result_heavy_ingestion() {
    let order = std::env::var("AR0027_RESULT_ORDER").map_or(0, |v| v.parse::<usize>().unwrap());
    assert!(order <= 2);
    let program = program();
    for group in 0..3 {
        let items = [5, 50, 500][(group + order) % 3];
        for reuses in [1, 8] {
            for index in 0..8 {
                sample(&program, &input(items, index), reuses);
            }
            let samples: Vec<_> = (0..32)
                .map(|index| sample(&program, &input(items, index + 8), reuses))
                .collect();
            for (phase, name) in [
                "admit-prepare",
                "execution",
                "serialize-validate",
                "release",
                "total",
            ]
            .into_iter()
            .enumerate()
            {
                let mut values: Vec<_> = samples.iter().map(|s| s[phase]).collect();
                values.sort_unstable();
                println!(
                    "ar0027-result order={order} items={items} uses={reuses} samples=32 phase={name} median_ns={} p95_ns={}",
                    u128::midpoint(values[15], values[16]),
                    values[30]
                );
            }
            println!(
                "ar0027-result items={items} uses={reuses} output_bytes={} timed_exact_outputs={} publication_eligible=false",
                input(items, 8).expected.len(),
                32 * reuses
            );
        }
    }
}
