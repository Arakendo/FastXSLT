//! Pinned XSLT30 corpus integration tests for the private golden runtime.

use std::{collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, TransformRequest, TransformSetBuilder, compile_resource, execute_transform_set,
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
const PATH_CASES: [&str; 10] = [
    "path-001", "path-002", "path-003", "path-004", "path-005", "path-006", "path-007", "path-008",
    "path-009", "path-010",
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

fn path_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/expr/path/_path-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned XSLT30 path test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:expr:path:test-set",
        &bytes,
        ParseLimits {
            max_events: 4_096,
            max_depth: 64,
        },
    )
    .expect("parse pinned XSLT30 path test set");
    (
        Document::from_parsed(parsed).expect("build path test-set document"),
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

fn execute_path_case(case_name: &str) -> (String, String) {
    let (test_set, set_path) = path_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", case_name)),
    )
    .expect("path case should exist in pinned suite");
    let environment_ref = find_element(&test_set, test_case, "environment", None)
        .and_then(|node| attribute(&test_set, node, "ref"))
        .expect("path case should reference an environment");
    let environment = find_element(
        &test_set,
        test_set.document_node(),
        "environment",
        Some(("name", environment_ref)),
    )
    .expect("referenced path environment should exist");
    let source_element = find_element(&test_set, environment, "source", None)
        .expect("path case should have a principal source");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("path case should name a stylesheet");
    let expected = find_element(&test_set, test_case, "assert-xml", None)
        .map(|node| test_set.string_value(node))
        .expect("path case should provide an XML assertion");
    let case_directory = set_path
        .parent()
        .expect("path test set should have a directory");
    let source = if let Some(content) = find_element(&test_set, source_element, "content", None) {
        test_set.string_value(content).into_bytes()
    } else {
        let source_file = attribute(&test_set, source_element, "file")
            .expect("path source should be inline or name a file");
        fs::read(case_directory.join(source_file))
            .expect("read upstream path source and close handle")
    };
    let stylesheet = fs::read(case_directory.join(stylesheet_file))
        .expect("read upstream path stylesheet and close handle");
    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(source_id.clone(), source)
        .expect("admit path source bytes");
    resources
        .admit(stylesheet_id.clone(), stylesheet)
        .expect("admit path stylesheet bytes");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile path case");
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
    .expect("admit path request");

    let results = execute_transform_set(set.seal()).expect("execute path case");
    (results.by_request[case_name].serialized.clone(), expected)
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
        0
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

        compile_resource(&snapshot, &stylesheet_id)
            .expect("every case in the complete admitted template set should compile");
    }
}

#[test]
fn executes_pinned_xslt30_template_001_through_005() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let (test_set, set_path) = suite_test_set();
    for case_name in [
        "template-001",
        "template-002",
        "template-003",
        "template-004",
        "template-005",
    ] {
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

#[test]
fn inventories_the_complete_pinned_path_test_set_without_denominator_loss() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let path_set = "set_file = \"tests/expr/path/_path-test-set.xml\"";
    assert_eq!(overlay.matches(path_set).count(), PATH_CASES.len());

    let (test_set, _) = path_test_set();
    for case_name in PATH_CASES {
        assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", case_name)),
        )
        .expect("overlay path case should exist in the complete pinned test set");
        assert!(find_element(&test_set, test_case, "environment", None).is_some());
        assert!(find_element(&test_set, test_case, "stylesheet", None).is_some());
        assert!(find_element(&test_set, test_case, "assert-xml", None).is_some());
    }
}

#[test]
fn executes_pinned_xslt30_path_001_child_axis_predicate() {
    let (test_set, set_path) = path_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", "path-001")),
    )
    .expect("path-001 should exist in pinned suite");
    let environment_ref = find_element(&test_set, test_case, "environment", None)
        .and_then(|node| attribute(&test_set, node, "ref"))
        .expect("path-001 should reference an environment");
    let environment = find_element(
        &test_set,
        test_set.document_node(),
        "environment",
        Some(("name", environment_ref)),
    )
    .expect("path-001 environment should exist");
    let source = find_element(&test_set, environment, "content", None)
        .map(|node| test_set.string_value(node))
        .expect("path-001 should have an inline principal source");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("path-001 should name a stylesheet");
    let expected = find_element(&test_set, test_case, "assert-xml", None)
        .map(|node| test_set.string_value(node))
        .expect("path-001 should provide an XML assertion");
    let stylesheet = fs::read(
        set_path
            .parent()
            .expect("path test set should have a directory")
            .join(stylesheet_file),
    )
    .expect("read upstream path-001 stylesheet and close handle");

    let source_id = "urn:w3c:xslt30:path-001:source";
    let stylesheet_id = "urn:w3c:xslt30:path-001:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(source_id, source.into_bytes())
        .expect("admit path-001 source");
    resources
        .admit(stylesheet_id, stylesheet)
        .expect("admit path-001 stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, stylesheet_id).expect("compile path-001");
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
        identity: "path-001".to_owned(),
        result_identity: "result:path-001".to_owned(),
        source_resource: source_id.to_owned(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit path-001 request");

    let results = execute_transform_set(set.seal()).expect("execute path-001");
    assert_same_result_element_string(&results.by_request["path-001"].serialized, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_002_from_a_file_backed_environment() {
    let (actual, expected) = execute_path_case("path-002");
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_003_ancestor_or_self_predicate() {
    let (actual, expected) = execute_path_case("path-003");
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_004_attribute_predicate() {
    let (actual, expected) = execute_path_case("path-004");
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_005_descendant_or_self_predicate() {
    let (actual, expected) = execute_path_case("path-005");
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_006_parent_predicate() {
    let (actual, expected) = execute_path_case("path-006");
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_007_constant_arithmetic_position() {
    let (actual, expected) = execute_path_case("path-007");
    assert_same_result_element_string(&actual, &expected, "out");
}
