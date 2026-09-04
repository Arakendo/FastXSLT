//! Test-only adapter from unchanged QT3 expressions to the production XSLT path.

use crate::compile::golden_stylesheet_experiment::{CompileFailure, compile_stylesheet};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

pub(crate) fn compile_expression(
    case_name: &str,
    expression: &str,
) -> Result<StylesheetProgram, CompileFailure> {
    let select = expression
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('\r', "&#xD;")
        .replace('\n', "&#xA;")
        .replace('\t', "&#x9;");
    let stylesheet = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="{select}"/></xsl:template></xsl:stylesheet>"#
    );
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{case_name}:stylesheet"),
        stylesheet.as_bytes(),
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("generated production-path stylesheet should parse");
    let document = Document::from_parsed(parsed).expect("generated stylesheet XDM should build");
    compile_stylesheet(&document)
}

pub(crate) fn execute_expression(program: &StylesheetProgram, case_name: &str) -> String {
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{case_name}:source"),
        b"<source/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("production-path source should parse");
    let source = Document::from_parsed(parsed).expect("production-path source XDM should build");
    execute_expression_with_source(program, &source, case_name)
        .unwrap_or_else(|failure| panic!("production execution failed: {case_name}: {failure}"))
}

pub(crate) fn execute_expression_with_source(
    program: &StylesheetProgram,
    source: &Document,
    case_name: &str,
) -> Result<String, String> {
    execute_compiled_stylesheet_for_test(program, source, &format!("qt3:{case_name}"))
}
