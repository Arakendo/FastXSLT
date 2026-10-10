//! Real sealed document admission while test-only ordinary-node frames are live.

use super::super::super::dynamic_document::prepare_document_from_snapshot;
use super::*;
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xslt::golden_semantics_experiment::DocumentRootReference;

fn snapshot(value: &str) -> ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(3, 128, 384));
    builder
        .admit(
            "https://example.test/live/a.xml",
            format!("<r><a>{value}</a></r>").into_bytes(),
        )
        .unwrap();
    builder
        .admit(
            "https://example.test/live/b.xml",
            b"<r><a>second</a></r>".to_vec(),
        )
        .unwrap();
    builder
        .admit("https://example.test/live/broken.xml", b"<r>".to_vec())
        .unwrap();
    builder.seal()
}

fn reference(name: &str) -> DocumentRootReference {
    DocumentRootReference {
        base: "https://example.test/live/style.xsl".to_owned(),
        reference: name.to_owned(),
        descendant_name: None,
    }
}

fn value(node: &RetainedNode) -> String {
    match node.borrowed() {
        crate::xdm::qualified_nodes::QualifiedSourceNode::Tree { document, id } => {
            document.string_value(id)
        }
        crate::xdm::qualified_nodes::QualifiedSourceNode::Namespace(_) => unreachable!(),
    }
}

#[test]
fn live_admission_preserves_parent_alias_and_failure_cleanup_in_both_frame_modes() {
    for complete in [false, true] {
        let resources = snapshot("first");
        let cache = RefCell::new(BTreeMap::new());
        let mut control = InvocationControl::unbounded();
        let first = prepare_document_from_snapshot(
            Some(&resources),
            None,
            &cache,
            "live",
            &reference("a.xml"),
            &mut control,
        )
        .unwrap();
        let first_id = first.document.children(first.document.document_node())[0];
        let weak_first = Arc::downgrade(&first.document);
        let globals = RuntimeGlobals::default();
        let mut parent = RuntimeVariables::default();
        parent.bind_retained_nodes_reference(
            "kept".to_owned(),
            retain(&first.document, &[first_id], &mut control).unwrap(),
        );
        let mut child = if complete {
            parent.clone_complete_for_sequence()
        } else {
            parent.clone()
        };
        assert!(child.bind_alias("alias".to_owned(), "kept", &globals));
        // This real cache insertion occurs while parent and child retain first.
        let second = prepare_document_from_snapshot(
            Some(&resources),
            None,
            &cache,
            "live",
            &reference("b.xml"),
            &mut control,
        )
        .unwrap();
        let second_id = second.document.children(second.document.document_node())[0];
        assert_eq!(first_id, second_id);
        let weak_second = Arc::downgrade(&second.document);
        child.bind_retained_nodes_reference(
            "kept".to_owned(),
            retain(&second.document, &[second_id], &mut control).unwrap(),
        );
        assert_eq!(
            value(&parent.retained_nodes_reference(&globals, "kept").unwrap()[0]),
            "first"
        );
        assert_eq!(
            value(&child.retained_nodes_reference(&globals, "alias").unwrap()[0]),
            "first"
        );
        assert_eq!(
            value(&child.retained_nodes_reference(&globals, "kept").unwrap()[0]),
            "second"
        );
        assert!(
            !child.retained_nodes_reference(&globals, "alias").unwrap()[0]
                .same_node(&child.retained_nodes_reference(&globals, "kept").unwrap()[0])
        );
        let before_failure = cache.borrow().len();
        assert!(
            prepare_document_from_snapshot(
                Some(&resources),
                None,
                &cache,
                "live",
                &reference("broken.xml"),
                &mut control
            )
            .is_err()
        );
        assert_eq!(cache.borrow().len(), before_failure);
        let again = prepare_document_from_snapshot(
            Some(&resources),
            None,
            &cache,
            "live",
            &reference("./a.xml"),
            &mut control,
        )
        .unwrap();
        assert!(Arc::ptr_eq(&first.document, &again.document));
        drop(again);
        drop(cache);
        drop(first);
        drop(second);
        drop(parent);
        assert!(weak_first.upgrade().is_some());
        assert!(weak_second.upgrade().is_some());
        drop(child);
        assert!(weak_first.upgrade().is_none());
        assert!(weak_second.upgrade().is_none());
    }
}

#[test]
fn live_admission_overlapping_snapshot_frames_never_share_invocation_maps() {
    let old = snapshot("old");
    let new = snapshot("new");
    let overlap = std::sync::Barrier::new(4);
    std::thread::scope(|scope| {
        let mut threads = Vec::new();
        for (resources, expected, complete) in [
            (&old, "old", false),
            (&old, "old", true),
            (&new, "new", false),
            (&new, "new", true),
        ] {
            let overlap = &overlap;
            threads.push(scope.spawn(move || {
                let cache = RefCell::new(BTreeMap::new());
                let mut control = InvocationControl::unbounded();
                let dynamic = prepare_document_from_snapshot(
                    Some(resources),
                    None,
                    &cache,
                    "overlap",
                    &reference("a.xml"),
                    &mut control,
                )
                .unwrap();
                let weak = Arc::downgrade(&dynamic.document);
                let id = dynamic.document.children(dynamic.document.document_node())[0];
                let globals = RuntimeGlobals::default();
                let mut parent = RuntimeVariables::default();
                parent.bind_retained_nodes_reference(
                    "kept".to_owned(),
                    retain(&dynamic.document, &[id], &mut control).unwrap(),
                );
                let mut child = if complete {
                    parent.clone_complete_for_sequence()
                } else {
                    parent.clone()
                };
                overlap.wait();
                assert!(child.bind_alias("alias".to_owned(), "kept", &globals));
                child.bind_source_nodes("kept".to_owned(), Vec::new());
                assert_eq!(
                    value(&child.retained_nodes_reference(&globals, "alias").unwrap()[0]),
                    expected
                );
                assert_eq!(
                    value(&parent.retained_nodes_reference(&globals, "kept").unwrap()[0]),
                    expected
                );
                assert!(child.retained_nodes_reference(&globals, "kept").is_none());
                drop(dynamic);
                drop(cache);
                drop(parent);
                assert!(weak.upgrade().is_some());
                drop(child);
                assert!(weak.upgrade().is_none());
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
    });
}
