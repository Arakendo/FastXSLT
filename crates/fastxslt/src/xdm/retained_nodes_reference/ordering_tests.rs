//! Ownership controls only: no cross-document ranking or runtime admission.

use super::tests::{document, root};
use super::*;

#[test]
fn mixed_same_origin_order_and_dedup_preserve_namespace_and_attribute_identity() {
    let owner = document(b"<r xmlns:p='urn:p' z='v'>text<a/></r>");
    let root = root(&owner);
    let tree_ids = [
        root,
        owner.attributes(root)[0],
        owner.children(root)[0],
        owner.children(root)[1],
    ];
    let retained = retain(&owner, &tree_ids, &mut InvocationControl::unbounded()).unwrap();
    let namespaces = owner
        .namespace_nodes_controlled(root, 8, &mut InvocationControl::unbounded())
        .unwrap();
    let expected = retained[..1]
        .iter()
        .map(RetainedNode::borrowed)
        .chain(namespaces.iter().copied())
        .chain(retained[1..].iter().map(RetainedNode::borrowed))
        .collect::<Vec<_>>();
    let mut shuffled = expected
        .iter()
        .rev()
        .copied()
        .chain(expected.iter().copied())
        .collect::<Vec<_>>();
    shuffled.sort_by(|left, right| left.compare_same_origin(*right).unwrap());
    shuffled.dedup_by(|left, right| left.same_node(*right));
    assert_eq!(shuffled.len(), expected.len());
    for (actual, expected) in shuffled.into_iter().zip(expected) {
        assert!(actual.same_node(expected));
    }
    // Colliding IDs/prefixes in an equal-byte reparse cannot enter this sort.
    let foreign = document(b"<r xmlns:p='urn:p' z='v'>text<a/></r>");
    let foreign_nodes = foreign
        .namespace_nodes_controlled(root, 8, &mut InvocationControl::unbounded())
        .unwrap();
    for left in &namespaces {
        for right in &foreign_nodes {
            assert!(!left.same_node(*right));
            assert_eq!(left.compare_same_origin(*right), None);
        }
    }
    let foreign_tree = retain(&foreign, &tree_ids, &mut InvocationControl::unbounded()).unwrap();
    let input = retained
        .iter()
        .chain(foreign_tree.iter())
        .chain(retained.iter());
    let mut unique: Vec<&RetainedNode> = Vec::new();
    for node in input {
        if !unique.iter().any(|previous| previous.same_node(node)) {
            unique.push(node);
        }
    }
    // Identity-only reference: preserving insertion order does not select
    // document order for a cross-origin union.
    assert_eq!(unique.len(), 8);
    assert!(unique[0].same_node(&retained[0]));
    assert!(unique[4].same_node(&foreign_tree[0]));
}

#[test]
fn same_origin_visible_normalization_agrees_across_complete_and_shared_views() {
    let owner = document(b"<r xmlns:p='urn:p' z='v'> <a/> </r>");
    let root = root(&owner);
    let child = owner.children(root)[1];
    let view = Arc::new(
        owner
            .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap(),
    );
    let complete = Arc::new(
        owner
            .derive_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap(),
    );
    let ids = [child, owner.attributes(root)[0], root, child];
    let original = retain(&owner, &ids, &mut InvocationControl::unbounded()).unwrap();
    for effective in [&view, &complete] {
        let values = retain(effective, &ids, &mut InvocationControl::unbounded()).unwrap();
        let mut mixed = values
            .iter()
            .map(RetainedNode::borrowed)
            .collect::<Vec<_>>();
        mixed.extend(
            effective
                .namespace_nodes_controlled(root, 8, &mut InvocationControl::unbounded())
                .unwrap(),
        );
        mixed.sort_by(|left, right| left.compare_same_origin(*right).unwrap());
        mixed.dedup_by(|left, right| left.same_node(*right));
        assert_eq!(mixed.len(), 5);
        assert!(mixed[0].same_node(original[2].borrowed()));
        assert!(mixed[3].same_node(original[1].borrowed()));
        assert!(mixed[4].same_node(original[0].borrowed()));
        assert_eq!(effective.children(root), &[child]);
    }
    assert_eq!(owner.children(root).len(), 3);
}

#[test]
fn concurrent_invocation_local_values_share_only_immutable_prepared_storage() {
    let owner = document(b"<r> <a>value</a> </r>");
    let baseline = Arc::strong_count(&owner);
    let root = root(&owner);
    let child = owner.children(root)[1];
    let overlap = std::sync::Barrier::new(4);
    std::thread::scope(|scope| {
        let mut threads = Vec::new();
        for strip in [false, true, false, true] {
            let prepared = &owner;
            let overlap = &overlap;
            threads.push(scope.spawn(move || {
                for _ in 0..32 {
                    let effective = if strip {
                        Arc::new(
                            prepared
                                .view_stripping_all_element_whitespace(
                                    &mut InvocationControl::unbounded(),
                                )
                                .unwrap(),
                        )
                    } else {
                        Arc::clone(prepared)
                    };
                    let nodes =
                        retain(&effective, &[child], &mut InvocationControl::unbounded()).unwrap();
                    // Every round has four live local selections before reading.
                    overlap.wait();
                    assert_eq!(nodes[0].owner.string_value(child), "value");
                    assert!(nodes[0].borrowed().same_node(QualifiedSourceNode::Tree {
                        document: prepared,
                        id: child
                    }));
                    assert_eq!(effective.children(root).len(), if strip { 1 } else { 3 });
                }
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
    });
    assert_eq!(Arc::strong_count(&owner), baseline);
    assert_eq!(owner.children(root).len(), 3);
}
