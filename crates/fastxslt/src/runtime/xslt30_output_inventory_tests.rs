//! Conserved admission for the complete XSLT30 `decl/output` denominator.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use super::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, compile_resource,
    execute_transform_set,
};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
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
    const CASE_NAME: &str = "output-0128";
    assert!(OVERLAY.contains(&format!("case_name = \"{CASE_NAME}\"")));
    assert!(OVERLAY.contains("execution = \"passed\""));

    let (test_set, set_path) = load_test_set();
    let directory = set_path.parent().expect("output test-set directory");
    let root = document_element(&test_set);
    let case = element_children(&test_set, root)
        .into_iter()
        .find(|node| {
            local_name(&test_set, *node) == "test-case"
                && attribute(&test_set, *node, "name") == Some(CASE_NAME)
        })
        .expect("pinned output-0128 case");
    let test = child_named(&test_set, case, "test").expect("output-0128 test");
    let stylesheet_file = child_named(&test_set, test, "stylesheet")
        .and_then(|node| attribute(&test_set, node, "file"))
        .expect("output-0128 stylesheet file");
    let environment = resolve_environment(&test_set, root, case).expect("output-0128 environment");
    let source = child_named(&test_set, environment, "source").expect("output-0128 source");
    let source_content = child_named(&test_set, source, "content").expect("inline source content");
    let result = child_named(&test_set, case, "result").expect("output-0128 result");
    let assertion = first_element_child(&test_set, result).expect("output-0128 assertion");
    assert_eq!(local_name(&test_set, assertion), "assert-serialization");
    assert_eq!(attribute(&test_set, assertion, "method"), Some("xml"));
    let expected_file = attribute(&test_set, assertion, "file").expect("expected file");

    let source_id = format!("urn:w3c:xslt30:{CASE_NAME}:source");
    let stylesheet_id = format!("urn:w3c:xslt30:{CASE_NAME}:stylesheet");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 65_536, 131_072));
    resources
        .admit(
            source_id.clone(),
            test_set.string_value(source_content).into_bytes(),
        )
        .expect("admit output-0128 source");
    resources
        .admit(
            stylesheet_id.clone(),
            fs::read(directory.join(stylesheet_file)).expect("read stylesheet and close handle"),
        )
        .expect("admit output-0128 stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, &stylesheet_id).expect("compile output-0128");
    assert_eq!(program.output.method.as_deref(), Some("xml"));
    assert_eq!(program.output.include_content_type, Some(true));

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
        identity: CASE_NAME.to_owned(),
        result_identity: format!("result:{CASE_NAME}"),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id,
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    })
    .expect("admit output-0128 request");
    let results = execute_transform_set(set.seal()).expect("execute output-0128");
    let actual = &results.by_request[CASE_NAME].serialized;
    let expected = fs::read_to_string(directory.join(expected_file))
        .expect("read expected serialization and close handle");
    let canonical_expected = expected.replace("\r\n", "\n");
    assert_eq!(actual, &canonical_expected);
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
