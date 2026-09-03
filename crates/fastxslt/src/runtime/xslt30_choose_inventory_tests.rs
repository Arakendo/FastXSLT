//! Conserved inventory and an executable slice of the XSLT30 `insn/choose` denominator.

use std::{collections::BTreeMap, collections::BTreeSet, collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, compile_resource,
    execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const TEST_SET: &str = "tests/insn/choose/_choose-test-set.xml";
const PASSED_CASES: [&str; 35] = [
    "choose-0101",
    "choose-0102",
    "choose-0201",
    "choose-0301",
    "choose-0401",
    "choose-0402",
    "choose-0403",
    "choose-0404",
    "choose-0501",
    "choose-0502",
    "choose-0601",
    "choose-0602",
    "choose-0603",
    "choose-0604",
    "choose-0605",
    "choose-0606",
    "choose-0609",
    "choose-0701",
    "choose-0702",
    "choose-0801",
    "choose-0901",
    "choose-1001",
    "choose-1101",
    "choose-1201",
    "choose-1202",
    "choose-1203",
    "choose-1204",
    "choose-1301",
    "choose-1401",
    "choose-1501",
    "choose-1502",
    "choose-1601",
    "choose-1703",
    "choose-1704",
    "choose-1706",
];
const ERROR_CASES: [&str; 4] = ["choose-1801", "choose-1802", "choose-1803", "choose-1804"];
const OVERLAY: &str = include_str!("../../../../corpus/overlays/xslt30/choose-denominator-v0.toml");

#[test]
fn inventories_complete_choose_denominator_before_selection() {
    let document = load_test_set();
    let cases = descendants_named(&document, document.document_node(), "test-case");
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<BTreeSet<_>>();

    assert_eq!(cases.len(), 55);
    assert_eq!(names.len(), cases.len());
    assert!(OVERLAY.contains(&format!("set_file = \"{TEST_SET}\"")));
    assert!(OVERLAY.contains("case_count = 55"));
    assert_eq!(OVERLAY.matches("[[case_override]]").count(), 39);
    for case_name in PASSED_CASES.into_iter().chain(ERROR_CASES) {
        assert!(names.contains(case_name));
        let record = overlay_case(case_name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains("execution = \"passed\""));
    }
}

#[test]
fn reports_unchanged_invalid_choose_structures() {
    let document = load_test_set();
    for case_name in ERROR_CASES {
        let case = case_named(&document, case_name);
        let expected_code = child_named(&document, case, "result")
            .and_then(|node| child_named(&document, node, "error"))
            .and_then(|node| attribute(&document, node, "code"))
            .expect("expected error code");
        let (snapshot, stylesheet_id, _) = sealed_case_resources(&document, case_name);
        let failure = compile_resource(&snapshot, &stylesheet_id)
            .expect_err("invalid choose structure must fail during compilation");
        assert_eq!(failure.code, expected_code, "{case_name}");
    }
}

#[test]
fn executes_unchanged_choose_and_if_cases() {
    for case_name in PASSED_CASES {
        execute_case(case_name);
    }
}

fn execute_case(case_name: &str) {
    let document = load_test_set();
    let case = case_named(&document, case_name);
    let result = child_named(&document, case, "result").expect("native result assertion");
    let (snapshot, stylesheet_id, source_id) = sealed_case_resources(&document, case_name);
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile choose case");
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
        result_identity: format!("urn:w3c:xslt30:insn:choose:{case_name}:result"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit transform request");
    let results = execute_transform_set(set.seal()).expect("execute choose case");
    let actual = &results.by_request[case_name].serialized;
    if let Some(assertion) = child_named(&document, result, "assert-xml") {
        assert_xml_equivalent(actual, &document.string_value(assertion), case_name);
    } else {
        let assertion = child_named(&document, result, "assert")
            .map(|node| document.string_value(node))
            .expect("admitted XML or exact root-string assertion");
        assert_root_string_equal(actual, &assertion, case_name);
    }
}

fn assert_root_string_equal(actual: &str, assertion: &str, case_name: &str) {
    let expected = assertion
        .trim()
        .strip_prefix("/out = \"")
        .and_then(|value| value.strip_suffix('"'))
        .expect("admitted exact /out string assertion");
    let actual_id = format!("urn:w3c:xslt30:insn:choose:{case_name}:actual");
    let parsed = parse_document(
        &actual_id,
        actual.as_bytes(),
        ParseLimits {
            max_events: 512,
            max_depth: 16,
        },
    )
    .expect("actual result XML");
    let actual = Document::from_parsed(parsed).expect("actual result XDM");
    let out = descendants_named(&actual, actual.document_node(), "out")
        .into_iter()
        .next()
        .expect("out result element");
    assert_eq!(actual.string_value(out), expected, "{case_name}");
}

fn sealed_case_resources(
    document: &Document,
    case_name: &str,
) -> (crate::resources::ResourceSnapshot, String, String) {
    let case = case_named(document, case_name);
    let environment_ref = child_named(document, case, "environment")
        .and_then(|node| attribute(document, node, "ref"))
        .expect("environment reference");
    let environment = descendants_named(document, document.document_node(), "environment")
        .into_iter()
        .find(|node| attribute(document, *node, "name") == Some(environment_ref))
        .expect("referenced environment");
    let source = child_named(document, environment, "source").expect("principal source");
    let stylesheet_file = child_named(document, case, "test")
        .and_then(|node| child_named(document, node, "stylesheet"))
        .and_then(|node| attribute(document, node, "file"))
        .expect("stylesheet file");
    let source_id = format!("urn:w3c:xslt30:insn:choose:{case_name}:source");
    let stylesheet_id = format!("https://example.invalid/xslt30/insn/choose/{stylesheet_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(source_id.clone(), source_bytes(document, source))
        .expect("admit principal source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(corpus_directory().join(stylesheet_file))
                .expect("read stylesheet and close handle"),
        )
        .expect("admit stylesheet");
    (resources.seal(), stylesheet_id, source_id)
}

fn case_named(document: &Document, case_name: &str) -> NodeId {
    descendants_named(document, document.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(document, *node, "name") == Some(case_name))
        .expect("pinned test case")
}

fn source_bytes(document: &Document, source: NodeId) -> Vec<u8> {
    if let Some(file) = attribute(document, source, "file") {
        return fs::read(corpus_directory().join(file)).expect("read source and close handle");
    }
    let content = child_named(document, source, "content").expect("inline source content");
    document.string_value(content).into_bytes()
}

fn overlay_case(case_name: &str) -> &str {
    OVERLAY
        .split("[[case_override]]")
        .find(|record| record.contains(&format!("case_name = \"{case_name}\"")))
        .expect("case override")
}

fn corpus_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/xslt30-test/tests/insn/choose")
}

fn load_test_set() -> Document {
    let bytes = fs::read(corpus_directory().join("_choose-test-set.xml"))
        .expect("read pinned test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:insn:choose:test-set",
        &bytes,
        ParseLimits {
            max_events: 16_384,
            max_depth: 64,
        },
    )
    .expect("parse pinned test set");
    Document::from_parsed(parsed).expect("build test-set document")
}

fn descendants_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in element_children(document, parent) {
        if local_name(document, child) == local {
            found.push(child);
        }
        found.extend(descendants_named(document, child, local));
    }
    found
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

fn assert_xml_equivalent(actual: &str, expected: &str, case_name: &str) {
    let limits = ParseLimits {
        max_events: 4_096,
        max_depth: 64,
    };
    let actual = Document::from_parsed(
        parse_document(
            &format!("urn:fastxslt:choose:{case_name}:actual"),
            actual.as_bytes(),
            limits,
        )
        .expect("actual result should parse"),
    )
    .expect("actual result should build");
    let expected = Document::from_parsed(
        parse_document(
            &format!("urn:fastxslt:choose:{case_name}:expected"),
            expected.as_bytes(),
            limits,
        )
        .expect("expected result should parse"),
    )
    .expect("expected result should build");
    assert_nodes_equal(
        &actual,
        actual.document_node(),
        &expected,
        expected.document_node(),
        case_name,
    );
}

fn assert_nodes_equal(
    actual: &Document,
    actual_node: NodeId,
    expected: &Document,
    expected_node: NodeId,
    case_name: &str,
) {
    assert_eq!(
        actual.kind(actual_node),
        expected.kind(expected_node),
        "{case_name}"
    );
    assert_eq!(
        actual.name(actual_node),
        expected.name(expected_node),
        "{case_name}"
    );
    assert_eq!(
        actual.value(actual_node),
        expected.value(expected_node),
        "{case_name}"
    );
    let actual_attributes = actual.attributes(actual_node);
    let expected_attributes = expected.attributes(expected_node);
    assert_eq!(
        actual_attributes.len(),
        expected_attributes.len(),
        "{case_name}"
    );
    for actual_attribute in actual_attributes {
        let name = actual.name(*actual_attribute);
        let value = actual.value(*actual_attribute);
        assert!(
            expected_attributes.iter().any(|expected_attribute| {
                expected.name(*expected_attribute) == name
                    && expected.value(*expected_attribute) == value
            }),
            "{case_name}: missing expected attribute {name:?}={value:?}"
        );
    }
    let actual_children = actual.children(actual_node);
    let expected_children = expected.children(expected_node);
    assert_eq!(
        actual_children.len(),
        expected_children.len(),
        "{case_name}"
    );
    for (actual_child, expected_child) in actual_children.iter().zip(expected_children) {
        assert_nodes_equal(actual, *actual_child, expected, *expected_child, case_name);
    }
}
