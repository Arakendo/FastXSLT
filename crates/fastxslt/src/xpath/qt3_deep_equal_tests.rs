//! Executable QT3 `fn-deep-equalint2args` typed-integer group.

use std::{fs, path::PathBuf};

use super::deep_equal_experiment::{evaluate, parse};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "fn/deep-equal.xml";
const INT_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalint2args-1", true),
    ("fn-deep-equalint2args-2", false),
    ("fn-deep-equalint2args-3", false),
    ("fn-deep-equalint2args-4", false),
    ("fn-deep-equalint2args-5", false),
];
const INTEGER_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalintg2args-1", true),
    ("fn-deep-equalintg2args-2", false),
    ("fn-deep-equalintg2args-3", false),
    ("fn-deep-equalintg2args-4", false),
    ("fn-deep-equalintg2args-5", false),
];

#[test]
fn executes_complete_qt3_deep_equal_xs_int_group() {
    execute_group("fn-deep-equalint2args-", &INT_CASES);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_integer_group() {
    execute_group("fn-deep-equalintg2args-", &INTEGER_CASES);
}

fn execute_group(prefix: &str, expected_cases: &[(&str, bool)]) {
    let overlay = include_str!("../../../../corpus/overlays/qt3/private-ledger-v0.toml");
    assert_eq!(
        overlay.matches(&format!("case_name = \"{prefix}")).count(),
        expected_cases.len()
    );
    let test_set = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .filter(|node| {
            attribute(&test_set, *node, "name").is_some_and(|name| name.starts_with(prefix))
        })
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), expected_cases.len());

    for (name, expected) in expected_cases.iter().copied() {
        let record = overlay_case(overlay, name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains("execution = \"passed\""));
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned QT3 deep-equal case");
        let expression = child_named(&test_set, case, "test")
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&test_set, case, "result")
            .and_then(|node| first_element_child(&test_set, node))
            .expect("QT3 boolean assertion");
        assert_eq!(
            local_name(&test_set, result),
            if expected {
                "assert-true"
            } else {
                "assert-false"
            }
        );

        let parsed = parse(
            &expression,
            &SourceLocation {
                resource: format!("urn:w3c:qt3:{name}:expression"),
                span: 0..expression.len(),
            },
        )
        .expect("parse admitted typed deep-equal expression");
        let mut control = InvocationControl::unbounded();
        let actual = evaluate(&parsed, None, &mut control).expect("evaluate typed deep-equal");
        assert_eq!(actual, expected, "native QT3 assertion for {name}");
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 1);
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 0);
    }
}

fn load_test_set() -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(SET_FILE);
    let bytes = fs::read(path).expect("read pinned QT3 deep-equal test set");
    let parsed = parse_document(
        "urn:w3c:qt3:fn-deep-equal:test-set",
        &bytes,
        ParseLimits {
            max_events: 30_000,
            max_depth: 64,
        },
    )
    .expect("parse QT3 deep-equal test set");
    Document::from_parsed(parsed).expect("build QT3 deep-equal catalog document")
}

fn overlay_case<'a>(overlay: &'a str, name: &str) -> &'a str {
    overlay
        .split("[[case]]")
        .find(|section| {
            section.contains(&format!("set_file = \"{SET_FILE}\""))
                && section.contains(&format!("case_name = \"{name}\""))
        })
        .expect("overlay must contain one typed deep-equal disposition")
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}

fn descendants_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in document.children(parent).iter().copied() {
        if document.kind(child) != NodeKind::Element {
            continue;
        }
        if local_name(document, child) == local {
            found.push(child);
        }
        found.extend(descendants_named(document, child, local));
    }
    found
}

fn child_named(document: &Document, parent: NodeId, local: &str) -> Option<NodeId> {
    document.children(parent).iter().copied().find(|node| {
        document.kind(*node) == NodeKind::Element && local_name(document, *node) == local
    })
}

fn first_element_child(document: &Document, parent: NodeId) -> Option<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .find(|node| document.kind(*node) == NodeKind::Element)
}

fn local_name(document: &Document, node: NodeId) -> &str {
    &document.name(node).expect("element node has a name").local
}
