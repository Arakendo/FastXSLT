//! Pinned XSLT30 corpus integration tests for the private golden runtime.

use std::{collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, FailureCategory, TransformRequest, TransformSetBuilder, compile_resource,
    execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const CASE_NAME: &str = "template-006";
const TEMPLATE_CASES: [&str; 6] = [
    "template-001",
    "template-002",
    "template-003",
    "template-004",
    "template-005",
    "template-006",
];

fn suite_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/template/_template-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned XSLT30 test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:decl:template:test-set",
        &bytes,
        ParseLimits {
            max_events: 4_096,
            max_depth: 64,
        },
    )
    .expect("parse pinned XSLT30 test set");
    (
        Document::from_parsed(parsed).expect("build test-set document"),
        path,
    )
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document
        .attributes(node)
        .iter()
        .copied()
        .find(|attribute| {
            document
                .name(*attribute)
                .is_some_and(|name| name.local == local)
        })
        .and_then(|attribute| document.value(attribute))
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
        let matches_name = document.name(child).is_some_and(|name| name.local == local);
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

fn assert_same_empty_document_element(actual: &str, expected: &str) {
    let limits = ParseLimits {
        max_events: 32,
        max_depth: 8,
    };
    let actual = Document::from_parsed(
        parse_document("urn:fastxslt:actual", actual.as_bytes(), limits)
            .expect("actual result should parse"),
    )
    .expect("actual result document should build");
    let expected = Document::from_parsed(
        parse_document("urn:w3c:expected", expected.as_bytes(), limits)
            .expect("expected result should parse"),
    )
    .expect("expected result document should build");
    let actual_root =
        find_element(&actual, actual.document_node(), "o", None).expect("actual document element");
    let expected_root = find_element(&expected, expected.document_node(), "o", None)
        .expect("expected document element");

    assert_eq!(actual.name(actual_root), expected.name(expected_root));
    assert!(actual.children(actual_root).is_empty());
    assert!(expected.children(expected_root).is_empty());
}

fn assert_same_result_element_string(actual: &str, expected: &str, local: &str) {
    let limits = ParseLimits {
        max_events: 32,
        max_depth: 8,
    };
    let actual_document = Document::from_parsed(
        parse_document("urn:fastxslt:actual", actual.as_bytes(), limits)
            .unwrap_or_else(|error| panic!("actual result should parse: {actual:?}: {error:?}")),
    )
    .expect("actual result should build");
    let expected_document = Document::from_parsed(
        parse_document("urn:w3c:expected", expected.trim().as_bytes(), limits)
            .expect("expected result should parse"),
    )
    .expect("expected result should build");
    let actual_element = find_element(
        &actual_document,
        actual_document.document_node(),
        local,
        None,
    )
    .expect("actual result element");
    let expected_element = find_element(
        &expected_document,
        expected_document.document_node(),
        local,
        None,
    )
    .expect("expected result element");

    assert_eq!(
        actual_document.name(actual_element),
        expected_document.name(expected_element)
    );
    assert_eq!(
        actual_document.string_value(actual_element),
        expected_document.string_value(expected_element)
    );
}

#[test]
fn classifies_the_complete_pinned_template_test_set_without_denominator_loss() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let template_set = "set_file = \"tests/decl/template/_template-test-set.xml\"";
    assert_eq!(overlay.matches(template_set).count(), TEMPLATE_CASES.len());
    assert_eq!(
        overlay
            .matches("selection = \"engine-unsupported\"")
            .count(),
        2
    );

    let (test_set, set_path) = suite_test_set();
    for case_name in TEMPLATE_CASES {
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", case_name)),
        )
        .expect("overlay case should exist in the complete pinned test set");
        assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));

        let spec = find_element(&test_set, test_case, "spec", None)
            .and_then(|node| attribute(&test_set, node, "value"))
            .expect("template case should retain an explicit spec dependency");
        assert!(matches!(spec, "XSLT10+" | "XSLT20+"));
        assert!(find_element(&test_set, test_case, "environment", None).is_some());
        assert!(find_element(&test_set, test_case, "assert-xml", None).is_some());

        let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("template case should reference one stylesheet");
        let stylesheet = fs::read(
            set_path
                .parent()
                .expect("test set should have a directory")
                .join(stylesheet_file),
        )
        .expect("read upstream stylesheet and close handle");
        let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8_192, 8_192));
        resources
            .admit(stylesheet_id.clone(), stylesheet)
            .expect("admit one upstream stylesheet");
        let snapshot = resources.seal();

        if matches!(
            case_name,
            "template-001" | "template-002" | "template-003" | CASE_NAME
        ) {
            compile_resource(&snapshot, &stylesheet_id)
                .expect("the admitted preview case should compile");
        } else {
            let expected_code = if case_name == "template-005" {
                "FXST1010"
            } else {
                "FXXP1001"
            };
            let failure = compile_resource(&snapshot, &stylesheet_id)
                .expect_err("the overlay must retain unsupported template cases visibly");
            assert_eq!(failure.category, FailureCategory::Unsupported);
            assert_eq!(failure.code, expected_code);
        }
    }
}

#[test]
fn executes_pinned_xslt30_node_kind_selection_and_modes() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let (test_set, set_path) = suite_test_set();
    for case_name in ["template-001", "template-002", "template-003"] {
        assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", case_name)),
        )
        .expect("overlay case should exist in pinned suite");
        let environment_ref = find_element(&test_set, test_case, "environment", None)
            .and_then(|node| attribute(&test_set, node, "ref"))
            .expect("case should reference an environment");
        let environment = find_element(
            &test_set,
            test_set.document_node(),
            "environment",
            Some(("name", environment_ref)),
        )
        .expect("referenced environment should exist");
        let source = find_element(&test_set, environment, "content", None)
            .map(|node| test_set.string_value(node))
            .expect("environment should contain the principal source");
        let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("case should name a stylesheet");
        let expected = find_element(&test_set, test_case, "assert-xml", None)
            .map(|node| test_set.string_value(node))
            .expect("case should provide an XML assertion");
        let stylesheet = fs::read(
            set_path
                .parent()
                .expect("test set should have a directory")
                .join(stylesheet_file),
        )
        .expect("read upstream stylesheet and close handle");
        let source_id = format!("urn:w3c:xslt30:{case_name}:source");
        let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
        resources
            .admit(source_id.clone(), source.into_bytes())
            .expect("admit upstream source");
        resources
            .admit(stylesheet_id.clone(), stylesheet)
            .expect("admit upstream stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, &stylesheet_id).expect("compile suite case");
        assert!(program.root_template.is_none());
        let mut set = TransformSetBuilder::new(
            snapshot,
            program,
            1,
            ExecutionPolicy {
                denied_sources: HashSet::new(),
                serialized_byte_limit: 4_096,
                work_limits: WorkLimits::unbounded(),
            },
        );
        set.add(TransformRequest {
            identity: case_name.to_owned(),
            result_identity: format!("result:{case_name}"),
            source_resource: source_id,
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit suite request");
        let results = execute_transform_set(set.seal()).expect("execute suite case");
        let actual = &results.by_request[case_name].serialized;
        assert_same_result_element_string(actual, &expected, "out");
    }
}

#[test]
fn executes_pinned_xslt30_template_006_from_its_upstream_test_set() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay.contains("case_name = \"template-006\""));

    let (test_set, set_path) = suite_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", CASE_NAME)),
    )
    .expect("overlay case should exist in pinned suite");
    let environment_ref = find_element(&test_set, test_case, "environment", None)
        .and_then(|node| attribute(&test_set, node, "ref"))
        .expect("case should reference an environment");
    let environment = find_element(
        &test_set,
        test_set.document_node(),
        "environment",
        Some(("name", environment_ref)),
    )
    .expect("referenced environment should exist");
    let source = find_element(&test_set, environment, "content", None)
        .map(|node| test_set.string_value(node))
        .expect("environment should contain the principal source");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("case should name a stylesheet");
    let expected = find_element(&test_set, test_case, "assert-xml", None)
        .map(|node| test_set.string_value(node))
        .expect("case should provide an XML assertion");
    let stylesheet = fs::read(
        set_path
            .parent()
            .expect("test set should have a directory")
            .join(stylesheet_file),
    )
    .expect("read upstream stylesheet and close handle");

    let source_id = "urn:w3c:xslt30:template-006:source";
    let stylesheet_id = "urn:w3c:xslt30:template-006:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(source_id, source.into_bytes())
        .expect("admit upstream source");
    resources
        .admit(stylesheet_id, stylesheet)
        .expect("admit upstream stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, stylesheet_id).expect("compile suite case");
    assert_eq!(program.output.method, None);
    let mut set = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 4_096,
            work_limits: WorkLimits::unbounded(),
        },
    );
    set.add(TransformRequest {
        identity: CASE_NAME.to_owned(),
        result_identity: "result:template-006".to_owned(),
        source_resource: source_id.to_owned(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit suite request");

    let results = execute_transform_set(set.seal()).expect("execute suite case");
    let actual = &results.by_request[CASE_NAME].serialized;

    assert_eq!(actual, "<?xml version=\"1.0\" encoding=\"UTF-8\"?><o></o>");
    assert_same_empty_document_element(actual, expected.trim());
}
