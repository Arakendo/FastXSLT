//! Test-only invocation-table candidate; selection borrows its owning table.

use super::*;

#[path = "table_measurements.rs"]
mod measurements;

#[derive(Default)]
struct OwnerTable {
    owners: Vec<Arc<Document>>,
    identity: Arc<()>,
}

#[derive(Debug)]
struct OwnerKey {
    table: Arc<()>,
    index: usize,
}

impl OwnerTable {
    fn admit(
        &mut self,
        owner: &Arc<Document>,
        max_owners: usize,
        control: &mut InvocationControl,
    ) -> Result<OwnerKey, TableFailure> {
        for (index, existing) in self.owners.iter().enumerate() {
            control.charge(WorkDomain::XdmNode, 1)?;
            // Storage reuse only. Equal-origin views must not lose relationships.
            if Arc::ptr_eq(existing, owner) {
                return Ok(OwnerKey {
                    table: self.identity.clone(),
                    index,
                });
            }
        }
        if self.owners.len() == max_owners {
            return Err(TableFailure::Capacity);
        }
        control.charge(WorkDomain::XdmNode, 1)?;
        let index = self.owners.len();
        self.owners.push(owner.clone());
        Ok(OwnerKey {
            table: self.identity.clone(),
            index,
        })
    }

    fn select(
        &self,
        owner: &OwnerKey,
        ids: &[NodeId],
        control: &mut InvocationControl,
    ) -> Result<TableSelection<'_>, TableFailure> {
        if !Arc::ptr_eq(&owner.table, &self.identity) {
            return Err(TableFailure::ForeignOwner);
        }
        if owner.index >= self.owners.len() {
            return Err(TableFailure::MissingOwner);
        }
        let mut nodes = Vec::new();
        for &id in ids {
            control.charge(WorkDomain::XPathNodeVisit, 1)?;
            nodes.push((owner.index, id));
        }
        Ok(TableSelection { table: self, nodes })
    }
}

struct TableSelection<'a> {
    table: &'a OwnerTable,
    nodes: Vec<(usize, NodeId)>,
}

impl TableSelection<'_> {
    fn borrowed(&self, index: usize) -> QualifiedSourceNode<'_> {
        let (owner, id) = self.nodes[index];
        QualifiedSourceNode::Tree {
            document: &self.table.owners[owner],
            id,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TableFailure {
    Capacity,
    MissingOwner,
    ForeignOwner,
    Control(ControlFailure),
}

impl From<ControlFailure> for TableFailure {
    fn from(value: ControlFailure) -> Self {
        Self::Control(value)
    }
}

#[test]
fn table_reference_preserves_origin_and_effective_owner_not_storage_identity() {
    let owner = super::tests::document(b"<r> <a/> </r>");
    let root = super::tests::root(&owner);
    let view = Arc::new(
        owner
            .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
            .unwrap(),
    );
    let other = super::tests::document(b"<r> <a/> </r>");
    let mut table = OwnerTable::default();
    let mut control = InvocationControl::unbounded();
    let first = table.admit(&owner, 3, &mut control).unwrap();
    assert_eq!(
        first.index,
        table.admit(&owner, 3, &mut control).unwrap().index
    );
    let effective = table.admit(&view, 3, &mut control).unwrap();
    let foreign = table.admit(&other, 3, &mut control).unwrap();
    assert_ne!(first.index, effective.index);
    let a = table.select(&first, &[root], &mut control).unwrap();
    let b = table.select(&effective, &[root], &mut control).unwrap();
    let c = table.select(&foreign, &[root], &mut control).unwrap();
    assert!(a.borrowed(0).same_node(b.borrowed(0)));
    assert!(!a.borrowed(0).same_node(c.borrowed(0)));
    assert_eq!(table.owners[first.index].children(root).len(), 3);
    assert_eq!(table.owners[effective.index].children(root).len(), 1);
    // A coincident index in another table is interpreted only by that table.
    let mut second_table = OwnerTable::default();
    let token = second_table.admit(&other, 1, &mut control).unwrap();
    let second = second_table.select(&token, &[root], &mut control).unwrap();
    assert!(!a.borrowed(0).same_node(second.borrowed(0)));
    assert!(matches!(
        second_table.select(&first, &[root], &mut control),
        Err(TableFailure::ForeignOwner)
    ));
}

#[test]
fn table_reference_admission_and_selection_fail_without_partial_retention() {
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    let owner = super::tests::document(b"<r/>");
    let weak = Arc::downgrade(&owner);
    let mut table = OwnerTable::default();
    let mut limits = WorkLimits::unbounded();
    limits.xdm_nodes = 0;
    let mut denied = InvocationControl::new(CancellationToken::new(), limits);
    assert!(matches!(
        table.admit(&owner, 1, &mut denied),
        Err(TableFailure::Control(_))
    ));
    assert!(table.owners.is_empty());
    assert!(matches!(
        table.admit(&owner, 0, &mut InvocationControl::unbounded()),
        Err(TableFailure::Capacity)
    ));
    let index = table
        .admit(&owner, 1, &mut InvocationControl::unbounded())
        .unwrap();
    let baseline = Arc::strong_count(&owner);
    limits = WorkLimits::unbounded();
    limits.xpath_node_visits = 1;
    let mut short = InvocationControl::new(CancellationToken::new(), limits);
    assert!(matches!(
        table.select(&index, &[owner.document_node(); 2], &mut short),
        Err(TableFailure::Control(_))
    ));
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(matches!(
        table.select(
            &index,
            &[owner.document_node()],
            &mut InvocationControl::new(cancel, WorkLimits::unbounded())
        ),
        Err(TableFailure::Control(ControlFailure::Cancelled { .. }))
    ));
    let invalid = OwnerKey {
        table: table.identity.clone(),
        index: 1,
    };
    assert!(matches!(
        table.select(&invalid, &[], &mut InvocationControl::unbounded()),
        Err(TableFailure::MissingOwner)
    ));
    assert_eq!(Arc::strong_count(&owner), baseline);
    drop(owner);
    assert!(weak.upgrade().is_some());
    drop(table);
    assert!(weak.upgrade().is_none());
}

#[test]
fn table_reference_reports_selection_capacity_and_owner_clone_pressure() {
    let owner = super::tests::document(b"<r><a/></r>");
    let root = super::tests::root(&owner);
    for count in [1, 16, 1024] {
        let ids = vec![root; count];
        let baseline = Arc::strong_count(&owner);
        let explicit = retain(&owner, &ids, &mut InvocationControl::unbounded()).unwrap();
        assert_eq!(Arc::strong_count(&owner), baseline + count);
        let mut table = OwnerTable::default();
        let index = table
            .admit(&owner, 1, &mut InvocationControl::unbounded())
            .unwrap();
        let selected = table
            .select(&index, &ids, &mut InvocationControl::unbounded())
            .unwrap();
        assert_eq!(Arc::strong_count(&owner), baseline + count + 1);
        for (position, expected) in explicit.iter().enumerate() {
            assert!(expected.borrowed().same_node(selected.borrowed(position)));
        }
        println!(
            "nodes={count} explicit_capacity_bytes={} table_capacity_bytes={} explicit_owner_clones={count} table_owner_clones=1",
            explicit.capacity() * std::mem::size_of::<RetainedNode>(),
            selected.nodes.capacity() * std::mem::size_of::<(usize, NodeId)>()
                + table.owners.capacity() * std::mem::size_of::<Arc<Document>>()
        );
        drop(selected);
        assert_eq!(table.owners.len(), 1); // table retains until invocation teardown
        drop(table);
        drop(explicit);
        assert_eq!(Arc::strong_count(&owner), baseline);
    }
}
