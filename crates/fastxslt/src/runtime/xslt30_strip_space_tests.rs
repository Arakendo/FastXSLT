//! Conserved XSLT30 `decl/strip-space` denominator and exact strip-all execution.

use std::{collections::BTreeMap, collections::BTreeSet, collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, compile_resource,
    execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt30_overlay_test_support::{
    DenominatorIdentity, ExecutionDisposition, SelectionDisposition,
    assert_denominator_default_disposition, assert_denominator_override_names,
    assert_strip_space_case_passed,
};

const CASE_NAME: &str = "strip-space-012";
#[test]
fn inventories_complete_strip_space_denominator_before_selection() {
    let document = load_test_set();
    let cases = element_children(&document, document_element(&document))
        .into_iter()
        .filter(|node| local_name(&document, *node) == "test-case")
        .collect::<Vec<_>>();
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<BTreeSet<_>>();

    assert_eq!(cases.len(), 30);
    assert_eq!(names.len(), cases.len());
    assert_eq!(names.first(), Some(&"strip-space-001"));
    assert_eq!(names.last(), Some(&"strip-space-029"));
    assert!(names.contains(CASE_NAME));
    assert_denominator_override_names(DenominatorIdentity::StripSpace, &[CASE_NAME]);
    assert_denominator_default_disposition(
        DenominatorIdentity::StripSpace,
        SelectionDisposition::HarnessUnsupported,
        ExecutionDisposition::NotRun,
    );
    assert_strip_space_case_passed(CASE_NAME);
}

#[test]
fn executes_unchanged_strip_space_012_through_visibility_view() {
    assert_strip_space_case_passed(CASE_NAME);
    let document = load_test_set();
    let case = case_named(&document, CASE_NAME);
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
    let source_content = child_named(&document, environment, "source")
        .and_then(|node| child_named(&document, node, "content"))
        .map(|node| document.string_value(node))
        .expect("inline principal source");
    let stylesheet_file = child_named(&document, case, "test")
        .and_then(|node| child_named(&document, node, "stylesheet"))
        .and_then(|node| attribute(&document, node, "file"))
        .expect("stylesheet file");
    let expected = child_named(&document, case, "result")
        .and_then(|node| child_named(&document, node, "assert-xml"))
        .map(|node| document.string_value(node))
        .expect("XML assertion");

    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/strip-space");
    let stylesheet_id =
        format!("https://example.invalid/xslt30/decl/strip-space/{stylesheet_file}");
    let source_id = format!("urn:w3c:xslt30:decl:strip-space:{CASE_NAME}:source");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 16_384, 32_768));
    resources
        .admit(source_id.clone(), source_content.into_bytes())
        .expect("admit inline source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file))
                .expect("read stylesheet and close source handle"),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile strip-space case");
    let mut set = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 8_192,
            work_limits: WorkLimits::unbounded(),
        },
    );
    set.add(TransformRequest {
        identity: CASE_NAME.to_owned(),
        result_identity: format!("urn:w3c:xslt30:decl:strip-space:{CASE_NAME}:result"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit strip-space request");
    let results = execute_transform_set(set.seal()).expect("execute strip-space case");
    let actual = &results.by_request[CASE_NAME].serialized;
    assert_eq!(without_xml_declaration(actual.trim()), expected.trim());
}

fn load_test_set() -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/strip-space/_strip-space-test-set.xml");
    let bytes = fs::read(path).expect("read pinned strip-space test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:decl:strip-space:test-set",
        &bytes,
        ParseLimits {
            max_events: 30_000,
            max_depth: 64,
        },
    )
    .expect("parse pinned strip-space test set");
    Document::from_parsed(parsed).expect("build strip-space test-set document")
}

fn case_named(document: &Document, name: &str) -> NodeId {
    element_children(document, document_element(document))
        .into_iter()
        .find(|node| {
            local_name(document, *node) == "test-case"
                && attribute(document, *node, "name") == Some(name)
        })
        .expect("pinned strip-space case")
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
