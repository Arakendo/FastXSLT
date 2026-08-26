//! Conserved admission test for the complete XSLT30 `expr/for` denominator.

use std::{fs, path::PathBuf};

use super::{FailureCategory, compile_resource};
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
        execution: "engine-unsupported",
    },
    CasePressure {
        name: "for-002",
        environment: None,
        initial_template: Some("main"),
        execution: "harness-unsupported",
    },
    CasePressure {
        name: "for-003",
        environment: Some("for03"),
        initial_template: None,
        execution: "engine-unsupported",
    },
    CasePressure {
        name: "for-004",
        environment: Some("for03"),
        initial_template: None,
        execution: "engine-unsupported",
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
