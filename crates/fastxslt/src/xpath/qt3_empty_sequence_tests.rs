//! Executable source-free tranche from the complete QT3 `fn:empty` set.

use std::{collections::BTreeSet, fs, path::PathBuf};

use super::qt3_production_path_test_support::{compile_expression, execute_expression};
use crate::qt3_overlay_test_support::{assert_private_case_passed, assert_selected_count};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "fn/empty.xml";
const NUMERIC_STEMS: [&str; 13] = [
    "fn-emptyint1args",
    "fn-emptyintg1args",
    "fn-emptydec1args",
    "fn-emptydbl1args",
    "fn-emptyflt1args",
    "fn-emptylng1args",
    "fn-emptyusht1args",
    "fn-emptynint1args",
    "fn-emptypint1args",
    "fn-emptyulng1args",
    "fn-emptynpi1args",
    "fn-emptynni1args",
    "fn-emptysht1args",
];
const EXISTS_SET_FILE: &str = "fn/exists.xml";
const EXISTS_NUMERIC_STEMS: [&str; 13] = [
    "fn-existsint1args",
    "fn-existsintg1args",
    "fn-existsdec1args",
    "fn-existsdbl1args",
    "fn-existsflt1args",
    "fn-existslng1args",
    "fn-existsusht1args",
    "fn-existsnint1args",
    "fn-existspint1args",
    "fn-existsulng1args",
    "fn-existsnpi1args",
    "fn-existsnni1args",
    "fn-existssht1args",
];

#[test]
fn executes_qt3_source_free_empty_sequence_tranche() {
    let selected = selected_case_names();
    assert_eq!(selected.len(), 47);
    assert_selected_count(SET_FILE, selected.len());
    let document = load_test_set(SET_FILE);
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
        assert_private_case_passed(SET_FILE, &case_name);
        execute_case(&document, &case_name);
    }
}

#[test]
fn executes_qt3_source_free_exists_sequence_tranche() {
    let mut selected = EXISTS_NUMERIC_STEMS
        .into_iter()
        .flat_map(|stem| (1..=3).map(move |suffix| format!("{stem}-{suffix}")))
        .collect::<Vec<_>>();
    selected.extend((1..=9).map(|suffix| format!("K-SeqExistsFunc-{suffix}")));
    assert_eq!(selected.len(), 48);
    assert_selected_count(EXISTS_SET_FILE, selected.len());
    let document = load_test_set(EXISTS_SET_FILE);
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
        assert_private_case_passed(EXISTS_SET_FILE, &case_name);
        execute_case(&document, &case_name);
    }
}

fn selected_case_names() -> Vec<String> {
    let mut selected = NUMERIC_STEMS
        .into_iter()
        .flat_map(|stem| (1..=3).map(move |suffix| format!("{stem}-{suffix}")))
        .collect::<Vec<_>>();
    selected.extend((1..=8).map(|suffix| format!("K-SeqEmptyFunc-{suffix}")));
    selected
}

fn execute_case(document: &Document, case_name: &str) {
    let case = descendants_named(document, document.document_node(), "test-case")
        .into_iter()
        .find(|case| attribute(document, *case, "name") == Some(case_name))
        .expect("selected QT3 case");
    let test = child_named(document, case, "test").expect("test expression");
    let source = document.string_value(test).trim().to_owned();
    let result = child_named(document, case, "result").expect("result metadata");
    match compile_expression(case_name, &source) {
        Err(failure) => {
            let error = descendants_named(document, result, "error")
                .into_iter()
                .next()
                .expect("invalid-arity case must expect an error");
            assert_eq!(attribute(document, error, "code"), Some(failure.code));
        }
        Ok(program) => {
            let actual = execute_expression(&program, case_name);
            let expected = expected_boolean(document, result)
                .unwrap_or_else(|| panic!("selected case lacks a boolean assertion: {case_name}"));
            assert_eq!(actual, expected.to_string(), "{case_name}: {source}");
        }
    }
}

fn expected_boolean(document: &Document, result: NodeId) -> Option<bool> {
    if !descendants_named(document, result, "assert-true").is_empty() {
        Some(true)
    } else if !descendants_named(document, result, "assert-false").is_empty() {
        Some(false)
    } else {
        None
    }
}

fn load_test_set(set_file: &str) -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(set_file);
    let bytes = fs::read(path).expect("read pinned QT3 cardinality test set");
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{set_file}"),
        &bytes,
        ParseLimits {
            max_events: 8_192,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 cardinality test set");
    Document::from_parsed(parsed).expect("build pinned QT3 cardinality test set")
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
