//! Conserved inventory of the complete XSLT30 `insn/apply-templates` denominator.

use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const CASE_NAMES: [&str; 50] = [
    "apply-templates-001",
    "apply-templates-002",
    "conflict-resolution-0101",
    "conflict-resolution-0102a",
    "conflict-resolution-0102b",
    "conflict-resolution-0102c",
    "conflict-resolution-0104a",
    "conflict-resolution-0104b",
    "conflict-resolution-0104c",
    "conflict-resolution-0106",
    "conflict-resolution-0107",
    "conflict-resolution-0108a",
    "conflict-resolution-0108b",
    "conflict-resolution-0108c",
    "conflict-resolution-0110a",
    "conflict-resolution-0110b",
    "conflict-resolution-0110c",
    "conflict-resolution-0112",
    "conflict-resolution-0201",
    "conflict-resolution-0401a",
    "conflict-resolution-0401b",
    "conflict-resolution-0401c",
    "conflict-resolution-0501",
    "conflict-resolution-0502",
    "conflict-resolution-0503",
    "conflict-resolution-0601",
    "conflict-resolution-0701",
    "conflict-resolution-0702",
    "conflict-resolution-0703",
    "conflict-resolution-0801",
    "conflict-resolution-0802",
    "conflict-resolution-0901",
    "conflict-resolution-1001",
    "conflict-resolution-1101",
    "conflict-resolution-1102",
    "conflict-resolution-1201",
    "conflict-resolution-1202a",
    "conflict-resolution-1202b",
    "conflict-resolution-1202c",
    "conflict-resolution-1204",
    "conflict-resolution-1205",
    "conflict-resolution-1301",
    "conflict-resolution-1401",
    "conflict-resolution-1402",
    "conflict-resolution-1501",
    "conflict-resolution-1601",
    "conflict-resolution-1602",
    "conflict-resolution-1603",
    "conflict-resolution-1701",
    "conflict-resolution-1801",
];

const PASSED_CASES: [&str; 41] = [
    "apply-templates-001",
    "apply-templates-002",
    "conflict-resolution-0101",
    "conflict-resolution-0102a",
    "conflict-resolution-0102c",
    "conflict-resolution-0104a",
    "conflict-resolution-0104c",
    "conflict-resolution-0106",
    "conflict-resolution-0107",
    "conflict-resolution-0108a",
    "conflict-resolution-0108c",
    "conflict-resolution-0110a",
    "conflict-resolution-0110c",
    "conflict-resolution-0112",
    "conflict-resolution-0201",
    "conflict-resolution-0401a",
    "conflict-resolution-0401c",
    "conflict-resolution-0501",
    "conflict-resolution-0502",
    "conflict-resolution-0503",
    "conflict-resolution-0601",
    "conflict-resolution-0701",
    "conflict-resolution-0702",
    "conflict-resolution-0703",
    "conflict-resolution-0801",
    "conflict-resolution-0802",
    "conflict-resolution-0901",
    "conflict-resolution-1001",
    "conflict-resolution-1101",
    "conflict-resolution-1102",
    "conflict-resolution-1201",
    "conflict-resolution-1202a",
    "conflict-resolution-1202c",
    "conflict-resolution-1204",
    "conflict-resolution-1205",
    "conflict-resolution-1501",
    "conflict-resolution-1601",
    "conflict-resolution-1602",
    "conflict-resolution-1603",
    "conflict-resolution-1701",
    "conflict-resolution-1801",
];

const OVERLAY: &str =
    include_str!("../../../../corpus/overlays/xslt30/apply-templates-denominator-v0.toml");

#[test]
fn inventories_complete_apply_templates_denominator_with_explicit_dispositions() {
    let document = load_test_set();
    let root = first_element_child(&document, document.document_node()).expect("test-set root");
    let cases = element_children(&document, root)
        .into_iter()
        .filter(|node| local_name(&document, *node) == "test-case")
        .collect::<Vec<_>>();
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<Vec<_>>();
    assert_eq!(names, CASE_NAMES);

    let mut assertion_shapes = BTreeMap::new();
    let mut principal_stylesheets = 0;
    let mut secondary_stylesheets = 0;
    for case in &cases {
        let test = child_named(&document, *case, "test").expect("test metadata");
        let stylesheets = element_children(&document, test)
            .into_iter()
            .filter(|node| local_name(&document, *node) == "stylesheet")
            .collect::<Vec<_>>();
        let primary_count = stylesheets
            .iter()
            .filter(|stylesheet| attribute(&document, **stylesheet, "role") != Some("secondary"))
            .count();
        assert_eq!(primary_count, 1, "each case has one principal stylesheet");
        principal_stylesheets += primary_count;
        secondary_stylesheets += stylesheets.len() - primary_count;

        let result = child_named(&document, *case, "result").expect("result metadata");
        let assertion = first_element_child(&document, result).expect("result assertion");
        *assertion_shapes
            .entry(local_name(&document, assertion).to_owned())
            .or_insert(0usize) += 1;
    }
    assert_eq!(
        assertion_shapes,
        BTreeMap::from([
            ("all-of".to_owned(), 1),
            ("assert-xml".to_owned(), 41),
            ("error".to_owned(), 8),
        ])
    );
    assert_eq!(principal_stylesheets, 50);
    assert_eq!(secondary_stylesheets, 1);

    assert!(OVERLAY.contains("case_count = 50"));
    assert!(OVERLAY.contains("selection = \"harness-unsupported\""));
    assert!(OVERLAY.contains("execution = \"not-run\""));
    assert_eq!(
        OVERLAY.matches("[[case_override]]").count(),
        PASSED_CASES.len()
    );
    for case_name in PASSED_CASES {
        assert!(names.contains(&case_name));
        let override_record = OVERLAY
            .split("[[case_override]]")
            .find(|section| section.contains(&format!("case_name = \"{case_name}\"")))
            .expect("passed case override");
        assert!(override_record.contains("selection = \"selected\""));
        assert!(override_record.contains("execution = \"passed\""));
    }
}

fn load_test_set() -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/insn/apply-templates/_apply-templates-test-set.xml");
    let bytes = fs::read(path).expect("read pinned apply-templates test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:insn:apply-templates:test-set",
        &bytes,
        ParseLimits {
            max_events: 20_000,
            max_depth: 64,
        },
    )
    .expect("parse pinned apply-templates test set");
    Document::from_parsed(parsed).expect("build apply-templates test-set document")
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
    element_children(document, parent).into_iter().next()
}

fn local_name(document: &Document, node: NodeId) -> &str {
    &document.name(node).expect("element name").local
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|candidate| {
        document
            .name(*candidate)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*candidate))
    })
}
