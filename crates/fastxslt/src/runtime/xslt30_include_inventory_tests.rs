//! Conserved preview of the complete XSLT30 `decl/include` denominator.

use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const CASE_NAMES: [&str; 16] = [
    "include-0101",
    "include-0102",
    "include-0103",
    "include-0104",
    "include-0105",
    "include-0201",
    "include-0202",
    "include-0301",
    "include-0401",
    "include-0501",
    "include-0601",
    "include-0701",
    "include-0702a",
    "include-0702b",
    "include-0702c",
    "include-0801",
];
const OVERLAY: &str =
    include_str!("../../../../corpus/overlays/xslt30/include-denominator-v0.toml");

#[test]
fn inventories_complete_include_denominator_without_admitting_dependency_semantics() {
    let document = load_test_set();
    let cases = element_children(&document, document_element(&document))
        .into_iter()
        .filter(|node| local_name(&document, *node) == "test-case")
        .collect::<Vec<_>>();
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<Vec<_>>();
    assert_eq!(names, CASE_NAMES);

    let mut primary_stylesheets = 0;
    let mut secondary_stylesheets = 0;
    let mut assertion_shapes = BTreeMap::new();
    for case in cases {
        let test = child_named(&document, case, "test").expect("test metadata");
        let stylesheets = element_children(&document, test)
            .into_iter()
            .filter(|node| local_name(&document, *node) == "stylesheet")
            .collect::<Vec<_>>();
        let primary_count = stylesheets
            .iter()
            .filter(|stylesheet| attribute(&document, **stylesheet, "role") != Some("secondary"))
            .count();
        assert_eq!(primary_count, 1, "each case has one principal stylesheet");
        primary_stylesheets += primary_count;
        secondary_stylesheets += stylesheets.len() - primary_count;

        let result = child_named(&document, case, "result").expect("result metadata");
        let assertion = first_element_child(&document, result).expect("result assertion");
        *assertion_shapes
            .entry(local_name(&document, assertion).to_owned())
            .or_insert(0usize) += 1;
    }

    assert_eq!(primary_stylesheets, 16);
    assert_eq!(secondary_stylesheets, 34);
    assert_eq!(
        assertion_shapes,
        BTreeMap::from([
            ("any-of".to_owned(), 1),
            ("assert-xml".to_owned(), 14),
            ("error".to_owned(), 1),
        ])
    );

    assert!(OVERLAY.contains("case_count = 16"));
    assert!(OVERLAY.contains("selection = \"harness-unsupported\""));
    assert!(OVERLAY.contains("execution = \"not-run\""));
    for case_name in CASE_NAMES {
        assert!(names.contains(&case_name));
    }
}

fn load_test_set() -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/include/_include-test-set.xml");
    let bytes = fs::read(path).expect("read pinned include test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:decl:include:test-set",
        &bytes,
        ParseLimits {
            max_events: 20_000,
            max_depth: 64,
        },
    )
    .expect("parse pinned include test set");
    Document::from_parsed(parsed).expect("build include test-set document")
}

fn document_element(document: &Document) -> NodeId {
    first_element_child(document, document.document_node()).expect("test-set document element")
}

fn element_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|node| document.kind(*node) == NodeKind::Element)
        .collect()
}

fn child_named(document: &Document, parent: NodeId, local: &str) -> Option<NodeId> {
    element_children(document, parent)
        .into_iter()
        .find(|node| local_name(document, *node) == local)
}

fn first_element_child(document: &Document, parent: NodeId) -> Option<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .find(|node| document.kind(*node) == NodeKind::Element)
}

fn local_name(document: &Document, node: NodeId) -> &str {
    &document.name(node).expect("element name").local
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}
