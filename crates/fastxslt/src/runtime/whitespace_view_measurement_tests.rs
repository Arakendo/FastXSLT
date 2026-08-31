//! Manual AR-0016 complete-reference versus visibility-view measurements.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

use super::{MultipleMatchPolicy, WhitespaceRepresentation, execute_program_with_parameters_using};

#[test]
#[ignore = "manual release-mode complete-reference versus visibility-view probe"]
fn measures_whitespace_reference_against_visibility_view() {
    const ITEMS: usize = 500;
    const ITERATIONS: usize = 2_000;
    const ITERATIONS_F64: f64 = 2_000.0;
    const SAMPLES: usize = 7;
    let mut source_xml = String::from("<root>\n");
    for index in 0..ITEMS {
        writeln!(source_xml, "  <item>{index}</item>").expect("writing into a String cannot fail");
    }
    source_xml.push_str("\n</root>");
    let source = Document::from_parsed(
        parse_document(
            "memory:whitespace-measurement.xml",
            source_xml.as_bytes(),
            ParseLimits {
                max_events: 4_096,
                max_depth: 8,
            },
        )
        .expect("measurement source should parse"),
    )
    .expect("measurement source should prepare");
    let stylesheet = Document::from_parsed(
        parse_document(
            "memory:whitespace-measurement.xsl",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:strip-space elements="*"/><xsl:template match="/"><out><xsl:value-of select="."/></out></xsl:template></xsl:stylesheet>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("measurement stylesheet should parse"),
    )
    .expect("measurement stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("measurement stylesheet should compile");

    let run = |representation, request_id| {
        let mut control = InvocationControl::unbounded();
        execute_program_with_parameters_using(
            &program,
            &source,
            &BTreeMap::new(),
            MultipleMatchPolicy::UseLast,
            request_id,
            representation,
            &mut control,
        )
        .expect("measurement transform should execute")
    };
    assert_eq!(
        run(WhitespaceRepresentation::CompleteReference, "reference"),
        run(WhitespaceRepresentation::VisibilityView, "view")
    );

    let mut reference_control = InvocationControl::unbounded();
    let reference = source
        .derive_stripping_all_element_whitespace(&mut reference_control)
        .expect("measurement reference should derive");
    let mut view_control = InvocationControl::unbounded();
    let view = source
        .view_stripping_all_element_whitespace(&mut view_control)
        .expect("measurement view should derive");
    let reference_bytes = reference.owned_capacity_bytes();
    let view_bytes = view.exclusive_view_capacity_bytes();

    let mut reference_ns = Vec::with_capacity(SAMPLES);
    let mut view_ns = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let started = Instant::now();
        for iteration in 0..ITERATIONS {
            black_box(run(
                WhitespaceRepresentation::CompleteReference,
                black_box("reference-measurement"),
            ));
            black_box(iteration);
        }
        reference_ns.push(started.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);

        let started = Instant::now();
        for iteration in 0..ITERATIONS {
            black_box(run(
                WhitespaceRepresentation::VisibilityView,
                black_box("view-measurement"),
            ));
            black_box(iteration);
        }
        view_ns.push(started.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);
    }
    reference_ns.sort_by(f64::total_cmp);
    view_ns.sort_by(f64::total_cmp);
    println!(
        "items={ITEMS} iterations={ITERATIONS} samples={SAMPLES} reference_median_ns={:.1} view_median_ns={:.1} speedup={:.2} reference_owned_bytes={reference_bytes} view_additional_bytes={view_bytes}",
        reference_ns[SAMPLES / 2],
        view_ns[SAMPLES / 2],
        reference_ns[SAMPLES / 2] / view_ns[SAMPLES / 2]
    );
}
