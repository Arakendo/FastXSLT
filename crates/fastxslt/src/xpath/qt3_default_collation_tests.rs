//! Executable QT3 `fn:default-collation` denominator.

use std::{collections::BTreeSet, fs, path::PathBuf};

use crate::qt3_overlay_test_support::{assert_private_case_passed, assert_selected_count};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

#[test]
fn executes_complete_qt3_default_collation_denominator() {
    let set_file = "fn/default-collation.xml";
    let selected = [
        "fn-default-collation-1",
        "fn-default-collation-2",
        "K-ContextDefaultCollationFunc-1",
        "K-ContextDefaultCollationFunc-2",
        "K-ContextDefaultCollationFunc-3",
        "cbcl-default-collation-001",
        "cbcl-default-collation-002",
    ];
    assert_selected_count(set_file, selected.len());

    let document = load_test_set(set_file);
    let catalog_names = descendants_named(&document, document.document_node(), "test-case")
        .into_iter()
        .map(|case| {
            attribute(&document, case, "name")
                .expect("case name")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();

    for case_name in selected {
        assert!(catalog_names.contains(case_name), "{case_name}");
        assert_private_case_passed(set_file, case_name);
        let case = descendants_named(&document, document.document_node(), "test-case")
            .into_iter()
            .find(|case| attribute(&document, *case, "name") == Some(case_name))
            .expect("selected QT3 default-collation case");
        let source = child_named(&document, case, "test")
            .map(|test| document.string_value(test).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&document, case, "result").expect("QT3 result metadata");
        let compiled = compile_production_expression(case_name, &source);
        if expects_error(&document, result, "XPST0017") {
            let failure = compiled.expect_err("invalid arity must fail production compilation");
            assert_eq!(failure.code, "XPST0017", "{case_name}: {source}");
        } else {
            let program = compiled.unwrap_or_else(|failure| {
                panic!("production compilation failed: {case_name}: {source}: {failure:?}")
            });
            let source_document = parse_source(case_name);
            let actual = execute_compiled_stylesheet_for_test(
                &program,
                &source_document,
                &format!("qt3:{case_name}"),
            )
            .unwrap_or_else(|failure| {
                panic!("production execution failed: {case_name}: {source}: {failure}")
            });
            assert_native_result(&document, result, &actual, case_name, &source);
        }
    }
}

fn compile_production_expression(
    case_name: &str,
    expression: &str,
) -> Result<
    crate::xslt::golden_semantics_experiment::StylesheetProgram,
    crate::compile::golden_stylesheet_experiment::CompileFailure,
> {
    let select = expression
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;");
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
    crate::compile::golden_stylesheet_experiment::compile_stylesheet(&document)
}

fn parse_source(case_name: &str) -> Document {
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{case_name}:source"),
        b"<source/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("production-path source should parse");
    Document::from_parsed(parsed).expect("production-path source XDM should build")
}

fn assert_native_result(
    document: &Document,
    result: NodeId,
    actual: &str,
    case_name: &str,
    source: &str,
) {
    if !descendants_named(document, result, "assert-true").is_empty() {
        assert_eq!(actual, "true", "{case_name}: {source}");
        return;
    }
    let assertion = descendants_named(document, result, "assert-string-value")
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("selected case lacks an admitted assertion: {case_name}"));
    let expected = document.string_value(assertion);
    assert_eq!(actual, expected, "{case_name}: {source}");
}

fn expects_error(document: &Document, result: NodeId, expected_code: &str) -> bool {
    descendants_named(document, result, "error")
        .into_iter()
        .next()
        .is_some_and(|error| attribute(document, error, "code") == Some(expected_code))
}

fn load_test_set(set_file: &str) -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(set_file);
    let bytes = fs::read(path).expect("read pinned QT3 default-collation test set");
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{set_file}"),
        &bytes,
        ParseLimits {
            max_events: 4_096,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 default-collation test set");
    Document::from_parsed(parsed).expect("build pinned QT3 default-collation test set")
}

fn child_named(document: &Document, parent: NodeId, local: &str) -> Option<NodeId> {
    document.children(parent).iter().copied().find(|child| {
        document.kind(*child) == NodeKind::Element
            && document
                .name(*child)
                .is_some_and(|name| name.local == local)
    })
}

fn descendants_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in document.children(parent).iter().copied() {
        if document.kind(child) != NodeKind::Element {
            continue;
        }
        if document.name(child).is_some_and(|name| name.local == local) {
            found.push(child);
        }
        found.extend(descendants_named(document, child, local));
    }
    found
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}
