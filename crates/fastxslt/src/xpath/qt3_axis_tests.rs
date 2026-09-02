//! Executable metadata-driven `QT3` location-axis slice.

use std::{fs, path::PathBuf};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

use super::count_experiment;

const QT3_NAMESPACE: &str = "http://www.w3.org/2010/09/qt-fots-catalog";
const STATIC_SYNTAX_ERROR_CASES: [(&str, &str); 22] = [
    ("Axes088", "/*/"),
    ("K2-Axes-5", "*:(:hey:)ncname"),
    ("K2-Axes-6", "*(:hey:):ncname"),
    ("K2-Axes-7", "* :ncname"),
    ("K2-Axes-8", "*(:hey:):ncname"),
    ("K2-Axes-9", "ncname :*"),
    ("K2-Axes-10", "name(:hey:):*"),
    ("K2-Axes-11", "* :ncname"),
    ("K2-Axes-12", "ncname: *"),
    ("K2-Axes-13", "*(:hey:):ncname"),
    ("K2-Axes-14", "ncname:(:hey:)*"),
    ("K2-Axes-15", "*(:hey:):(:hey:) ncname"),
    ("K2-Axes-16", "*:(:hey:)ncname"),
    ("K2-Axes-17", "*:"),
    ("K2-Axes-29", "preceding-or-ancestor::*"),
    ("K2-Axes-34", "nametest//"),
    ("K2-Axes-35", "nametest/"),
    ("K2-Axes-37", "parent::self()"),
    ("K2-Axes-46", "//"),
    ("K2-Axes-77", "preceeding::node()"),
    ("K2-Axes-90", "prefix:"),
    ("K2-Axes-91", "prefix:"),
];
const CASES: [(&str, &str, usize); 182] = [
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
    ("Axes020-1", "fn:count(//center/self::*)", 1),
    ("Axes021-1", "fn:count(//center/self::center)", 1),
    ("Axes023-1", "fn:count(//center/self::node())", 1),
    (
        "Axes027-1",
        "fn:count(//center/@center-attr-3/self::node())",
        1,
    ),
    ("Axes030-1", "fn:count(//center/text()/self::node())", 0),
    ("Axes030-2", "fn:count(//center/text()/self::node())", 1),
    ("Axes031-1", "fn:count(//center/descendant::*)", 0),
    ("Axes031-2", "fn:count(//center/descendant::*)", 0),
    ("Axes031-3", "fn:count(//center/descendant::*)", 1),
    ("Axes031-4", "fn:count(//center/descendant::*)", 5),
    ("Axes032-1", "fn:count(//center/descendant::south)", 0),
    ("Axes032-2", "fn:count(//center/descendant::south)", 0),
    ("Axes032-3", "fn:count(//center/descendant::south)", 1),
    ("Axes032-4", "fn:count(//center/descendant::south)", 8),
    ("Axes033-1", "fn:count(//center/descendant::node())", 0),
    ("Axes033-2", "fn:count(//center/descendant::node())", 1),
    ("Axes033-3", "fn:count(//center/descendant::node())", 1),
    ("Axes033-4", "fn:count(//center/descendant::node())", 21),
    ("Axes034-1", "fn:count(//center/descendant-or-self::*)", 1),
    ("Axes034-2", "fn:count(//center/descendant-or-self::*)", 6),
    (
        "Axes035-1",
        "fn:count(//center/descendant-or-self::south)",
        0,
    ),
    (
        "Axes035-2",
        "fn:count(//center/descendant-or-self::south)",
        0,
    ),
    (
        "Axes035-3",
        "fn:count(//center/descendant-or-self::south)",
        1,
    ),
    (
        "Axes035-4",
        "fn:count(//center/descendant-or-self::south)",
        8,
    ),
    (
        "Axes036-1",
        "fn:count(//center/descendant-or-self::center)",
        1,
    ),
    (
        "Axes036-2",
        "fn:count(//center/descendant-or-self::center)",
        9,
    ),
    (
        "Axes037-1",
        "fn:count(//center/descendant-or-self::node())",
        1,
    ),
    (
        "Axes037-2",
        "fn:count(//center/descendant-or-self::node())",
        22,
    ),
    (
        "Axes041-1",
        "fn:count(//center/@center-attr-3/descendant-or-self::node())",
        1,
    ),
    (
        "Axes043-1",
        "fn:count(//center/text()/descendant-or-self::node())",
        0,
    ),
    (
        "Axes043-2",
        "fn:count(//center/text()/descendant-or-self::node())",
        1,
    ),
    ("Axes044-1", "fn:count(/child::*)", 1),
    ("Axes044-2", "fn:count(/child::*)", 1),
    ("Axes045-1", "fn:count(/child::far-north)", 0),
    ("Axes045-2", "fn:count(/child::far-north)", 1),
    ("Axes046-1", "fn:count(/child::node())", 1),
    ("Axes046-2", "fn:count(/child::node())", 7),
    ("Axes047-1", "fn:count(/*)", 1),
    ("Axes047-2", "fn:count(/*)", 1),
    ("Axes048-1", "fn:count(/far-north)", 0),
    ("Axes048-2", "fn:count(/far-north)", 1),
    ("Axes049-1", "fn:count(/node())", 1),
    ("Axes049-2", "fn:count(/node())", 7),
    ("Axes055-1", "fn:count(/self::node())", 1),
    ("Axes056-1", "fn:count(/descendant::*)", 1),
    ("Axes056-2", "fn:count(/descendant::*)", 15),
    ("Axes056-3", "fn:count(/descendant::*)", 16),
    ("Axes057-1", "fn:count(/descendant::south)", 0),
    ("Axes057-2", "fn:count(/descendant::south)", 1),
    ("Axes057-3", "fn:count(/descendant::south)", 1),
    ("Axes057-4", "fn:count(/descendant::south)", 8),
    ("Axes058-1", "fn:count(/descendant::node())", 1),
    ("Axes058-2", "fn:count(/descendant::node())", 56),
    ("Axes058-3", "fn:count(/descendant::node())", 58),
    ("Axes059-1", "fn:count(/descendant-or-self::*)", 1),
    ("Axes059-2", "fn:count(/descendant-or-self::*)", 15),
    ("Axes060-1", "fn:count(/descendant-or-self::south)", 0),
    ("Axes060-2", "fn:count(/descendant-or-self::south)", 1),
    ("Axes060-3", "fn:count(/descendant-or-self::south)", 1),
    ("Axes060-4", "fn:count(/descendant-or-self::south)", 8),
    ("Axes061-1", "fn:count(/descendant-or-self::node())", 57),
    ("Axes061-2", "fn:count(/descendant-or-self::node())", 59),
    ("Axes062-1", "fn:count(//child::*)", 1),
    ("Axes062-2", "fn:count(//child::*)", 15),
    ("Axes063-1", "fn:count(//child::south)", 0),
    ("Axes063-2", "fn:count(//child::south)", 1),
    ("Axes063-3", "fn:count(//child::south)", 1),
    ("Axes063-4", "fn:count(//child::south)", 8),
    ("Axes064-1", "fn:count(//child::node())", 1),
    ("Axes064-2", "fn:count(//child::node())", 56),
    ("Axes064-3", "fn:count(//child::node())", 58),
    ("Axes065-1", "fn:count(//*)", 1),
    ("Axes065-2", "fn:count(//*)", 15),
    ("Axes066-1", "fn:count(//south)", 0),
    ("Axes066-2", "fn:count(//south)", 1),
    ("Axes066-3", "fn:count(//south)", 1),
    ("Axes066-4", "fn:count(//south)", 8),
    ("Axes067-1", "fn:count(//node())", 1),
    ("Axes067-2", "fn:count(//node())", 56),
    ("Axes067-3", "fn:count(//node())", 58),
    ("Axes068-1", "fn:count(//attribute::*)", 0),
    ("Axes068-2", "fn:count(//attribute::*)", 1),
    ("Axes068-3", "fn:count(//attribute::*)", 14),
    ("Axes069-1", "fn:count(//attribute::mark)", 0),
    ("Axes069-2", "fn:count(//attribute::mark)", 1),
    ("Axes069-3", "fn:count(//attribute::mark)", 6),
    ("Axes070-1", "fn:count(//@*)", 0),
    ("Axes070-2", "fn:count(//@*)", 1),
    ("Axes070-3", "fn:count(//@*)", 14),
    ("Axes071-1", "fn:count(//@mark)", 0),
    ("Axes071-2", "fn:count(//@mark)", 1),
    ("Axes071-3", "fn:count(//@mark)", 6),
    ("Axes072-1", "fn:count(//self::*)", 1),
    ("Axes072-2", "fn:count(//self::*)", 15),
    ("Axes073-1", "fn:count(//self::node())", 57),
    ("Axes073-2", "fn:count(//self::node())", 59),
    ("Axes074-1", "fn:count(//center//child::*)", 0),
    ("Axes074-2", "fn:count(//center//child::*)", 0),
    ("Axes074-3", "fn:count(//center//child::*)", 1),
    ("Axes074-4", "fn:count(//center//child::*)", 12),
    ("Axes075-1", "fn:count(//center//child::south)", 0),
    ("Axes075-2", "fn:count(//center//child::south)", 0),
    ("Axes075-3", "fn:count(//center//child::south)", 1),
    ("Axes075-4", "fn:count(//center//child::south)", 8),
    ("Axes076-1", "fn:count(//center//child::node())", 0),
    ("Axes076-2", "fn:count(//center//child::node())", 1),
    ("Axes076-3", "fn:count(//center//child::node())", 1),
    ("Axes076-4", "fn:count(//center//child::node())", 37),
    ("Axes077-1", "fn:count(//center//*)", 0),
    ("Axes077-2", "fn:count(//center//*)", 1),
    ("Axes077-3", "fn:count(//center//*)", 12),
    ("Axes078-1", "fn:count(//center//south)", 0),
    ("Axes078-2", "fn:count(//center//south)", 0),
    ("Axes078-3", "fn:count(//center//south)", 1),
    ("Axes078-4", "fn:count(//center//south)", 8),
    ("Axes079-1", "fn:count(//center//node())", 0),
    ("Axes079-2", "fn:count(//center//node())", 1),
    ("Axes079-3", "fn:count(//center//node())", 1),
    ("Axes079-4", "fn:count(//center//node())", 37),
    ("Axes080-1", "fn:count(//west//attribute::*)", 0),
    ("Axes080-2", "fn:count(//west//attribute::*)", 1),
    ("Axes080-3", "fn:count(//west//attribute::*)", 4),
    (
        "Axes081-1",
        "fn:count(//center//attribute::center-attr-2)",
        0,
    ),
    (
        "Axes081-2",
        "fn:count(//center//attribute::center-attr-2)",
        0,
    ),
    (
        "Axes081-3",
        "fn:count(//center//attribute::center-attr-2)",
        1,
    ),
    (
        "Axes081-4",
        "fn:count(//center//attribute::center-attr-2)",
        4,
    ),
    ("Axes082-1", "fn:count(//west//attribute::node())", 0),
    ("Axes082-2", "fn:count(//west//attribute::node())", 1),
    ("Axes082-3", "fn:count(//west//attribute::node())", 4),
    ("Axes083-1", "fn:count(//west//@*)", 0),
    ("Axes083-2", "fn:count(//west//@*)", 1),
    ("Axes083-3", "fn:count(//west//@*)", 4),
    ("Axes084-1", "fn:count(//center//@center-attr-2)", 0),
    ("Axes084-2", "fn:count(//center//@center-attr-2)", 0),
    ("Axes084-3", "fn:count(//center//@center-attr-2)", 1),
    ("Axes084-4", "fn:count(//center//@center-attr-2)", 4),
    ("Axes084-5", "fn:count(//text()[normalize-space()])", 827),
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
fn executes_qt3_axes001_through_axes084_admitted_location_path_groups() {
    crate::qt3_overlay_test_support::assert_selected_count(
        "prod/AxisStep.xml",
        CASES.len() + STATIC_SYNTAX_ERROR_CASES.len(),
    );
    let (test_set, set_path) = load_axis_test_set();
    let set_directory = set_path
        .parent()
        .expect("QT3 test set should have a directory");

    for (case_name, expected_expression, expected_count) in CASES {
        crate::qt3_overlay_test_support::assert_private_case_passed("prod/AxisStep.xml", case_name);
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
        let source_byte_limit = if case_name == "Axes084-5" {
            65_536
        } else {
            4_096
        };
        let mut resources =
            ResourceSetBuilder::new(ResourceLimits::new(1, source_byte_limit, source_byte_limit));
        resources
            .admit(resource_id.clone(), bytes)
            .expect("admit QT3 source into bounded memory");
        let snapshot = resources.seal();
        let source_bytes = snapshot
            .get(&resource_id)
            .expect("sealed QT3 source should remain available");
        let max_events = if case_name == "Axes084-5" {
            16_384
        } else {
            2_048
        };
        let parsed = parse_document(
            &resource_id,
            source_bytes,
            ParseLimits {
                max_events,
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

#[test]
fn reports_admitted_qt3_axis_static_syntax_errors() {
    use super::path_experiment::{PathFailure, parse_location_path};

    let (test_set, _) = load_axis_test_set();
    for (case_name, expected_expression) in STATIC_SYNTAX_ERROR_CASES {
        crate::qt3_overlay_test_support::assert_private_case_passed("prod/AxisStep.xml", case_name);
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", case_name)),
        )
        .expect("admitted QT3 syntax case");
        let expression = find_element(&test_set, test_case, "test", None)
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 syntax expression");
        assert_eq!(expression, expected_expression, "{case_name}");
        let asserted_error = find_element(&test_set, test_case, "error", None)
            .and_then(|node| attribute(&test_set, node, "code"))
            .expect("QT3 static error assertion");
        assert_eq!(asserted_error, "XPST0003", "{case_name}");

        let location = SourceLocation {
            resource: format!("urn:w3c:qt3:{case_name}:expression"),
            span: 0..expression.len(),
        };
        let failure = parse_location_path(&expression, location.clone())
            .expect_err("invalid QT3 path syntax must fail");
        match failure {
            PathFailure::Invalid {
                standard_code,
                detail,
                location: actual_location,
            } => {
                assert_eq!(standard_code, "XPST0003", "{case_name}");
                assert!(!detail.is_empty(), "{case_name}");
                assert_eq!(actual_location, location, "{case_name}");
            }
            PathFailure::Unsupported { .. } => {
                panic!("{case_name} must be classified as invalid syntax")
            }
        }
    }
}
