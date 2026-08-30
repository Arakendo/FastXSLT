//! Conserved admission for the complete XSLT30 `decl/output` denominator.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use super::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, compile_resource,
    execute_program, execute_transform_set, serialize_xml_bytes,
};
use crate::execution_control_experiment::{CancellationToken, InvocationControl, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "tests/decl/output/_output-test-set.xml";
const CASE_COUNT: usize = 232;
const OVERLAY: &str = include_str!("../../../../corpus/overlays/xslt30/output-denominator-v0.toml");

#[derive(Default)]
struct InventoryObservation {
    names: BTreeSet<String>,
    assertions: BTreeMap<String, usize>,
    specs: BTreeMap<String, usize>,
    features: BTreeMap<String, usize>,
    environment_shapes: BTreeMap<String, usize>,
    direct_stylesheets: usize,
    resolved_environment_stylesheets: usize,
    source_files: usize,
    inline_sources: usize,
    expected_file_references: usize,
}

#[test]
fn executes_output_0128_without_injecting_html_content_type_metadata() {
    let execution = execute_assert_serialization_case("output-0128", "xml");
    assert_eq!(execution.method.as_deref(), Some("xml"));
    assert_eq!(execution.include_content_type, Some(true));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0110_with_explicit_xhtml_declaration_omission() {
    let execution = execute_assert_serialization_case("output-0110", "xhtml");
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert!(execution.omit_xml_declaration);
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_xslt30_true_lexicals_for_xhtml_declaration_omission() {
    for case_name in ["output-0110a", "output-0110b"] {
        let execution = execute_assert_serialization_case(case_name, "xhtml");
        assert_eq!(execution.method.as_deref(), Some("xhtml"));
        assert!(execution.omit_xml_declaration, "{case_name}");
        assert_eq!(
            execution.expected.as_deref(),
            Some(execution.actual.as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn executes_output_0121_with_the_default_xhtml_declaration() {
    let execution = execute_assert_serialization_case("output-0121", "xhtml");
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert!(!execution.omit_xml_declaration);
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0127_through_bounded_all_of_serialization_matches() {
    const CASE_NAME: &str = "output-0127";
    let execution = execute_output_case(CASE_NAME, None);
    assert_eq!(execution.method.as_deref(), Some("xhtml"));
    assert_eq!(execution.include_content_type, Some(false));
    assert!(execution.expected.is_none());

    let (test_set, _) = load_test_set();
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(CASE_NAME)
        })
        .expect("pinned output-0127 case");
    let result = child_named(&test_set, case, "result").expect("output-0127 result");
    let all_of = first_element_child(&test_set, result).expect("output-0127 assertion");
    assert_eq!(local_name(&test_set, all_of), "all-of");
    let patterns = element_children(&test_set, all_of)
        .into_iter()
        .map(|assertion| {
            assert_eq!(local_name(&test_set, assertion), "serialization-matches");
            test_set.string_value(assertion)
        })
        .collect::<Vec<_>>();
    assert_eq!(patterns.len(), 2);
    for pattern in patterns {
        assert!(
            matches_literal_whitespace_pattern(&execution.actual, &pattern)
                .expect("output-0127 pattern belongs to the admitted comparator subset"),
            "pattern did not match: {pattern}"
        );
    }
}

#[test]
fn bounded_serialization_matcher_requires_whitespace_and_rejects_other_operators() {
    assert_eq!(
        matches_literal_whitespace_pattern(
            "<html xmlns=\"urn.test\">",
            "<html\\s+xmlns=\"urn.test\">"
        ),
        Some(true)
    );
    assert_eq!(
        matches_literal_whitespace_pattern(
            "<htmlxmlns=\"urn.test\">",
            "<html\\s+xmlns=\"urn.test\">"
        ),
        Some(false)
    );
    assert_eq!(
        matches_literal_whitespace_pattern("alpha", "alpha|beta"),
        None
    );
    assert_eq!(matches_literal_whitespace_pattern("42", "\\d+"), None);
}

#[test]
fn executes_explicit_false_lexicals_for_retained_xhtml_declaration() {
    for case_name in ["output-0148", "output-0148a", "output-0148b"] {
        let execution = execute_assert_serialization_case(case_name, "xhtml");
        assert_eq!(execution.method.as_deref(), Some("xhtml"));
        assert!(!execution.omit_xml_declaration, "{case_name}");
        assert_eq!(
            execution.expected.as_deref(),
            Some(execution.actual.as_str()),
            "{case_name}"
        );
    }
}

#[test]
fn executes_output_0129_as_descendant_text_without_injected_markup() {
    let execution = execute_assert_serialization_case("output-0129", "html");
    assert_eq!(execution.method.as_deref(), Some("text"));
    assert_eq!(execution.include_content_type, Some(true));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0166_as_utf8_without_a_byte_order_mark() {
    let execution = execute_assert_serialization_case("output-0166", "text");
    assert_eq!(execution.method.as_deref(), Some("xml"));
    assert_eq!(execution.encoding.as_deref(), Some("UTF-8"));
    assert_eq!(execution.byte_order_mark, Some(false));
    assert!(!execution.actual.starts_with('\u{feff}'));
    assert_eq!(
        execution.expected.as_deref(),
        Some(execution.actual.as_str())
    );
}

#[test]
fn executes_output_0165_as_utf8_bytes_with_a_byte_order_mark() {
    const CASE_NAME: &str = "output-0165";
    let bytes = execute_output_bytes_case(CASE_NAME);
    assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    assert_eq!(
        &bytes[3..],
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><html><body>Hello</body></html>"
    );
}

#[test]
fn executes_text_output_with_and_without_a_utf8_byte_order_mark() {
    let with_mark = execute_output_bytes_case("output-0171");
    assert_eq!(with_mark, b"\xef\xbb\xbfHello");

    let without_mark = execute_output_bytes_case("output-0172");
    assert_eq!(without_mark, b"Hello");
}

#[test]
fn executes_xhtml_byte_order_mark_boolean_variants() {
    let body = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>Hello</body></html>";
    for case_name in ["output-0136", "output-0136a", "output-0136b"] {
        let bytes = execute_output_bytes_case(case_name);
        assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]), "{case_name}");
        assert_eq!(&bytes[3..], body, "{case_name}");
    }
    for case_name in ["output-0137", "output-0137a", "output-0137b"] {
        assert_eq!(execute_output_bytes_case(case_name), body, "{case_name}");
    }
}

#[test]
fn executes_xhtml_utf8_multibyte_text_bytes() {
    let bytes = execute_output_bytes_case("output-0139");
    assert_eq!(
        bytes,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>HelloÁ</body></html>"
            .as_bytes()
    );
    assert!(bytes.ends_with(b"Hello\xc3\x81</body></html>"));
}

struct SerializationExecution {
    method: Option<String>,
    encoding: Option<String>,
    include_content_type: Option<bool>,
    byte_order_mark: Option<bool>,
    omit_xml_declaration: bool,
    actual: String,
    expected: Option<String>,
}

fn execute_assert_serialization_case(
    case_name: &str,
    assertion_method: &str,
) -> SerializationExecution {
    execute_output_case(case_name, Some(assertion_method))
}

fn execute_output_case(case_name: &str, assertion_method: Option<&str>) -> SerializationExecution {
    assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    assert!(OVERLAY.contains("execution = \"passed\""));

    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("output test-set directory");
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(case_name)
        })
        .expect("pinned output case");
    let test = child_named(&test_set, case, "test").expect("output test");
    let stylesheet_file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("output stylesheet file");
    let environment = resolve_environment(&test_set, root, case).expect("output environment");
    let source = child_named(&test_set, environment, "source").expect("output source");
    let source_content = child_named(&test_set, source, "content").expect("inline source content");
    let expected_file = assertion_method.map(|method| {
        let result = child_named(&test_set, case, "result").expect("output result");
        let assertion = first_element_child(&test_set, result).expect("output assertion");
        assert_eq!(local_name(&test_set, assertion), "assert-serialization");
        assert_eq!(attribute(&test_set, assertion, "method"), Some(method));
        attribute(&test_set, assertion, "file")
            .expect("expected file")
            .to_owned()
    });

    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(
            source_id.clone(),
            test_set.string_value(source_content).into_bytes(),
        )
        .expect("admit output source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit output stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile output case");
    let method = program.output.method.clone();
    let encoding = program.output.encoding.clone();
    let include_content_type = program.output.include_content_type;
    let byte_order_mark = program.output.byte_order_mark;
    let omit_xml_declaration = program.output.omit_xml_declaration;

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
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit output request");
    let results = execute_transform_set(set.seal()).expect("execute output case");
    let actual = results.by_request[case_name].serialized.clone();
    let expected = expected_file.map(|expected_file| {
        fs::read_to_string(directory.join(expected_file))
            .expect("read expected serialization and close handle")
            .replace("\r\n", "\n")
    });
    SerializationExecution {
        method,
        encoding,
        include_content_type,
        byte_order_mark,
        omit_xml_declaration,
        actual,
        expected,
    }
}

fn execute_output_bytes_case(case_name: &str) -> Vec<u8> {
    assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("output test-set directory");
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(case_name)
        })
        .expect("pinned byte-output case");
    let test = child_named(&test_set, case, "test").expect("byte-output test");
    let stylesheet_file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("byte-output stylesheet file");
    let environment = resolve_environment(&test_set, root, case).expect("byte-output environment");
    let source = child_named(&test_set, environment, "source").expect("byte-output source");
    let source_content = child_named(&test_set, source, "content").expect("inline source content");
    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let source_bytes = test_set.string_value(source_content).into_bytes();

    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(source_id.clone(), source_bytes.clone())
        .expect("admit byte-output source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file))
                .expect("read and close byte-output stylesheet"),
        )
        .expect("admit byte-output stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile byte-output case");
    assert_eq!(program.output.encoding.as_deref(), Some("UTF-8"));

    let parsed = parse_document(
        &source_id,
        &source_bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
    )
    .expect("parse byte-output source");
    let source = Document::from_parsed(parsed).expect("build byte-output source XDM");
    let mut control = InvocationControl::unbounded();
    let result = execute_program(&program, &source, case_name, &mut control)
        .expect("execute byte-output case");
    serialize_xml_bytes(&result, &program.output, case_name, 8_192, &mut control)
        .expect("serialize UTF-8 output with a byte-order mark")
}

fn matches_literal_whitespace_pattern(actual: &str, pattern: &str) -> Option<bool> {
    if pattern.contains(['[', ']', '(', ')', '|', '*', '?', '^', '$', '{', '}'])
        || pattern.replace("\\s+", "").contains('\\')
    {
        return None;
    }
    let mut remainder = actual;
    for (index, literal) in pattern.split("\\s+").enumerate() {
        let Some(position) = remainder.find(literal) else {
            return Some(false);
        };
        let skipped = &remainder[..position];
        if index > 0 && (skipped.is_empty() || !skipped.chars().all(char::is_whitespace)) {
            return Some(false);
        }
        remainder = &remainder[position + literal.len()..];
    }
    Some(true)
}

#[test]
fn inventories_complete_output_denominator_and_seals_each_environment() {
    assert!(OVERLAY.contains(&format!("set_file = \"{SET_FILE}\"")));
    assert!(OVERLAY.contains(&format!("case_count = {CASE_COUNT}")));
    assert!(OVERLAY.contains("selection = \"harness-unsupported\""));
    assert!(OVERLAY.contains("execution = \"not-run\""));

    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("output test-set directory");
    let root = document_element(&test_set);
    assert_eq!(
        dependency_records(&test_set, root, "feature"),
        vec![("serialization", Some("true"))]
    );
    assert_unique_environments(&test_set, root);

    let observation = observe_cases(&test_set, root, directory);
    assert_complete_inventory(&observation);
}

fn assert_unique_environments(document: &Document, root: NodeId) {
    let environments = element_children(document, root)
        .into_iter()
        .filter(|node| local_name(document, *node) == "environment")
        .collect::<Vec<_>>();
    let environment_names = environments
        .iter()
        .map(|node| {
            attribute(document, *node, "name")
                .expect("top-level environment identity")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(environment_names.len(), environments.len());
}

fn observe_cases(document: &Document, root: NodeId, directory: &Path) -> InventoryObservation {
    let cases = element_children(document, root)
        .into_iter()
        .filter(|node| local_name(document, *node) == "test-case")
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), CASE_COUNT);

    let mut observation = InventoryObservation::default();
    for case in cases {
        observe_case(document, root, case, directory, &mut observation);
    }
    observation
}

fn observe_case(
    document: &Document,
    root: NodeId,
    case: NodeId,
    directory: &Path,
    observation: &mut InventoryObservation,
) {
    let case_name = attribute(document, case, "name").expect("output case identity");
    assert!(
        observation.names.insert(case_name.to_owned()),
        "duplicate case {case_name}"
    );
    count_dependencies(document, case, "spec", &mut observation.specs);
    count_dependencies(document, case, "feature", &mut observation.features);

    let result = child_named(document, case, "result").expect("output case result");
    let assertion = first_element_child(document, result).expect("output assertion");
    *observation
        .assertions
        .entry(local_name(document, assertion).to_owned())
        .or_insert(0) += 1;
    observation.expected_file_references += verify_expected_files(document, result, directory);

    let environment = resolve_environment(document, root, case);
    let environment_shape = match child_named(document, case, "environment") {
        Some(reference) if attribute(document, reference, "ref").is_some() => "ref",
        Some(_) => "inline",
        None => "missing",
    };
    *observation
        .environment_shapes
        .entry(environment_shape.to_owned())
        .or_insert(0) += 1;

    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(16, 65_536, 524_288));
    let mut admitted = Vec::new();
    if let Some(test) = child_named(document, case, "test") {
        observation.direct_stylesheets += admit_stylesheets(
            document,
            test,
            directory,
            case_name,
            "test",
            &mut resources,
            &mut admitted,
        );
    }
    if let Some(environment) = environment {
        observation.resolved_environment_stylesheets += admit_stylesheets(
            document,
            environment,
            directory,
            case_name,
            "environment",
            &mut resources,
            &mut admitted,
        );
        let (files, inline) = admit_sources(
            document,
            environment,
            directory,
            case_name,
            &mut resources,
            &mut admitted,
        );
        observation.source_files += files;
        observation.inline_sources += inline;
    }
    assert!(
        !admitted.is_empty(),
        "{case_name} has no admitted engine input"
    );
    let snapshot = resources.seal();
    for identity in admitted {
        assert!(snapshot.get(&identity).is_some(), "missing {identity}");
    }
}

fn assert_complete_inventory(observation: &InventoryObservation) {
    assert_eq!(observation.names.len(), CASE_COUNT);
    assert_eq!(
        observation.assertions,
        BTreeMap::from([
            ("all-of".to_owned(), 89),
            ("any-of".to_owned(), 29),
            ("assert-serialization".to_owned(), 43),
            ("assert-serialization-error".to_owned(), 14),
            ("error".to_owned(), 6),
            ("not".to_owned(), 4),
            ("serialization-matches".to_owned(), 47),
        ])
    );
    assert_eq!(
        observation.specs,
        BTreeMap::from([
            ("XSLT10+".to_owned(), 1),
            ("XSLT20".to_owned(), 7),
            ("XSLT20+".to_owned(), 133),
            ("XSLT30+".to_owned(), 91)
        ])
    );
    assert_eq!(
        observation.features,
        BTreeMap::from([
            ("HTML4".to_owned(), 1),
            ("HTML5".to_owned(), 2),
            ("XPath_3.1".to_owned(), 14),
            ("higher_order_functions".to_owned(), 1),
        ])
    );
    assert_eq!(
        observation.environment_shapes,
        BTreeMap::from([
            ("inline".to_owned(), 3),
            ("missing".to_owned(), 27),
            ("ref".to_owned(), 202)
        ])
    );
    assert_eq!(observation.direct_stylesheets, 223);
    assert_eq!(observation.resolved_environment_stylesheets, 18);
    assert_eq!(observation.source_files, 7);
    assert_eq!(observation.inline_sources, 186);
    assert_eq!(observation.expected_file_references, 50);

    for (family, count) in &observation.assertions {
        assert!(OVERLAY.contains(&format!("name = \"{family}\"\ncount = {count}")));
    }
}

fn admit_stylesheets(
    document: &Document,
    owner: NodeId,
    directory: &Path,
    case_name: &str,
    owner_kind: &str,
    resources: &mut ResourceSetBuilder,
    admitted: &mut Vec<String>,
) -> usize {
    let stylesheets = element_children(document, owner)
        .into_iter()
        .filter(|node| local_name(document, *node) == "stylesheet")
        .collect::<Vec<_>>();
    for (ordinal, stylesheet) in stylesheets.iter().enumerate() {
        let file = attribute(document, *stylesheet, "file").expect("stylesheet file");
        let identity = format!(
            "urn:w3c:xslt30:decl:output:{case_name}:{owner_kind}:stylesheet:{ordinal}:{file}"
        );
        resources
            .admit(
                identity.clone(),
                fs::read(directory.join(file)).expect("read stylesheet and close handle"),
            )
            .expect("admit bounded stylesheet");
        admitted.push(identity);
    }
    stylesheets.len()
}

fn admit_sources(
    document: &Document,
    environment: NodeId,
    directory: &Path,
    case_name: &str,
    resources: &mut ResourceSetBuilder,
    admitted: &mut Vec<String>,
) -> (usize, usize) {
    let sources = element_children(document, environment)
        .into_iter()
        .filter(|node| local_name(document, *node) == "source")
        .collect::<Vec<_>>();
    let mut file_count = 0;
    let mut inline_count = 0;
    for (ordinal, source) in sources.iter().enumerate() {
        let (suffix, bytes) = if let Some(file) = attribute(document, *source, "file") {
            file_count += 1;
            (
                file.to_owned(),
                fs::read(directory.join(file)).expect("read source and close handle"),
            )
        } else {
            inline_count += 1;
            let content = child_named(document, *source, "content").expect("inline source content");
            (
                "inline".to_owned(),
                document.string_value(content).into_bytes(),
            )
        };
        let identity = format!("urn:w3c:xslt30:decl:output:{case_name}:source:{ordinal}:{suffix}");
        resources
            .admit(identity.clone(), bytes)
            .expect("admit bounded source");
        admitted.push(identity);
    }
    (file_count, inline_count)
}

fn verify_expected_files(document: &Document, result: NodeId, directory: &Path) -> usize {
    descendants(result, document)
        .into_iter()
        .filter_map(|node| attribute(document, node, "file"))
        .inspect(|file| {
            assert!(
                directory.join(file).is_file(),
                "missing expected file {file}"
            );
        })
        .count()
}

fn resolve_environment(document: &Document, root: NodeId, case: NodeId) -> Option<NodeId> {
    let reference = child_named(document, case, "environment")?;
    let Some(name) = attribute(document, reference, "ref") else {
        return Some(reference);
    };
    element_children(document, root).into_iter().find(|node| {
        local_name(document, *node) == "environment"
            && attribute(document, *node, "name") == Some(name)
    })
}

fn count_dependencies(
    document: &Document,
    case: NodeId,
    kind: &str,
    counts: &mut BTreeMap<String, usize>,
) {
    for (value, _) in dependency_records(document, case, kind) {
        *counts.entry(value.to_owned()).or_insert(0) += 1;
    }
}

fn dependency_records<'a>(
    document: &'a Document,
    owner: NodeId,
    kind: &str,
) -> Vec<(&'a str, Option<&'a str>)> {
    let Some(dependencies) = child_named(document, owner, "dependencies") else {
        return Vec::new();
    };
    element_children(document, dependencies)
        .into_iter()
        .filter(|node| local_name(document, *node) == kind)
        .map(|node| {
            (
                attribute(document, node, "value").expect("dependency value"),
                attribute(document, node, "satisfied"),
            )
        })
        .collect()
}

fn load_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/output/_output-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned output test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:decl:output:test-set",
        &bytes,
        ParseLimits {
            max_events: 50_000,
            max_depth: 128,
        },
    )
    .expect("parse pinned output test set");
    (
        Document::from_parsed(parsed).expect("build output test-set document"),
        path,
    )
}

fn document_element(document: &Document) -> NodeId {
    first_element_child(document, document.document_node()).expect("test-set document element")
}

fn descendants(parent: NodeId, document: &Document) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in element_children(document, parent) {
        found.push(child);
        found.extend(descendants(child, document));
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
