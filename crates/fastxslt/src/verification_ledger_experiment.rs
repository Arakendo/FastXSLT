use std::{fs, path::PathBuf};

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const QT3_NAMESPACE: &str = "http://www.w3.org/2010/09/qt-fots-catalog";
const QT3_REVISION: &str = "83993587711dbd5c18ed846385ec37d079d6e492";
const XSLT30_NAMESPACE: &str = "http://www.w3.org/2012/10/xslt-test-catalog";
const XSLT30_REVISION: &str = "6f8fd9e966ae74a251a2604abef9d904c7bc5c9b";

#[derive(Debug, PartialEq, Eq)]
struct NativeCaseIdentity {
    suite: &'static str,
    revision: &'static str,
    test_set: &'static str,
    case_name: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Qt3CaseObservation {
    identity: NativeCaseIdentity,
    environment_ref: String,
    expression: String,
    assertion: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Xslt30CaseObservation {
    identity: NativeCaseIdentity,
    environment_ref: String,
    stylesheet_ref: String,
    dependency: String,
    assertion_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionDisposition {
    EngineUnsupported,
    HarnessUnsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionDisposition {
    Selected,
}

#[derive(Debug, PartialEq, Eq)]
struct PrivateCaseRecord<'a> {
    identity: &'a NativeCaseIdentity,
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
}

fn corpus_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn load_test_set(relative: &str) -> Document {
    let path = corpus_path(relative);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let parsed = parse_document(
        &path.display().to_string(),
        &bytes,
        ParseLimits {
            max_events: 100_000,
            max_depth: 64,
        },
    )
    .unwrap_or_else(|error| panic!("parse {}: {error:?}", path.display()));
    Document::from_parsed(parsed)
        .unwrap_or_else(|error| panic!("build {}: {error:?}", path.display()))
}

fn is_element(document: &Document, node: NodeId, namespace: &str, local: &str) -> bool {
    document.kind(node) == NodeKind::Element
        && document
            .name(node)
            .is_some_and(|name| name.namespace.as_deref() == Some(namespace) && name.local == local)
}

fn attribute(document: &Document, node: NodeId, local: &str) -> Option<String> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
            .map(str::to_owned)
    })
}

fn child(document: &Document, parent: NodeId, namespace: &str, local: &str) -> NodeId {
    document
        .children(parent)
        .iter()
        .copied()
        .find(|node| is_element(document, *node, namespace, local))
        .unwrap_or_else(|| panic!("missing {{{namespace}}}{local}"))
}

fn descendant_case(
    document: &Document,
    node: NodeId,
    namespace: &str,
    case_name: &str,
) -> Option<NodeId> {
    if is_element(document, node, namespace, "test-case")
        && attribute(document, node, "name").as_deref() == Some(case_name)
    {
        return Some(node);
    }
    document
        .children(node)
        .iter()
        .find_map(|child| descendant_case(document, *child, namespace, case_name))
}

fn case(document: &Document, namespace: &str, case_name: &str) -> NodeId {
    descendant_case(document, document.document_node(), namespace, case_name)
        .unwrap_or_else(|| panic!("missing case {case_name}"))
}

fn element_local(document: &Document, node: NodeId) -> String {
    document
        .name(node)
        .expect("assertion nodes are named elements")
        .local
        .clone()
}

fn collect_assertion_paths(
    document: &Document,
    node: NodeId,
    namespace: &str,
    prefix: &str,
    paths: &mut Vec<String>,
) {
    for child in document.children(node).iter().copied().filter(|child| {
        document.kind(*child) == NodeKind::Element
            && document
                .name(*child)
                .is_some_and(|name| name.namespace.as_deref() == Some(namespace))
    }) {
        let local = element_local(document, child);
        let path = if prefix.is_empty() {
            local
        } else {
            format!("{prefix}/{local}")
        };
        paths.push(path.clone());
        collect_assertion_paths(document, child, namespace, &path, paths);
    }
}

fn observe_qt3_axis_case() -> Qt3CaseObservation {
    const TEST_SET: &str = "prod/AxisStep.xml";
    let document = load_test_set("vendor/qt3tests/prod/AxisStep.xml");
    let case = case(&document, QT3_NAMESPACE, "Axes001-1");
    let environment = child(&document, case, QT3_NAMESPACE, "environment");
    let test = child(&document, case, QT3_NAMESPACE, "test");
    let result = child(&document, case, QT3_NAMESPACE, "result");
    let assertion = document
        .children(result)
        .iter()
        .copied()
        .find(|node| document.kind(*node) == NodeKind::Element)
        .expect("QT3 result has an assertion element");

    Qt3CaseObservation {
        identity: NativeCaseIdentity {
            suite: "QT3",
            revision: QT3_REVISION,
            test_set: TEST_SET,
            case_name: attribute(&document, case, "name").expect("case name"),
        },
        environment_ref: attribute(&document, environment, "ref").expect("environment ref"),
        expression: document.string_value(test).trim().to_owned(),
        assertion: element_local(&document, assertion),
    }
}

fn observe_xslt30_message_case() -> Xslt30CaseObservation {
    const TEST_SET: &str = "tests/attr/avt/_avt-test-set.xml";
    let document = load_test_set("vendor/xslt30-test/tests/attr/avt/_avt-test-set.xml");
    let case = case(&document, XSLT30_NAMESPACE, "avt-0701");
    let environment = child(&document, case, XSLT30_NAMESPACE, "environment");
    let dependencies = child(&document, case, XSLT30_NAMESPACE, "dependencies");
    let spec = child(&document, dependencies, XSLT30_NAMESPACE, "spec");
    let test = child(&document, case, XSLT30_NAMESPACE, "test");
    let stylesheet = child(&document, test, XSLT30_NAMESPACE, "stylesheet");
    let result = child(&document, case, XSLT30_NAMESPACE, "result");
    let mut assertion_paths = Vec::new();
    collect_assertion_paths(
        &document,
        result,
        XSLT30_NAMESPACE,
        "",
        &mut assertion_paths,
    );

    Xslt30CaseObservation {
        identity: NativeCaseIdentity {
            suite: "XSLT30",
            revision: XSLT30_REVISION,
            test_set: TEST_SET,
            case_name: attribute(&document, case, "name").expect("case name"),
        },
        environment_ref: attribute(&document, environment, "ref").expect("environment ref"),
        stylesheet_ref: attribute(&document, stylesheet, "file").expect("stylesheet file"),
        dependency: attribute(&document, spec, "value").expect("spec dependency"),
        assertion_paths,
    }
}

fn classify_qt3(case: &Qt3CaseObservation) -> ExecutionDisposition {
    if case.assertion != "assert-eq" {
        return ExecutionDisposition::HarnessUnsupported;
    }
    ExecutionDisposition::EngineUnsupported
}

fn classify_xslt30(case: &Xslt30CaseObservation) -> ExecutionDisposition {
    const SUPPORTED_ASSERTIONS: &[&str] = &["assert-xml"];
    if case.dependency != "XSLT20+"
        || case
            .assertion_paths
            .iter()
            .any(|path| !SUPPORTED_ASSERTIONS.contains(&path.as_str()))
    {
        return ExecutionDisposition::HarnessUnsupported;
    }
    ExecutionDisposition::EngineUnsupported
}

#[test]
fn suite_specific_records_preserve_materially_different_native_metadata() {
    let qt3 = observe_qt3_axis_case();
    assert_eq!(qt3.identity.case_name, "Axes001-1");
    assert_eq!(qt3.environment_ref, "TreeTrunc");
    assert_eq!(qt3.expression, "fn:count(//center/child::*)");
    assert_eq!(qt3.assertion, "assert-eq");
    assert_eq!(classify_qt3(&qt3), ExecutionDisposition::EngineUnsupported);

    let xslt30 = observe_xslt30_message_case();
    assert_eq!(xslt30.identity.case_name, "avt-0701");
    assert_eq!(xslt30.environment_ref, "avt-07");
    assert_eq!(xslt30.stylesheet_ref, "avt-0701.xsl");
    assert_eq!(xslt30.dependency, "XSLT20+");
    assert_eq!(
        xslt30.assertion_paths,
        [
            "all-of",
            "all-of/assert-xml",
            "all-of/assert-message",
            "all-of/assert-message/assert-eq",
        ]
    );
    assert_eq!(
        classify_xslt30(&xslt30),
        ExecutionDisposition::HarnessUnsupported
    );

    let records = [
        PrivateCaseRecord {
            identity: &qt3.identity,
            selection: SelectionDisposition::Selected,
            execution: classify_qt3(&qt3),
        },
        PrivateCaseRecord {
            identity: &xslt30.identity,
            selection: SelectionDisposition::Selected,
            execution: classify_xslt30(&xslt30),
        },
    ];
    assert_eq!(records.len(), 2);
    assert_eq!(
        records
            .iter()
            .filter(|record| record.selection == SelectionDisposition::Selected)
            .count(),
        2
    );
}

#[test]
fn unknown_assertion_metadata_is_a_visible_harness_gap() {
    let mut case = observe_qt3_axis_case();
    case.assertion = "future-assertion-family".to_owned();

    assert_eq!(
        classify_qt3(&case),
        ExecutionDisposition::HarnessUnsupported
    );

    let mut case = observe_xslt30_message_case();
    case.dependency = "future-feature-contract".to_owned();
    assert_eq!(
        classify_xslt30(&case),
        ExecutionDisposition::HarnessUnsupported
    );
}
