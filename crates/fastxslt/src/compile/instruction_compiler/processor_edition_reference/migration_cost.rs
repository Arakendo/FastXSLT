//! Fixed R0 P07 cost/control comparison, not a host throughput benchmark.
use super::processor_edition_reference::{Edition, lower_module};
use super::*;
use crate::compile::golden_stylesheet_experiment as compiler;
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use compiler::stylesheet_preprocessing::StrictPreprocessingFailure;
use std::{hint::black_box, time::Instant};

type Lower =
    fn(&Document, &mut InvocationControl) -> Result<StylesheetProgram, StrictPreprocessingFailure>;

fn document(xml: &str) -> Document {
    Document::from_parsed(
        parse_document(
            "urn:r0-cost",
            xml.as_bytes(),
            ParseLimits {
                max_events: 1000,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
#[ignore = "manual release-mode R0 compile/warm cost probe"]
fn r0_selected_migration_cost_probe() {
    assert!(!black_box(cfg!(debug_assertions)), "run with --release");
    let source = document("<r/>");
    let lowers: [(&str, Lower); 2] = [
        ("root-reference", |sheet, control| {
            lower_module(sheet, Edition::Legacy, control)
        }),
        ("selected-candidate", |sheet, control| {
            compile_selected_legacy_import_module(sheet, Vec::new(), control, true)
        }),
    ];
    println!(
        "R0_COST_BEGIN\ncompile=preparsed-module+strict-preprocessing+lowering+drop; warm=shared-execution+serialization; acquisition/XML-parse excluded"
    );
    for (fixture, body, expected) in [
        ("text", "<xsl:text>ok</xsl:text>", "ok"),
        (
            "fallback",
            "<xsl:sequence select='1'><xsl:fallback><xsl:text>ok</xsl:text></xsl:fallback></xsl:sequence>",
            "ok",
        ),
        (
            "conditional",
            "<xsl:if test='false()'><xsl:foo/></xsl:if><xsl:text>ok</xsl:text>",
            "ok",
        ),
    ] {
        let sheet = document(&format!(
            "<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform' version='2.0'><xsl:template match='/'>{body}</xsl:template></xsl:stylesheet>"
        ));
        let programs: Vec<_> = lowers
            .iter()
            .map(|(lane, lower)| {
                let mut control = InvocationControl::unbounded();
                let program = lower(&sheet, &mut control).unwrap();
                assert_eq!(
                    execute_compiled_stylesheet_for_test(&program, &source, "cost").unwrap(),
                    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>{expected}")
                );
                println!(
                    "charge\t{fixture}\t{lane}\txdm-node\t{}",
                    control.consumed(WorkDomain::XdmNode)
                );
                program
            })
            .collect();
        for (_, lower) in lowers {
            for _ in 0..100 {
                black_box(lower(black_box(&sheet), &mut InvocationControl::unbounded()).unwrap());
            }
        }
        for round in 0..5 {
            for lane in [round % 2, (round + 1) % 2] {
                let (name, lower) = lowers[lane];
                let started = Instant::now();
                for _ in 0..2000 {
                    black_box(
                        lower(black_box(&sheet), &mut InvocationControl::unbounded()).unwrap(),
                    );
                }
                let compile_ns = started.elapsed().as_nanos() / 2000;
                let started = Instant::now();
                for _ in 0..5000 {
                    black_box(
                        execute_compiled_stylesheet_for_test(
                            black_box(&programs[lane]),
                            black_box(&source),
                            "cost",
                        )
                        .unwrap(),
                    );
                }
                let warm_ns = started.elapsed().as_nanos() / 5000;
                println!(
                    "sample\t{fixture}\t{round}\t{name}\tcompile-ns\t{compile_ns}\twarm-ns\t{warm_ns}"
                );
            }
        }
    }
    println!("R0_COST_END");
}
