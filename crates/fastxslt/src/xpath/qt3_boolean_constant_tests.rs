//! Executable QT3 `fn:true` and `fn:false` constant-boolean groups.

use std::{collections::BTreeSet, fs, path::PathBuf};

use super::constant_boolean_experiment::{
    BooleanParseFailure, ScalarValue, evaluate_scalar, parse_scalar,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::qt3_overlay_test_support::{assert_private_case_passed, assert_selected_count};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SELECTED_SUFFIXES: [&str; 24] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17",
    "18", "19", "20", "21", "K1", "K2", "K3",
];
const NOT_NUMERIC_STEMS: [&str; 13] = [
    "fn-notint1args",
    "fn-notintg1args",
    "fn-notdec1args",
    "fn-notdbl1args",
    "fn-notflt1args",
    "fn-notlng1args",
    "fn-notusht1args",
    "fn-notnint1args",
    "fn-notpint1args",
    "fn-notulng1args",
    "fn-notnpi1args",
    "fn-notnni1args",
    "fn-notsht1args",
];
const BOOLEAN_NUMERIC_STEMS: [&str; 13] = [
    "fn-booleanint1args",
    "fn-booleanintg1args",
    "fn-booleandec1args",
    "fn-booleandbl1args",
    "fn-booleanflt1args",
    "fn-booleanlng1args",
    "fn-booleanusht1args",
    "fn-booleannint1args",
    "fn-booleanpint1args",
    "fn-booleanulng1args",
    "fn-booleannpi1args",
    "fn-booleannni1args",
    "fn-booleansht1args",
];

#[test]
fn executes_qt3_true_and_false_constant_boolean_groups() {
    for (set_file, stem, expected_constant) in [
        ("fn/true.xml", "true", true),
        ("fn/false.xml", "false", false),
    ] {
        assert_selected_count(set_file, SELECTED_SUFFIXES.len());
        let document = load_test_set(set_file);
        let catalog_names = descendants_named(&document, document.document_node(), "test-case")
            .into_iter()
            .map(|case| attribute(&document, case, "name").expect("case name"))
            .collect::<BTreeSet<_>>();
        for suffix in SELECTED_SUFFIXES {
            let case_name = case_name(stem, suffix);
            assert!(catalog_names.contains(case_name.as_str()), "{case_name}");
            assert_private_case_passed(set_file, &case_name);
            execute_case(&document, &case_name, expected_constant);
        }
    }
}

#[test]
fn executes_qt3_source_free_not_effective_boolean_value_tranche() {
    let set_file = "fn/not.xml";
    let mut selected = NOT_NUMERIC_STEMS
        .into_iter()
        .flat_map(|stem| (1..=3).map(move |suffix| format!("{stem}-{suffix}")))
        .collect::<Vec<_>>();
    selected.extend((1..=21).map(|suffix| format!("fn-not-{suffix}")));
    selected.extend((24..=26).map(|suffix| format!("fn-not-{suffix}")));
    selected.push("fn-not-27".to_owned());
    selected.extend((1..=9).map(|suffix| format!("K-NotFunc-{suffix}")));
    selected.push("cbcl-not-002".to_owned());
    assert_eq!(selected.len(), 74);
    assert_selected_count(set_file, selected.len());
    let document = load_test_set(set_file);
    let catalog_names = descendants_named(&document, document.document_node(), "test-case")
        .into_iter()
        .map(|case| {
            attribute(&document, case, "name")
                .expect("case name")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();

    for case_name in selected {
        assert!(catalog_names.contains(&case_name), "{case_name}");
        assert_private_case_passed(set_file, &case_name);
        execute_not_case(&document, &case_name);
    }
}

#[test]
fn executes_qt3_source_free_boolean_effective_boolean_value_tranche() {
    let set_file = "fn/boolean.xml";
    let mut selected = BOOLEAN_NUMERIC_STEMS
        .into_iter()
        .flat_map(|stem| (1..=3).map(move |suffix| format!("{stem}-{suffix}")))
        .collect::<Vec<_>>();
    selected.extend((1..=49).map(|suffix| format!("fn-boolean-mixed-args-{suffix:03}")));
    selected.push("fn-boolean-050".to_owned());
    selected.extend((5..=7).map(|suffix| format!("boolean-{suffix:03}")));
    selected.extend(
        (1..=15)
            .chain(17..=31)
            .map(|suffix| format!("K-SeqBooleanFunc-{suffix}")),
    );
    assert_eq!(selected.len(), 122);
    assert_selected_count(set_file, selected.len());
    let document = load_test_set(set_file);
    let catalog_names = descendants_named(&document, document.document_node(), "test-case")
        .into_iter()
        .map(|case| {
            attribute(&document, case, "name")
                .expect("case name")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();

    for case_name in selected {
        assert!(catalog_names.contains(&case_name), "{case_name}");
        assert_private_case_passed(set_file, &case_name);
        execute_not_case(&document, &case_name);
    }
}

fn execute_not_case(document: &Document, case_name: &str) {
    let case = descendants_named(document, document.document_node(), "test-case")
        .into_iter()
        .find(|case| attribute(document, *case, "name") == Some(case_name))
        .expect("selected QT3 case");
    let test = child_named(document, case, "test").expect("test expression");
    let source = document.string_value(test).trim().to_owned();
    let result = child_named(document, case, "result").expect("result metadata");

    match parse_scalar(&source) {
        Err(BooleanParseFailure::InvalidArity) => {
            let error = descendants_named(document, result, "error")
                .into_iter()
                .next()
                .expect("invalid-arity case must expect an error");
            assert_eq!(attribute(document, error, "code"), Some("XPST0017"));
        }
        Err(BooleanParseFailure::InvalidEffectiveBooleanValue) => {
            assert_expected_error(document, result, "FORG0006");
        }
        Err(BooleanParseFailure::Unsupported) => {
            panic!("selected QT3 expression is outside the admitted grammar: {case_name}: {source}")
        }
        Ok(expression) => {
            let mut control = InvocationControl::unbounded();
            let actual =
                evaluate_scalar(&expression, &mut control).expect("evaluate scalar expression");
            assert_native_result(document, result, &actual, case_name, &source);
            assert!(control.consumed(WorkDomain::XPathOperation) > 0);
        }
    }
}

fn case_name(stem: &str, suffix: &str) -> String {
    match suffix {
        "K1" | "K2" | "K3" => format!("K-{}Func-{}", title_case(stem), &suffix[1..]),
        _ => format!("fn-{stem}-{suffix}"),
    }
}

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
        .unwrap_or_default()
}

fn execute_case(document: &Document, case_name: &str, expected_constant: bool) {
    let case = descendants_named(document, document.document_node(), "test-case")
        .into_iter()
        .find(|case| attribute(document, *case, "name") == Some(case_name))
        .expect("selected QT3 case");
    let test = child_named(document, case, "test").expect("test expression");
    let source = document.string_value(test).trim().to_owned();
    let result = child_named(document, case, "result").expect("result metadata");

    match parse_scalar(&source) {
        Err(BooleanParseFailure::InvalidArity) => {
            let error = descendants_named(document, result, "error")
                .into_iter()
                .next()
                .expect("invalid-arity case must expect an error");
            assert_eq!(attribute(document, error, "code"), Some("XPST0017"));
        }
        Err(BooleanParseFailure::InvalidEffectiveBooleanValue) => {
            assert_expected_error(document, result, "FORG0006");
        }
        Err(BooleanParseFailure::Unsupported) => {
            panic!("selected QT3 expression is outside the admitted grammar: {case_name}: {source}")
        }
        Ok(expression) => {
            let mut control = InvocationControl::unbounded();
            let actual =
                evaluate_scalar(&expression, &mut control).expect("evaluate scalar expression");
            assert_native_result(document, result, &actual, case_name, &source);
            assert!(control.consumed(WorkDomain::XPathOperation) > 0);
            if case_name.ends_with("-1") {
                assert_eq!(actual, ScalarValue::Boolean(expected_constant));
                assert!(
                    !descendants_named(document, result, "assert-type").is_empty(),
                    "{case_name} retains its native xs:boolean type assertion"
                );
            }
        }
    }
}

fn assert_expected_error(document: &Document, result: NodeId, expected_code: &str) {
    let error = descendants_named(document, result, "error")
        .into_iter()
        .next()
        .expect("selected error case must own a native error assertion");
    assert_eq!(attribute(document, error, "code"), Some(expected_code));
}

fn assert_native_result(
    document: &Document,
    result: NodeId,
    actual: &ScalarValue,
    case_name: &str,
    source: &str,
) {
    if let Some(expected) = expected_boolean(document, result) {
        assert_eq!(
            actual,
            &ScalarValue::Boolean(expected),
            "{case_name}: {source}"
        );
        return;
    }
    let assertion = descendants_named(document, result, "assert-eq")
        .into_iter()
        .next()
        .or_else(|| {
            descendants_named(document, result, "assert-string-value")
                .into_iter()
                .next()
        })
        .unwrap_or_else(|| panic!("selected case lacks an admitted assertion: {case_name}"));
    let expected = document.string_value(assertion);
    let expected = expected.trim().trim_matches(['"', '\'']);
    let actual = match actual {
        ScalarValue::Boolean(value) => value.to_string(),
        ScalarValue::String(value) => value.clone(),
        ScalarValue::Integer(value) => value.to_string(),
    };
    assert_eq!(actual, expected, "{case_name}: {source}");
}

fn expected_boolean(document: &Document, result: NodeId) -> Option<bool> {
    if !descendants_named(document, result, "assert-true").is_empty() {
        Some(true)
    } else if !descendants_named(document, result, "assert-false").is_empty() {
        Some(false)
    } else {
        None
    }
}

fn load_test_set(set_file: &str) -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(set_file);
    let bytes = fs::read(path).expect("read pinned QT3 boolean test set");
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{set_file}"),
        &bytes,
        ParseLimits {
            max_events: 8_192,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 boolean test set");
    Document::from_parsed(parsed).expect("build pinned QT3 boolean test set")
}

fn child_named(document: &Document, parent: NodeId, local: &str) -> Option<NodeId> {
    document.children(parent).iter().copied().find(|child| {
        document.kind(*child) == NodeKind::Element
            && document
                .name(*child)
                .is_some_and(|name| name.local == local)
    })
}

fn descendants_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in document.children(parent).iter().copied() {
        if document.kind(child) != NodeKind::Element {
            continue;
        }
        if document.name(child).is_some_and(|name| name.local == local) {
            found.push(child);
        }
        found.extend(descendants_named(document, child, local));
    }
    found
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}
