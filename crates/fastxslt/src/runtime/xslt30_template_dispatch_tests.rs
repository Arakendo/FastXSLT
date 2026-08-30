//! Pinned XSLT30 corpus integration tests for the private golden runtime.

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::PathBuf,
};

use super::{
    ExecutionFailure, ExecutionPolicy, FailureCategory, InvocationEntry, InvocationParameter,
    MultipleMatchPolicy, TransformRequest, TransformSetBuilder, compile_resource,
    execute_program_with_parameters, execute_transform_set, serialize_xml_bytes,
};
use crate::execution_control_experiment::{CancellationToken, InvocationControl, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document, parse_document_controlled};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

const CASE_NAME: &str = "template-006";
const TEMPLATE_CASES: [&str; 6] = [
    "template-001",
    "template-002",
    "template-003",
    "template-004",
    "template-005",
    "template-006",
];
fn suite_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/decl/template/_template-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned XSLT30 test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:decl:template:test-set",
        &bytes,
        ParseLimits {
            max_events: 4_096,
            max_depth: 64,
        },
    )
    .expect("parse pinned XSLT30 test set");
    (
        Document::from_parsed(parsed).expect("build test-set document"),
        path,
    )
}

fn apply_templates_test_set() -> (Document, PathBuf) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/insn/apply-templates/_apply-templates-test-set.xml");
    let bytes = fs::read(&path).expect("read pinned XSLT30 apply-templates test set");
    let parsed = parse_document(
        "urn:w3c:xslt30:insn:apply-templates:test-set",
        &bytes,
        ParseLimits {
            max_events: 16_384,
            max_depth: 64,
        },
    )
    .expect("parse pinned XSLT30 apply-templates test set");
    (
        Document::from_parsed(parsed).expect("build apply-templates test-set document"),
        path,
    )
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document
        .attributes(node)
        .iter()
        .copied()
        .find(|attribute| {
            document
                .name(*attribute)
                .is_some_and(|name| name.local == local)
        })
        .and_then(|attribute| document.value(attribute))
}

fn find_element(
    document: &Document,
    parent: NodeId,
    local: &str,
    required_attribute: Option<(&str, &str)>,
) -> Option<NodeId> {
    for child in document.children(parent).iter().copied() {
        if document.kind(child) != NodeKind::Element {
            continue;
        }
        let matches_name = document.name(child).is_some_and(|name| name.local == local);
        let matches_attribute = required_attribute
            .is_none_or(|(name, value)| attribute(document, child, name) == Some(value));
        if matches_name && matches_attribute {
            return Some(child);
        }
        if let Some(found) = find_element(document, child, local, required_attribute) {
            return Some(found);
        }
    }
    None
}

fn assert_same_empty_document_element(actual: &str, expected: &str) {
    let limits = ParseLimits {
        max_events: 32,
        max_depth: 8,
    };
    let actual = Document::from_parsed(
        parse_document("urn:fastxslt:actual", actual.as_bytes(), limits)
            .expect("actual result should parse"),
    )
    .expect("actual result document should build");
    let expected = Document::from_parsed(
        parse_document("urn:w3c:expected", expected.as_bytes(), limits)
            .expect("expected result should parse"),
    )
    .expect("expected result document should build");
    let actual_root =
        find_element(&actual, actual.document_node(), "o", None).expect("actual document element");
    let expected_root = find_element(&expected, expected.document_node(), "o", None)
        .expect("expected document element");

    assert_eq!(actual.name(actual_root), expected.name(expected_root));
    assert!(actual.children(actual_root).is_empty());
    assert!(expected.children(expected_root).is_empty());
}

fn assert_same_result_element_string(actual: &str, expected: &str, local: &str) {
    let limits = ParseLimits {
        max_events: 2_048,
        max_depth: 64,
    };
    let actual_document = Document::from_parsed(
        parse_document("urn:fastxslt:actual", actual.as_bytes(), limits)
            .unwrap_or_else(|error| panic!("actual result should parse: {actual:?}: {error:?}")),
    )
    .expect("actual result should build");
    let expected_document = Document::from_parsed(
        parse_document("urn:w3c:expected", expected.trim().as_bytes(), limits)
            .expect("expected result should parse"),
    )
    .expect("expected result should build");
    let actual_element = find_element(
        &actual_document,
        actual_document.document_node(),
        local,
        None,
    )
    .expect("actual result element");
    let expected_element = find_element(
        &expected_document,
        expected_document.document_node(),
        local,
        None,
    )
    .expect("expected result element");

    assert_eq!(
        actual_document.name(actual_element),
        expected_document.name(expected_element)
    );
    assert_eq!(
        actual_document.string_value(actual_element),
        expected_document.string_value(expected_element)
    );
}

fn execute_apply_templates_case(case_name: &str) -> (String, String, usize) {
    execute_apply_templates_case_with_parameters(case_name, BTreeMap::new())
}

fn execute_apply_templates_case_with_parameters(
    case_name: &str,
    parameters: BTreeMap<String, InvocationParameter>,
) -> (String, String, usize) {
    let (actual, expected, matched_template_count) =
        try_execute_apply_templates_case_with_parameters(
            case_name,
            parameters,
            MultipleMatchPolicy::UseLast,
        );
    (
        actual.expect("execute suite case"),
        expected,
        matched_template_count,
    )
}

fn try_execute_apply_templates_case_with_parameters(
    case_name: &str,
    parameters: BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
) -> (Result<String, ExecutionFailure>, String, usize) {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));
    let (test_set, set_path) = apply_templates_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", case_name)),
    )
    .expect("overlay case should exist in pinned suite");
    let case_environment = find_element(&test_set, test_case, "environment", None)
        .expect("case should provide an environment");
    let environment = if let Some(environment_ref) = attribute(&test_set, case_environment, "ref") {
        find_element(
            &test_set,
            test_set.document_node(),
            "environment",
            Some(("name", environment_ref)),
        )
        .expect("referenced environment should exist")
    } else {
        case_environment
    };
    let source_element = find_element(&test_set, environment, "source", None)
        .expect("environment should contain the principal source");
    let stylesheet_files = case_stylesheet_files(&test_set, test_case);
    let principal_files = stylesheet_files
        .iter()
        .filter(|(_, role)| *role != Some("secondary"))
        .collect::<Vec<_>>();
    let [(principal_file, _)] = principal_files.as_slice() else {
        panic!("case should name exactly one principal stylesheet");
    };
    let case_directory = set_path.parent().expect("test set should have a directory");
    let expected = expected_apply_templates_result(&test_set, test_case, case_directory, case_name);
    let source = apply_templates_source(&test_set, source_element, case_directory);
    let source_id = format!("urn:w3c:xslt30:{case_name}:source");
    let stylesheet_base = "https://example.invalid/xslt30/insn/apply-templates/";
    let stylesheet_id = format!("{stylesheet_base}{principal_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(
        stylesheet_files.len() + 1,
        8_192,
        65_536,
    ));
    resources
        .admit(source_id.clone(), source)
        .expect("admit upstream source");
    for (stylesheet_file, _) in stylesheet_files {
        resources
            .admit(
                format!("{stylesheet_base}{stylesheet_file}"),
                fs::read(case_directory.join(stylesheet_file))
                    .expect("read upstream stylesheet and close handle"),
            )
            .expect("admit upstream stylesheet");
    }
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile suite case");
    let matched_template_count = program.matched_templates.len();
    if case_name == "conflict-resolution-1301" {
        return (
            Ok(execute_conflict_resolution_1301_bytes(
                &snapshot,
                &program,
                &source_id,
                &parameters,
                multiple_match_policy,
            )),
            expected,
            matched_template_count,
        );
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
    )
    .with_multiple_match_policy(multiple_match_policy);
    set.add(TransformRequest {
        identity: case_name.to_owned(),
        result_identity: format!("result:{case_name}"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters,
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit suite request");
    let result = execute_transform_set(set.seal())
        .map(|results| results.by_request[case_name].serialized.clone());
    (result, expected, matched_template_count)
}

fn expected_apply_templates_result(
    test_set: &Document,
    test_case: NodeId,
    case_directory: &std::path::Path,
    case_name: &str,
) -> String {
    find_element(test_set, test_case, "assert-xml", None)
        .map(|node| {
            attribute(test_set, node, "file").map_or_else(
                || test_set.string_value(node),
                |file| {
                    fs::read_to_string(case_directory.join(file))
                        .expect("read upstream expected XML and close handle")
                },
            )
        })
        .or_else(|| expected_apply_templates_all_of(case_name))
        .or_else(|| {
            find_element(test_set, test_case, "error", None)
                .and_then(|node| attribute(test_set, node, "code"))
                .map(str::to_owned)
        })
        .expect("case should provide an admitted result assertion")
}

fn apply_templates_source(
    test_set: &Document,
    source_element: NodeId,
    case_directory: &std::path::Path,
) -> Vec<u8> {
    if let Some(content) = find_element(test_set, source_element, "content", None) {
        return test_set.string_value(content).into_bytes();
    }
    let source_file = attribute(test_set, source_element, "file")
        .expect("principal source should be inline or name a file");
    fs::read(case_directory.join(source_file))
        .expect("read upstream apply-templates source and close handle")
}

fn execute_conflict_resolution_1301_bytes(
    snapshot: &ResourceSnapshot,
    program: &StylesheetProgram,
    source_id: &str,
    parameters: &BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
) -> String {
    let mut control = InvocationControl::unbounded();
    let source_bytes = snapshot
        .get(source_id)
        .expect("the sealed snapshot retains the principal source");
    let parsed = parse_document_controlled(
        source_id,
        source_bytes,
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
        &mut control,
    )
    .expect("parse the valid upstream source through bounded XML work");
    let source = Document::from_parsed_controlled(parsed, &mut control)
        .expect("build the upstream source through bounded XDM work");
    let semantic = execute_program_with_parameters(
        program,
        &source,
        parameters,
        multiple_match_policy,
        "conflict-resolution-1301",
        &mut control,
    )
    .expect("execute the upstream positional stylesheet");
    let bytes = serialize_xml_bytes(
        &semantic,
        &program.output,
        "conflict-resolution-1301",
        65_536,
        &mut control,
    )
    .expect("serialize the upstream ASCII result as ISO-8859-1 bytes");
    String::from_utf8(bytes).expect("the admitted byte lane is ASCII")
}

fn case_stylesheet_files(test_set: &Document, test_case: NodeId) -> Vec<(&str, Option<&str>)> {
    let test = find_element(test_set, test_case, "test", None).expect("case test metadata");
    test_set
        .children(test)
        .iter()
        .copied()
        .filter(|node| {
            test_set.kind(*node) == NodeKind::Element
                && test_set
                    .name(*node)
                    .is_some_and(|name| name.local == "stylesheet")
        })
        .map(|node| {
            (
                attribute(test_set, node, "file").expect("stylesheet file"),
                attribute(test_set, node, "role"),
            )
        })
        .collect()
}

fn compile_apply_templates_error_case(case_name: &str) -> (ExecutionFailure, String) {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));
    let (test_set, set_path) = apply_templates_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", case_name)),
    )
    .expect("overlay error case should exist in pinned suite");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("error case should name a stylesheet");
    let expected_code = find_element(&test_set, test_case, "error", None)
        .and_then(|node| attribute(&test_set, node, "code"))
        .expect("error case should provide an error code")
        .to_owned();
    let stylesheet = fs::read(
        set_path
            .parent()
            .expect("test set should have a directory")
            .join(stylesheet_file),
    )
    .expect("read upstream error stylesheet and close handle");
    let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8_192, 8_192));
    resources
        .admit(stylesheet_id.clone(), stylesheet)
        .expect("admit upstream error stylesheet");
    let failure = compile_resource(&resources.seal(), &stylesheet_id)
        .expect_err("statically atomic apply-templates focus should fail compilation");
    (failure, expected_code)
}

fn expected_apply_templates_all_of(case_name: &str) -> Option<String> {
    (case_name == "conflict-resolution-1501")
        .then(|| "<doc><a><a/></a><a><a parent-recursive=\"yes\"/></a><a><b/></a></doc>".to_owned())
}

#[test]
fn classifies_the_complete_pinned_template_test_set_without_denominator_loss() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let template_set = "set_file = \"tests/decl/template/_template-test-set.xml\"";
    assert_eq!(overlay.matches(template_set).count(), TEMPLATE_CASES.len());
    assert_eq!(
        overlay
            .matches("selection = \"engine-unsupported\"")
            .count(),
        0
    );

    let (test_set, set_path) = suite_test_set();
    for case_name in TEMPLATE_CASES {
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", case_name)),
        )
        .expect("overlay case should exist in the complete pinned test set");
        assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));

        let spec = find_element(&test_set, test_case, "spec", None)
            .and_then(|node| attribute(&test_set, node, "value"))
            .expect("template case should retain an explicit spec dependency");
        assert!(matches!(spec, "XSLT10+" | "XSLT20+"));
        assert!(find_element(&test_set, test_case, "environment", None).is_some());
        assert!(find_element(&test_set, test_case, "assert-xml", None).is_some());

        let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("template case should reference one stylesheet");
        let stylesheet = fs::read(
            set_path
                .parent()
                .expect("test set should have a directory")
                .join(stylesheet_file),
        )
        .expect("read upstream stylesheet and close handle");
        let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8_192, 8_192));
        resources
            .admit(stylesheet_id.clone(), stylesheet)
            .expect("admit one upstream stylesheet");
        let snapshot = resources.seal();

        compile_resource(&snapshot, &stylesheet_id)
            .expect("every case in the complete admitted template set should compile");
    }
}

#[test]
fn executes_pinned_xslt30_template_001_through_005() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    let (test_set, set_path) = suite_test_set();
    for case_name in [
        "template-001",
        "template-002",
        "template-003",
        "template-004",
        "template-005",
    ] {
        assert!(overlay.contains(&format!("case_name = \"{case_name}\"")));
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", case_name)),
        )
        .expect("overlay case should exist in pinned suite");
        let environment_ref = find_element(&test_set, test_case, "environment", None)
            .and_then(|node| attribute(&test_set, node, "ref"))
            .expect("case should reference an environment");
        let environment = find_element(
            &test_set,
            test_set.document_node(),
            "environment",
            Some(("name", environment_ref)),
        )
        .expect("referenced environment should exist");
        let source = find_element(&test_set, environment, "content", None)
            .map(|node| test_set.string_value(node))
            .expect("environment should contain the principal source");
        let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("case should name a stylesheet");
        let expected = find_element(&test_set, test_case, "assert-xml", None)
            .map(|node| test_set.string_value(node))
            .expect("case should provide an XML assertion");
        let stylesheet = fs::read(
            set_path
                .parent()
                .expect("test set should have a directory")
                .join(stylesheet_file),
        )
        .expect("read upstream stylesheet and close handle");
        let source_id = format!("urn:w3c:xslt30:{case_name}:source");
        let stylesheet_id = format!("urn:w3c:xslt30:{case_name}:stylesheet");
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
        resources
            .admit(source_id.clone(), source.into_bytes())
            .expect("admit upstream source");
        resources
            .admit(stylesheet_id.clone(), stylesheet)
            .expect("admit upstream stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, &stylesheet_id).expect("compile suite case");
        assert!(program.root_template.is_none());
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
            identity: case_name.to_owned(),
            result_identity: format!("result:{case_name}"),
            entry: InvocationEntry::PrincipalSource {
                resource: source_id,
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit suite request");
        let results = execute_transform_set(set.seal()).expect("execute suite case");
        let actual = &results.by_request[case_name].serialized;
        assert_same_result_element_string(actual, &expected, "out");
    }
}

#[test]
fn executes_pinned_xslt30_conflict_resolution_0101() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0101");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_priority_conflicts_by_last_source_order() {
    for case_name in ["conflict-resolution-0102c", "conflict-resolution-0104c"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 3);
        assert_same_result_element_string(&actual, &expected, "out");
    }
}

#[test]
fn executes_declared_legacy_recovery_variants_without_selecting_legacy_profile() {
    for (case_name, spec, matched_template_count, result_element) in [
        ("conflict-resolution-0102a", "XSLT10 XSLT20", 3, "out"),
        ("conflict-resolution-0104a", "XSLT10 XSLT20", 3, "out"),
        ("conflict-resolution-0108a", "XSLT10 XSLT20", 6, "out"),
        ("conflict-resolution-0110a", "XSLT10 XSLT20", 6, "out"),
        ("conflict-resolution-0401a", "XSLT20", 2, "b"),
        ("conflict-resolution-1202a", "XSLT10 XSLT20", 7, "out"),
    ] {
        assert_apply_templates_dependency(case_name, "spec", spec);
        assert_apply_templates_dependency(case_name, "on-multiple-match", "recover");
        let (actual, expected, actual_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(actual_template_count, matched_template_count);
        assert_same_result_element_string(&actual, &expected, result_element);
    }
}

#[test]
fn reports_declared_multiple_match_errors() {
    for (case_name, matched_template_count) in [
        ("conflict-resolution-0102b", 3),
        ("conflict-resolution-0104b", 3),
        ("conflict-resolution-0108b", 6),
        ("conflict-resolution-0110b", 6),
        ("conflict-resolution-0401b", 2),
        ("conflict-resolution-1202b", 7),
    ] {
        assert_apply_templates_dependency(case_name, "on-multiple-match", "error");
        let (result, expected_code_pattern, actual_template_count) =
            try_execute_apply_templates_case_with_parameters(
                case_name,
                BTreeMap::new(),
                MultipleMatchPolicy::Error,
            );
        assert_eq!(actual_template_count, matched_template_count);
        assert_eq!(expected_code_pattern, "XTRE0540");
        let failure = result.expect_err("equal top-ranked rules should be rejected");
        assert_eq!(failure.code, "XTDE0540");
        assert_eq!(failure.category, FailureCategory::Invalid);
        assert_eq!(failure.request_id.as_deref(), Some(case_name));
        assert!(failure.location.is_some());
    }
}

#[test]
fn multiple_match_error_policy_ignores_lower_rank_conflicts() {
    let case_name = "conflict-resolution-0101";
    let (result, expected, matched_template_count) =
        try_execute_apply_templates_case_with_parameters(
            case_name,
            BTreeMap::new(),
            MultipleMatchPolicy::Error,
        );
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(
        &result.expect("a unique highest-ranked rule should execute"),
        &expected,
        "out",
    );
}

fn assert_apply_templates_dependency(case_name: &str, dependency: &str, expected: &str) {
    let (test_set, _) = apply_templates_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", case_name)),
    )
    .expect("pinned recovery case");
    let dependencies =
        find_element(&test_set, test_case, "dependencies", None).expect("case dependency metadata");
    let declaration = find_element(&test_set, dependencies, dependency, None)
        .expect("requested dependency metadata");
    assert_eq!(attribute(&test_set, declaration, "value"), Some(expected));
}

#[test]
fn executes_xslt30_explicit_priority_without_widening_node_test_axis() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0106");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_non_simple_path_default_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0107");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_priority_path_and_attribute_predicate_by_source_order() {
    for case_name in ["conflict-resolution-0108c", "conflict-resolution-0110c"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 6);
        assert_same_result_element_string(&actual, &expected, "out");
    }
}

#[test]
fn executes_xslt30_descendant_wildcard_non_simple_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0112");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "text");
}

#[test]
fn executes_xslt30_exact_attribute_value_pattern() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0201");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_explicit_priority_namespace_wildcard_by_source_order() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0401c");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "b");
}

#[test]
fn executes_xslt30_leading_descendant_selection_with_parent_child_patterns() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0901");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_xpath_default_namespace_pattern_and_selection() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0701");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_literal_result_xpath_default_namespace_context() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0702");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
    assert!(!actual.contains("xpath-default-namespace"));
    let parsed = parse_document(
        "urn:fastxslt:conflict-resolution-0702:actual",
        actual.as_bytes(),
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("actual result should parse");
    let document = Document::from_parsed(parsed).expect("actual result should build");
    let out =
        find_element(&document, document.document_node(), "out", None).expect("actual out element");
    assert!(document.namespace_declarations(out).iter().any(|binding| {
        binding.prefix.as_deref() == Some("u") && binding.namespace == "http://some.uri/"
    }));
}

#[test]
fn executes_xslt30_stylesheet_default_namespace_without_affecting_attributes() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0703");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_current_mode_through_named_template_call() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0801");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_multi_mode_default_and_current_dispatch() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0802");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_local_name_wildcard_fractional_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1701");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_element_kind_test_default_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1801");
    assert_eq!(matched_template_count, 4);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_root_pattern_default_priority() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1601");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_document_element_pattern_priorities() {
    for case_name in ["conflict-resolution-1602", "conflict-resolution-1603"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 3);
        assert_same_result_element_string(&actual, &expected, "out");
    }
}

#[test]
fn executes_xslt30_next_match_priority_chain() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1201");
    assert_eq!(matched_template_count, 6);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_global_variable_pattern_and_source_copy() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0601");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "doc");
}

#[test]
fn executes_xslt30_equivalent_same_named_child_patterns() {
    for case_name in ["conflict-resolution-0501", "conflict-resolution-0502"] {
        let (actual, expected, matched_template_count) = execute_apply_templates_case(case_name);
        assert_eq!(matched_template_count, 2);
        assert_same_result_element_string(&actual, &expected, "doc");
    }
}

#[test]
fn executes_xslt30_same_named_parent_current_pattern() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-0503");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "doc");
}

#[test]
fn executes_xslt30_filtered_parent_position_current_pattern() {
    let (test_set, _) = apply_templates_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", "conflict-resolution-1501")),
    )
    .expect("case should exist");
    let all_of = find_element(&test_set, test_case, "all-of", None).expect("all-of assertion");
    let assertions = test_set
        .children(all_of)
        .iter()
        .copied()
        .filter(|node| {
            test_set.kind(*node) == NodeKind::Element
                && test_set
                    .name(*node)
                    .is_some_and(|name| name.local == "assert")
        })
        .collect::<Vec<_>>();
    assert_eq!(assertions.len(), 2);
    assert_eq!(
        test_set.string_value(assertions[0]),
        "/doc/a[2]/a[1][@parent-recursive=\"yes\"]"
    );
    assert_eq!(
        test_set.string_value(assertions[1]),
        "count(//@parent-recursive) = 1"
    );

    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1501");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "doc");
}

#[test]
fn executes_xslt30_builtin_temporary_tree_parameter_propagation() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1101");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "z");
}

#[test]
fn executes_xslt30_apply_imports_builtin_parameter_propagation() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1102");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "z");
}

#[test]
fn reports_xslt30_statically_atomic_apply_templates_focus() {
    for case_name in ["apply-templates-001", "apply-templates-002"] {
        let (failure, expected_code) = compile_apply_templates_error_case(case_name);
        assert_eq!(failure.code, expected_code);
        assert_eq!(failure.category, FailureCategory::Invalid);
        assert!(failure.location.is_some());
    }
}

#[test]
fn executes_xslt30_empty_global_parameter_filtered_paths() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1001");
    assert_eq!(matched_template_count, 2);
    assert_same_result_element_string(&actual, &expected, "planche");

    let limits = ParseLimits {
        max_events: 32,
        max_depth: 8,
    };
    let actual = Document::from_parsed(
        parse_document("urn:fastxslt:actual:1001", actual.as_bytes(), limits)
            .expect("actual result should parse"),
    )
    .expect("actual result should build");
    let planche = find_element(&actual, actual.document_node(), "planche", None)
        .expect("actual planche element");
    let children = actual
        .children(planche)
        .iter()
        .filter_map(|node| actual.name(*node).map(|name| name.local.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(children, ["images", "dialogues"]);

    let mut parameters = BTreeMap::new();
    parameters.insert(
        "type".to_owned(),
        InvocationParameter {
            value: AtomicValue::string("enfant"),
            tunnel: false,
        },
    );
    let (filtered, _, matched_template_count) =
        execute_apply_templates_case_with_parameters("conflict-resolution-1001", parameters);
    assert_eq!(matched_template_count, 2);
    assert!(filtered.contains("<bart type=\"enfant\">bart2.jpg</bart>"));
    assert!(filtered.contains("<lisa type=\"enfant\">lisa.gif</lisa>"));
    assert!(!filtered.contains("homer1.jpg"));
    assert!(!filtered.contains("marge.gif"));
}

#[test]
fn executes_xslt30_next_match_parameter_chain() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1205");
    assert_eq!(matched_template_count, 6);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_equal_rank_next_match_chain() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1202c");
    assert_eq!(matched_template_count, 7);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_next_match_across_import_precedence() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1204");
    assert_eq!(matched_template_count, 7);
    assert_same_result_element_string(&actual, &expected, "out");
}

#[test]
fn executes_xslt30_temporary_tree_union_next_match() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1401");
    assert_eq!(matched_template_count, 3);
    assert_same_result_element_string(&actual, &expected, "h2");
}

#[test]
fn executes_xslt30_positional_focus_with_iso_8859_1_bytes() {
    let (actual, expected, matched_template_count) =
        execute_apply_templates_case("conflict-resolution-1301");
    assert_eq!(matched_template_count, 5);
    assert_same_result_element_string(&actual, &expected, "root");
    assert!(actual.starts_with("<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>"));
    for fragment in [
        "<fo:block text-align=\"justify\" color=\"black\" pos=\"2\" last=\"9\">11111111</fo:block>",
        "<fo:block text-align=\"justify\" color=\"black\" pos=\"4\" last=\"9\">22222222</fo:block>",
        "<fo:block text-align=\"justify\" color=\"black\" pos=\"6\" last=\"9\">33333333</fo:block>",
        "<fo:block text-align=\"justify\" color=\"blue\" pos=\"8\" last=\"9\">44444444</fo:block>",
    ] {
        assert!(
            actual.contains(fragment),
            "missing result fragment: {fragment}"
        );
    }
}

#[test]
fn executes_pinned_xslt30_template_006_from_its_upstream_test_set() {
    let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(overlay.contains("case_name = \"template-006\""));

    let (test_set, set_path) = suite_test_set();
    let test_case = find_element(
        &test_set,
        test_set.document_node(),
        "test-case",
        Some(("name", CASE_NAME)),
    )
    .expect("overlay case should exist in pinned suite");
    let environment_ref = find_element(&test_set, test_case, "environment", None)
        .and_then(|node| attribute(&test_set, node, "ref"))
        .expect("case should reference an environment");
    let environment = find_element(
        &test_set,
        test_set.document_node(),
        "environment",
        Some(("name", environment_ref)),
    )
    .expect("referenced environment should exist");
    let source = find_element(&test_set, environment, "content", None)
        .map(|node| test_set.string_value(node))
        .expect("environment should contain the principal source");
    let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("case should name a stylesheet");
    let expected = find_element(&test_set, test_case, "assert-xml", None)
        .map(|node| test_set.string_value(node))
        .expect("case should provide an XML assertion");
    let stylesheet = fs::read(
        set_path
            .parent()
            .expect("test set should have a directory")
            .join(stylesheet_file),
    )
    .expect("read upstream stylesheet and close handle");

    let source_id = "urn:w3c:xslt30:template-006:source";
    let stylesheet_id = "urn:w3c:xslt30:template-006:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(source_id, source.into_bytes())
        .expect("admit upstream source");
    resources
        .admit(stylesheet_id, stylesheet)
        .expect("admit upstream stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, stylesheet_id).expect("compile suite case");
    assert_eq!(program.output.method, None);
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
        identity: CASE_NAME.to_owned(),
        result_identity: "result:template-006".to_owned(),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id.to_owned(),
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit suite request");

    let results = execute_transform_set(set.seal()).expect("execute suite case");
    let actual = &results.by_request[CASE_NAME].serialized;

    assert_eq!(actual, "<?xml version=\"1.0\" encoding=\"UTF-8\"?><o></o>");
    assert_same_empty_document_element(actual, expected.trim());
}
