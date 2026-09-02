//! Conserved inventory of the XSLT30 `insn/apply-imports` denominator.

use std::{fs, path::PathBuf};

use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const TEST_SET: &str = "tests/insn/apply-imports/_apply-imports-test-set.xml";
const OVERLAY: &str =
    include_str!("../../../../corpus/overlays/xslt30/apply-imports-denominator-v0.toml");

#[test]
fn inventories_and_seals_complete_apply_imports_denominator() {
    let (document, directory) = load_test_set();
    let cases = descendants_named(&document, document.document_node(), "test-case");
    assert_eq!(cases.len(), 1);
    let case = cases[0];
    assert_eq!(
        attribute(&document, case, "name"),
        Some("apply-imports-001")
    );
    assert_eq!(dependency(&document, case, "spec"), Some("XSLT30+"));
    assert!(OVERLAY.contains(&format!("set_file = \"{TEST_SET}\"")));
    assert!(OVERLAY.contains("case_count = 1"));
    assert_eq!(OVERLAY.matches("[[case_override]]").count(), 0);
    assert!(OVERLAY.contains("selection = \"harness-unsupported\""));

    let test = child_named(&document, case, "test").expect("test metadata");
    let stylesheets = element_children(&document, test)
        .into_iter()
        .filter(|node| local_name(&document, *node) == "stylesheet")
        .collect::<Vec<_>>();
    assert_eq!(stylesheets.len(), 3);
    assert_eq!(
        attribute(&document, stylesheets[0], "role"),
        Some("principal")
    );
    assert_eq!(
        attribute(&document, stylesheets[1], "role"),
        Some("secondary")
    );
    assert_eq!(
        attribute(&document, stylesheets[2], "role"),
        Some("secondary")
    );
    let assertion = child_named(&document, case, "result")
        .and_then(|node| child_named(&document, node, "assert"))
        .expect("native assertion");
    assert_eq!(
        document.string_value(assertion).trim(),
        "/out = \"R1R2BQ3BQ4AP5\""
    );

    let environment = descendants_named(&document, document.document_node(), "environment")
        .into_iter()
        .find(|node| attribute(&document, *node, "name") == Some("apply-imports-A"))
        .expect("native environment");
    let source = child_named(&document, environment, "source")
        .and_then(|node| child_named(&document, node, "content"))
        .expect("inline source");
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 65_536, 131_072));
    resources
        .admit(
            "urn:w3c:xslt30:apply-imports-001:source",
            document.string_value(source).into_bytes(),
        )
        .expect("admit inline source");
    for stylesheet in stylesheets {
        let file = attribute(&document, stylesheet, "file").expect("stylesheet file");
        resources
            .admit(
                format!("https://example.invalid/xslt30/insn/apply-imports/{file}"),
                fs::read(directory.join(file)).expect("read stylesheet and close handle"),
            )
            .expect("admit stylesheet module");
    }
    let snapshot = resources.seal();
    assert!(
        snapshot
            .get("urn:w3c:xslt30:apply-imports-001:source")
            .is_some()
    );
    for file in [
        "apply-imports-001.xsl",
        "apply-imports-001a.xsl",
        "apply-imports-001b.xsl",
    ] {
        assert!(
            snapshot
                .get(&format!(
                    "https://example.invalid/xslt30/insn/apply-imports/{file}"
                ))
                .is_some()
        );
    }
}

fn load_test_set() -> (Document, PathBuf) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/xslt30-test/tests/insn/apply-imports");
    let bytes = fs::read(directory.join("_apply-imports-test-set.xml"))
        .expect("read pinned test set and close handle");
    let parsed = parse_document(
        "urn:w3c:xslt30:insn:apply-imports:test-set",
        &bytes,
        ParseLimits {
            max_events: 2_048,
            max_depth: 32,
        },
    )
    .expect("parse pinned test set");
    (
        Document::from_parsed(parsed).expect("build test-set document"),
        directory,
    )
}

fn descendants_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in element_children(document, parent) {
        if local_name(document, child) == local {
            found.push(child);
        }
        found.extend(descendants_named(document, child, local));
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

fn dependency<'a>(document: &'a Document, case: NodeId, kind: &str) -> Option<&'a str> {
    child_named(document, case, "dependencies")
        .and_then(|node| child_named(document, node, kind))
        .and_then(|node| attribute(document, node, "value"))
}
