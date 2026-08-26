//! Conserved admission for the complete XSLT30 `expr/castable` denominator.

use std::{collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, FailureCategory, InvocationEntry, TransformRequest, TransformSetBuilder,
    compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "tests/expr/castable/_castable-test-set.xml";

#[derive(Clone, Copy)]
struct CasePressure {
    name: &'static str,
    environment: Option<&'static str>,
    spec: &'static str,
    features: &'static [&'static str],
    assertion: &'static str,
    selection: &'static str,
    execution: &'static str,
}

const CASES: [CasePressure; 9] = [
    CasePressure {
        name: "castable-001",
        environment: Some("castbl01"),
        spec: "XSLT20+",
        features: &[],
        assertion: "assert-xml",
        selection: "selected",
        execution: "passed",
    },
    CasePressure {
        name: "castable-002",
        environment: Some("castbl01"),
        spec: "XSLT20+",
        features: &[],
        assertion: "assert-xml",
        selection: "selected",
        execution: "passed",
    },
    CasePressure {
        name: "castable-003",
        environment: Some("castbl01"),
        spec: "XSLT20+",
        features: &[],
        assertion: "assert-xml",
        selection: "selected",
        execution: "passed",
    },
    selected_engine_case("castable-004", "assert-xml"),
    CasePressure {
        name: "castable-005",
        environment: None,
        spec: "XSLT30+",
        features: &["schema_aware", "higher_order_functions"],
        assertion: "assert",
        selection: "excluded-by-profile",
        execution: "not-run",
    },
    CasePressure {
        name: "castable-006",
        environment: None,
        spec: "XSLT30+",
        features: &["schema_aware", "higher_order_functions"],
        assertion: "assert",
        selection: "excluded-by-profile",
        execution: "not-run",
    },
    selected_harness_case("castable-007"),
    selected_harness_case("castable-008"),
    selected_harness_case("castable-009"),
];

#[test]
fn executes_native_xslt30_castable_001() {
    let (actual, expected) = execute_principal_case("castable-001");
    assert_eq!(actual, expected.trim());
}

#[test]
fn executes_native_xslt30_castable_002_with_typed_local_variables() {
    let (actual, expected) = execute_principal_case("castable-002");
    assert_eq!(actual, expected.trim());
}

#[test]
fn executes_native_xslt30_castable_003_with_numeric_conversions() {
    let (actual, expected) = execute_principal_case("castable-003");
    assert_eq!(actual, expected.trim());
}

fn execute_principal_case(case_name: &str) -> (String, String) {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay_case(overlay, case_name).contains("execution = \"passed\""));
    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("castable test-set directory");
    let test_case = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some(case_name))
        .expect("native castable case");
    let test = child_named(&test_set, test_case, "test").expect("native test");
    let stylesheet_file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("native stylesheet file");
    let environment = descendants_named(&test_set, test_set.document_node(), "environment")
        .into_iter()
        .find(|node| attribute(&test_set, *node, "name") == Some("castbl01"))
        .expect("native environment");
    let source_file = child_named(&test_set, environment, "source")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("native source file");
    let result = child_named(&test_set, test_case, "result").expect("native result");
    let assertion = first_element_child(&test_set, result).expect("native assertion");
    let expected_file = attribute(&test_set, assertion, "file").expect("native expected file");
    let expected = fs::read_to_string(directory.join(expected_file))
        .expect("read native assertion and close handle");
    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = stylesheet_id(case_name);
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(
            source_id.clone(),
            fs::read(directory.join(source_file)).expect("read native source and close handle"),
        )
        .expect("admit native source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file))
                .expect("read native stylesheet and close handle"),
        )
        .expect("admit native stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, &stylesheet_id).expect("compile native castable case");
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
            resource: source_id,
        },
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit native request");

    let results = execute_transform_set(set.seal()).expect("execute native castable case");
    (results.by_request[case_name].serialized.clone(), expected)
}

const fn selected_engine_case(name: &'static str, assertion: &'static str) -> CasePressure {
    CasePressure {
        name,
        environment: Some("castbl01"),
        spec: "XSLT20+",
        features: &[],
        assertion,
        selection: "selected",
        execution: "engine-unsupported",
    }
}

const fn selected_harness_case(name: &'static str) -> CasePressure {
    CasePressure {
        name,
        environment: None,
        spec: "XSLT30+",
        features: &[],
        assertion: "all-of",
        selection: "selected",
        execution: "harness-unsupported",
    }
}

#[test]
fn admits_complete_xslt30_castable_test_set_without_denominator_loss() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let admitted: Vec<_> = overlay
        .split("[[case]]")
        .filter(|section| section.contains(&format!("set_file = \"{SET_FILE}\"")))
        .collect();
    assert_eq!(admitted.len(), CASES.len());

    let (test_set, set_path) = load_test_set();
    let test_cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(test_cases.len(), CASES.len());
    let directory = set_path
        .parent()
        .expect("castable test set should have a directory");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(13, 65_536, 524_288));

    for pressure in CASES {
        let record = overlay_case(overlay, pressure.name);
        assert!(record.contains(&format!("selection = \"{}\"", pressure.selection)));
        assert!(record.contains(&format!("execution = \"{}\"", pressure.execution)));

        let test_case = test_cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(pressure.name))
            .expect("overlay identity should exist in pinned castable test set");
        assert_eq!(
            dependency(&test_set, test_case, "spec"),
            vec![pressure.spec]
        );
        assert_eq!(
            dependency(&test_set, test_case, "feature"),
            pressure.features
        );
        let environment = child_named(&test_set, test_case, "environment")
            .and_then(|node| attribute(&test_set, node, "ref"));
        assert_eq!(environment, pressure.environment);
        let result = child_named(&test_set, test_case, "result").expect("case result");
        let assertion = first_element_child(&test_set, result).expect("case assertion");
        assert_eq!(local_name(&test_set, assertion), pressure.assertion);

        let test = child_named(&test_set, test_case, "test").expect("case test");
        let stylesheet = child_named(&test_set, test, "stylesheet").expect("case stylesheet");
        let stylesheet_file = attribute(&test_set, stylesheet, "file")
            .expect("castable case should name a stylesheet");
        let stylesheet_bytes = fs::read(directory.join(stylesheet_file))
            .expect("read castable stylesheet and close handle");
        resources
            .admit(stylesheet_id(pressure.name), stylesheet_bytes)
            .expect("admit castable stylesheet");

        if let Some(environment_name) = pressure.environment {
            let environment = descendants_named(&test_set, test_set.document_node(), "environment")
                .into_iter()
                .find(|node| attribute(&test_set, *node, "name") == Some(environment_name))
                .expect("referenced castable environment should exist");
            let source = child_named(&test_set, environment, "source")
                .and_then(|node| attribute(&test_set, node, "file"))
                .expect("castable environment should name a source");
            let source_bytes =
                fs::read(directory.join(source)).expect("read castable source and close handle");
            resources
                .admit(
                    format!("urn:w3c:xslt30:{}:source", pressure.name),
                    source_bytes,
                )
                .expect("admit castable source");
        }

        if let Some(expected_file) = attribute(&test_set, assertion, "file") {
            assert!(
                fs::metadata(directory.join(expected_file))
                    .expect("file-backed castable assertion should exist")
                    .len()
                    > 0
            );
        }
    }

    let snapshot = resources.seal();
    for pressure in CASES
        .iter()
        .filter(|pressure| pressure.execution == "engine-unsupported")
    {
        let failure = compile_resource(&snapshot, &stylesheet_id(pressure.name))
            .expect_err("engine-unsupported castable case should fail compilation");
        assert_eq!(
            failure.category,
            FailureCategory::Unsupported,
            "{} should remain valid-but-unsupported: {failure:?}",
            pressure.name
        );
    }
}

fn stylesheet_id(case_name: &str) -> String {
    format!("urn:w3c:xslt30:{case_name}:stylesheet")
}

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/expr/castable/_castable-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned castable test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:expr:castable:test-set",
        &bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
    )
    .expect("parse pinned castable test set");
    (
        Document::from_parsed(parsed).expect("build castable test-set document"),
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
    &document
        .name(node)
        .expect("element should have a name")
        .local
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
    let Some(dependencies) = child_named(document, case, "dependencies") else {
        return Vec::new();
    };
    document
        .children(dependencies)
        .iter()
        .copied()
        .filter(|node| {
            document.kind(*node) == NodeKind::Element && local_name(document, *node) == kind
        })
        .filter_map(|node| attribute(document, node, "value"))
        .collect()
}

fn overlay_case<'a>(overlay: &'a str, case_name: &str) -> &'a str {
    overlay
        .split("[[case]]")
        .find(|section| {
            section.contains(&format!("set_file = \"{SET_FILE}\""))
                && section.contains(&format!("case_name = \"{case_name}\""))
        })
        .expect("admitted castable case should have one overlay record")
}
