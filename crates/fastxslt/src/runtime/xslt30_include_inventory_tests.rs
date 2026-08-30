//! Conserved preview of the complete XSLT30 `decl/include` denominator.

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

const CASE_NAMES: [&str; 16] = [
    "include-0101",
    "include-0102",
    "include-0103",
    "include-0104",
    "include-0105",
    "include-0201",
    "include-0202",
    "include-0301",
    "include-0401",
    "include-0501",
    "include-0601",
    "include-0701",
    "include-0702a",
    "include-0702b",
    "include-0702c",
    "include-0801",
];
const PASSED_CASES: [&str; 10] = [
    "include-0103",
    "include-0104",
    "include-0105",
    "include-0201",
    "include-0202",
    "include-0301",
    "include-0401",
    "include-0501",
    "include-0601",
    "include-0701",
];
const OVERLAY: &str =
    include_str!("../../../../corpus/overlays/xslt30/include-denominator-v0.toml");
const PRINCIPAL_ID: &str = "https://example.invalid/xslt30/decl/include/include-0401.xsl";
const SECONDARY_ID: &str = "https://example.invalid/xslt30/decl/include/include-0401a.xsl";
const SOURCE_ID: &str = "urn:w3c:xslt30:decl:include:include-0401:source";

#[test]
fn executes_include_0401_through_one_sealed_relative_module() {
    let document = load_test_set();
    let case = element_children(&document, document_element(&document))
        .into_iter()
        .find(|node| attribute(&document, *node, "name") == Some("include-0401"))
        .expect("pinned include-0401 case");
    let test = child_named(&document, case, "test").expect("test metadata");
    let stylesheet_files = element_children(&document, test)
        .into_iter()
        .filter(|node| local_name(&document, *node) == "stylesheet")
        .map(|node| attribute(&document, node, "file").expect("stylesheet file"))
        .collect::<Vec<_>>();
    assert_eq!(stylesheet_files, ["include-0401.xsl", "include-0401a.xsl"]);
    let environment = child_named(&document, case, "environment").expect("inline environment");
    let source = child_named(&document, environment, "source").expect("principal source");
    let content = child_named(&document, source, "content").expect("inline source content");
    let result = child_named(&document, case, "result").expect("case result");
    let assertion = first_element_child(&document, result).expect("result assertion");
    assert_eq!(local_name(&document, assertion), "assert-xml");
    let expected = document.string_value(assertion);

    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/include");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 8_192, 16_384));
    resources
        .admit(SOURCE_ID, document.string_value(content).into_bytes())
        .expect("admit inline source");
    resources
        .admit(
            PRINCIPAL_ID,
            fs::read(directory.join(stylesheet_files[0]))
                .expect("read principal stylesheet and close handle"),
        )
        .expect("admit principal stylesheet");
    resources
        .admit(
            SECONDARY_ID,
            fs::read(directory.join(stylesheet_files[1]))
                .expect("read secondary stylesheet and close handle"),
        )
        .expect("admit secondary stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, PRINCIPAL_ID).expect("compile included stylesheet");
    assert_eq!(
        program
            .root_template
            .as_ref()
            .expect("simplified stylesheet implicit root template")
            .location
            .resource,
        SECONDARY_ID
    );
    assert!(program.global_bindings.iter().any(|binding| {
        binding.name == "greeting"
            && matches!(
                &binding.default,
                crate::xslt::golden_semantics_experiment::GlobalBindingDefault::Text(value)
                    if value == "Hi there!"
            )
    }));
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
        identity: "include-0401".to_owned(),
        result_identity: "urn:w3c:xslt30:decl:include:include-0401:result".to_owned(),
        entry: InvocationEntry::PrincipalSource {
            resource: SOURCE_ID.to_owned(),
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit include-0401 request");
    let results = execute_transform_set(set.seal()).expect("execute include-0401");
    let actual = &results.by_request["include-0401"].serialized;
    assert_eq!(
        without_xml_declaration(actual.trim()),
        without_xml_declaration(expected.trim())
    );
}

#[test]
fn executes_include_0201_apply_imports_builtin_fallback() {
    let (actual, expected) = execute_inline_case_without_dependencies("include-0201");
    assert_eq!(
        without_xml_declaration(actual.trim()),
        without_xml_declaration(expected.trim())
    );
}

#[test]
fn executes_include_0301_repeated_apply_imports() {
    let execution = execute_inline_case_with_dependencies("include-0301");
    assert_eq!(execution.import_precedences.len(), 2);
    assert!(execution.import_precedences[0] < execution.import_precedences[1]);
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

#[test]
fn executes_include_0202_apply_imports_with_parameter_and_computed_attribute() {
    let execution = execute_inline_case_with_dependencies("include-0202");
    assert_eq!(execution.import_precedences.len(), 2);
    assert!(execution.import_precedences[0] < execution.import_precedences[1]);
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

#[test]
fn executes_include_0105_principal_global_override_and_imported_named_template() {
    let execution = execute_inline_case_with_dependencies("include-0105");
    assert!(execution.import_precedences.is_empty());
    assert_eq!(
        execution.global_text_defaults,
        [("test".to_owned(), "OK".to_owned())]
    );
    assert_eq!(execution.named_template_names, ["two"]);
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

#[test]
fn executes_include_0601_imported_simplified_root_and_text_fallback() {
    let execution = execute_inline_case_with_dependencies("include-0601");
    assert_eq!(execution.import_precedences, [-1, 0]);
    assert!(execution.global_text_defaults.is_empty());
    assert!(execution.named_template_names.is_empty());
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

#[test]
fn executes_include_0501_later_import_global_parameter_precedence() {
    let execution = execute_inline_case_with_dependencies("include-0501");
    assert!(execution.import_precedences.is_empty());
    assert_eq!(
        execution.global_text_defaults,
        [
            (
                "first".to_owned(),
                "aaa, as defined in first.xsl".to_owned()
            ),
            (
                "second".to_owned(),
                "ZZZ, as defined in second.xsl".to_owned()
            ),
        ]
    );
    assert!(execution.named_template_names.is_empty());
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

#[test]
fn executes_include_0103_embedded_stylesheet_fragment_and_base() {
    let execution = execute_inline_case_with_dependencies("include-0103");
    assert!(execution.import_precedences.is_empty());
    assert!(execution.global_text_defaults.is_empty());
    assert_eq!(execution.named_template_names, ["x"]);
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

#[test]
fn executes_include_0104_apply_imports_across_included_rule() {
    let execution = execute_inline_case_with_dependencies("include-0104");
    assert_eq!(execution.import_precedences, [-1, 0, 0]);
    assert!(execution.global_text_defaults.is_empty());
    assert!(execution.named_template_names.is_empty());
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

#[test]
fn executes_include_0701_two_includes_with_leaf_imports() {
    let execution = execute_inline_case_with_dependencies("include-0701");
    assert_eq!(
        execution
            .import_precedences
            .iter()
            .filter(|precedence| **precedence < 0)
            .count(),
        4
    );
    assert_eq!(
        execution
            .import_precedences
            .iter()
            .filter(|precedence| **precedence == 0)
            .count(),
        6
    );
    assert_xml_equivalent(&execution.actual, &execution.expected);
}

struct DependencyCaseExecution {
    actual: String,
    expected: String,
    import_precedences: Vec<i32>,
    global_text_defaults: Vec<(String, String)>,
    named_template_names: Vec<String>,
}

fn execute_inline_case_with_dependencies(case_name: &str) -> DependencyCaseExecution {
    let document = load_test_set();
    let case = element_children(&document, document_element(&document))
        .into_iter()
        .find(|node| attribute(&document, *node, "name") == Some(case_name))
        .expect("pinned dependency case");
    let test = child_named(&document, case, "test").expect("test metadata");
    let stylesheet_files = element_children(&document, test)
        .into_iter()
        .filter(|node| local_name(&document, *node) == "stylesheet")
        .map(|node| attribute(&document, node, "file").expect("stylesheet file"))
        .collect::<Vec<_>>();
    assert!(matches!(stylesheet_files.len(), 2 | 3 | 5));
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/include");
    let environment = case_environment(&document, case);
    let source = child_named(&document, environment, "source").expect("principal source");
    let source_bytes = child_named(&document, source, "content").map_or_else(
        || {
            fs::read(directory.join(attribute(&document, source, "file").expect("source file")))
                .expect("read source file")
        },
        |content| document.string_value(content).into_bytes(),
    );
    let assertion = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "assert-xml",
    )
    .expect("XML assertion");
    let expected = attribute(&document, assertion, "file").map_or_else(
        || document.string_value(assertion),
        |file| fs::read_to_string(directory.join(file)).expect("read expected XML"),
    );
    let principal_id = format!(
        "https://example.invalid/xslt30/decl/include/{}",
        stylesheet_files[0]
    );
    let source_id = format!("urn:w3c:xslt30:decl:include:{case_name}:source");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(6, 12_288, 49_152));
    resources
        .admit(source_id.clone(), source_bytes)
        .expect("admit inline source");
    resources
        .admit(
            principal_id.clone(),
            fs::read(directory.join(stylesheet_files[0])).expect("read principal stylesheet"),
        )
        .expect("admit principal stylesheet");
    for stylesheet_file in &stylesheet_files[1..] {
        resources
            .admit(
                format!("https://example.invalid/xslt30/decl/include/{stylesheet_file}"),
                fs::read(directory.join(stylesheet_file)).expect("read imported stylesheet"),
            )
            .expect("admit imported stylesheet");
    }
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &principal_id).expect("compile imported stylesheet");
    let (import_precedences, global_text_defaults, named_template_names) =
        observe_dependency_program(&program);
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
        result_identity: format!("urn:w3c:xslt30:decl:include:{case_name}:result"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit dependency request");
    let results = execute_transform_set(set.seal()).expect("execute dependency case");
    DependencyCaseExecution {
        actual: results.by_request[case_name].serialized.clone(),
        expected,
        import_precedences,
        global_text_defaults,
        named_template_names,
    }
}

type DependencyProgramObservation = (Vec<i32>, Vec<(String, String)>, Vec<String>);

fn observe_dependency_program(
    program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
) -> DependencyProgramObservation {
    let precedences = program
        .matched_templates
        .iter()
        .map(|template| template.import_precedence)
        .collect();
    let text_defaults = program
        .global_bindings
        .iter()
        .filter_map(|binding| match &binding.default {
            crate::xslt::golden_semantics_experiment::GlobalBindingDefault::Text(value) => {
                Some((binding.name.clone(), value.clone()))
            }
            _ => None,
        })
        .collect();
    let named_templates = program
        .named_templates
        .iter()
        .map(|template| template.name.clone())
        .collect();
    (precedences, text_defaults, named_templates)
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

fn assert_xml_equivalent(actual: &str, expected: &str) {
    let limits = ParseLimits {
        max_events: 256,
        max_depth: 32,
    };
    let actual = Document::from_parsed(
        parse_document(
            "urn:fastxslt:include:actual",
            actual.trim().as_bytes(),
            limits,
        )
        .expect("actual XML result should parse"),
    )
    .expect("actual XML result should build");
    let expected = Document::from_parsed(
        parse_document(
            "urn:fastxslt:include:expected",
            expected.trim().as_bytes(),
            limits,
        )
        .expect("expected XML result should parse"),
    )
    .expect("expected XML result should build");
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
    let actual_attributes = actual.attributes(actual_node);
    let expected_attributes = expected.attributes(expected_node);
    assert_eq!(actual_attributes.len(), expected_attributes.len());
    for expected_attribute in expected_attributes {
        let expected_name = expected.name(*expected_attribute).expect("attribute name");
        let actual_attribute = actual_attributes
            .iter()
            .find(|attribute| actual.name(**attribute) == Some(expected_name))
            .expect("matching actual attribute");
        assert_eq!(
            actual.value(*actual_attribute),
            expected.value(*expected_attribute)
        );
    }
    let actual_children = actual.children(actual_node);
    let expected_children = expected.children(expected_node);
    assert_eq!(actual_children.len(), expected_children.len());
    for (actual_child, expected_child) in actual_children.iter().zip(expected_children) {
        assert_xml_nodes_equal(actual, *actual_child, expected, *expected_child);
    }
}

fn execute_inline_case_without_dependencies(case_name: &str) -> (String, String) {
    let document = load_test_set();
    let case = element_children(&document, document_element(&document))
        .into_iter()
        .find(|node| attribute(&document, *node, "name") == Some(case_name))
        .expect("pinned include case");
    let environment_ref = child_named(&document, case, "environment")
        .and_then(|node| attribute(&document, node, "ref"))
        .expect("case environment reference");
    let environment = element_children(&document, document_element(&document))
        .into_iter()
        .find(|node| {
            local_name(&document, *node) == "environment"
                && attribute(&document, *node, "name") == Some(environment_ref)
        })
        .expect("referenced environment");
    let source = child_named(&document, environment, "source").expect("principal source");
    let content = child_named(&document, source, "content").expect("inline source content");
    let test = child_named(&document, case, "test").expect("test metadata");
    let stylesheet_file = child_named(&document, test, "stylesheet")
        .and_then(|node| attribute(&document, node, "file"))
        .expect("principal stylesheet file");
    let result = child_named(&document, case, "result").expect("result metadata");
    let assertion = child_named(&document, result, "assert-xml").expect("XML assertion");
    let expected = document.string_value(assertion);

    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/include");
    let stylesheet_id = format!("https://example.invalid/xslt30/decl/include/{stylesheet_file}");
    let source_id = format!("urn:w3c:xslt30:decl:include:{case_name}:source");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            source_id.clone(),
            document.string_value(content).into_bytes(),
        )
        .expect("admit inline source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file))
                .expect("read principal stylesheet and close handle"),
        )
        .expect("admit principal stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile include case");
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
        result_identity: format!("urn:w3c:xslt30:decl:include:{case_name}:result"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit include request");
    let results = execute_transform_set(set.seal()).expect("execute include case");
    (results.by_request[case_name].serialized.clone(), expected)
}

#[test]
fn inventories_complete_include_denominator_with_explicit_dispositions() {
    let document = load_test_set();
    let cases = element_children(&document, document_element(&document))
        .into_iter()
        .filter(|node| local_name(&document, *node) == "test-case")
        .collect::<Vec<_>>();
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test-case name"))
        .collect::<Vec<_>>();
    assert_eq!(names, CASE_NAMES);

    let mut primary_stylesheets = 0;
    let mut secondary_stylesheets = 0;
    let mut assertion_shapes = BTreeMap::new();
    for case in cases {
        let test = child_named(&document, case, "test").expect("test metadata");
        let stylesheets = element_children(&document, test)
            .into_iter()
            .filter(|node| local_name(&document, *node) == "stylesheet")
            .collect::<Vec<_>>();
        let primary_count = stylesheets
            .iter()
            .filter(|stylesheet| attribute(&document, **stylesheet, "role") != Some("secondary"))
            .count();
        assert_eq!(primary_count, 1, "each case has one principal stylesheet");
        primary_stylesheets += primary_count;
        secondary_stylesheets += stylesheets.len() - primary_count;

        let result = child_named(&document, case, "result").expect("result metadata");
        let assertion = first_element_child(&document, result).expect("result assertion");
        *assertion_shapes
            .entry(local_name(&document, assertion).to_owned())
            .or_insert(0usize) += 1;
    }

    assert_eq!(primary_stylesheets, 16);
    assert_eq!(secondary_stylesheets, 34);
    assert_eq!(
        assertion_shapes,
        BTreeMap::from([
            ("any-of".to_owned(), 1),
            ("assert-xml".to_owned(), 14),
            ("error".to_owned(), 1),
        ])
    );

    assert!(OVERLAY.contains("case_count = 16"));
    assert!(OVERLAY.contains("selection = \"harness-unsupported\""));
    assert!(OVERLAY.contains("execution = \"not-run\""));
    for case_name in PASSED_CASES {
        let case_override = OVERLAY
            .split("[[case_override]]")
            .find(|section| section.contains(&format!("case_name = \"{case_name}\"")))
            .expect("passed case overlay override");
        assert!(case_override.contains("selection = \"selected\""));
        assert!(case_override.contains("execution = \"passed\""));
    }
    for case_name in CASE_NAMES {
        assert!(names.contains(&case_name));
    }
}

fn load_test_set() -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/include/_include-test-set.xml");
    let bytes = fs::read(path).expect("read pinned include test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:decl:include:test-set",
        &bytes,
        ParseLimits {
            max_events: 20_000,
            max_depth: 64,
        },
    )
    .expect("parse pinned include test set");
    Document::from_parsed(parsed).expect("build include test-set document")
}

fn document_element(document: &Document) -> NodeId {
    first_element_child(document, document.document_node()).expect("test-set document element")
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

fn without_xml_declaration(xml: &str) -> &str {
    if xml.starts_with("<?xml") {
        xml.find("?>").map_or(xml, |end| &xml[end + 2..])
    } else {
        xml
    }
}
