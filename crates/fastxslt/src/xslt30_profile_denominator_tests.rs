//! Typed conservation for complete XSLT30 test sets excluded by one profile feature.

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde::Deserialize;

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const EXPECTED_SUITE: &str = "W3C XSLT 3.0 test suite";
const EXPECTED_REVISION: &str = "6f8fd9e966ae74a251a2604abef9d904c7bc5c9b";
const OVERLAYS: [&str; 3] = [
    include_str!("../../../corpus/overlays/xslt30/treat-as-denominator-v0.toml"),
    include_str!("../../../corpus/overlays/xslt30/type-expr-denominator-v0.toml"),
    include_str!("../../../corpus/overlays/xslt30/type-functions-denominator-v0.toml"),
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Denominator {
    suite: String,
    revision: String,
    set_file: String,
    case_count: usize,
    profile_exclusion: ProfileExclusion,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileExclusion {
    feature: String,
    selection: String,
    execution: String,
    rationale: String,
}

#[test]
fn conserves_complete_schema_aware_expression_denominators() {
    let mut set_files = BTreeSet::new();
    for source in OVERLAYS {
        let overlay: Denominator = toml::from_str(source).expect("typed XSLT30 profile overlay");
        assert_eq!(overlay.suite, EXPECTED_SUITE);
        assert_eq!(overlay.revision, EXPECTED_REVISION);
        assert!(set_files.insert(overlay.set_file.clone()));
        assert_eq!(overlay.profile_exclusion.feature, "schema_aware");
        assert_eq!(overlay.profile_exclusion.selection, "excluded-profile");
        assert_eq!(overlay.profile_exclusion.execution, "not-run");
        assert!(!overlay.profile_exclusion.rationale.trim().is_empty());
        assert_profile_dependency_covers_every_case(&overlay);
    }
}

fn assert_profile_dependency_covers_every_case(overlay: &Denominator) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test")
        .join(&overlay.set_file);
    let bytes = fs::read(path).expect("read pinned XSLT30 test set and close handle");
    let parsed = parse_document(
        &format!("urn:w3c:xslt30:{}", overlay.set_file),
        &bytes,
        ParseLimits {
            max_events: 100_000,
            max_depth: 64,
        },
    )
    .expect("parse pinned XSLT30 test set");
    let document = Document::from_parsed(parsed).expect("build pinned test-set XDM");
    let root = element_children(&document, document.document_node())
        .into_iter()
        .next()
        .expect("test-set root");
    assert!(has_feature_dependency(
        &document,
        root,
        &overlay.profile_exclusion.feature
    ));
    let cases = children_named(&document, root, "test-case");
    assert_eq!(cases.len(), overlay.case_count);
    let names = cases
        .iter()
        .map(|case| attribute(&document, *case, "name").expect("test case name"))
        .collect::<BTreeSet<_>>();
    assert_eq!(names.len(), overlay.case_count);
}

fn has_feature_dependency(document: &Document, root: NodeId, feature: &str) -> bool {
    children_named(document, root, "dependencies")
        .into_iter()
        .flat_map(|dependencies| children_named(document, dependencies, "feature"))
        .any(|dependency| attribute(document, dependency, "value") == Some(feature))
}

fn children_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    element_children(document, parent)
        .into_iter()
        .filter(|node| document.name(*node).is_some_and(|name| name.local == local))
        .collect()
}

fn element_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|node| document.kind(*node) == NodeKind::Element)
        .collect()
}

fn attribute<'a>(document: &'a Document, element: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(element).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}
