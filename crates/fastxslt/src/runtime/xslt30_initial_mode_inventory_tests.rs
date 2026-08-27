//! Conserved admission for the complete XSLT30 `misc/initial-mode` denominator.

use std::{fs, path::PathBuf};

use super::{FailureCategory, compile_resource};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "tests/misc/initial-mode/_initial-mode-test-set.xml";

struct CaseMetadata {
    name: &'static str,
    spec: Option<&'static str>,
    mode: &'static str,
    assertion: &'static str,
    error: Option<&'static str>,
    compile_code: &'static str,
}

const CASES: [CaseMetadata; 5] = [
    CaseMetadata {
        name: "initial-mode-001",
        spec: Some("XSLT20+"),
        mode: "inimode",
        assertion: "assert-xml",
        error: None,
        compile_code: "FXST1009",
    },
    CaseMetadata {
        name: "initial-mode-002",
        spec: Some("XSLT10+"),
        mode: "inimode",
        assertion: "error",
        error: Some("XTDE0045"),
        compile_code: "FXST1009",
    },
    CaseMetadata {
        name: "initial-mode-003",
        spec: Some("XSLT20+"),
        mode: "inimode",
        assertion: "error",
        error: Some("XTDE0050"),
        compile_code: "FXST1009",
    },
    CaseMetadata {
        name: "initial-mode-004",
        spec: Some("XSLT30+"),
        mode: "flobble",
        assertion: "assert-xml",
        error: None,
        compile_code: "FXST1011",
    },
    CaseMetadata {
        name: "initial-mode-005",
        spec: None,
        mode: "b",
        assertion: "assert-xml",
        error: None,
        compile_code: "FXST1015",
    },
];

#[test]
fn admits_complete_initial_mode_denominator_and_reaches_engine_boundaries() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let (test_set, set_path) = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(cases.len(), CASES.len());
    let directory = set_path.parent().expect("initial-mode test-set directory");

    for metadata in &CASES {
        let record = overlay_case(overlay, metadata.name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains("execution = \"engine-unsupported\""));

        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(metadata.name))
            .expect("overlay case identity should exist upstream");
        assert_eq!(dependency(&test_set, case, "spec"), metadata.spec);
        let test = child_named(&test_set, case, "test").expect("test metadata");
        let initial_mode =
            child_named(&test_set, test, "initial-mode").expect("initial-mode entry metadata");
        assert_eq!(
            attribute(&test_set, initial_mode, "name"),
            Some(metadata.mode)
        );
        let result = child_named(&test_set, case, "result").expect("result metadata");
        let assertion = first_element_child(&test_set, result).expect("result assertion");
        assert_eq!(local_name(&test_set, assertion), metadata.assertion);
        assert_eq!(attribute(&test_set, assertion, "code"), metadata.error);

        let stylesheet_file = child_named(&test_set, test, "stylesheet")
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("stylesheet file");
        let stylesheet = fs::read(directory.join(stylesheet_file))
            .expect("read stylesheet and close upstream handle");
        let stylesheet_id = format!("urn:w3c:xslt30:{}:stylesheet", metadata.name);
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 65_536, 65_536));
        resources
            .admit(stylesheet_id.clone(), stylesheet)
            .expect("admit stylesheet bytes");
        let snapshot = resources.seal();
        let failure = compile_resource(&snapshot, &stylesheet_id)
            .expect_err("initial-mode case should reach an explicit engine gap");
        assert_eq!(failure.category, FailureCategory::Unsupported);
        assert_eq!(failure.code, metadata.compile_code);
    }
}

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test")
        .join(SET_FILE);
    let bytes = fs::read(&path).expect("read pinned initial-mode test set");
    let parsed = parse_document(
        "urn:w3c:xslt30:initial-mode:test-set",
        &bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
    )
    .expect("parse pinned initial-mode test set");
    (
        Document::from_parsed(parsed).expect("build initial-mode catalog document"),
        path,
    )
}

fn overlay_case<'a>(overlay: &'a str, name: &str) -> &'a str {
    overlay
        .split("[[case]]")
        .find(|section| {
            section.contains(&format!("set_file = \"{SET_FILE}\""))
                && section.contains(&format!("case_name = \"{name}\""))
        })
        .expect("overlay must contain one initial-mode disposition")
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
    document.name(node).map_or("", |name| name.local.as_str())
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}

fn dependency<'a>(document: &'a Document, case: NodeId, name: &str) -> Option<&'a str> {
    child_named(document, case, "dependencies")
        .and_then(|dependencies| child_named(document, dependencies, name))
        .and_then(|dependency| attribute(document, dependency, "value"))
}
