//! AR-0028 test-only retained ordinary-node oracle. No runtime admission.
//! Lifetime retention delegates identity/navigation to the qualified carrier.

use std::sync::Arc;

use super::owned_tree_experiment::{Document, NodeId};
use super::qualified_nodes::QualifiedSourceNode;
use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[path = "retained_nodes_reference/ordering_tests.rs"]
mod ordering_tests;

#[path = "retained_nodes_reference/table_reference.rs"]
mod table_reference;

#[derive(Debug, Clone)]
pub(crate) struct RetainedNode {
    owner: Arc<Document>,
    id: NodeId,
}

impl RetainedNode {
    pub(crate) fn borrowed(&self) -> QualifiedSourceNode<'_> {
        QualifiedSourceNode::Tree {
            document: &self.owner,
            id: self.id,
        }
    }

    pub(crate) fn same_node(&self, other: &Self) -> bool {
        self.borrowed().same_node(other.borrowed())
    }

    fn parent(&self) -> Option<Self> {
        self.owner.parent(self.id).map(|id| Self {
            owner: self.owner.clone(),
            id,
        })
    }
}

pub(crate) fn retain(
    owner: &Arc<Document>,
    ids: &[NodeId],
    control: &mut InvocationControl,
) -> Result<Vec<RetainedNode>, ControlFailure> {
    let mut retained = Vec::new();
    for &id in ids {
        // A test-reference retention charge, not a selected production formula.
        // Charge before cloning the owner and growing the retained sequence.
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        retained.push(RetainedNode {
            owner: owner.clone(),
            id,
        });
    }
    Ok(retained)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    pub(super) fn document(bytes: &[u8]) -> Arc<Document> {
        Arc::new(
            Document::from_parsed(
                parse_document(
                    "memory:retained.xml",
                    bytes,
                    ParseLimits {
                        max_events: 100,
                        max_depth: 20,
                    },
                )
                .unwrap(),
            )
            .unwrap(),
        )
    }

    pub(super) fn root(owner: &Document) -> NodeId {
        owner.children(owner.document_node())[0]
    }

    #[test]
    fn colliding_ids_and_equal_payloads_do_not_alias_after_focus_switch() {
        let first = document(b"<r><a>first</a></r>");
        let second = document(b"<r><a>second</a></r>");
        let equal_reparse = document(b"<r><a>first</a></r>");
        let id = first.children(root(&first))[0];
        assert_eq!(id, second.children(root(&second))[0]);
        let nodes = retain(&first, &[id], &mut InvocationControl::unbounded()).unwrap();
        let foreign = retain(&second, &[id], &mut InvocationControl::unbounded()).unwrap();
        let equal = retain(&equal_reparse, &[id], &mut InvocationControl::unbounded()).unwrap();
        assert!(!nodes[0].same_node(&foreign[0]));
        assert!(!nodes[0].same_node(&equal[0]));
        assert_eq!(nodes[0].owner.string_value(nodes[0].id), "first");
        assert_eq!(foreign[0].owner.string_value(foreign[0].id), "second");
        assert!(
            nodes[0]
                .borrowed()
                .compare_same_origin(foreign[0].borrowed())
                .is_none()
        );
        assert!(
            nodes[0]
                .parent()
                .unwrap()
                .borrowed()
                .same_node(QualifiedSourceNode::Tree {
                    document: &first,
                    id: root(&first)
                })
        );
    }

    #[test]
    fn repeated_owner_aliases_keep_identity_and_last_value_releases_owner() {
        let owner = document(b"<r><a/></r>");
        let weak = Arc::downgrade(&owner);
        let id = root(&owner);
        let first = retain(&owner, &[id], &mut InvocationControl::unbounded()).unwrap();
        let second = retain(&owner, &[id], &mut InvocationControl::unbounded()).unwrap();
        assert!(first[0].same_node(&second[0]));
        let clone = first.clone();
        drop(owner);
        drop(first);
        drop(second);
        assert!(weak.upgrade().is_some());
        assert_eq!(clone[0].owner.name(id).unwrap().local, "r");
        drop(clone);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn effective_owners_preserve_identity_navigation_and_namespace_coexistence() {
        let owner = document(b"<r xmlns:p='urn:p'> <a/> </r>");
        let root = root(&owner);
        let a = owner.children(root)[1];
        let original = retain(&owner, &[root, a], &mut InvocationControl::unbounded()).unwrap();
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
        for effective in [&view, &complete] {
            assert!(!Arc::ptr_eq(&owner, effective));
            let retained =
                retain(effective, &[root, a], &mut InvocationControl::unbounded()).unwrap();
            assert!(retained[0].same_node(&original[0]));
            assert!(retained[1].same_node(&original[1]));
            assert_eq!(retained[0].owner.children(root), &[a]);
            assert!(
                retained[0]
                    .borrowed()
                    .compare_same_origin(retained[1].borrowed())
                    .unwrap()
                    .is_lt()
            );
            let namespaces = effective
                .namespace_nodes_controlled(a, 8, &mut InvocationControl::unbounded())
                .unwrap();
            for namespace in namespaces {
                assert!(!retained[1].borrowed().same_node(namespace));
                assert!(
                    retained[1]
                        .borrowed()
                        .same_node(namespace.parent().unwrap())
                );
            }
        }
        assert_eq!(owner.children(root).len(), 3);
    }

    #[test]
    fn retention_failure_and_cancellation_release_partial_owner_clones() {
        let owner = document(b"<r><a/></r>");
        let ids = [root(&owner), owner.children(root(&owner))[0]];
        let baseline = Arc::strong_count(&owner);
        let mut exact = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xpath_node_visits: 2,
                ..WorkLimits::unbounded()
            },
        );
        let values = retain(&owner, &ids, &mut exact).unwrap();
        assert_eq!(Arc::strong_count(&owner), baseline + 2);
        drop(values);
        let mut short = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xpath_node_visits: 1,
                ..WorkLimits::unbounded()
            },
        );
        assert_eq!(
            retain(&owner, &ids, &mut short).unwrap_err(),
            ControlFailure::BudgetExhausted {
                domain: WorkDomain::XPathNodeVisit,
                limit: 1,
                consumed: 1,
                attempted: 1,
            }
        );
        assert_eq!(Arc::strong_count(&owner), baseline);
        let cancel = CancellationToken::new();
        cancel.cancel();
        let mut cancelled = InvocationControl::new(cancel, WorkLimits::unbounded());
        assert_eq!(
            retain(&owner, &ids, &mut cancelled).unwrap_err(),
            ControlFailure::Cancelled {
                domain: WorkDomain::XPathNodeVisit
            }
        );
        assert_eq!(Arc::strong_count(&owner), baseline);
        assert_eq!(
            retain(&owner, &ids, &mut InvocationControl::unbounded())
                .unwrap()
                .len(),
            2
        );
    }
}
