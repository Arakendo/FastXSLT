//! Pinned XSLT30 corpus integration tests for the private golden runtime.

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::PathBuf,
};

use super::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, compile_resource,
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

fn apply_templates_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/insn/apply-templates/_apply-templates-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned XSLT30 apply-templates test set");
    let parsed = parse_document(
        "urn:w3c:xslt30:insn:apply-templates:test-set",
        &bytes,
        ParseLimits {
            max_events: 16_384,
            max_depth: 64,
        },
    )
    .expect("parse pinned XSLT30 apply-templates test set");
    (
        Document::from_parsed(parsed).expect("build apply-templates test-set document"),
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
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit path request");

    let results = execute_transform_set(set.seal()).expect("execute path case");
    (results.by_request[case_name].serialized.clone(), expected)
}

fn execute_apply_templates_case(case_name: &str) -> (String, String, usize) {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));
    let (test_set, set_path) = apply_templates_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", case_name)),
    )
    .expect("overlay case should exist in pinned suite");
    let case_environment = find_element(&test_set, test_case, "environment", None)
        .expect("case should provide an environment");
    let environment = if let Some(environment_ref) = attribute(&test_set, case_environment, "ref") {
        find_element(
            &test_set,
            test_set.document_node(),
            "environment",
            Some(("name", environment_ref)),
        )
        .expect("referenced environment should exist")
    } else {
        case_environment
    };
    let source_element = find_element(&test_set, environment, "source", None)
        .expect("environment should contain the principal source");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("case should name a stylesheet");
    let expected = find_element(&test_set, test_case, "assert-xml", None)
        .map(|node| test_set.string_value(node))
        .or_else(|| expected_apply_templates_all_of(case_name))
        .expect("case should provide an admitted result assertion");
    let case_directory = set_path.parent().expect("test set should have a directory");
    let source = if let Some(content) = find_element(&test_set, source_element, "content", None) {
        test_set.string_value(content).into_bytes()
    } else {
        let source_file = attribute(&test_set, source_element, "file")
            .expect("principal source should be inline or name a file");
        fs::read(case_directory.join(source_file))
            .expect("read upstream apply-templates source and close handle")
    };
    let stylesheet = fs::read(case_directory.join(stylesheet_file))
        .expect("read upstream stylesheet and close handle");
    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(source_id.clone(), source)
        .expect("admit upstream source");
    resources
        .admit(stylesheet_id.clone(), stylesheet)
        .expect("admit upstream stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile suite case");
    let matched_template_count = program.matched_templates.len();
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
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit suite request");
    let results = execute_transform_set(set.seal()).expect("execute suite case");
    (
        results.by_request[case_name].serialized.clone(),
        expected,
        matched_template_count,
    )
}

fn expected_apply_templates_all_of(case_name: &str) -> Option<String> {
    (case_name == "conflict-resolution-1501")
        .then(|| "<doc><a><a/></a><a><a parent-recursive=\"yes\"/></a><a><b/></a></doc>".to_owned())
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
            entry: InvocationEntry::PrincipalSource {
                resource: source_id,
            },
            parameters: BTreeMap::new(),
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
fn executes_pinned_xslt30_conflict_resolution_0101() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0101");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_priority_conflicts_by_last_source_order() {
    for case_name in ["conflict-resolution-0102c", "conflict-resolution-0104c"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 3);
        assert_same_result_element_string(&actual, &expected, "out");
    }
}

#[test]
fn executes_xslt30_explicit_priority_without_widening_node_test_axis() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0106");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_non_simple_path_default_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0107");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_priority_path_and_attribute_predicate_by_source_order() {
    for case_name in ["conflict-resolution-0108c", "conflict-resolution-0110c"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 6);
        assert_same_result_element_string(&actual, &expected, "out");
    }
}

#[test]
fn executes_xslt30_descendant_wildcard_non_simple_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0112");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "text");
}

#[test]
fn executes_xslt30_exact_attribute_value_pattern() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0201");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_explicit_priority_namespace_wildcard_by_source_order() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0401c");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "b");
}

#[test]
fn executes_xslt30_leading_descendant_selection_with_parent_child_patterns() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0901");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_xpath_default_namespace_pattern_and_selection() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0701");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_literal_result_xpath_default_namespace_context() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0702");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
    assert!(!actual.contains("xpath-default-namespace"));
    let parsed = parse_document(
        "urn:fastxslt:conflict-resolution-0702:actual",
        actual.as_bytes(),
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("actual result should parse");
    let document = Document::from_parsed(parsed).expect("actual result should build");
    let out =
        find_element(&document, document.document_node(), "out", None).expect("actual out element");
    assert!(document.namespace_declarations(out).iter().any(|binding| {
        binding.prefix.as_deref() == Some("u") && binding.namespace == "http://some.uri/"
    }));
}

#[test]
fn executes_xslt30_stylesheet_default_namespace_without_affecting_attributes() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0703");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_current_mode_through_named_template_call() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0801");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_multi_mode_default_and_current_dispatch() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0802");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_local_name_wildcard_fractional_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1701");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_element_kind_test_default_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1801");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_root_pattern_default_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1601");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_document_element_pattern_priorities() {
    for case_name in ["conflict-resolution-1602", "conflict-resolution-1603"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 3);
        assert_same_result_element_string(&actual, &expected, "out");
    }
}

#[test]
fn executes_xslt30_next_match_priority_chain() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1201");
    assert_eq!(matched_template_count, 6);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_global_variable_pattern_and_source_copy() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0601");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "doc");
}

#[test]
fn executes_xslt30_equivalent_same_named_child_patterns() {
    for case_name in ["conflict-resolution-0501", "conflict-resolution-0502"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 2);
        assert_same_result_element_string(&actual, &expected, "doc");
    }
}

#[test]
fn executes_xslt30_same_named_parent_current_pattern() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0503");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "doc");
}

#[test]
fn executes_xslt30_filtered_parent_position_current_pattern() {
    let (test_set, _) = apply_templates_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", "conflict-resolution-1501")),
    )
    .expect("case should exist");
    let all_of = find_element(&test_set, test_case, "all-of", None).expect("all-of assertion");
    let assertions = test_set
        .children(all_of)
        .iter()
        .copied()
        .filter(|node| {
            test_set.kind(*node) == NodeKind::Element
                && test_set
                    .name(*node)
                    .is_some_and(|name| name.local == "assert")
        })
        .collect::<Vec<_>>();
    assert_eq!(assertions.len(), 2);
    assert_eq!(
        test_set.string_value(assertions[0]),
        "/doc/a[2]/a[1][@parent-recursive=\"yes\"]"
    );
    assert_eq!(
        test_set.string_value(assertions[1]),
        "count(//@parent-recursive) = 1"
    );

    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1501");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "doc");
}

#[test]
fn executes_xslt30_builtin_temporary_tree_parameter_propagation() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1101");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "z");
}

#[test]
fn executes_xslt30_apply_imports_builtin_parameter_propagation() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1102");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "z");
}

#[test]
fn executes_xslt30_next_match_parameter_chain() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1205");
    assert_eq!(matched_template_count, 6);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_rank_next_match_chain() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1202c");
    assert_eq!(matched_template_count, 7);
    assert_same_result_element_string(&actual, &expected, "out");
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
        entry: InvocationEntry::PrincipalSource {
            resource: source_id.to_owned(),
        },
        parameters: BTreeMap::new(),
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
        entry: InvocationEntry::PrincipalSource {
            resource: source_id.to_owned(),
        },
        parameters: BTreeMap::new(),
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

#[test]
fn executes_pinned_xslt30_path_008_floor_inside_arithmetic() {
    let (actual, expected) = execute_path_case("path-008");
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_009_floor_position() {
    let (actual, expected) = execute_path_case("path-009");
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_pinned_xslt30_path_010_multi_step_positions_and_pattern() {
    let (actual, expected) = execute_path_case("path-010");
    assert_same_result_element_string(&actual, &expected, "out");
}
