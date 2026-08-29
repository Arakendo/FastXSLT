//! Executable metadata-driven `QT3` child- and attribute-axis slice.

use std::{fs, path::PathBuf};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

use super::count_experiment;

const QT3_NAMESPACE: &str = "http://www.w3.org/2010/09/qt-fots-catalog";
const CASES: [(&str, &str, usize); 45] = [
    ("Axes001-1", "fn:count(//center/child::*)", 0),
    ("Axes001-2", "fn:count(//center/child::*)", 1),
    ("Axes001-3", "fn:count(//center/child::*)", 6),
    ("Axes002-1", "fn:count(//center/child::south-east)", 0),
    ("Axes002-2", "fn:count(//center/child::south-east)", 0),
    ("Axes002-3", "fn:count(//center/child::south-east)", 1),
    ("Axes002-4", "fn:count(//center/child::south-east)", 2),
    ("Axes003-1", "fn:count(//center/child::node())", 0),
    ("Axes003-2", "fn:count(//center/child::node())", 1),
    ("Axes003-3", "fn:count(//center/child::node())", 1),
    ("Axes003-4", "fn:count(//center/child::node())", 19),
    ("Axes004-1", "fn:count(//center/*)", 0),
    ("Axes004-2", "fn:count(//center/*)", 1),
    ("Axes004-3", "fn:count(//center/*)", 6),
    ("Axes005-1", "fn:count(//center/south-east)", 0),
    ("Axes005-2", "fn:count(//center/south-east)", 0),
    ("Axes005-3", "fn:count(//center/south-east)", 1),
    ("Axes005-4", "fn:count(//center/south-east)", 2),
    ("Axes006-1", "fn:count(//center/node())", 0),
    ("Axes006-2", "fn:count(//center/node())", 1),
    ("Axes006-3", "fn:count(//center/node())", 1),
    ("Axes006-4", "fn:count(//center/node())", 19),
    ("Axes007-1", "fn:count(//west/attribute::*)", 0),
    ("Axes007-2", "fn:count(//west/attribute::*)", 1),
    ("Axes007-3", "fn:count(//west/attribute::*)", 4),
    ("Axes008-1", "fn:count(//west/attribute::west-attr-2)", 0),
    ("Axes008-2", "fn:count(//west/attribute::west-attr-2)", 0),
    ("Axes008-3", "fn:count(//west/attribute::west-attr-2)", 1),
    ("Axes009-1", "fn:count(//west/attribute::node())", 0),
    ("Axes009-2", "fn:count(//west/attribute::node())", 1),
    ("Axes009-3", "fn:count(//west/attribute::node())", 4),
    ("Axes010-1", "fn:count(//west/@*)", 0),
    ("Axes010-2", "fn:count(//west/@*)", 1),
    ("Axes010-3", "fn:count(//west/@*)", 4),
    ("Axes011-1", "fn:count(//west/@west-attr-2)", 0),
    ("Axes011-2", "fn:count(//west/@west-attr-2)", 0),
    ("Axes011-3", "fn:count(//west/@west-attr-2)", 1),
    ("Axes012-1", "fn:count( / )", 1),
    ("Axes013-1", "fn:count(//center/parent::*)", 1),
    ("Axes014-1", "fn:count(/far-north/parent::*)", 0),
    ("Axes015-1", "fn:count(//center/parent::near-north)", 1),
    ("Axes016-1", "fn:count(//center/parent::nowhere)", 0),
    ("Axes017-1", "fn:count(//center/parent::node())", 1),
    ("Axes018-1", "fn:count(/far-north/parent::node())", 1),
    ("Axes019-1", "fn:count(//center/..)", 1),
];

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}

fn find_element(
    document: &Document,
    parent: NodeId,
    local: &str,
    required_attribute: Option<(&str, &str)>,
) -> Option<NodeId> {
    for child in document.children(parent).iter().copied() {
        if document.kind(child) != NodeKind::Element {
            continue;
        }
        let matches_name = document.name(child).is_some_and(|name| {
            name.namespace.as_deref() == Some(QT3_NAMESPACE) && name.local == local
        });
        let matches_attribute = required_attribute
            .is_none_or(|(name, value)| attribute(document, child, name) == Some(value));
        if matches_name && matches_attribute {
            return Some(child);
        }
        if let Some(found) = find_element(document, child, local, required_attribute) {
            return Some(found);
        }
    }
    None
}

fn load_axis_test_set() -> (Document, PathBuf) {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/qt3tests/prod/AxisStep.xml");
    let bytes = fs::read(&path).expect("read pinned QT3 AxisStep test set and close handle");
    let parsed = parse_document(
        "urn:w3c:qt3:prod:AxisStep:test-set",
        &bytes,
        ParseLimits {
            max_events: 100_000,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 AxisStep test set");
    (
        Document::from_parsed(parsed).expect("build QT3 AxisStep test-set document"),
        path,
    )
}

#[test]
fn executes_complete_qt3_axes001_through_axes019_location_path_groups() {
    let overlay = include_str!("../../../../corpus/overlays/qt3/private-ledger-v0.toml");
    let selected_records: Vec<_> = overlay
        .split("[[case]]")
        .filter(|record| {
            CASES
                .iter()
                .any(|(case_name, _, _)| record.contains(&format!("case_name = \"{case_name}\"")))
        })
        .collect();
    assert_eq!(selected_records.len(), CASES.len());
    assert_eq!(
        selected_records
            .iter()
            .filter(|record| record.contains("execution = \"passed\""))
            .count(),
        CASES.len()
    );
    let (test_set, set_path) = load_axis_test_set();
    let set_directory = set_path
        .parent()
        .expect("QT3 test set should have a directory");

    for (case_name, expected_expression, expected_count) in CASES {
        assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", case_name)),
        )
        .expect("overlay case should exist in pinned QT3 set");
        let environment_ref = find_element(&test_set, test_case, "environment", None)
            .and_then(|node| attribute(&test_set, node, "ref"))
            .expect("QT3 case should reference an environment");
        let environment = find_element(
            &test_set,
            test_set.document_node(),
            "environment",
            Some(("name", environment_ref)),
        )
        .expect("referenced QT3 environment should exist");
        let source_file = find_element(&test_set, environment, "source", None)
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("QT3 environment should name a source file");
        let expression = find_element(&test_set, test_case, "test", None)
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 case should contain an expression");
        let asserted = find_element(&test_set, test_case, "assert-eq", None)
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 case should contain assert-eq")
            .parse::<usize>()
            .expect("axis assertion should be an unsigned integer");
        assert_eq!(asserted, expected_count);
        assert_eq!(expression, expected_expression);

        let bytes = fs::read(set_directory.join(source_file))
            .expect("read QT3 source and close import handle");
        let resource_id = format!("urn:w3c:qt3:{case_name}:source");
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4_096, 4_096));
        resources
            .admit(resource_id.clone(), bytes)
            .expect("admit QT3 source into bounded memory");
        let snapshot = resources.seal();
        let source_bytes = snapshot
            .get(&resource_id)
            .expect("sealed QT3 source should remain available");
        let parsed = parse_document(
            &resource_id,
            source_bytes,
            ParseLimits {
                max_events: 2_048,
                max_depth: 64,
            },
        )
        .expect("parse admitted QT3 source");
        let document = Document::from_parsed(parsed).expect("build QT3 source XDM");
        let mut control = InvocationControl::unbounded();
        let actual = count_experiment::evaluate(
            &expression,
            &document,
            SourceLocation {
                resource: format!("urn:w3c:qt3:{case_name}:expression"),
                span: 0..expression.len(),
            },
            &mut control,
        )
        .expect("execute admitted QT3 count expression");

        assert_eq!(actual, asserted, "native QT3 assertion for {case_name}");
        assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
    }
}
