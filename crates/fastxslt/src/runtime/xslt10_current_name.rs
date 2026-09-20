//! Private charged execution for the bounded XSLT 1.0 `name(current())` slice.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};

use super::{ExecutionFailure, control_failure};

pub(super) fn has_descendant_or_following(
    source: &Document,
    context: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let mut descendants = Vec::new();
    collect_descendant_elements(source, context, request_id, control, &mut descendants)?;
    if descendants
        .iter()
        .copied()
        .any(|candidate| same_lexical_name(source, context, candidate))
    {
        return Ok(true);
    }
    let subtree_end = descendants
        .iter()
        .copied()
        .map(|node| source.document_order(node))
        .max()
        .unwrap_or_else(|| source.document_order(context));
    let mut all_elements = Vec::new();
    collect_descendant_elements(
        source,
        source.document_node(),
        request_id,
        control,
        &mut all_elements,
    )?;
    Ok(all_elements.into_iter().any(|candidate| {
        source.document_order(candidate) > subtree_end
            && same_lexical_name(source, context, candidate)
    }))
}

pub(super) fn count_descendants_same_name(
    source: &Document,
    context: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    let mut all_elements = Vec::new();
    collect_descendant_elements(
        source,
        source.document_node(),
        request_id,
        control,
        &mut all_elements,
    )?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(all_elements
        .into_iter()
        .filter(|candidate| same_lexical_name(source, context, *candidate))
        .count())
}

pub(super) fn select_children_of_same_name_elements(
    source: &Document,
    context: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut all_elements = Vec::new();
    collect_descendant_elements(
        source,
        source.document_node(),
        request_id,
        control,
        &mut all_elements,
    )?;
    let mut selected = Vec::new();
    for matching in all_elements
        .into_iter()
        .filter(|candidate| same_lexical_name(source, context, *candidate))
    {
        for child in source.children(matching).iter().copied() {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            if source.kind(child) == NodeKind::Element {
                selected.push(child);
            }
        }
    }
    Ok(selected)
}

fn collect_descendant_elements(
    source: &Document,
    parent: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
    selected: &mut Vec<NodeId>,
) -> Result<(), ExecutionFailure> {
    for child in source.children(parent).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.kind(child) == NodeKind::Element {
            selected.push(child);
        }
        collect_descendant_elements(source, child, request_id, control, selected)?;
    }
    Ok(())
}

fn same_lexical_name(source: &Document, left: NodeId, right: NodeId) -> bool {
    source.name(left) == source.name(right) && source.prefix(left) == source.prefix(right)
}
