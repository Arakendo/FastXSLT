//! Conserved admission and execution for the complete XSLT30 `fn/deep-equal` denominator.

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

const SET_FILE: &str = "tests/fn/deep-equal/_deep-equal-test-set.xml";
const CASES: [&str; 2] = ["deep-equal-001", "deep-equal-002"];

#[test]
fn admits_complete_deep_equal_denominator_without_loss() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let (test_set, _) = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(cases.len(), CASES.len());

    for name in CASES {
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned deep-equal case");
        assert_eq!(dependency(&test_set, case, "spec"), Some("XSLT20+"));
        assert_eq!(
            child_named(&test_set, case, "environment")
                .and_then(|node| attribute(&test_set, node, "ref")),
            Some("deepeq01")
        );
        let assertion = child_named(&test_set, case, "result")
            .and_then(|node| first_element_child(&test_set, node))
            .expect("result assertion");
        assert_eq!(local_name(&test_set, assertion), "assert-xml");
        let record = overlay_case(overlay, name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains("execution = \"native-pass\""));
    }
}

#[test]
fn executes_attribute_deep_equality_without_using_node_identity() {
    let (actual, expected) = execute_case("deep-equal-001");
    assert_eq!(
        expected,
        "<out><true>true</true><false>false</false><false>false</false></out>"
    );
    assert_eq!(
        actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out><true>true</true><false>false</false><false>false</false></out>"
    );
}

#[test]
fn executes_comment_deep_equality_by_kind_and_value() {
    let (actual, expected) = execute_case("deep-equal-002");
    assert_eq!(expected, "<out><true>true</true><false>false</false></out>");
    assert_eq!(
        actual,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out><true>true</true><false>false</false></out>"
    );
}

fn execute_case(name: &str) -> (String, String) {
    let (test_set, set_path) = load_test_set();
    let case = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some(name))
        .expect("pinned deep-equal case");
    let environment = descendants_named(&test_set, test_set.document_node(), "environment")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some("deepeq01"))
        .expect("deep-equal environment");
    let source = child_named(&test_set, environment, "source")
        .and_then(|node| child_named(&test_set, node, "content"))
        .map(|node| test_set.string_value(node).into_bytes())
        .expect("inline source content");
    let test = child_named(&test_set, case, "test").expect("test metadata");
    let stylesheet_file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("stylesheet file");
    let stylesheet = fs::read(
        set_path
            .parent()
            .expect("test-set directory")
            .join(stylesheet_file),
    )
    .expect("read pinned stylesheet");
    let expected = child_named(&test_set, case, "result")
        .and_then(|node| child_named(&test_set, node, "assert-xml"))
        .map(|node| test_set.string_value(node).trim().to_owned())
        .expect("inline XML assertion");

    let source_id = format!("urn:w3c:xslt30:{name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{name}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(source_id.clone(), source)
        .expect("admit source");
    resources
        .admit(stylesheet_id.clone(), stylesheet)
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile deep-equal case");
    let mut builder = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 4_096,
            work_limits: WorkLimits::unbounded(),
        },
    );
    builder
        .add(TransformRequest {
            identity: name.to_owned(),
            result_identity: format!("result:{name}"),
            entry: InvocationEntry::PrincipalSource {
                resource: source_id,
            },
            parameters: BTreeMap::default(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit deep-equal request");
    let result = execute_transform_set(builder.seal()).expect("execute deep-equal case");
    (result.by_request[name].serialized.clone(), expected)
}

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test")
        .join(SET_FILE);
    let bytes = fs::read(&path).expect("read pinned deep-equal test set");
    let parsed = parse_document(
        "urn:w3c:xslt30:deep-equal:test-set",
        &bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
    )
    .expect("parse deep-equal test set");
    (
        Document::from_parsed(parsed).expect("build deep-equal catalog document"),
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
        .expect("overlay must contain one deep-equal disposition")
}

fn dependency<'a>(document: &'a Document, case: NodeId, kind: &str) -> Option<&'a str> {
    descendants_named(document, case, "dependencies")
        .into_iter()
        .flat_map(|node| document.children(node).iter().copied())
        .find(|node| {
            document.kind(*node) == NodeKind::Element && local_name(document, *node) == kind
        })
        .and_then(|node| attribute(document, node, "value"))
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
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
    &document.name(node).expect("element node has a name").local
}
