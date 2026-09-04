//! Executable QT3 `fn:iri-to-uri` tranche.

use std::{collections::BTreeSet, fs, path::PathBuf};

use super::qt3_production_path_test_support::{compile_expression, execute_expression};
use crate::qt3_overlay_test_support::{assert_private_case_passed, assert_selected_count};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

#[test]
fn executes_qt3_iri_to_uri_denominator_tranche() {
    let set_file = "fn/iri-to-uri.xml";
    let mut selected = (1..=6)
        .map(|suffix| format!("fn-iri-to-uri1args-{suffix}"))
        .collect::<Vec<_>>();
    selected.extend((1..=17).map(|suffix| format!("fn-iri-to-uri-{suffix}")));
    selected.push("fn-iri-to-uri-18A".to_owned());
    selected.extend((19..=26).map(|suffix| format!("fn-iri-to-uri-{suffix}")));
    selected.extend((1..=3).map(|suffix| format!("K-IRIToURIfunc-{suffix}")));
    selected.extend((1..=10).map(|suffix| format!("K2-IRIToURIfunc-{suffix}")));
    assert_eq!(selected.len(), 45);
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
        assert!(catalog_names.contains(&case_name), "{case_name}");
        assert_private_case_passed(set_file, &case_name);
        let case = descendants_named(&document, document.document_node(), "test-case")
            .into_iter()
            .find(|case| attribute(&document, *case, "name") == Some(&case_name))
            .expect("selected QT3 iri-to-uri case");
        let source = child_named(&document, case, "test")
            .map(|test| document.string_value(test).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&document, case, "result").expect("QT3 result metadata");
        let compiled = compile_expression(&case_name, &source);
        if let Some(expected_code) = expected_error(&document, result) {
            let failure =
                compiled.expect_err("invalid expression must fail production compilation");
            assert_eq!(failure.code, expected_code, "{case_name}: {source}");
        } else {
            let program = compiled.unwrap_or_else(|failure| {
                panic!("production compilation failed: {case_name}: {source}: {failure:?}")
            });
            let actual = execute_expression(&program, &case_name);
            assert_native_result(&document, result, &actual, &case_name, &source);
        }
    }
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
    assert_eq!(
        actual,
        document.string_value(assertion),
        "{case_name}: {source}"
    );
}

fn expected_error(document: &Document, result: NodeId) -> Option<&str> {
    descendants_named(document, result, "error")
        .into_iter()
        .next()
        .and_then(|error| attribute(document, error, "code"))
}

fn load_test_set(set_file: &str) -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(set_file);
    let bytes = fs::read(path).expect("read pinned QT3 iri-to-uri test set");
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{set_file}"),
        &bytes,
        ParseLimits {
            max_events: 16_384,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 iri-to-uri test set");
    Document::from_parsed(parsed).expect("build pinned QT3 iri-to-uri test set")
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
