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
const PASSED_CASES: [&str; 8] = [
    "root-0101",
    "root-0102",
    "root-0103",
    "root-0104",
    "root-0201",
    "root-0301",
    "root-0401",
    "root-0601",
];
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
    assert_eq!(OVERLAY.matches("[[case_override]]").count(), 8);
    for case_name in PASSED_CASES {
        assert!(names.contains(case_name));
        let record = overlay_case(case_name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains("execution = \"passed\""));
    }
}

#[test]
fn executes_unchanged_root_location_path_cases() {
    for case_name in PASSED_CASES {
        execute_case(case_name);
    }
}

#[test]
fn generated_root_identity_is_stable_for_element_and_document_arguments() {
    execute_case("root-0301");
    execute_case("root-0401");
}

#[test]
fn root_path_and_source_node_variable_enforce_argument_cardinality() {
    let stylesheets = [
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><out><xsl:value-of select="root(doc/item)"/></out></xsl:template></xsl:stylesheet>"#.as_slice(),
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:variable name="nodes" select="doc/item"/><out><xsl:value-of select="root($nodes)"/></out></xsl:template></xsl:stylesheet>"#.as_slice(),
    ];
    for (index, stylesheet) in stylesheets.into_iter().enumerate() {
        let source_id = format!("urn:fastxslt:fn-root:cardinality:{index}:source");
        let stylesheet_id = format!("urn:fastxslt:fn-root:cardinality:{index}:stylesheet");
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
        resources
            .admit(&source_id, b"<doc><item/><item/></doc>".to_vec())
            .expect("admit cardinality source");
        resources
            .admit(&stylesheet_id, stylesheet.to_vec())
            .expect("admit cardinality stylesheet");
        let snapshot = resources.seal();
        let program =
            compile_resource(&snapshot, &stylesheet_id).expect("compile cardinality control");
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
            identity: format!("cardinality-{index}"),
            result_identity: format!("urn:fastxslt:fn-root:cardinality:{index}:result"),
            entry: InvocationEntry::PrincipalSource {
                resource: source_id,
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit cardinality request");

        let failure = execute_transform_set(set.seal()).expect_err("two root arguments must fail");
        assert_eq!(failure.code, "XPTY0004");
    }
}

fn execute_case(case_name: &str) {
    let document = load_test_set();
    let case = case_named(&document, case_name);
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
    let source = child_named(&document, environment, "source").expect("source metadata");
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
    let source_id = format!("urn:w3c:xslt30:fn:root:{case_name}:source");
    let stylesheet_id = format!("https://example.invalid/xslt30/fn/root/{stylesheet_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(
            source_id.clone(),
            source_bytes(&document, source, &directory),
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
        identity: case_name.to_owned(),
        result_identity: format!("urn:w3c:xslt30:fn:root:{case_name}:result"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit request");
    let results = execute_transform_set(set.seal()).expect("execute root case");
    assert_xml_equivalent(
        &results.by_request[case_name].serialized,
        &expected,
        case_name,
    );
}

fn source_bytes(document: &Document, source: NodeId, directory: &std::path::Path) -> Vec<u8> {
    if let Some(file) = attribute(document, source, "file") {
        return fs::read(directory.join(file)).expect("read source and close handle");
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

fn assert_xml_equivalent(actual: &str, expected: &str, case_name: &str) {
    let limits = ParseLimits {
        max_events: 4_096,
        max_depth: 64,
    };
    let actual = Document::from_parsed(
        parse_document(
            &format!("urn:fastxslt:fn-root:{case_name}:actual"),
            actual.as_bytes(),
            limits,
        )
        .expect("actual result should parse"),
    )
    .expect("actual result should build");
    let expected = Document::from_parsed(
        parse_document(
            &format!("urn:fastxslt:fn-root:{case_name}:expected"),
            expected.as_bytes(),
            limits,
        )
        .expect("expected result should parse"),
    )
    .expect("expected result should build");
    assert_xml_nodes_equal(
        &actual,
        actual.document_node(),
        &expected,
        expected.document_node(),
        case_name,
    );
}

fn assert_xml_nodes_equal(
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
        assert_xml_nodes_equal(actual, *actual_child, expected, *expected_child, case_name);
    }
}
