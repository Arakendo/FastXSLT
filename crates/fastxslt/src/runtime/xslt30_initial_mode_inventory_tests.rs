//! Conserved admission for the complete XSLT30 `misc/initial-mode` denominator.

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::PathBuf,
};

use super::{
    ExecutionPolicy, FailureCategory, InvocationEntry, InvocationParameter, TransformRequest,
    TransformSetBuilder, compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "tests/misc/initial-mode/_initial-mode-test-set.xml";
const INITIAL_MODE_004: &str = "initial-mode-004";
const INITIAL_MODE_004_SOURCE: &str = "urn:w3c:xslt30:initial-mode-004:source";
const INITIAL_MODE_004_STYLESHEET: &str = "urn:w3c:xslt30:initial-mode-004:stylesheet";

struct CaseMetadata {
    name: &'static str,
    spec: Option<&'static str>,
    mode: &'static str,
    assertion: &'static str,
    error: Option<&'static str>,
    compile_code: Option<&'static str>,
}

const CASES: [CaseMetadata; 5] = [
    CaseMetadata {
        name: "initial-mode-001",
        spec: Some("XSLT20+"),
        mode: "inimode",
        assertion: "assert-xml",
        error: None,
        compile_code: Some("FXST1009"),
    },
    CaseMetadata {
        name: "initial-mode-002",
        spec: Some("XSLT10+"),
        mode: "inimode",
        assertion: "error",
        error: Some("XTDE0045"),
        compile_code: None,
    },
    CaseMetadata {
        name: "initial-mode-003",
        spec: Some("XSLT20+"),
        mode: "inimode",
        assertion: "error",
        error: Some("XTDE0050"),
        compile_code: None,
    },
    CaseMetadata {
        name: "initial-mode-004",
        spec: Some("XSLT30+"),
        mode: "flobble",
        assertion: "assert-xml",
        error: None,
        compile_code: None,
    },
    CaseMetadata {
        name: "initial-mode-005",
        spec: None,
        mode: "b",
        assertion: "assert-xml",
        error: None,
        compile_code: Some("FXST1015"),
    },
];

#[test]
fn admits_complete_initial_mode_denominator_and_reaches_engine_boundaries() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let (test_set, set_path) = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(cases.len(), CASES.len());
    let directory = set_path.parent().expect("initial-mode test-set directory");

    for metadata in &CASES {
        let record = overlay_case(overlay, metadata.name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains(if metadata.compile_code.is_some() {
            "execution = \"engine-unsupported\""
        } else {
            "execution = \"native-pass\""
        }));

        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(metadata.name))
            .expect("overlay case identity should exist upstream");
        assert_eq!(dependency(&test_set, case, "spec"), metadata.spec);
        let test = child_named(&test_set, case, "test").expect("test metadata");
        let initial_mode =
            child_named(&test_set, test, "initial-mode").expect("initial-mode entry metadata");
        assert_eq!(
            attribute(&test_set, initial_mode, "name"),
            Some(metadata.mode)
        );
        let result = child_named(&test_set, case, "result").expect("result metadata");
        let assertion = first_element_child(&test_set, result).expect("result assertion");
        assert_eq!(local_name(&test_set, assertion), metadata.assertion);
        assert_eq!(attribute(&test_set, assertion, "code"), metadata.error);

        let stylesheet_file = child_named(&test_set, test, "stylesheet")
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("stylesheet file");
        let stylesheet = fs::read(directory.join(stylesheet_file))
            .expect("read stylesheet and close upstream handle");
        let stylesheet_id = format!("urn:w3c:xslt30:{}:stylesheet", metadata.name);
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 65_536, 65_536));
        resources
            .admit(stylesheet_id.clone(), stylesheet)
            .expect("admit stylesheet bytes");
        let snapshot = resources.seal();
        if let Some(compile_code) = metadata.compile_code {
            let failure = compile_resource(&snapshot, &stylesheet_id)
                .expect_err("initial-mode case should reach an explicit engine gap");
            assert_eq!(failure.category, FailureCategory::Unsupported);
            assert_eq!(failure.code, compile_code);
        } else {
            compile_resource(&snapshot, &stylesheet_id)
                .expect("native initial-mode case should compile");
        }
    }
}

#[test]
fn executes_initial_mode_002_as_the_expected_unavailable_mode_error() {
    const CASE_NAME: &str = "initial-mode-002";
    const SOURCE_ID: &str = "urn:w3c:xslt30:initial-mode-002:source";
    const STYLESHEET_ID: &str = "urn:w3c:xslt30:initial-mode-002:stylesheet";
    let (test_set, set_path) = load_test_set();
    let case = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some(CASE_NAME))
        .expect("pinned initial-mode-002 metadata");
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
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(SOURCE_ID, b"<doc/>".to_vec())
        .expect("admit catalog source");
    resources
        .admit(STYLESHEET_ID, stylesheet)
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile initial-mode-002");
    assert_eq!(program.output.indent, Some(true));
    assert_eq!(program.matched_templates[0].mode.as_deref(), Some("#all"));
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

    let failure = builder
        .add(TransformRequest {
            identity: CASE_NAME.to_owned(),
            result_identity: format!("result:{CASE_NAME}"),
            entry: InvocationEntry::InitialMode {
                resource: SOURCE_ID.to_owned(),
                name: "inimode".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect_err("mode #all must not make an arbitrary initial mode available");
    assert_eq!(failure.code, "XTDE0045");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some(CASE_NAME));
}

#[test]
fn executes_initial_mode_003_as_the_expected_missing_required_parameter_error() {
    const CASE_NAME: &str = "initial-mode-003";
    const SOURCE_ID: &str = "urn:w3c:xslt30:initial-mode-003:source";
    const STYLESHEET_ID: &str = "urn:w3c:xslt30:initial-mode-003:stylesheet";
    let (test_set, set_path) = load_test_set();
    let case = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some(CASE_NAME))
        .expect("pinned initial-mode-003 metadata");
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
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(SOURCE_ID, b"<doc/>".to_vec())
        .expect("admit catalog source");
    resources
        .admit(STYLESHEET_ID, stylesheet)
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile initial-mode-003");
    assert_eq!(program.output.indent, Some(true));
    assert!(program.global_bindings[0].required);
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
            identity: CASE_NAME.to_owned(),
            result_identity: format!("result:{CASE_NAME}"),
            entry: InvocationEntry::InitialMode {
                resource: SOURCE_ID.to_owned(),
                name: "inimode".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit known initial mode");

    let failure = execute_transform_set(builder.seal())
        .expect_err("missing required global parameter must fail before transformation");
    assert_eq!(failure.code, "XTDE0050");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some(CASE_NAME));
}

#[test]
fn executes_initial_mode_004_with_local_qname_and_tunnel_parameters() {
    let (test_set, set_path) = load_test_set();
    let case = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some(INITIAL_MODE_004))
        .expect("pinned initial-mode-004 metadata");
    let test = child_named(&test_set, case, "test").expect("test metadata");
    let initial_mode = child_named(&test_set, test, "initial-mode").expect("initial mode");
    let parameter_nodes = children_named(&test_set, initial_mode, "param");
    assert_eq!(parameter_nodes.len(), 2);
    assert_eq!(attribute(&test_set, parameter_nodes[0], "name"), Some("a"));
    assert_eq!(
        attribute(&test_set, parameter_nodes[0], "select"),
        Some("1234")
    );
    assert_eq!(
        attribute(&test_set, parameter_nodes[1], "name"),
        Some("my:b")
    );
    assert_eq!(
        attribute(&test_set, parameter_nodes[1], "tunnel"),
        Some("yes")
    );
    assert_eq!(
        attribute(&test_set, parameter_nodes[1], "select"),
        Some("999")
    );
    let expected = child_named(&test_set, case, "result")
        .and_then(|node| child_named(&test_set, node, "assert-xml"))
        .map(|node| test_set.string_value(node))
        .expect("XML assertion");
    assert_eq!(expected.trim(), "<out><doc/>1234 999</out>");

    let environment = descendants_named(&test_set, test_set.document_node(), "environment")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some("inimode001"))
        .expect("referenced source environment");
    let source = child_named(&test_set, environment, "source")
        .and_then(|node| child_named(&test_set, node, "content"))
        .map(|node| test_set.string_value(node).into_bytes())
        .expect("inline source content");
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
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(INITIAL_MODE_004_SOURCE, source)
        .expect("admit inline source");
    resources
        .admit(INITIAL_MODE_004_STYLESHEET, stylesheet)
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, INITIAL_MODE_004_STYLESHEET).expect("compile initial-mode-004");
    let mut builder = TransformSetBuilder::new(
        snapshot,
        program,
        2,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 4_096,
            work_limits: WorkLimits::unbounded(),
        },
    );
    builder
        .add(initial_mode_004_request(INITIAL_MODE_004, "999", true))
        .expect("admit initial-mode invocation");
    builder
        .add(initial_mode_004_request(
            "initial-mode-004-wrong-tunnel",
            "must not bind",
            false,
        ))
        .expect("admit tunnel-mismatch control invocation");

    let results = execute_transform_set(builder.seal()).expect("execute initial-mode-004");
    assert_eq!(
        results.by_request[INITIAL_MODE_004].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out><doc></doc>1234 999</out>"
    );
    assert_eq!(
        results.by_request["initial-mode-004-wrong-tunnel"].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out><doc></doc>1234 </out>"
    );
}

fn initial_mode_004_request(identity: &str, b_value: &str, b_tunnel: bool) -> TransformRequest {
    TransformRequest {
        identity: identity.to_owned(),
        result_identity: format!("result:{identity}"),
        entry: InvocationEntry::InitialMode {
            resource: INITIAL_MODE_004_SOURCE.to_owned(),
            name: "flobble".to_owned(),
        },
        parameters: BTreeMap::from([
            (
                "a".to_owned(),
                InvocationParameter {
                    value: AtomicValue::string("1234"),
                    tunnel: false,
                },
            ),
            (
                "Q{http://my.net/}b".to_owned(),
                InvocationParameter {
                    value: AtomicValue::string(b_value),
                    tunnel: b_tunnel,
                },
            ),
        ]),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    }
}

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test")
        .join(SET_FILE);
    let bytes = fs::read(&path).expect("read pinned initial-mode test set");
    let parsed = parse_document(
        "urn:w3c:xslt30:initial-mode:test-set",
        &bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
    )
    .expect("parse pinned initial-mode test set");
    (
        Document::from_parsed(parsed).expect("build initial-mode catalog document"),
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
        .expect("overlay must contain one initial-mode disposition")
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

fn children_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|node| {
            document.kind(*node) == NodeKind::Element && local_name(document, *node) == local
        })
        .collect()
}

fn first_element_child(document: &Document, parent: NodeId) -> Option<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .find(|node| document.kind(*node) == NodeKind::Element)
}

fn local_name(document: &Document, node: NodeId) -> &str {
    document.name(node).map_or("", |name| name.local.as_str())
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}

fn dependency<'a>(document: &'a Document, case: NodeId, name: &str) -> Option<&'a str> {
    child_named(document, case, "dependencies")
        .and_then(|dependencies| child_named(document, dependencies, name))
        .and_then(|dependency| attribute(document, dependency, "value"))
}
