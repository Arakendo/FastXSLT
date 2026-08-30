//! Pinned XSLT30 evidence for expanded-QName mode identity.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
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

const TEST_SET: &str = "tests/attr/mode/_mode-test-set.xml";
const SELECTED_CASES: [&str; 4] = ["mode-0105", "mode-0106", "mode-0107", "mode-0108"];
const OVERLAY: &str = include_str!("../../../../corpus/overlays/xslt30/mode-denominator-v0.toml");

#[test]
fn inventories_the_complete_mode_denominator_before_selection() {
    let document = load_test_set();
    let cases = element_children(&document, document_element(&document))
        .into_iter()
        .filter(|node| local_name(&document, *node) == "test-case")
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 169);
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<BTreeSet<_>>();
    assert_eq!(names.len(), cases.len());
    assert_eq!(names.first(), Some(&"mode-0001"));
    assert_eq!(names.last(), Some(&"mode-1905"));
    assert!(OVERLAY.contains("case_count = 169"));
    assert!(OVERLAY.contains("selection = \"harness-unsupported\""));
    assert_eq!(OVERLAY.matches("[[case_override]]").count(), 4);
    for case_name in SELECTED_CASES {
        assert!(names.contains(case_name));
        assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    }
}

#[test]
fn executes_qualified_and_unqualified_mode_names_as_distinct_expanded_qnames() {
    let qualified = execute_case("mode-0105");
    let unqualified = execute_case("mode-0106");
    assert_eq!(
        without_xml_declaration(qualified.0.trim()),
        qualified.1.trim()
    );
    assert_eq!(
        without_xml_declaration(unqualified.0.trim()),
        unqualified.1.trim()
    );
    assert!(qualified.0.contains("mode-foo:a:a-text"));
    assert!(unqualified.0.contains("mode-a:a-text"));
}

#[test]
fn executes_mode_0107_from_a_global_temporary_document_focus() {
    let (actual, expected) = execute_case("mode-0107");
    assert_xml_equivalent(&actual, &expected);
}

#[test]
fn executes_mode_0108_with_for_each_temporary_document_focus() {
    let (actual, expected) = execute_case("mode-0108");
    assert_xml_equivalent(&actual, &expected);
}

fn execute_case(case_name: &str) -> (String, String) {
    let private_overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(private_overlay.contains(&format!("case_name = \"{case_name}\"")));
    let document = load_test_set();
    let case = element_children(&document, document_element(&document))
        .into_iter()
        .find(|node| attribute(&document, *node, "name") == Some(case_name))
        .expect("selected mode case");
    let environment_ref = child_named(&document, case, "environment")
        .and_then(|node| attribute(&document, node, "ref"))
        .expect("environment reference");
    let environment = element_children(&document, document_element(&document))
        .into_iter()
        .find(|node| {
            local_name(&document, *node) == "environment"
                && attribute(&document, *node, "name") == Some(environment_ref)
        })
        .expect("referenced environment");
    let content = child_named(
        &document,
        child_named(&document, environment, "source").expect("principal source"),
        "content",
    )
    .expect("inline source content");
    let stylesheet_file = child_named(
        &document,
        child_named(&document, case, "test").expect("test metadata"),
        "stylesheet",
    )
    .and_then(|node| attribute(&document, node, "file"))
    .expect("stylesheet file");
    let expected = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "assert-xml",
    )
    .map(|node| document.string_value(node))
    .expect("XML assertion");

    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/xslt30-test/tests/attr/mode");
    let source_id = format!("urn:w3c:xslt30:attr:mode:{case_name}:source");
    let stylesheet_id = format!("https://example.invalid/xslt30/attr/mode/{stylesheet_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            source_id.clone(),
            document.string_value(content).into_bytes(),
        )
        .expect("admit source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile selected mode case");
    if matches!(case_name, "mode-0105" | "mode-0106") {
        assert!(program.matched_templates.iter().any(|template| {
            template
                .modes
                .iter()
                .any(|mode| mode == "Q{http://foo.com}a")
        }));
    }
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
    .expect("admit mode request");
    let results = execute_transform_set(set.seal()).expect("execute selected mode case");
    (results.by_request[case_name].serialized.clone(), expected)
}

fn load_test_set() -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test")
        .join(TEST_SET);
    let bytes = fs::read(path).expect("read pinned mode test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:attr:mode:test-set",
        &bytes,
        ParseLimits {
            max_events: 65_536,
            max_depth: 64,
        },
    )
    .expect("parse mode test set");
    Document::from_parsed(parsed).expect("build mode test-set document")
}

fn document_element(document: &Document) -> NodeId {
    element_children(document, document.document_node())[0]
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

fn without_xml_declaration(xml: &str) -> &str {
    xml.strip_prefix("<?xml version=\"1.0\" encoding=\"UTF-8\"?>")
        .unwrap_or(xml)
}

fn assert_xml_equivalent(actual: &str, expected: &str) {
    let limits = ParseLimits {
        max_events: 64,
        max_depth: 16,
    };
    let actual = Document::from_parsed(
        parse_document("urn:fastxslt:mode:actual", actual.as_bytes(), limits)
            .expect("actual mode result should parse"),
    )
    .expect("actual mode result should build");
    let expected = Document::from_parsed(
        parse_document("urn:fastxslt:mode:expected", expected.as_bytes(), limits)
            .expect("expected mode result should parse"),
    )
    .expect("expected mode result should build");
    assert_xml_nodes_equal(
        &actual,
        actual.document_node(),
        &expected,
        expected.document_node(),
    );
}

fn assert_xml_nodes_equal(
    actual: &Document,
    actual_node: NodeId,
    expected: &Document,
    expected_node: NodeId,
) {
    assert_eq!(actual.kind(actual_node), expected.kind(expected_node));
    assert_eq!(actual.name(actual_node), expected.name(expected_node));
    assert_eq!(actual.value(actual_node), expected.value(expected_node));
    assert_eq!(
        actual.attributes(actual_node).len(),
        expected.attributes(expected_node).len()
    );
    let actual_children = actual.children(actual_node);
    let expected_children = expected.children(expected_node);
    assert_eq!(actual_children.len(), expected_children.len());
    for (actual_child, expected_child) in actual_children.iter().zip(expected_children) {
        assert_xml_nodes_equal(actual, *actual_child, expected, *expected_child);
    }
}
