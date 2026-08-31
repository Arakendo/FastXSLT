//! Pinned XSLT30 evidence for expanded-QName mode identity.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::PathBuf,
};

use super::{
    ExecutionPolicy, InvocationEntry, InvocationParameter, TransformRequest, TransformSetBuilder,
    compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits, parse_document};

const TEST_SET: &str = "tests/attr/mode/_mode-test-set.xml";
const SELECTED_CASES: [&str; 25] = [
    "mode-0101",
    "mode-0102",
    "mode-0103",
    "mode-0104",
    "mode-0105",
    "mode-0106",
    "mode-0107",
    "mode-0108",
    "mode-0201",
    "mode-0301",
    "mode-0401",
    "mode-0501",
    "mode-0601",
    "mode-0701",
    "mode-0901",
    "mode-1001",
    "mode-1101",
    "mode-1102",
    "mode-1103",
    "mode-1104",
    "mode-1105",
    "mode-1201",
    "mode-1202",
    "mode-1203",
    "mode-1204",
];
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
    assert_eq!(
        OVERLAY.matches("[[case_override]]").count(),
        SELECTED_CASES.len()
    );
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

#[test]
fn executes_basic_mode_selection_isolation_and_builtin_rules() {
    for case_name in [
        "mode-0101",
        "mode-0102",
        "mode-0103",
        "mode-0104",
        "mode-0201",
    ] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_mode_preservation_and_typed_node_dispatch() {
    for case_name in [
        "mode-0301",
        "mode-0401",
        "mode-0501",
        "mode-0601",
        "mode-0701",
    ] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_equivalent_prefixed_and_punctuated_mode_names() {
    for case_name in ["mode-0901", "mode-1001"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_native_initial_mode_and_current_mode_continuation() {
    for case_name in [
        "mode-1101",
        "mode-1102",
        "mode-1103",
        "mode-1104",
        "mode-1105",
    ] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_all_mode_priority_and_next_match() {
    for case_name in ["mode-1201", "mode-1202", "mode-1203", "mode-1204"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

fn execute_case(case_name: &str) -> (String, String) {
    let private_overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(private_overlay.contains(&format!("case_name = \"{case_name}\"")));
    let document = load_test_set();
    let case = element_children(&document, document_element(&document))
        .into_iter()
        .find(|node| attribute(&document, *node, "name") == Some(case_name))
        .expect("selected mode case");
    let environment = case_environment(&document, case);
    let source = child_named(&document, environment, "source").expect("principal source");
    let content = child_named(&document, source, "content").expect("inline source content");
    let test = child_named(&document, case, "test").expect("test metadata");
    let stylesheet_file = child_named(&document, test, "stylesheet")
        .and_then(|node| attribute(&document, node, "file"))
        .expect("stylesheet file");
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/xslt30-test/tests/attr/mode");
    let assertion = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "assert-xml",
    )
    .expect("XML assertion");
    let expected = attribute(&document, assertion, "file").map_or_else(
        || document.string_value(assertion),
        |file| fs::read_to_string(directory.join(file)).expect("read expected XML result"),
    );

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
    let entry = case_entry(&document, test, source, &source_id);
    let parameters = case_parameters(&document, test);
    set.add(TransformRequest {
        identity: case_name.to_owned(),
        result_identity: format!("result:{case_name}"),
        entry,
        parameters,
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit mode request");
    let results = execute_transform_set(set.seal()).expect("execute selected mode case");
    (results.by_request[case_name].serialized.clone(), expected)
}

fn case_entry(
    document: &Document,
    test: NodeId,
    source: NodeId,
    source_id: &str,
) -> InvocationEntry {
    child_named(document, test, "initial-mode").map_or_else(
        || InvocationEntry::PrincipalSource {
            resource: source_id.to_owned(),
        },
        |initial_mode| {
            let name = attribute(document, initial_mode, "name")
                .expect("initial mode name")
                .to_owned();
            match attribute(document, source, "select") {
                None => InvocationEntry::InitialMode {
                    resource: source_id.to_owned(),
                    name,
                },
                Some("/doc") => InvocationEntry::InitialModeElement {
                    resource: source_id.to_owned(),
                    name,
                    element: ExpandedName {
                        namespace: None,
                        local: "doc".to_owned(),
                    },
                },
                Some(select) => panic!("unsupported initial context selection: {select}"),
            }
        },
    )
}

fn case_environment(document: &Document, case: NodeId) -> NodeId {
    let declaration = child_named(document, case, "environment").expect("case environment");
    attribute(document, declaration, "ref").map_or(declaration, |reference| {
        element_children(document, document_element(document))
            .into_iter()
            .find(|node| {
                local_name(document, *node) == "environment"
                    && attribute(document, *node, "name") == Some(reference)
            })
            .expect("referenced environment")
    })
}

fn case_parameters(document: &Document, test: NodeId) -> BTreeMap<String, InvocationParameter> {
    element_children(document, test)
        .into_iter()
        .filter(|node| local_name(document, *node) == "param")
        .map(|parameter| {
            let name = attribute(document, parameter, "name")
                .expect("parameter name")
                .to_owned();
            let select = attribute(document, parameter, "select").expect("parameter select");
            let value = select
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .expect("admitted mode parameter is one quoted string");
            (
                name,
                InvocationParameter {
                    value: AtomicValue::string(value),
                    tunnel: false,
                },
            )
        })
        .collect()
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
