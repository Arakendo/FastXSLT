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

pub(super) fn has_prior_descendant_same_name(
    source: &Document,
    context: NodeId,
    position: f64,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let expected_name = lexical_name(source, context);
    has_prior_descendant_named(source, position, &expected_name, request_id, control)
}

pub(super) fn has_prior_descendant_named(
    source: &Document,
    position: f64,
    expected_name: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
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
    let mut candidate_position = 1.0;
    for candidate in all_elements {
        if candidate_position >= position {
            break;
        }
        if lexical_name(source, candidate) == expected_name {
            return Ok(true);
        }
        candidate_position += 1.0;
    }
    Ok(false)
}

pub(super) fn select_children_of_same_name_elements(
    source: &Document,
    context: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let expected_name = lexical_name(source, context);
    select_children_of_named_elements(source, &expected_name, request_id, control)
}

pub(super) fn select_children_of_named_elements(
    source: &Document,
    expected_name: &str,
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
        .filter(|candidate| lexical_name(source, *candidate) == expected_name)
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

pub(super) fn has_prior_child_of_named_elements_same_name(
    source: &Document,
    context: NodeId,
    parent_name: &str,
    position: f64,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let selected = select_children_of_named_elements(source, parent_name, request_id, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut candidate_position = 1.0;
    for candidate in selected {
        if candidate_position >= position {
            break;
        }
        if same_lexical_name(source, context, candidate) {
            return Ok(true);
        }
        candidate_position += 1.0;
    }
    Ok(false)
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

fn lexical_name(source: &Document, node: NodeId) -> String {
    source.name(node).map_or_else(String::new, |name| {
        source.prefix(node).map_or_else(
            || name.local.clone(),
            |prefix| format!("{prefix}:{}", name.local),
        )
    })
}
