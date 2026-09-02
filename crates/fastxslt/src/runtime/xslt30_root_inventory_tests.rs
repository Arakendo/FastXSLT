//! Conserved XSLT30 `fn/root` denominator and exact `root(.)` execution.

use std::{collections::BTreeMap, collections::BTreeSet, collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, compile_resource,
    execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const TEST_SET: &str = "tests/fn/root/_root-test-set.xml";
const PASSED_CASE: &str = "root-0101";
const OVERLAY: &str = include_str!("../../../../corpus/overlays/xslt30/root-denominator-v0.toml");

#[test]
fn inventories_complete_root_denominator_before_selection() {
    let document = load_test_set();
    let cases = element_children(&document, document_element(&document))
        .into_iter()
        .filter(|node| local_name(&document, *node) == "test-case")
        .collect::<Vec<_>>();
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<BTreeSet<_>>();

    assert_eq!(cases.len(), 10);
    assert_eq!(names.len(), cases.len());
    assert_eq!(names.first(), Some(&"root-0101"));
    assert_eq!(names.last(), Some(&"root-0601"));
    assert!(OVERLAY.contains(&format!("set_file = \"{TEST_SET}\"")));
    assert!(OVERLAY.contains("case_count = 10"));
    assert_eq!(OVERLAY.matches("[[case_override]]").count(), 1);
    assert!(names.contains(PASSED_CASE));
    assert!(OVERLAY.contains("case_name = \"root-0101\""));
    assert!(OVERLAY.contains("execution = \"passed\""));
}

#[test]
fn executes_unchanged_root_context_case() {
    let document = load_test_set();
    let case = case_named(&document, PASSED_CASE);
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
    let source_file = child_named(&document, environment, "source")
        .and_then(|node| attribute(&document, node, "file"))
        .expect("source file");
    let stylesheet_file = child_named(&document, case, "test")
        .and_then(|node| child_named(&document, node, "stylesheet"))
        .and_then(|node| attribute(&document, node, "file"))
        .expect("stylesheet file");
    let assertion = child_named(&document, case, "result")
        .and_then(|node| child_named(&document, node, "assert-xml"))
        .expect("XML assertion");
    let directory = corpus_directory();
    let expected = attribute(&document, assertion, "file").map_or_else(
        || document.string_value(assertion),
        |file| fs::read_to_string(directory.join(file)).expect("read expected XML"),
    );
    let source_id = format!("urn:w3c:xslt30:fn:root:{PASSED_CASE}:source");
    let stylesheet_id = format!("https://example.invalid/xslt30/fn/root/{stylesheet_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(
            source_id.clone(),
            fs::read(directory.join(source_file)).expect("read source and close handle"),
        )
        .expect("admit source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile root case");
    let mut set = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 16_384,
            work_limits: WorkLimits::unbounded(),
        },
    );
    set.add(TransformRequest {
        identity: PASSED_CASE.to_owned(),
        result_identity: format!("urn:w3c:xslt30:fn:root:{PASSED_CASE}:result"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit request");
    let results = execute_transform_set(set.seal()).expect("execute root case");
    assert_eq!(
        normalized_assert_xml(&results.by_request[PASSED_CASE].serialized),
        normalized_assert_xml(&expected)
    );
}

fn corpus_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/xslt30-test/tests/fn/root")
}

fn load_test_set() -> Document {
    let bytes = fs::read(corpus_directory().join("_root-test-set.xml"))
        .expect("read pinned test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:fn:root:test-set",
        &bytes,
        ParseLimits {
            max_events: 8_192,
            max_depth: 64,
        },
    )
    .expect("parse pinned test set");
    Document::from_parsed(parsed).expect("build test-set document")
}

fn case_named(document: &Document, name: &str) -> NodeId {
    element_children(document, document_element(document))
        .into_iter()
        .find(|node| {
            local_name(document, *node) == "test-case"
                && attribute(document, *node, "name") == Some(name)
        })
        .expect("pinned case")
}

fn document_element(document: &Document) -> NodeId {
    document
        .children(document.document_node())
        .iter()
        .copied()
        .find(|node| document.kind(*node) == NodeKind::Element)
        .expect("test-set document element")
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
    if xml.starts_with("<?xml") {
        xml.find("?>").map_or(xml, |end| &xml[end + 2..])
    } else {
        xml
    }
}

fn normalized_assert_xml(xml: &str) -> String {
    without_xml_declaration(xml.trim()).replace("\r\n", "\n")
}
