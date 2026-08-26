//! Conserved integration tests for the complete XSLT30 `expr/for` denominator.

use std::{collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, FailureCategory, InvocationEntry, TransformRequest, TransformSetBuilder,
    compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "tests/expr/for/_for-test-set.xml";

#[derive(Clone, Copy)]
struct CasePressure {
    name: &'static str,
    environment: Option<&'static str>,
    initial_template: Option<&'static str>,
    execution: &'static str,
}

const CASES: [CasePressure; 4] = [
    CasePressure {
        name: "for-001",
        environment: Some("for01"),
        initial_template: None,
        execution: "passed",
    },
    CasePressure {
        name: "for-002",
        environment: None,
        initial_template: Some("main"),
        execution: "passed",
    },
    CasePressure {
        name: "for-003",
        environment: Some("for03"),
        initial_template: None,
        execution: "passed",
    },
    CasePressure {
        name: "for-004",
        environment: Some("for03"),
        initial_template: None,
        execution: "passed",
    },
];

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}

#[test]
fn executes_native_xslt30_for_001_in_order() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay_case(overlay, "for-001").contains("execution = \"passed\""));
    let (actual, expected) = execute_principal_case("for-001");
    assert_eq!(actual, expected.trim());
}

#[test]
fn executes_native_xslt30_for_003_with_outer_focus_preserved() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay_case(overlay, "for-003").contains("execution = \"passed\""));
    let (actual, expected) = execute_principal_case("for-003");
    assert_eq!(expected.trim(), "<out>0</out>");
    assert_eq!(
        actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>0</out>"
    );
}

#[test]
fn executes_native_xslt30_for_004_with_exact_decimal_sum() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay_case(overlay, "for-004").contains("execution = \"passed\""));
    let (actual, expected) = execute_principal_case("for-004");
    assert_eq!(expected.trim(), "<out>36.02</out>");
    assert_eq!(
        actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>"
    );
}

#[test]
fn executes_source_free_native_xslt30_for_002_initial_template() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay_case(overlay, "for-002").contains("execution = \"passed\""));
    let (test_set, set_path) = load_test_set();
    let directory = set_path
        .parent()
        .expect("for test set should have a directory");
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", "for-002")),
    )
    .expect("for-002 should exist in the pinned test set");
    assert!(find_element(&test_set, test_case, "environment", None).is_none());
    let initial_template = find_element(&test_set, test_case, "initial-template", None)
        .and_then(|node| attribute(&test_set, node, "name"))
        .expect("for-002 should declare an initial template");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("for-002 should name a stylesheet");
    let expected = find_element(&test_set, test_case, "assert-xml", None)
        .map(|node| test_set.string_value(node))
        .expect("for-002 should provide inline expected XML");
    let stylesheet = fs::read(directory.join(stylesheet_file))
        .expect("read for-002 stylesheet and close import handle");
    let stylesheet_id = "urn:w3c:xslt30:for-002:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8_192, 8_192));
    resources
        .admit(stylesheet_id, stylesheet)
        .expect("admit for-002 stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, stylesheet_id).expect("compile native for-002");
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
        identity: "for-002".to_owned(),
        result_identity: "result:for-002".to_owned(),
        entry: InvocationEntry::InitialTemplate {
            name: initial_template.to_owned(),
        },
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit source-free for-002 request");

    let results = execute_transform_set(set.seal()).expect("execute native for-002");
    assert_eq!(expected.trim(), "<out>11, 12, 21, 22</out>");
    assert_eq!(
        results.by_request["for-002"].serialized,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>{}",
            expected.trim()
        )
    );
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

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/expr/for/_for-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned XSLT30 for test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:expr:for:test-set",
        &bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
    )
    .expect("parse pinned XSLT30 for test set");
    (
        Document::from_parsed(parsed).expect("build XSLT30 for test-set document"),
        path,
    )
}

fn execute_principal_case(case_name: &str) -> (String, String) {
    let (test_set, set_path) = load_test_set();
    let directory = set_path
        .parent()
        .expect("for test set should have a directory");
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", case_name)),
    )
    .expect("for case should exist in the pinned test set");
    let environment_ref = find_element(&test_set, test_case, "environment", None)
        .and_then(|node| attribute(&test_set, node, "ref"))
        .expect("principal for case should reference an environment");
    let environment = find_element(
        &test_set,
        test_set.document_node(),
        "environment",
        Some(("name", environment_ref)),
    )
    .expect("referenced for environment should exist");
    let source_file = find_element(&test_set, environment, "source", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("for environment should name a source");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("for case should name a stylesheet");
    let assertion = find_element(&test_set, test_case, "assert-xml", None)
        .expect("for case should provide expected XML");
    let expected = attribute(&test_set, assertion, "file").map_or_else(
        || test_set.string_value(assertion),
        |file| {
            fs::read_to_string(directory.join(file))
                .expect("read file-backed for assertion and close handle")
        },
    );
    let source = fs::read(directory.join(source_file))
        .expect("read upstream for source and close import handle");
    let stylesheet = fs::read(directory.join(stylesheet_file))
        .expect("read upstream for stylesheet and close import handle");
    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(source_id.clone(), source)
        .expect("admit native for source");
    resources
        .admit(stylesheet_id.clone(), stylesheet)
        .expect("admit native for stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile native for case");
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
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit native for request");

    let results = execute_transform_set(set.seal()).expect("execute native for case");
    (results.by_request[case_name].serialized.clone(), expected)
}

fn overlay_case<'a>(overlay: &'a str, case_name: &str) -> &'a str {
    overlay
        .split("[[case]]")
        .find(|section| {
            section.contains(&format!("set_file = \"{SET_FILE}\""))
                && section.contains(&format!("case_name = \"{case_name}\""))
        })
        .expect("admitted case should have one overlay record")
}

#[test]
fn admits_complete_xslt30_for_test_set_without_denominator_loss() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let admitted: Vec<_> = overlay
        .split("[[case]]")
        .filter(|section| section.contains(&format!("set_file = \"{SET_FILE}\"")))
        .collect();
    assert_eq!(admitted.len(), CASES.len());

    let (test_set, set_path) = load_test_set();
    let directory = set_path
        .parent()
        .expect("for test set should have a directory");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(7, 8_192, 32_768));

    for pressure in CASES {
        let record = overlay_case(overlay, pressure.name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains(&format!("execution = \"{}\"", pressure.execution)));

        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", pressure.name)),
        )
        .expect("overlay identity should exist in pinned test set");
        let spec = find_element(&test_set, test_case, "spec", None)
            .and_then(|node| attribute(&test_set, node, "value"));
        assert_eq!(spec, Some("XSLT20+"));

        let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("for case should name one stylesheet");
        let stylesheet = fs::read(directory.join(stylesheet_file))
            .expect("read upstream for stylesheet and close handle");
        resources
            .admit(
                format!("urn:w3c:xslt30:{}:stylesheet", pressure.name),
                stylesheet,
            )
            .expect("admit upstream for stylesheet");

        let environment_ref = find_element(&test_set, test_case, "environment", None)
            .and_then(|node| attribute(&test_set, node, "ref"));
        assert_eq!(environment_ref, pressure.environment);
        if let Some(environment_name) = pressure.environment {
            let environment = find_element(
                &test_set,
                test_set.document_node(),
                "environment",
                Some(("name", environment_name)),
            )
            .expect("referenced for environment should exist");
            let source_file = find_element(&test_set, environment, "source", None)
                .and_then(|node| attribute(&test_set, node, "file"))
                .expect("for environment should name a source file");
            let source = fs::read(directory.join(source_file))
                .expect("read upstream for source and close handle");
            resources
                .admit(format!("urn:w3c:xslt30:{}:source", pressure.name), source)
                .expect("admit upstream for source");
        }

        let initial_template = find_element(&test_set, test_case, "initial-template", None)
            .and_then(|node| attribute(&test_set, node, "name"));
        assert_eq!(initial_template, pressure.initial_template);

        let assertion = find_element(&test_set, test_case, "assert-xml", None)
            .expect("for case should provide an XML assertion");
        if let Some(file) = attribute(&test_set, assertion, "file") {
            let expected = fs::read(directory.join(file))
                .expect("read upstream expected XML and close handle");
            assert!(!expected.is_empty());
        } else {
            assert!(!test_set.string_value(assertion).trim().is_empty());
        }
    }

    let snapshot = resources.seal();
    for pressure in CASES {
        let stylesheet_id = format!("urn:w3c:xslt30:{}:stylesheet", pressure.name);
        assert!(snapshot.get(&stylesheet_id).is_some());
        if pressure.execution == "engine-unsupported" {
            let failure = compile_resource(&snapshot, &stylesheet_id)
                .expect_err("engine-unsupported for case should fail compilation");
            assert_eq!(
                failure.category,
                FailureCategory::Unsupported,
                "{} should remain valid-but-unsupported: {failure:?}",
                pressure.name
            );
        }
    }
}
