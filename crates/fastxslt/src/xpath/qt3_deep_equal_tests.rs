//! Executable QT3 `fn:deep-equal` typed-atomic groups.

use std::{fs, path::PathBuf};

use super::deep_equal_boolean_experiment::{evaluate, parse};
use super::deep_equal_experiment::DeepEqualFailureKind;
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SET_FILE: &str = "fn/deep-equal.xml";
const INT_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalint2args-1", true),
    ("fn-deep-equalint2args-2", false),
    ("fn-deep-equalint2args-3", false),
    ("fn-deep-equalint2args-4", false),
    ("fn-deep-equalint2args-5", false),
];
const INTEGER_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalintg2args-1", true),
    ("fn-deep-equalintg2args-2", false),
    ("fn-deep-equalintg2args-3", false),
    ("fn-deep-equalintg2args-4", false),
    ("fn-deep-equalintg2args-5", false),
];
const DECIMAL_CASES: [(&str, bool); 5] = [
    ("fn-deep-equaldec2args-1", true),
    ("fn-deep-equaldec2args-2", false),
    ("fn-deep-equaldec2args-3", false),
    ("fn-deep-equaldec2args-4", false),
    ("fn-deep-equaldec2args-5", false),
];
const LONG_CASES: [(&str, bool); 5] = [
    ("fn-deep-equallng2args-1", true),
    ("fn-deep-equallng2args-2", false),
    ("fn-deep-equallng2args-3", false),
    ("fn-deep-equallng2args-4", false),
    ("fn-deep-equallng2args-5", false),
];
const UNSIGNED_SHORT_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalusht2args-1", true),
    ("fn-deep-equalusht2args-2", false),
    ("fn-deep-equalusht2args-3", false),
    ("fn-deep-equalusht2args-4", false),
    ("fn-deep-equalusht2args-5", false),
];
const NEGATIVE_INTEGER_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalnint2args-1", true),
    ("fn-deep-equalnint2args-2", false),
    ("fn-deep-equalnint2args-3", false),
    ("fn-deep-equalnint2args-4", false),
    ("fn-deep-equalnint2args-5", false),
];
const POSITIVE_INTEGER_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalpint2args-1", true),
    ("fn-deep-equalpint2args-2", false),
    ("fn-deep-equalpint2args-3", false),
    ("fn-deep-equalpint2args-4", false),
    ("fn-deep-equalpint2args-5", false),
];
const UNSIGNED_LONG_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalulng2args-1", true),
    ("fn-deep-equalulng2args-2", false),
    ("fn-deep-equalulng2args-3", false),
    ("fn-deep-equalulng2args-4", false),
    ("fn-deep-equalulng2args-5", false),
];
const NON_POSITIVE_INTEGER_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalnpi2args-1", true),
    ("fn-deep-equalnpi2args-2", false),
    ("fn-deep-equalnpi2args-3", false),
    ("fn-deep-equalnpi2args-4", false),
    ("fn-deep-equalnpi2args-5", false),
];
const NON_NEGATIVE_INTEGER_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalnni2args-1", true),
    ("fn-deep-equalnni2args-2", false),
    ("fn-deep-equalnni2args-3", false),
    ("fn-deep-equalnni2args-4", false),
    ("fn-deep-equalnni2args-5", false),
];
const SHORT_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalsht2args-1", true),
    ("fn-deep-equalsht2args-2", false),
    ("fn-deep-equalsht2args-3", false),
    ("fn-deep-equalsht2args-4", false),
    ("fn-deep-equalsht2args-5", false),
];
const FLOAT_CASES: [(&str, bool); 5] = [
    ("fn-deep-equalflt2args-1", true),
    ("fn-deep-equalflt2args-2", false),
    ("fn-deep-equalflt2args-3", false),
    ("fn-deep-equalflt2args-4", false),
    ("fn-deep-equalflt2args-5", false),
];
const DOUBLE_CASES: [(&str, bool); 5] = [
    ("fn-deep-equaldbl2args-1", true),
    ("fn-deep-equaldbl2args-2", false),
    ("fn-deep-equaldbl2args-3", false),
    ("fn-deep-equaldbl2args-4", false),
    ("fn-deep-equaldbl2args-5", false),
];
const ARITY_ERROR_CASES: [&str; 5] = [
    "K-SeqDeepEqualFunc-1",
    "K-SeqDeepEqualFunc-2",
    "K-SeqDeepEqualFunc-3",
    "K2-SeqDeepEqualFunc-8",
    "K2-SeqDeepEqualFunc-9",
];
const CODEPOINT_AND_NAN_CASES: [(&str, usize); 5] = [
    ("K-SeqDeepEqualFunc-6", 2),
    ("K-SeqDeepEqualFunc-8", 2),
    ("K-SeqDeepEqualFunc-9", 2),
    ("K-SeqDeepEqualFunc-10", 2),
    ("K-SeqDeepEqualFunc-11", 2),
];
const BOOLEAN_COMPOSITION_CASES: [(&str, usize); 9] = [
    ("K-SeqDeepEqualFunc-7", 1),
    ("K-SeqDeepEqualFunc-12", 2),
    ("K-SeqDeepEqualFunc-13", 2),
    ("K-SeqDeepEqualFunc-14", 2),
    ("K-SeqDeepEqualFunc-15", 2),
    ("K-SeqDeepEqualFunc-16", 2),
    ("K-SeqDeepEqualFunc-18", 4),
    ("K-SeqDeepEqualFunc-19", 3),
    ("K-SeqDeepEqualFunc-20", 2),
];
const QNAME_CASES: [(&str, usize); 2] =
    [("K-SeqDeepEqualFunc-17", 2), ("K-SeqDeepEqualFunc-21", 4)];
const BINARY_CASES: [(&str, usize); 3] = [
    ("K-SeqDeepEqualFunc-22", 3),
    ("K-SeqDeepEqualFunc-23", 2),
    ("K-SeqDeepEqualFunc-24", 4),
];
const LITERAL_INDEX_OF_CASES: [(&str, usize); 4] = [
    ("K-SeqDeepEqualFunc-32", 2),
    ("K-SeqDeepEqualFunc-33", 2),
    ("K-SeqDeepEqualFunc-34", 3),
    ("K-SeqDeepEqualFunc-35", 3),
];
const ORDERED_AND_EMPTY_SEQUENCE_CASES: [(&str, usize); 18] = [
    ("K-SeqDeepEqualFunc-25", 4),
    ("K-SeqDeepEqualFunc-26", 4),
    ("K-SeqDeepEqualFunc-27", 3),
    ("K-SeqDeepEqualFunc-28", 2),
    ("K-SeqDeepEqualFunc-29", 2),
    ("K-SeqDeepEqualFunc-30", 3),
    ("K-SeqDeepEqualFunc-31", 4),
    ("K-SeqDeepEqualFunc-36", 4),
    ("K-SeqDeepEqualFunc-37", 4),
    ("K-SeqDeepEqualFunc-38", 4),
    ("K-SeqDeepEqualFunc-39", 3),
    ("K-SeqDeepEqualFunc-40", 3),
    ("K-SeqDeepEqualFunc-41", 3),
    ("K-SeqDeepEqualFunc-42", 1),
    ("K-SeqDeepEqualFunc-43", 1),
    ("K-SeqDeepEqualFunc-44", 1),
    ("K-SeqDeepEqualFunc-45", 1),
    ("K-SeqDeepEqualFunc-46", 1),
];
const UNEQUAL_LENGTH_TAIL_CASES: [(&str, usize); 5] = [
    ("K-SeqDeepEqualFunc-47", 1),
    ("K-SeqDeepEqualFunc-48", 1),
    ("K-SeqDeepEqualFunc-49", 1),
    ("K-SeqDeepEqualFunc-50", 1),
    ("K-SeqDeepEqualFunc-51", 1),
];
const LITERAL_RANGE_CASES: [(&str, usize); 4] = [
    ("K-SeqDeepEqualFunc-52", 1),
    ("K-SeqDeepEqualFunc-53", 1),
    ("K-SeqDeepEqualFunc-54", 1),
    ("K-SeqDeepEqualFunc-55", 1),
];
const STRING_DERIVED_CASES: [(&str, usize); 1] = [("K2-SeqDeepEqualFunc-35", 2)];
const HTML_ASCII_COLLATION_CASES: [(&str, bool, usize); 2] = [
    ("K-SeqDeepEqualFunc-64", true, 3),
    ("K-SeqDeepEqualFunc-65", false, 3),
];
const STANDARD_COLLATION_ERROR_CASES: [(&str, &str); 2] = [
    ("K-SeqDeepEqualFunc-4", "FOCH0002"),
    ("K-SeqDeepEqualFunc-5", "XPTY0004"),
];
const UNTYPED_DURATION_CASES: [(&str, bool, usize); 1] = [("cbcl-deep-equal-008", false, 3)];
const ARRAY_LITERAL_CASES: [(&str, bool, usize); 7] = [
    ("fn-deep-equal-arrays-1", true, 3),
    ("fn-deep-equal-arrays-2", true, 7),
    ("fn-deep-equal-arrays-3", true, 6),
    ("fn-deep-equal-arrays-4", false, 3),
    ("fn-deep-equal-arrays-5", false, 2),
    ("fn-deep-equal-arrays-6", false, 3),
    ("fn-deep-equal-arrays-7", true, 4),
];
const ARRAY_STRING_SEQUENCE_CASES: [(&str, bool, usize); 6] = [
    ("fn-deep-equal-arrays-11", true, 12),
    ("fn-deep-equal-arrays-12", false, 8),
    ("fn-deep-equal-arrays-14", true, 18),
    ("fn-deep-equal-arrays-15", false, 1),
    ("fn-deep-equal-arrays-16", false, 18),
    ("fn-deep-equal-arrays-17", false, 6),
];
const MAP_LITERAL_CASES: [(&str, bool, usize); 6] = [
    ("fn-deep-equal-maps-1", true, 3),
    ("fn-deep-equal-maps-2", false, 3),
    ("fn-deep-equal-maps-3", false, 3),
    ("fn-deep-equal-maps-4", true, 9),
    ("fn-deep-equal-arrays-8", true, 6),
    ("fn-deep-equal-arrays-9", true, 12),
];
const MAP_NUMERIC_CASES: [(&str, bool, usize); 6] = [
    ("fn-deep-equal-maps-5", true, 6),
    ("fn-deep-equal-maps-6", true, 6),
    ("fn-deep-equal-maps-7", true, 6),
    ("fn-deep-equal-maps-8", true, 6),
    ("fn-deep-equal-maps-9", true, 13),
    ("fn-deep-equal-maps-10", false, 9),
];
const COMPOSITE_UPDATE_CASES: [(&str, bool, usize); 2] = [
    ("fn-deep-equal-arrays-18", true, 18),
    ("fn-deep-equal-maps-15", true, 6),
];
const MIXED_ATOMIC_CASES: [(&str, bool, usize); 31] = [
    ("fn-deep-equal-mix-args-001", false, 2),
    ("fn-deep-equal-mix-args-002", true, 3),
    ("fn-deep-equal-mix-args-003", true, 2),
    ("fn-deep-equal-mix-args-004", false, 2),
    ("fn-deep-equal-mix-args-005", true, 2),
    ("fn-deep-equal-mix-args-006", true, 2),
    ("fn-deep-equal-mix-args-007", true, 1),
    ("fn-deep-equal-mix-args-008", true, 1),
    ("fn-deep-equal-mix-args-009", true, 1),
    ("fn-deep-equal-mix-args-010", false, 2),
    ("fn-deep-equal-mix-args-011", true, 2),
    ("fn-deep-equal-mix-args-012", true, 2),
    ("fn-deep-equal-mix-args-013", true, 2),
    ("fn-deep-equal-mix-args-014", false, 2),
    ("fn-deep-equal-mix-args-015", true, 2),
    ("fn-deep-equal-mix-args-016", true, 2),
    ("fn-deep-equal-mix-args-017", true, 2),
    ("fn-deep-equal-mix-args-018", true, 2),
    ("fn-deep-equal-mix-args-019", false, 2),
    ("fn-deep-equal-mix-args-020", true, 2),
    ("fn-deep-equal-mix-args-021", true, 2),
    ("fn-deep-equal-mix-args-022", true, 2),
    ("fn-deep-equal-mix-args-023", true, 2),
    ("fn-deep-equal-mix-args-024", true, 2),
    ("fn-deep-equal-mix-args-025", true, 2),
    ("fn-deep-equal-mix-args-026", true, 2),
    ("fn-deep-equal-mix-args-027", true, 2),
    ("fn-deep-equal-mix-args-028", false, 2),
    ("fn-deep-equal-mix-args-029", false, 2),
    ("fn-deep-equal-mix-args-030", false, 2),
    ("fn-deep-equal-mix-args-031", false, 2),
];

#[test]
fn executes_complete_qt3_deep_equal_xs_int_group() {
    execute_group("fn-deep-equalint2args-", &INT_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_integer_group() {
    execute_group("fn-deep-equalintg2args-", &INTEGER_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_decimal_group() {
    execute_group("fn-deep-equaldec2args-", &DECIMAL_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_long_group() {
    execute_group("fn-deep-equallng2args-", &LONG_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_unsigned_short_group() {
    execute_group("fn-deep-equalusht2args-", &UNSIGNED_SHORT_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_negative_integer_group() {
    execute_group("fn-deep-equalnint2args-", &NEGATIVE_INTEGER_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_positive_integer_group() {
    execute_group("fn-deep-equalpint2args-", &POSITIVE_INTEGER_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_unsigned_long_group() {
    execute_group("fn-deep-equalulng2args-", &UNSIGNED_LONG_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_non_positive_integer_group() {
    execute_group("fn-deep-equalnpi2args-", &NON_POSITIVE_INTEGER_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_non_negative_integer_group() {
    execute_group("fn-deep-equalnni2args-", &NON_NEGATIVE_INTEGER_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_short_group() {
    execute_group("fn-deep-equalsht2args-", &SHORT_CASES, 1);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_float_group() {
    execute_group("fn-deep-equalflt2args-", &FLOAT_CASES, 2);
}

#[test]
fn executes_complete_qt3_deep_equal_xs_double_group() {
    execute_group("fn-deep-equaldbl2args-", &DOUBLE_CASES, 2);
}

#[test]
fn classifies_selected_qt3_deep_equal_arity_errors() {
    let test_set = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(
        cases
            .iter()
            .filter(|node| {
                attribute(&test_set, **node, "name")
                    .is_some_and(|name| ARITY_ERROR_CASES.contains(&name))
            })
            .count(),
        ARITY_ERROR_CASES.len()
    );

    for name in ARITY_ERROR_CASES {
        crate::qt3_overlay_test_support::assert_private_case_passed(SET_FILE, name);
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned QT3 deep-equal arity case");
        let expression = child_named(&test_set, case, "test")
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&test_set, case, "result")
            .and_then(|node| first_element_child(&test_set, node))
            .expect("QT3 error assertion");
        assert_eq!(local_name(&test_set, result), "error");
        assert_eq!(attribute(&test_set, result, "code"), Some("XPST0017"));

        let location = SourceLocation {
            resource: format!("urn:w3c:qt3:{name}:expression"),
            span: 0..expression.len(),
        };
        let failure = parse(&expression, &location).expect_err("reject invalid function arity");
        assert_eq!(failure.location, location);
        assert_eq!(
            failure.kind,
            DeepEqualFailureKind::InvalidArity {
                standard_code: "XPST0017"
            }
        );
    }
}

#[test]
fn executes_qt3_codepoint_collation_and_paired_nan_tranche() {
    execute_named_true_cases(&CODEPOINT_AND_NAN_CASES);
}

#[test]
fn executes_qt3_boolean_composition_tranche() {
    execute_named_true_cases(&BOOLEAN_COMPOSITION_CASES);
}

#[test]
fn executes_qt3_qname_tranche() {
    execute_named_true_cases(&QNAME_CASES);
}

#[test]
fn executes_qt3_binary_tranche() {
    execute_named_true_cases(&BINARY_CASES);
}

#[test]
fn executes_qt3_literal_index_of_tranche() {
    execute_named_true_cases(&LITERAL_INDEX_OF_CASES);
}

#[test]
fn executes_qt3_ordered_and_empty_sequence_tranche() {
    execute_named_true_cases(&ORDERED_AND_EMPTY_SEQUENCE_CASES);
}

#[test]
fn executes_qt3_unequal_length_tail_tranche() {
    execute_named_true_cases(&UNEQUAL_LENGTH_TAIL_CASES);
}

#[test]
fn executes_qt3_literal_range_tranche() {
    execute_named_true_cases(&LITERAL_RANGE_CASES);
}

#[test]
fn executes_qt3_string_derived_ncname_case() {
    execute_named_true_cases(&STRING_DERIVED_CASES);
}

#[test]
fn executes_qt3_html_ascii_case_insensitive_collation_cases() {
    execute_named_boolean_cases(&HTML_ASCII_COLLATION_CASES);
}

#[test]
fn reports_qt3_standard_collation_errors() {
    let test_set = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");

    for (name, standard_code) in STANDARD_COLLATION_ERROR_CASES {
        crate::qt3_overlay_test_support::assert_private_case_passed(SET_FILE, name);
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned QT3 collation error case");
        let expression = child_named(&test_set, case, "test")
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&test_set, case, "result").expect("QT3 result");
        let error = descendants_named(&test_set, result, "error")
            .into_iter()
            .find(|node| attribute(&test_set, *node, "code") == Some(standard_code))
            .expect("QT3 any-of permits the selected standard error");
        assert_eq!(attribute(&test_set, error, "code"), Some(standard_code));

        let location = SourceLocation {
            resource: format!("urn:w3c:qt3:{name}:expression"),
            span: 0..expression.len(),
        };
        let failure = parse(&expression, &location).expect_err("reject invalid collation");
        assert_eq!(failure.location, location);
        let actual_code = match failure.kind {
            DeepEqualFailureKind::InvalidCollation { standard_code }
            | DeepEqualFailureKind::InvalidCollationType { standard_code } => standard_code,
            other => panic!("unexpected collation failure: {other:?}"),
        };
        assert_eq!(actual_code, standard_code);
    }
}

#[test]
fn executes_qt3_array_literal_tranche() {
    execute_named_boolean_cases(&ARRAY_LITERAL_CASES);
}

#[test]
fn executes_qt3_array_string_and_sequence_tranche() {
    execute_named_boolean_cases(&ARRAY_STRING_SEQUENCE_CASES);
}

#[test]
fn executes_qt3_map_literal_tranche() {
    execute_named_boolean_cases(&MAP_LITERAL_CASES);
}

#[test]
fn executes_qt3_map_numeric_equivalence_tranche() {
    execute_named_boolean_cases(&MAP_NUMERIC_CASES);
}

#[test]
fn executes_qt3_literal_composite_update_tranche() {
    execute_named_boolean_cases(&COMPOSITE_UPDATE_CASES);
}

#[test]
fn executes_qt3_untyped_atomic_duration_boundary() {
    execute_named_boolean_cases(&UNTYPED_DURATION_CASES);
}

fn execute_named_true_cases(expected_cases: &[(&str, usize)]) {
    let test_set = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(
        cases
            .iter()
            .filter(|node| {
                attribute(&test_set, **node, "name").is_some_and(|name| {
                    expected_cases.iter().any(|(expected, _)| *expected == name)
                })
            })
            .count(),
        expected_cases.len()
    );

    for (name, expected_operations) in expected_cases.iter().copied() {
        crate::qt3_overlay_test_support::assert_private_case_passed(SET_FILE, name);
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned QT3 named deep-equal case");
        let expression = child_named(&test_set, case, "test")
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&test_set, case, "result")
            .and_then(|node| first_element_child(&test_set, node))
            .expect("QT3 boolean assertion");
        assert_eq!(local_name(&test_set, result), "assert-true");

        let parsed = parse(
            &expression,
            &SourceLocation {
                resource: format!("urn:w3c:qt3:{name}:expression"),
                span: 0..expression.len(),
            },
        )
        .expect("parse admitted named deep-equal expression");
        let mut control = InvocationControl::unbounded();
        assert!(evaluate(&parsed, None, &mut control).expect("evaluate named deep-equal case"));
        assert_eq!(
            control.consumed(WorkDomain::XPathOperation),
            expected_operations
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 0);
    }
}

fn execute_named_boolean_cases(expected_cases: &[(&str, bool, usize)]) {
    let test_set = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(
        cases
            .iter()
            .filter(|node| {
                attribute(&test_set, **node, "name").is_some_and(|name| {
                    expected_cases
                        .iter()
                        .any(|(expected, _, _)| *expected == name)
                })
            })
            .count(),
        expected_cases.len()
    );

    for (name, expected, expected_operations) in expected_cases.iter().copied() {
        crate::qt3_overlay_test_support::assert_private_case_passed(SET_FILE, name);
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned QT3 boolean deep-equal case");
        let expression = child_named(&test_set, case, "test")
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&test_set, case, "result")
            .and_then(|node| first_element_child(&test_set, node))
            .expect("QT3 boolean assertion");
        assert_eq!(
            local_name(&test_set, result),
            if expected {
                "assert-true"
            } else {
                "assert-false"
            }
        );

        let parsed = parse(
            &expression,
            &SourceLocation {
                resource: format!("urn:w3c:qt3:{name}:expression"),
                span: 0..expression.len(),
            },
        )
        .expect("parse admitted boolean deep-equal expression");
        let mut control = InvocationControl::unbounded();
        assert_eq!(
            evaluate(&parsed, None, &mut control).expect("evaluate boolean deep-equal case"),
            expected
        );
        assert_eq!(
            control.consumed(WorkDomain::XPathOperation),
            expected_operations
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 0);
    }
}

#[test]
fn executes_complete_qt3_deep_equal_mixed_atomic_group() {
    let test_set = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case");
    assert_eq!(
        cases
            .iter()
            .filter(|node| {
                attribute(&test_set, **node, "name")
                    .is_some_and(|name| name.starts_with("fn-deep-equal-mix-args-"))
            })
            .count(),
        MIXED_ATOMIC_CASES.len()
    );
    for (name, expected, expected_operations) in MIXED_ATOMIC_CASES {
        crate::qt3_overlay_test_support::assert_private_case_passed(SET_FILE, name);
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned QT3 mixed deep-equal case");
        let expression = child_named(&test_set, case, "test")
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&test_set, case, "result")
            .and_then(|node| first_element_child(&test_set, node))
            .expect("QT3 boolean assertion");
        assert_eq!(
            local_name(&test_set, result),
            if expected {
                "assert-true"
            } else {
                "assert-false"
            }
        );

        let parsed = parse(
            &expression,
            &SourceLocation {
                resource: format!("urn:w3c:qt3:{name}:expression"),
                span: 0..expression.len(),
            },
        )
        .expect("parse admitted mixed deep-equal expression");
        let mut control = InvocationControl::unbounded();
        let actual = evaluate(&parsed, None, &mut control).expect("evaluate mixed deep-equal");
        assert_eq!(actual, expected, "native QT3 assertion for {name}");
        assert_eq!(
            control.consumed(WorkDomain::XPathOperation),
            expected_operations
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 0);
    }
}

fn execute_group(prefix: &str, expected_cases: &[(&str, bool)], expected_operations: usize) {
    let test_set = load_test_set();
    let cases = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .filter(|node| {
            attribute(&test_set, *node, "name").is_some_and(|name| name.starts_with(prefix))
        })
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), expected_cases.len());

    for (name, expected) in expected_cases.iter().copied() {
        crate::qt3_overlay_test_support::assert_private_case_passed(SET_FILE, name);
        let case = cases
            .iter()
            .copied()
            .find(|node| attribute(&test_set, *node, "name") == Some(name))
            .expect("pinned QT3 deep-equal case");
        let expression = child_named(&test_set, case, "test")
            .map(|node| test_set.string_value(node).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&test_set, case, "result")
            .and_then(|node| first_element_child(&test_set, node))
            .expect("QT3 boolean assertion");
        assert_eq!(
            local_name(&test_set, result),
            if expected {
                "assert-true"
            } else {
                "assert-false"
            }
        );

        let parsed = parse(
            &expression,
            &SourceLocation {
                resource: format!("urn:w3c:qt3:{name}:expression"),
                span: 0..expression.len(),
            },
        )
        .expect("parse admitted typed deep-equal expression");
        let mut control = InvocationControl::unbounded();
        let actual = evaluate(&parsed, None, &mut control).expect("evaluate typed deep-equal");
        assert_eq!(actual, expected, "native QT3 assertion for {name}");
        assert_eq!(
            control.consumed(WorkDomain::XPathOperation),
            expected_operations
        );
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 0);
    }
}

fn load_test_set() -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(SET_FILE);
    let bytes = fs::read(path).expect("read pinned QT3 deep-equal test set");
    let parsed = parse_document(
        "urn:w3c:qt3:fn-deep-equal:test-set",
        &bytes,
        ParseLimits {
            max_events: 30_000,
            max_depth: 64,
        },
    )
    .expect("parse QT3 deep-equal test set");
    Document::from_parsed(parsed).expect("build QT3 deep-equal catalog document")
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
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
    &document.name(node).expect("element node has a name").local
}
