//! Executable QT3 `fn:codepoint-equal` tranche.

use std::{collections::BTreeSet, fs, path::PathBuf};

use super::case_conversion_experiment::{CaseFailure, CaseValue, evaluate};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::qt3_overlay_test_support::{assert_private_case_passed, assert_selected_count};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

#[test]
fn executes_qt3_codepoint_equal_denominator_tranche() {
    let set_file = "fn/codepoint-equal.xml";
    let mut selected = vec![
        "fn-codepoint-equal-1".to_owned(),
        "fn-codepoint-equal-2".to_owned(),
        "fn-codepoint-equal-2a".to_owned(),
        "fn-codepoint-equal-2b".to_owned(),
    ];
    selected.extend((3..=21).map(|suffix| format!("fn-codepoint-equal-{suffix}")));
    assert_eq!(selected.len(), 23);
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
            .expect("selected QT3 codepoint-equal case");
        let source = child_named(&document, case, "test")
            .map(|test| document.string_value(test).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&document, case, "result").expect("QT3 result metadata");
        let mut control = InvocationControl::unbounded();

        match evaluate(&source, &mut control) {
            Ok(actual) => assert_native_result(&document, result, &actual, &case_name, &source),
            Err(CaseFailure::InvalidArity) => {
                assert_expected_error(&document, result, "XPST0017");
            }
            Err(CaseFailure::InvalidArgumentType) => {
                assert_expected_error(&document, result, "XPTY0004");
            }
            Err(failure) => {
                panic!("selected QT3 expression failed: {case_name}: {source}: {failure:?}")
            }
        }
        assert!(control.consumed(WorkDomain::XPathOperation) > 0);
    }
}

fn assert_native_result(
    document: &Document,
    result: NodeId,
    actual: &CaseValue,
    case_name: &str,
    source: &str,
) {
    for (assertion, expected) in [
        ("assert-true", CaseValue::Boolean(true)),
        ("assert-false", CaseValue::Boolean(false)),
        ("assert-empty", CaseValue::Empty),
    ] {
        if !descendants_named(document, result, assertion).is_empty() {
            assert_eq!(actual, &expected, "{case_name}: {source}");
            return;
        }
    }
    panic!("selected case lacks an admitted assertion: {case_name}");
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
    let bytes = fs::read(path).expect("read pinned QT3 codepoint-equal test set");
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{set_file}"),
        &bytes,
        ParseLimits {
            max_events: 8_192,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 codepoint-equal test set");
    Document::from_parsed(parsed).expect("build pinned QT3 codepoint-equal test set")
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
