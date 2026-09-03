//! Conserved admission and execution for the XSLT30 `expr/data-manipulation` denominator.

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
use crate::xslt30_overlay_test_support::{
    assert_private_case_passed, assert_private_set_case_names,
};

const SET_FILE: &str = "tests/expr/data-manipulation/_data-manipulation-test-set.xml";
const CASE_COUNT: usize = 28;
const PASSING_CASE_COUNT: usize = 28;

#[test]
fn executes_complete_native_data_manipulation_test_set() {
    for ordinal in 1..=PASSING_CASE_COUNT {
        let case_name = case_name(ordinal);
        let (actual, expected) = execute_case(&case_name);
        assert_xml_text(&actual, &expected, &case_name);
    }
}

#[test]
fn admits_complete_data_manipulation_test_set_without_denominator_loss() {
    let case_names = (1..=CASE_COUNT).map(case_name).collect::<Vec<_>>();
    let case_name_refs = case_names.iter().map(String::as_str).collect::<Vec<_>>();
    assert_private_set_case_names(SET_FILE, &case_name_refs);

    let (test_set, set_path) = load_test_set();
    let test_cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(test_cases.len(), CASE_COUNT);
    let directory = set_path.parent().expect("test-set directory");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(56, 65_536, 1_048_576));

    for ordinal in 1..=CASE_COUNT {
        let name = case_name(ordinal);
        assert_private_case_passed(SET_FILE, &name);

        let case = test_cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(&name))
            .expect("overlay identity should exist in the pinned test set");
        assert_eq!(dependency(&test_set, case, "spec"), vec!["XSLT10+"]);
        let result = child_named(&test_set, case, "result").expect("case result");
        let assertion = first_element_child(&test_set, result).expect("case assertion");
        assert_eq!(local_name(&test_set, assertion), "assert-xml");
        verify_expected_resource(&test_set, assertion, directory);

        let test = child_named(&test_set, case, "test").expect("case test");
        let stylesheet_file = child_named(&test_set, test, "stylesheet")
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("case stylesheet file");
        resources
            .admit(
                stylesheet_id(&name),
                fs::read(directory.join(stylesheet_file))
                    .expect("read stylesheet and close handle"),
            )
            .expect("admit stylesheet");
        let environment_name = child_named(&test_set, case, "environment")
            .and_then(|node| attribute(&test_set, node, "ref"))
            .expect("referenced environment");
        let source = referenced_source_bytes(&test_set, environment_name, directory);
        resources
            .admit(source_id(&name), source)
            .expect("admit logical source");
    }
    let _snapshot = resources.seal();
}

fn execute_case(case_name: &str) -> (String, String) {
    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("test-set directory");
    let case = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some(case_name))
        .expect("native case");
    let test = child_named(&test_set, case, "test").expect("case test");
    let stylesheet_file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("stylesheet file");
    let environment_name = child_named(&test_set, case, "environment")
        .and_then(|node| attribute(&test_set, node, "ref"))
        .expect("referenced environment");
    let result = child_named(&test_set, case, "result").expect("case result");
    let assertion = first_element_child(&test_set, result).expect("case assertion");
    let expected = expected_text(&test_set, assertion, directory);

    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(
            source_id(case_name),
            referenced_source_bytes(&test_set, environment_name, directory),
        )
        .expect("admit source");
    resources
        .admit(
            stylesheet_id(case_name),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, &stylesheet_id(case_name)).expect("compile native case");
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
        identity: case_name.to_owned(),
        result_identity: format!("result:{case_name}"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id(case_name),
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit request");
    let results = execute_transform_set(set.seal()).expect("execute native case");
    (results.by_request[case_name].serialized.clone(), expected)
}

fn referenced_source_bytes(
    document: &Document,
    name: &str,
    directory: &std::path::Path,
) -> Vec<u8> {
    let environment = descendants_named(document, document.document_node(), "environment")
        .into_iter()
        .find(|node| attribute(document, *node, "name") == Some(name))
        .expect("referenced environment should exist");
    let source = child_named(document, environment, "source").expect("environment source");
    if let Some(file) = attribute(document, source, "file") {
        fs::read(directory.join(file)).expect("read source and close handle")
    } else {
        let content = child_named(document, source, "content").expect("inline source content");
        document.string_value(content).into_bytes()
    }
}

fn expected_text(document: &Document, assertion: NodeId, directory: &std::path::Path) -> String {
    attribute(document, assertion, "file").map_or_else(
        || document.string_value(assertion),
        |file| fs::read_to_string(directory.join(file)).expect("read expected and close handle"),
    )
}

fn verify_expected_resource(document: &Document, assertion: NodeId, directory: &std::path::Path) {
    if let Some(file) = attribute(document, assertion, "file") {
        assert!(
            fs::metadata(directory.join(file))
                .expect("expected file")
                .len()
                > 0
        );
    } else {
        assert!(!document.string_value(assertion).is_empty());
    }
}

fn assert_xml_text(actual: &str, expected: &str, case_name: &str) {
    for (identity, xml) in [("actual", actual), ("expected", expected)] {
        parse_document(
            &format!("urn:fastxslt:{case_name}:{identity}"),
            xml.trim().as_bytes(),
            ParseLimits {
                max_events: 256,
                max_depth: 32,
            },
        )
        .unwrap_or_else(|failure| panic!("{identity} XML should parse: {failure:?}"));
    }
    assert_eq!(
        without_xml_declaration(actual.trim()),
        without_xml_declaration(expected.trim()),
        "{case_name}"
    );
}

fn without_xml_declaration(xml: &str) -> &str {
    if xml.starts_with("<?xml") {
        xml.find("?>").map_or(xml, |end| &xml[end + 2..])
    } else {
        xml
    }
}

fn case_name(ordinal: usize) -> String {
    format!("data-manipulation-{ordinal:03}")
}

fn stylesheet_id(case_name: &str) -> String {
    format!("urn:w3c:xslt30:{case_name}:stylesheet")
}

fn source_id(case_name: &str) -> String {
    format!("urn:w3c:xslt30:{case_name}:source")
}

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../vendor/xslt30-test/tests/expr/data-manipulation/_data-manipulation-test-set.xml",
    );
    let bytes = fs::read(&path).expect("read pinned test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:expr:data-manipulation:test-set",
        &bytes,
        ParseLimits {
            max_events: 4_096,
            max_depth: 64,
        },
    )
    .expect("parse pinned test set");
    (
        Document::from_parsed(parsed).expect("build test-set document"),
        path,
    )
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

fn dependency<'a>(document: &'a Document, case: NodeId, kind: &str) -> Vec<&'a str> {
    child_named(document, case, "dependencies")
        .map(|dependencies| {
            document
                .children(dependencies)
                .iter()
                .copied()
                .filter(|node| {
                    document.kind(*node) == NodeKind::Element && local_name(document, *node) == kind
                })
                .filter_map(|node| attribute(document, node, "value"))
                .collect()
        })
        .unwrap_or_default()
}
