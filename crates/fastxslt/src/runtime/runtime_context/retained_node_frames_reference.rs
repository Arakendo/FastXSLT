//! AR-0028 test-only qualified ordinary-node sequence frame experiment.
//! Uses actual lexical shadowing and clone hooks; no compiled consumer.

use super::*;
use crate::xdm::retained_nodes_reference::{RetainedNode, retain};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

#[path = "retained_node_live_admission_tests.rs"]
mod live_admission_tests;

impl RuntimeVariables {
    pub(super) fn bind_retained_nodes_reference(&mut self, name: String, nodes: Vec<RetainedNode>) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.retained_nodes_reference).insert(name, nodes);
    }

    pub(super) fn retained_nodes_reference<'a>(
        &'a self,
        globals: &'a RuntimeGlobals,
        name: &str,
    ) -> Option<&'a Vec<RetainedNode>> {
        self.retained_nodes_reference.get(name).or_else(|| {
            self.allows_global_fallback(name)
                .then(|| globals.retained_nodes_reference.get(name))
                .flatten()
        })
    }
}

fn document(value: &str) -> Arc<Document> {
    Arc::new(
        Document::from_parsed(
            parse_document(
                "urn:frame-reference",
                format!("<r> <a>{value}</a> </r>").as_bytes(),
                ParseLimits {
                    max_events: 32,
                    max_depth: 8,
                },
            )
            .unwrap(),
        )
        .unwrap(),
    )
}

fn nodes(owner: &Arc<Document>) -> Vec<RetainedNode> {
    let root = owner.children(owner.document_node())[0];
    retain(
        owner,
        &[root, owner.children(root)[1]],
        &mut InvocationControl::unbounded(),
    )
    .unwrap()
}

#[test]
fn retained_node_frames_reference_cow_and_complete_clone_preserve_mixed_owners() {
    let first = document("first");
    let second = document("second");
    let first_nodes = nodes(&first);
    let second_nodes = nodes(&second);
    let mut mixed = first_nodes.clone();
    mixed.extend(second_nodes.clone());
    let mut globals = RuntimeGlobals::default();
    globals
        .retained_nodes_reference
        .insert("global".to_owned(), first_nodes.clone());
    let mut parent = RuntimeVariables::default();
    parent.bind_retained_nodes_reference("mixed".to_owned(), mixed.clone());
    for complete in [false, true] {
        let mut child = if complete {
            parent.clone_complete_for_sequence()
        } else {
            parent.clone()
        };
        assert_eq!(
            Arc::ptr_eq(
                &parent.retained_nodes_reference,
                &child.retained_nodes_reference
            ),
            !complete
        );
        assert!(child.bind_alias("alias".to_owned(), "mixed", &globals));
        assert!(child.bind_alias("global-alias".to_owned(), "global", &globals));
        child.bind_retained_nodes_reference("mixed".to_owned(), second_nodes.clone());
        for (expected, actual) in mixed
            .iter()
            .zip(child.retained_nodes_reference(&globals, "alias").unwrap())
        {
            assert!(expected.same_node(actual));
        }
        assert_eq!(
            parent
                .retained_nodes_reference(&globals, "mixed")
                .unwrap()
                .len(),
            4
        );
        assert_eq!(
            child
                .retained_nodes_reference(&globals, "mixed")
                .unwrap()
                .len(),
            2
        );
        child.bind_source_nodes("global".to_owned(), Vec::new());
        assert!(child.retained_nodes_reference(&globals, "global").is_none());
        assert!(
            parent
                .retained_nodes_reference(&globals, "global")
                .is_some()
        );
        assert!(
            child
                .retained_nodes_reference(&globals, "global-alias")
                .unwrap()[0]
                .same_node(&first_nodes[0])
        );
        child.bind_source_nodes("alias".to_owned(), Vec::new());
        assert!(child.retained_nodes_reference(&globals, "alias").is_none());
        child.bind_retained_nodes_reference("alias".to_owned(), first_nodes.clone());
        assert!(child.source_nodes(&globals, "alias").is_none());
    }
}

#[test]
fn retained_node_frames_reference_unrelated_mutation_does_not_detach_and_cleanup_releases() {
    let owner = document("kept");
    let weak = Arc::downgrade(&owner);
    let mut parent = RuntimeVariables::default();
    parent.bind_retained_nodes_reference("kept".to_owned(), nodes(&owner));
    let mut child = parent.clone();
    child.bind_source_nodes("other".to_owned(), Vec::new());
    assert!(Arc::ptr_eq(
        &parent.retained_nodes_reference,
        &child.retained_nodes_reference
    ));
    let complete = parent.clone_complete_for_sequence();
    drop(owner);
    drop(parent);
    drop(child);
    assert!(weak.upgrade().is_some());
    drop(complete);
    assert!(weak.upgrade().is_none());
}

#[test]
fn retained_node_frames_reference_effective_navigation_survives_frame_clones() {
    let original = document("visible");
    let globals = RuntimeGlobals::default();
    let mut parent = RuntimeVariables::default();
    parent.bind_retained_nodes_reference("original".to_owned(), nodes(&original));
    for effective in [
        original
            .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap(),
        original
            .derive_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap(),
    ] {
        let effective = Arc::new(effective);
        let root = effective.children(effective.document_node())[0];
        let selection = retain(
            &effective,
            &[root, effective.children(root)[0]],
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        for complete in [false, true] {
            let mut child = if complete {
                parent.clone_complete_for_sequence()
            } else {
                parent.clone()
            };
            child.bind_retained_nodes_reference("effective".to_owned(), selection.clone());
            let first = child
                .retained_nodes_reference(&globals, "original")
                .unwrap();
            let second = child
                .retained_nodes_reference(&globals, "effective")
                .unwrap();
            for (first, second) in first.iter().zip(second) {
                assert!(first.same_node(second));
            }
            assert_eq!(effective.children(root).len(), 1);
            assert_eq!(original.children(root).len(), 3);
        }
    }
}
