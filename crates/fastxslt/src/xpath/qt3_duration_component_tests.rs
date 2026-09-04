//! Executable QT3 duration-component denominators.

use std::{collections::BTreeSet, fs, path::PathBuf};

use super::qt3_production_path_test_support::{compile_expression, execute_expression};
use crate::qt3_overlay_test_support::{assert_private_case_passed, assert_selected_count};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

#[test]
fn executes_complete_qt3_years_from_duration_denominator() {
    execute_duration_denominator("years", "Years", 20);
}

#[test]
fn executes_complete_qt3_months_from_duration_denominator() {
    execute_duration_denominator("months", "Months", 20);
}

#[test]
fn executes_complete_qt3_days_from_duration_denominator() {
    execute_duration_denominator("days", "Days", 20);
}

#[test]
fn executes_complete_qt3_hours_from_duration_denominator() {
    execute_duration_denominator("hours", "Hours", 20);
}

#[test]
fn executes_complete_qt3_minutes_from_duration_denominator() {
    execute_duration_denominator("minutes", "Minutes", 21);
}

fn execute_duration_denominator(component: &str, component_title: &str, numbered_cases: usize) {
    let set_file = format!("fn/{component}-from-duration.xml");
    let mut selected = (1..=3)
        .map(|suffix| format!("fn-{component}-from-duration1args-{suffix}"))
        .collect::<Vec<_>>();
    selected.extend(
        (1..=numbered_cases).map(|suffix| format!("fn-{component}-from-duration-{suffix}")),
    );
    selected.extend((1..=7).map(|suffix| format!("K-{component_title}FromDurationFunc-{suffix}")));
    selected.push(format!("cbcl-{component}-from-duration-001"));
    assert_eq!(selected.len(), numbered_cases + 11);
    assert_selected_count(&set_file, selected.len());

    let document = load_test_set(&set_file);
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
        assert_private_case_passed(&set_file, &case_name);
        let case = descendants_named(&document, document.document_node(), "test-case")
            .into_iter()
            .find(|case| attribute(&document, *case, "name") == Some(&case_name))
            .expect("selected QT3 years-from-duration case");
        let source = child_named(&document, case, "test")
            .map(|test| document.string_value(test).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&document, case, "result").expect("QT3 result metadata");
        match compile_expression(&case_name, &source) {
            Ok(program) => {
                let actual = execute_expression(&program, &case_name);
                assert_serialized_result(&document, result, &actual, &case_name, &source);
            }
            Err(failure) => {
                assert_expected_error(&document, result, failure.code);
            }
        }
    }
}

fn assert_serialized_result(
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
    if !descendants_named(document, result, "assert-false").is_empty() {
        assert_eq!(actual, "false", "{case_name}: {source}");
        return;
    }
    if let Some(assertion) = descendants_named(document, result, "assert-eq")
        .into_iter()
        .next()
    {
        let expected = document.string_value(assertion);
        assert_eq!(actual, expected.trim(), "{case_name}: {source}");
        return;
    }
    let assertion = descendants_named(document, result, "assert-string-value")
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("selected case lacks an admitted assertion: {case_name}"));
    let expected = document.string_value(assertion);
    assert_eq!(actual, expected.trim(), "{case_name}: {source}");
}

fn assert_expected_error(document: &Document, result: NodeId, expected_code: &str) {
    let error = descendants_named(document, result, "error")
        .into_iter()
        .next()
        .expect("selected error case must own a native error assertion");
    assert_eq!(attribute(document, error, "code"), Some(expected_code));
}

fn load_test_set(set_file: &str) -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(set_file);
    let bytes = fs::read(path).expect("read pinned QT3 duration test set");
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{set_file}"),
        &bytes,
        ParseLimits {
            max_events: 16_384,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 duration test set");
    Document::from_parsed(parsed).expect("build pinned QT3 duration test set")
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
