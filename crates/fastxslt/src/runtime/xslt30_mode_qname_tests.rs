//! Pinned XSLT30 evidence for expanded-QName mode identity.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::PathBuf,
};

use super::{
    ExecutionFailure, ExecutionPolicy, FailureCategory, InvocationEntry, InvocationParameter,
    MultipleMatchPolicy, TransformRequest, TransformSetBuilder, compile_resource,
    execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits, parse_document};

const TEST_SET: &str = "tests/attr/mode/_mode-test-set.xml";
const SELECTED_CASES: [&str; 80] = [
    "mode-0001",
    "mode-0003",
    "mode-0005",
    "mode-0015",
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
    "mode-0801a",
    "mode-0801b",
    "mode-0801c",
    "mode-0803",
    "mode-0805",
    "mode-0806",
    "mode-0901",
    "mode-1001",
    "mode-1101",
    "mode-1102",
    "mode-1103",
    "mode-1104",
    "mode-1105",
    "mode-1108",
    "mode-1201",
    "mode-1202",
    "mode-1203",
    "mode-1204",
    "mode-1301",
    "mode-1405",
    "mode-1407",
    "mode-1409",
    "mode-1411",
    "mode-1415",
    "mode-1417",
    "mode-1419",
    "mode-1421",
    "mode-1423",
    "mode-1431",
    "mode-1433",
    "mode-1434",
    "mode-1435",
    "mode-1439",
    "mode-1444",
    "mode-1445",
    "mode-1446",
    "mode-1447",
    "mode-1501",
    "mode-1502",
    "mode-1507",
    "mode-1508",
    "mode-1509",
    "mode-1601",
    "mode-1602",
    "mode-1603",
    "mode-1604",
    "mode-1605",
    "mode-1606",
    "mode-1607",
    "mode-1608",
    "mode-1609",
    "mode-1610",
    "mode-1611",
    "mode-1612",
    "mode-1613",
    "mode-1614",
    "mode-1615",
    "mode-1616",
    "mode-1617",
    "mode-1618",
    "mode-1619",
    "mode-1904",
];
const STREAMING_EXCLUDED_CASES: [&str; 26] = [
    "mode-0002",
    "mode-0004",
    "mode-0006",
    "mode-0008",
    "mode-0010",
    "mode-0012",
    "mode-0014",
    "mode-1406",
    "mode-1408",
    "mode-1410",
    "mode-1412",
    "mode-1414",
    "mode-1416",
    "mode-1418",
    "mode-1420",
    "mode-1422",
    "mode-1424",
    "mode-1426",
    "mode-1428",
    "mode-1430",
    "mode-1432",
    "mode-1436",
    "mode-1437",
    "mode-1438",
    "mode-1506",
    "mode-1903",
];
const PACKAGE_EXCLUDED_CASES: [&str; 19] = [
    "mode-1701",
    "mode-1701a",
    "mode-1702",
    "mode-1702a",
    "mode-1703",
    "mode-1704",
    "mode-1705",
    "mode-1705a",
    "mode-1705b",
    "mode-1706",
    "mode-1707",
    "mode-1708",
    "mode-1709",
    "mode-1710",
    "mode-1711",
    "mode-1712",
    "mode-1713",
    "mode-1714err",
    "mode-1803",
];
const LARGE_RESULT_CASES: [&str; 8] = [
    "mode-1405",
    "mode-1407",
    "mode-1409",
    "mode-1411",
    "mode-1415",
    "mode-1423",
    "mode-1445",
    "mode-1446",
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
        SELECTED_CASES.len() + STREAMING_EXCLUDED_CASES.len() + PACKAGE_EXCLUDED_CASES.len()
    );
    for case_name in SELECTED_CASES {
        assert!(names.contains(case_name));
        assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    }
    for case_name in STREAMING_EXCLUDED_CASES {
        assert!(names.contains(case_name));
        let case = cases
            .iter()
            .copied()
            .find(|case| attribute(&document, *case, "name") == Some(case_name))
            .expect("streaming-excluded case");
        assert_eq!(
            case_dependency(&document, case, "feature"),
            Some("streaming")
        );
        assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    }
    for case_name in PACKAGE_EXCLUDED_CASES {
        assert!(names.contains(case_name));
        let case = cases
            .iter()
            .copied()
            .find(|case| attribute(&document, *case, "name") == Some(case_name))
            .expect("package-excluded case");
        let test = child_named(&document, case, "test").expect("test metadata");
        let has_package_artifact = child_named(&document, test, "package").is_some();
        let has_stylesheet_wrapped_package = case_name == "mode-1803"
            && child_named(&document, test, "stylesheet")
                .and_then(|stylesheet| attribute(&document, stylesheet, "file"))
                == Some("mode-1803.xsl");
        assert!(
            has_package_artifact || has_stylesheet_wrapped_package,
            "{case_name} must retain its native package artifact"
        );
        assert!(OVERLAY.contains(&format!("case_name = \"{case_name}\"")));
    }
    assert_eq!(
        OVERLAY.matches("selection = \"selected\"").count(),
        SELECTED_CASES.len()
    );
    assert_eq!(
        OVERLAY
            .matches("selection = \"excluded-by-profile\"")
            .count(),
        45
    );
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
fn executes_inherited_default_mode_on_literal_and_template_elements() {
    for case_name in [
        "mode-1601",
        "mode-1602",
        "mode-1603",
        "mode-1604",
        "mode-1605",
        "mode-1606",
    ] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_inherited_default_mode_on_if_and_nested_elements() {
    for case_name in [
        "mode-1610",
        "mode-1611",
        "mode-1612",
        "mode-1613",
        "mode-1614",
        "mode-1615",
    ] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_inherited_default_mode_with_for_each_focus() {
    for case_name in ["mode-1607", "mode-1608", "mode-1609"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_included_module_default_mode_in_its_native_static_context() {
    for case_name in ["mode-1616", "mode-1617"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_nested_default_modes_across_element_and_attribute_focus() {
    for case_name in ["mode-1618", "mode-1619"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_native_multiple_match_recovery_and_error_policies() {
    for case_name in ["mode-0801a", "mode-0801c"] {
        let document = load_test_set();
        let case = find_case(&document, case_name);
        if case_name == "mode-0801a" {
            assert_eq!(
                case_dependency(&document, case, "on-multiple-match"),
                Some("recover")
            );
        } else {
            assert_eq!(case_dependency(&document, case, "spec"), Some("XSLT30+"));
            assert_eq!(case_dependency(&document, case, "on-multiple-match"), None);
        }
        let (actual, expected) = execute_case_with_policy(case_name, MultipleMatchPolicy::UseLast)
            .expect("recovery policy should choose the later equal-ranked rule");
        assert_xml_equivalent(&actual, expected.as_deref().expect("XML assertion"));
    }

    let document = load_test_set();
    let case = find_case(&document, "mode-0801b");
    assert_eq!(
        case_dependency(&document, case, "on-multiple-match"),
        Some("error")
    );
    let expected_error = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "error",
    )
    .and_then(|error| attribute(&document, error, "code"));
    assert_eq!(expected_error, Some("XTRE0540"));

    let failure = execute_case_with_policy("mode-0801b", MultipleMatchPolicy::Error)
        .expect_err("error policy should reject equal highest-ranked rules");
    assert_eq!(failure.code, "XTDE0540");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some("mode-0801b"));
    assert!(failure.location.is_some());
}

#[test]
fn executes_warning_disabled_mode_declarations() {
    for case_name in ["mode-0803", "mode-0805"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn rejects_invalid_warning_on_multiple_match_boolean() {
    let document = load_test_set();
    let case = find_case(&document, "mode-0806");
    let expected_error = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "error",
    )
    .and_then(|error| attribute(&document, error, "code"));
    assert_eq!(expected_error, Some("XTSE0020"));

    let failure = compile_case_only("mode-0806")
        .expect_err("mixed-case boolean should fail stylesheet compilation");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert!(failure.location.is_some());
}

#[test]
fn rejects_invalid_warning_and_typed_mode_booleans() {
    for case_name in ["mode-1444", "mode-1447"] {
        let document = load_test_set();
        let case = find_case(&document, case_name);
        let expected_error = child_named(
            &document,
            child_named(&document, case, "result").expect("result metadata"),
            "error",
        )
        .and_then(|error| attribute(&document, error, "code"));
        assert_eq!(expected_error, Some("XTSE0020"));

        let failure = compile_case_only(case_name)
            .expect_err("invalid mode boolean should fail stylesheet compilation");
        assert_eq!(failure.code, "XTSE0020");
        assert_eq!(failure.category, FailureCategory::Invalid);
        assert!(failure.location.is_some());
    }
}

#[test]
fn reports_typed_mode_requirement_for_the_native_untyped_source() {
    let document = load_test_set();
    let case = find_case(&document, "mode-1439");
    let expected_error = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "error",
    )
    .and_then(|error| attribute(&document, error, "code"));
    assert_eq!(expected_error, Some("XTTE3100"));

    let failure = execute_case_with_policy("mode-1439", MultipleMatchPolicy::UseLast)
        .expect_err("typed mode should reject the native untyped source");
    assert_eq!(failure.code, "XTTE3100");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(
        failure
            .location
            .as_ref()
            .map(|location| location.resource.as_str()),
        Some("https://example.invalid/xslt30/attr/mode/mode-1439.xsl")
    );
}

#[test]
fn reports_fail_on_no_match_for_the_native_unmatched_text_node() {
    let document = load_test_set();
    let case = find_case(&document, "mode-1431");
    let expected_error = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "error",
    )
    .and_then(|error| attribute(&document, error, "code"));
    assert_eq!(expected_error, Some("XTDE0555"));

    let failure = execute_case_with_policy("mode-1431", MultipleMatchPolicy::UseLast)
        .expect_err("fail-on-no-match mode should reject the first unmatched text node");
    assert_eq!(failure.code, "XTDE0555");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some("mode-1431"));
    assert_eq!(
        failure
            .location
            .as_ref()
            .map(|location| location.resource.as_str()),
        Some("https://example.invalid/xslt30/attr/mode/mode-1431.xsl")
    );
}

#[test]
fn executes_fail_on_no_match_when_every_visited_node_has_a_rule() {
    let (actual, expected) = execute_case("mode-1423");
    assert_xml_equivalent(&actual, &expected);
}

#[test]
fn executes_shallow_copy_with_both_false_typed_lexicals() {
    for case_name in ["mode-1445", "mode-1446"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_shallow_copy_over_the_complete_native_mode_source() {
    for case_name in ["mode-1411", "mode-1415"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_shallow_skip_with_explicit_copy_rules() {
    for case_name in ["mode-1417", "mode-1419", "mode-1421"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_default_mode_variants_over_the_native_source() {
    for case_name in ["mode-1433", "mode-1434", "mode-1435"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn rejects_nonempty_mode_declaration_with_native_static_error() {
    let document = load_test_set();
    let case = find_case(&document, "mode-1108");
    let any_of = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "any-of",
    )
    .expect("native alternative error assertion");
    let expected_errors = element_children(&document, any_of)
        .into_iter()
        .filter(|node| local_name(&document, *node) == "error")
        .filter_map(|error| attribute(&document, error, "code"))
        .collect::<BTreeSet<_>>();
    assert!(expected_errors.contains("XTSE0260"));

    let failure = compile_case_only("mode-1108")
        .expect_err("nonempty mode should fail stylesheet compilation");
    assert_eq!(failure.code, "XTSE0260");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert!(failure.location.is_some());
}

#[test]
fn rejects_invalid_mode_visibility_and_name_combinations() {
    for case_name in ["mode-1507", "mode-1508", "mode-1509"] {
        let document = load_test_set();
        let case = find_case(&document, case_name);
        let expected_error = child_named(
            &document,
            child_named(&document, case, "result").expect("result metadata"),
            "error",
        )
        .and_then(|error| attribute(&document, error, "code"));
        assert_eq!(expected_error, Some("XTSE0020"));

        let failure = compile_case_only(case_name)
            .expect_err("invalid mode visibility should fail stylesheet compilation");
        assert_eq!(failure.code, "XTSE0020");
        assert_eq!(failure.category, FailureCategory::Invalid);
        assert!(failure.location.is_some());
    }
}

#[test]
fn rejects_conflicting_same_precedence_mode_declarations() {
    let document = load_test_set();
    let case = find_case(&document, "mode-1502");
    let expected_error = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "error",
    )
    .and_then(|error| attribute(&document, error, "code"));
    assert_eq!(expected_error, Some("XTSE0545"));

    let failure = compile_case_only("mode-1502")
        .expect_err("conflicting mode declarations should fail stylesheet compilation");
    assert_eq!(failure.code, "XTSE0545");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert!(failure.location.is_some());
}

#[test]
fn rejects_conflicting_same_precedence_mode_visibility() {
    let document = load_test_set();
    let case = find_case(&document, "mode-1904");
    let expected_error = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "error",
    )
    .and_then(|error| attribute(&document, error, "code"));
    assert_eq!(expected_error, Some("XTSE0545"));

    let failure = compile_case_only("mode-1904")
        .expect_err("conflicting mode visibility should fail stylesheet compilation");
    assert_eq!(failure.code, "XTSE0545");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert!(failure.location.is_some());
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

#[test]
fn executes_mode_1301_with_stylesheet_dependent_whitespace_reference() {
    let (actual, expected) = execute_case("mode-1301");
    assert_xml_equivalent(&actual, &expected);
}

#[test]
fn executes_text_only_copy_builtin_rules_in_named_and_unnamed_modes() {
    for (case_name, prefix) in [
        ("mode-1405", "The First Book of Moses, Called GENESIS."),
        ("mode-1407", "The First Book of Moses, Called GENESIS."),
        ("mode-1409", "THE FIRST BOOK OF MOSES, CALLED GENESIS."),
    ] {
        let (actual, expected) = execute_case_with_policy(case_name, MultipleMatchPolicy::UseLast)
            .expect("selected text-only-copy case should execute");
        assert!(expected.is_none(), "native case uses a semantic assertion");
        let normalized = without_xml_declaration(&actual)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            normalized.starts_with(prefix),
            "{case_name} result did not start with {prefix:?}"
        );
    }
}

#[test]
fn executes_all_and_current_modes_across_copied_node_kinds() {
    let (actual, expected) = execute_case("mode-1501");
    assert_xml_equivalent(&actual, &expected);
    assert!(actual.contains("<?pi PI ?>"));
}

#[test]
fn executes_parentless_temporary_node_on_no_match_policies() {
    for case_name in ["mode-0001", "mode-0003", "mode-0005"] {
        let (actual, expected) = execute_case(case_name);
        assert_xml_equivalent(&actual, &expected);
    }
}

#[test]
fn executes_temporary_element_attribute_on_no_match_policies() {
    let (actual, expected) = execute_case("mode-0015");
    assert_xml_equivalent(&actual, &expected);
}

fn execute_case(case_name: &str) -> (String, String) {
    let (actual, expected) = execute_case_with_policy(case_name, MultipleMatchPolicy::UseLast)
        .expect("selected mode case should execute");
    (
        actual,
        expected.expect("selected positive case has an XML assertion"),
    )
}

fn execute_case_with_policy(
    case_name: &str,
    multiple_match_policy: MultipleMatchPolicy,
) -> Result<(String, Option<String>), ExecutionFailure> {
    let private_overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
    assert!(private_overlay.contains(&format!("case_name = \"{case_name}\"")));
    let document = load_test_set();
    let case = find_case(&document, case_name);
    let environment = case_environment(&document, case);
    let source = environment.and_then(|environment| child_named(&document, environment, "source"));
    let test = child_named(&document, case, "test").expect("test metadata");
    let stylesheet_files = element_children(&document, test)
        .into_iter()
        .filter(|node| local_name(&document, *node) == "stylesheet")
        .map(|node| attribute(&document, node, "file").expect("stylesheet file"))
        .collect::<Vec<_>>();
    let stylesheet_file = *stylesheet_files.first().expect("principal stylesheet file");
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/xslt30-test/tests/attr/mode");
    let assertion = child_named(
        &document,
        child_named(&document, case, "result").expect("result metadata"),
        "assert-xml",
    );
    let expected = assertion.map(|assertion| {
        attribute(&document, assertion, "file").map_or_else(
            || document.string_value(assertion),
            |file| fs::read_to_string(directory.join(file)).expect("read expected XML result"),
        )
    });

    let source_id = format!("urn:w3c:xslt30:attr:mode:{case_name}:source");
    let stylesheet_id = format!("https://example.invalid/xslt30/attr/mode/{stylesheet_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 32_768, 65_536));
    if let Some(source) = source {
        let source_bytes = child_named(&document, source, "content").map_or_else(
            || {
                let file = attribute(&document, source, "file").expect("source content or file");
                fs::read(directory.join(file)).expect("read source and close handle")
            },
            |content| document.string_value(content).into_bytes(),
        );
        resources
            .admit(source_id.clone(), source_bytes)
            .expect("admit source");
    }
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit stylesheet");
    for secondary_file in stylesheet_files.iter().skip(1) {
        resources
            .admit(
                format!("https://example.invalid/xslt30/attr/mode/{secondary_file}"),
                fs::read(directory.join(secondary_file))
                    .expect("read secondary stylesheet and close handle"),
            )
            .expect("admit secondary stylesheet");
    }
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
            serialized_byte_limit: if LARGE_RESULT_CASES.contains(&case_name) {
                16_384
            } else {
                4_096
            },
            work_limits: WorkLimits::unbounded(),
        },
    )
    .with_multiple_match_policy(multiple_match_policy);
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
    let results = execute_transform_set(set.seal())?;
    Ok((results.by_request[case_name].serialized.clone(), expected))
}

fn find_case(document: &Document, case_name: &str) -> NodeId {
    element_children(document, document_element(document))
        .into_iter()
        .find(|node| attribute(document, *node, "name") == Some(case_name))
        .expect("selected mode case")
}

fn compile_case_only(case_name: &str) -> Result<(), ExecutionFailure> {
    let document = load_test_set();
    let case = find_case(&document, case_name);
    let test = child_named(&document, case, "test").expect("test metadata");
    let stylesheet_file = child_named(&document, test, "stylesheet")
        .and_then(|node| attribute(&document, node, "file"))
        .expect("stylesheet file");
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/xslt30-test/tests/attr/mode");
    let stylesheet_id = format!("https://example.invalid/xslt30/attr/mode/{stylesheet_file}");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8_192, 8_192));
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit stylesheet");
    compile_resource(&resources.seal(), &stylesheet_id).map(|_| ())
}

fn case_dependency<'a>(document: &'a Document, case: NodeId, name: &str) -> Option<&'a str> {
    child_named(document, case, "dependencies")
        .and_then(|dependencies| child_named(document, dependencies, name))
        .and_then(|dependency| attribute(document, dependency, "value"))
}

fn case_entry(
    document: &Document,
    test: NodeId,
    source: Option<NodeId>,
    source_id: &str,
) -> InvocationEntry {
    if let Some(name) = child_named(document, test, "initial-template")
        .and_then(|initial_template| attribute(document, initial_template, "name"))
    {
        return InvocationEntry::InitialTemplate {
            name: normalize_catalog_qname(document, test, name),
        };
    }
    child_named(document, test, "initial-mode").map_or_else(
        || {
            assert!(source.is_some(), "principal-source entry requires a source");
            InvocationEntry::PrincipalSource {
                resource: source_id.to_owned(),
            }
        },
        |initial_mode| {
            let lexical_name =
                attribute(document, initial_mode, "name").expect("initial mode name");
            let name = normalize_catalog_qname(document, initial_mode, lexical_name);
            let source = source.expect("initial-mode entry requires a source");
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

fn normalize_catalog_qname(document: &Document, node: NodeId, lexical: &str) -> String {
    let Some((prefix, local)) = lexical.split_once(':') else {
        return lexical.to_owned();
    };
    let mut current = Some(node);
    while let Some(candidate) = current {
        if let Some(binding) = document
            .namespace_declarations(candidate)
            .iter()
            .find(|binding| binding.prefix.as_deref() == Some(prefix))
        {
            return format!("Q{{{}}}{local}", binding.namespace);
        }
        current = document.parent(candidate);
    }
    panic!("unbound catalog QName prefix: {prefix}");
}

fn case_environment(document: &Document, case: NodeId) -> Option<NodeId> {
    let declaration = child_named(document, case, "environment")?;
    Some(
        attribute(document, declaration, "ref").map_or(declaration, |reference| {
            element_children(document, document_element(document))
                .into_iter()
                .find(|node| {
                    local_name(document, *node) == "environment"
                        && attribute(document, *node, "name") == Some(reference)
                })
                .expect("referenced environment")
        }),
    )
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
        max_events: 4_096,
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
