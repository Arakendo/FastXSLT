//! Conserved inventory and initial execution of XSLT30 `insn/call-template`.

use std::{collections::BTreeMap, collections::BTreeSet, collections::HashSet, fs, path::PathBuf};

use super::{
    ExecutionPolicy, InvocationEntry, InvocationParameter, TransformRequest, TransformSetBuilder,
    compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const TEST_SET: &str = "tests/insn/call-template/_call-template-test-set.xml";
const RESULT_CASES: [&str; 13] = [
    "call-template-0101",
    "call-template-0102",
    "call-template-0103",
    "call-template-0201",
    "call-template-0801",
    "call-template-0802",
    "call-template-0109",
    "call-template-1101",
    "call-template-1501",
    "call-template-1701",
    "call-template-1801",
    "call-template-1802",
    "call-template-1803",
];
const ERROR_CASES: [(&str, &str); 6] = [
    ("call-template-0001", "XPDY0002"),
    ("call-template-0104", "XTDE0040"),
    ("call-template-0105", "XTDE0040"),
    ("call-template-0106", "XTSE0080"),
    ("call-template-0107", "XTDE0040"),
    ("call-template-0401a", "XTDE0700"),
];
const PROFILE_EXCLUDED_CASES: [&str; 1] = ["call-template-0401"];
const OVERLAY: &str =
    include_str!("../../../../corpus/overlays/xslt30/call-template-denominator-v0.toml");

#[test]
fn inventories_complete_call_template_denominator_before_selection() {
    let document = load_test_set();
    let cases = descendants_named(&document, document.document_node(), "test-case");
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<BTreeSet<_>>();
    assert_eq!(cases.len(), 42);
    assert_eq!(names.len(), cases.len());
    assert!(OVERLAY.contains(&format!("set_file = \"{TEST_SET}\"")));
    assert!(OVERLAY.contains("case_count = 42"));
    assert_eq!(OVERLAY.matches("[[case_override]]").count(), 20);
    for case_name in RESULT_CASES
        .into_iter()
        .chain(ERROR_CASES.into_iter().map(|(case_name, _)| case_name))
    {
        assert!(names.contains(case_name));
        let record = overlay_case(case_name);
        assert!(record.contains("selection = \"selected\""));
        assert!(record.contains("execution = \"passed\""));
    }
    for case_name in PROFILE_EXCLUDED_CASES {
        assert!(names.contains(case_name));
        let record = overlay_case(case_name);
        assert!(record.contains("selection = \"excluded-profile\""));
        assert!(record.contains("execution = \"not-run\""));
    }
}

#[test]
fn executes_unchanged_initial_and_called_named_templates() {
    for case_name in RESULT_CASES {
        execute_case(case_name);
    }
}

#[test]
fn reports_unchanged_initial_template_errors() {
    for (case_name, expected_code) in ERROR_CASES {
        execute_error_case(case_name, expected_code);
    }
}

fn execute_case(case_name: &str) {
    let document = load_test_set();
    let case = case_named(&document, case_name);
    let test = child_named(&document, case, "test").expect("test metadata");
    let expected = child_named(&document, case, "result")
        .and_then(|node| child_named(&document, node, "assert-xml"))
        .map(|node| document.string_value(node))
        .expect("inline XML assertion");
    let environment_ref = child_named(&document, case, "environment")
        .and_then(|node| attribute(&document, node, "ref"))
        .expect("environment reference");
    let environment = descendants_named(&document, document.document_node(), "environment")
        .into_iter()
        .find(|node| attribute(&document, *node, "name") == Some(environment_ref))
        .expect("referenced environment");
    let source = child_named(&document, environment, "source").expect("principal source");
    let source_id = format!("urn:w3c:xslt30:insn:call-template:{case_name}:source");
    let stylesheets = element_children(&document, test)
        .into_iter()
        .filter(|node| local_name(&document, *node) == "stylesheet")
        .collect::<Vec<_>>();
    let stylesheet_file = stylesheets
        .first()
        .copied()
        .and_then(|node| attribute(&document, node, "file"))
        .expect("stylesheet file");
    let stylesheet_id =
        format!("https://example.invalid/xslt30/insn/call-template/{stylesheet_file}");
    let mut resources =
        ResourceSetBuilder::new(ResourceLimits::new(1 + stylesheets.len(), 65_536, 262_144));
    resources
        .admit(source_id.clone(), source_bytes(&document, source))
        .expect("admit principal source");
    for stylesheet in stylesheets {
        let file = attribute(&document, stylesheet, "file").expect("stylesheet file");
        resources
            .admit(
                format!("https://example.invalid/xslt30/insn/call-template/{file}"),
                fs::read(corpus_directory().join(file)).expect("read stylesheet and close handle"),
            )
            .expect("admit stylesheet");
    }
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile call-template case");
    let entry = child_named(&document, test, "initial-template").map_or_else(
        || InvocationEntry::PrincipalSource {
            resource: source_id.clone(),
        },
        |node| InvocationEntry::InitialTemplateWithSource {
            resource: source_id.clone(),
            name: catalog_eqname(
                &document,
                node,
                attribute(&document, node, "name").expect("initial template name"),
            ),
        },
    );
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
        result_identity: format!("urn:w3c:xslt30:insn:call-template:{case_name}:result"),
        entry,
        parameters: case_parameters(&document, test),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit transform request");
    let results = execute_transform_set(set.seal()).expect("execute call-template case");
    assert_xml_equivalent(
        &results.by_request[case_name].serialized,
        &expected,
        case_name,
    );
}

fn execute_error_case(case_name: &str, expected_code: &str) {
    let document = load_test_set();
    let case = case_named(&document, case_name);
    let test = child_named(&document, case, "test").expect("test metadata");
    let source = child_named(&document, case, "environment")
        .and_then(|node| attribute(&document, node, "ref"))
        .map(|environment_ref| {
            descendants_named(&document, document.document_node(), "environment")
                .into_iter()
                .find(|node| attribute(&document, *node, "name") == Some(environment_ref))
                .and_then(|environment| child_named(&document, environment, "source"))
                .expect("referenced principal source")
        });
    let source_id = format!("urn:w3c:xslt30:insn:call-template:{case_name}:source");
    let stylesheet = child_named(&document, test, "stylesheet").expect("principal stylesheet");
    let stylesheet_file = attribute(&document, stylesheet, "file").expect("stylesheet file");
    let stylesheet_id =
        format!("https://example.invalid/xslt30/insn/call-template/{stylesheet_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(
        1 + usize::from(source.is_some()),
        65_536,
        131_072,
    ));
    if let Some(source) = source {
        resources
            .admit(source_id.clone(), source_bytes(&document, source))
            .expect("admit principal source");
    }
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(corpus_directory().join(stylesheet_file))
                .expect("read stylesheet and close handle"),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();

    let program = match compile_resource(&snapshot, &stylesheet_id) {
        Ok(program) => program,
        Err(failure) => {
            assert_eq!(failure.code, expected_code, "{case_name}");
            return;
        }
    };
    let initial_template_node =
        child_named(&document, test, "initial-template").expect("dynamic error initial template");
    let initial_template = catalog_eqname(
        &document,
        initial_template_node,
        attribute(&document, initial_template_node, "name")
            .expect("dynamic error case should name an initial template"),
    );
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
    let entry = if source.is_some() {
        InvocationEntry::InitialTemplateWithSource {
            resource: source_id,
            name: initial_template,
        }
    } else {
        InvocationEntry::InitialTemplate {
            name: initial_template,
        }
    };
    let admission = set.add(TransformRequest {
        identity: case_name.to_owned(),
        result_identity: format!("urn:w3c:xslt30:insn:call-template:{case_name}:result"),
        entry,
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    });
    let failure = match admission {
        Err(failure) => failure,
        Ok(()) => execute_transform_set(set.seal())
            .expect_err("dynamic initial-template error should fail execution"),
    };
    assert_eq!(failure.code, expected_code, "{case_name}");
}

fn catalog_eqname(document: &Document, element: NodeId, lexical: &str) -> String {
    let Some((prefix, local)) = lexical.split_once(':') else {
        return lexical.to_owned();
    };
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(binding) = document
            .namespace_declarations(node)
            .iter()
            .find(|binding| binding.prefix.as_deref() == Some(prefix))
        {
            return format!("Q{{{}}}{local}", binding.namespace);
        }
        current = document.parent(node);
    }
    panic!("catalog QName prefix should be bound: {lexical}");
}

fn source_bytes(document: &Document, source: NodeId) -> Vec<u8> {
    if let Some(file) = attribute(document, source, "file") {
        return fs::read(corpus_directory().join(file)).expect("read source and close handle");
    }
    let content = child_named(document, source, "content").expect("inline source content");
    document.string_value(content).into_bytes()
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
                .expect("admitted call-template parameter is one quoted string");
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

fn case_named(document: &Document, case_name: &str) -> NodeId {
    descendants_named(document, document.document_node(), "test-case")
        .into_iter()
        .find(|node| attribute(document, *node, "name") == Some(case_name))
        .expect("pinned test case")
}

fn overlay_case(case_name: &str) -> &str {
    OVERLAY
        .split("[[case_override]]")
        .find(|record| record.contains(&format!("case_name = \"{case_name}\"")))
        .expect("case override")
}

fn corpus_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/insn/call-template")
}

fn load_test_set() -> Document {
    let bytes = fs::read(corpus_directory().join("_call-template-test-set.xml"))
        .expect("read pinned test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:insn:call-template:test-set",
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
            &format!("urn:fastxslt:call-template:{case_name}:actual"),
            actual.as_bytes(),
            limits,
        )
        .expect("actual result should parse"),
    )
    .expect("actual result should build");
    let expected = Document::from_parsed(
        parse_document(
            &format!("urn:fastxslt:call-template:{case_name}:expected"),
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
