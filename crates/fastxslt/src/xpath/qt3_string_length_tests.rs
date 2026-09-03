//! Executable QT3 `fn:string-length` source-free tranche.

use std::{collections::BTreeSet, fs, path::PathBuf};

use super::string_length_experiment::{
    StringLengthFailure, StringLengthValue, evaluate, evaluate_document_path,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::qt3_overlay_test_support::{assert_private_case_passed, assert_selected_count};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SOURCE_FREE_CASES: [&str; 32] = [
    "fn-string-length1args-1",
    "fn-string-length1args-2",
    "fn-string-length1args-3",
    "fn-string-length-1",
    "fn-string-length-2",
    "fn-string-length-3",
    "fn-string-length-4",
    "fn-string-length-5",
    "fn-string-length-6",
    "fn-string-length-7",
    "fn-string-length-8",
    "fn-string-length-9",
    "fn-string-length-10",
    "fn-string-length-11",
    "fn-string-length-12",
    "fn-string-length-13",
    "fn-string-length-14",
    "fn-string-length-15",
    "fn-string-length-16",
    "fn-string-length-17",
    "fn-string-length-18",
    "fn-string-length-20",
    "fn-string-length-24",
    "fn-string-length-25",
    "K-StringLengthFunc-1",
    "K-StringLengthFunc-2",
    "K-StringLengthFunc-3",
    "K-StringLengthFunc-4",
    "K-StringLengthFunc-5",
    "K-StringLengthFunc-6",
    "K-StringLengthFunc-7",
    "K-StringLengthFunc-8",
];

#[test]
fn executes_qt3_source_free_string_length_tranche() {
    let set_file = "fn/string-length.xml";
    assert_selected_count(set_file, 33);
    let document = load_test_set(set_file);
    let catalog_names = descendants_named(&document, document.document_node(), "test-case")
        .into_iter()
        .map(|case| {
            attribute(&document, case, "name")
                .expect("case name")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();

    for case_name in SOURCE_FREE_CASES {
        assert!(catalog_names.contains(case_name), "{case_name}");
        assert_private_case_passed(set_file, case_name);
        let case = descendants_named(&document, document.document_node(), "test-case")
            .into_iter()
            .find(|case| attribute(&document, *case, "name") == Some(case_name))
            .expect("selected QT3 string-length case");
        let source = child_named(&document, case, "test")
            .map(|test| document.string_value(test).trim().to_owned())
            .expect("QT3 expression");
        let result = child_named(&document, case, "result").expect("QT3 result metadata");
        let mut control = InvocationControl::unbounded();

        match evaluate(&source, &mut control) {
            Ok(actual) => assert_native_result(&document, result, &actual, case_name, &source),
            Err(StringLengthFailure::InvalidArity) => {
                assert_expected_error(&document, result, "XPST0017");
            }
            Err(StringLengthFailure::MissingContext) => {
                assert_expected_error(&document, result, "XPDY0002");
            }
            Err(StringLengthFailure::InvalidArgumentType) => {
                assert_expected_error(&document, result, "XPTY0004");
            }
            Err(failure) => {
                panic!("selected QT3 expression failed: {case_name}: {source}: {failure:?}")
            }
        }
        assert!(control.consumed(WorkDomain::XPathOperation) > 0);
    }
}

#[test]
fn reports_qt3_document_sequence_string_length_type_error() {
    let set_file = "fn/string-length.xml";
    let case_name = "fn-string-length-19";
    let test_set = load_test_set(set_file);
    let case = descendants_named(&test_set, test_set.document_node(), "test-case")
        .into_iter()
        .find(|case| attribute(&test_set, *case, "name") == Some(case_name))
        .expect("selected QT3 document string-length case");
    let source = child_named(&test_set, case, "test")
        .map(|test| test_set.string_value(test).trim().to_owned())
        .expect("QT3 expression");
    let result = child_named(&test_set, case, "result").expect("QT3 result metadata");
    let document = load_context_document(&test_set, case, case_name);
    let mut control = InvocationControl::unbounded();

    assert_private_case_passed(set_file, case_name);
    let failure = evaluate_document_path(
        &source,
        &document,
        &SourceLocation {
            resource: format!("urn:w3c:qt3:{case_name}:expression"),
            span: 0..source.len(),
        },
        &mut control,
    )
    .expect_err("multi-node argument must fail string-length conversion");
    assert_eq!(failure, StringLengthFailure::InvalidArgumentType);
    assert_expected_error(&test_set, result, "XPTY0004");
    assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
}

fn assert_native_result(
    document: &Document,
    result: NodeId,
    actual: &StringLengthValue,
    case_name: &str,
    source: &str,
) {
    if !descendants_named(document, result, "assert-true").is_empty() {
        assert_eq!(
            actual,
            &StringLengthValue::Boolean(true),
            "{case_name}: {source}"
        );
        return;
    }
    if !descendants_named(document, result, "assert-false").is_empty() {
        assert_eq!(
            actual,
            &StringLengthValue::Boolean(false),
            "{case_name}: {source}"
        );
        return;
    }
    if let Some(assertion) = descendants_named(document, result, "assert-eq")
        .into_iter()
        .next()
    {
        let expected = document.string_value(assertion);
        let expected = expected.trim().trim_matches(['"', '\'']);
        assert_eq!(lexical_value(actual), expected, "{case_name}: {source}");
        return;
    }
    if let Some(assertion) = descendants_named(document, result, "assert-type")
        .into_iter()
        .next()
    {
        assert_eq!(document.string_value(assertion).trim(), "xs:integer");
        assert!(
            matches!(actual, StringLengthValue::Integer(_)),
            "{case_name}: {source}"
        );
        return;
    }
    panic!("selected case lacks an admitted assertion: {case_name}");
}

fn lexical_value(value: &StringLengthValue) -> String {
    match value {
        StringLengthValue::Empty => String::new(),
        StringLengthValue::Boolean(value) => value.to_string(),
        StringLengthValue::Integer(value) => value.to_string(),
        StringLengthValue::String(value) => value.clone(),
    }
}

fn assert_expected_error(document: &Document, result: NodeId, expected_code: &str) {
    let error = descendants_named(document, result, "error")
        .into_iter()
        .next()
        .expect("selected error case must own a native error assertion");
    assert_eq!(attribute(document, error, "code"), Some(expected_code));
}

fn load_test_set(set_file: &str) -> Document {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(set_file);
    let bytes = fs::read(path).expect("read pinned QT3 string-length test set");
    let parsed = parse_document(
        &format!("urn:w3c:qt3:{set_file}"),
        &bytes,
        ParseLimits {
            max_events: 8_192,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 string-length test set");
    Document::from_parsed(parsed).expect("build pinned QT3 string-length test set")
}

fn load_context_document(test_set: &Document, case: NodeId, case_name: &str) -> Document {
    let environment_ref = child_named(test_set, case, "environment")
        .and_then(|environment| attribute(test_set, environment, "ref"))
        .expect("QT3 document case must reference an environment");
    let catalog = load_test_set("catalog.xml");
    let environment = descendants_named(&catalog, catalog.document_node(), "environment")
        .into_iter()
        .find(|environment| attribute(&catalog, *environment, "name") == Some(environment_ref))
        .expect("QT3 catalog environment");
    let source_file = descendants_named(&catalog, environment, "source")
        .into_iter()
        .find(|source| attribute(&catalog, *source, "role") == Some("."))
        .and_then(|source| attribute(&catalog, source, "file"))
        .expect("QT3 context source");
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(source_file);
    let bytes = fs::read(path).expect("read QT3 context source and close handle");
    let resource_id = format!("urn:w3c:qt3:{case_name}:source");
    let byte_limit = bytes.len().max(1);
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, byte_limit, byte_limit));
    resources
        .admit(resource_id.clone(), bytes)
        .expect("admit QT3 context source into bounded memory");
    let snapshot = resources.seal();
    let source = snapshot
        .get(&resource_id)
        .expect("sealed QT3 context source");
    let parsed = parse_document(
        &resource_id,
        source,
        ParseLimits {
            max_events: source.len().max(1),
            max_depth: 256,
        },
    )
    .expect("parse QT3 context source");
    Document::from_parsed(parsed).expect("build QT3 context XDM")
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
