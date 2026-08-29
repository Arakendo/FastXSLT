//! Typed evaluation for relative wildcard paths filtered by an atomic variable.

use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xslt::golden_semantics_experiment::VariableFilteredElementPath;

use super::runtime_failure::{ExecutionFailure, control_failure};

pub(super) fn select(
    source: &Document,
    context: NodeId,
    path: &VariableFilteredElementPath,
    variables: &BTreeMap<String, AtomicValue>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let Some(value) = variables.get(&path.variable) else {
        return Ok(Vec::new());
    };
    let mut current = vec![context];
    for required in &path.parent_steps {
        let mut next = Vec::new();
        for parent in current {
            for child in source.children(parent) {
                charge_visit(control, request_id)?;
                if source.kind(*child) == NodeKind::Element && source.name(*child) == Some(required)
                {
                    next.push(*child);
                }
            }
        }
        current = next;
    }
    let mut selected = Vec::new();
    for parent in current {
        for child in source.children(parent) {
            charge_visit(control, request_id)?;
            if source.kind(*child) == NodeKind::Element
                && element_attribute_equals(source, *child, path, value, request_id, control)?
            {
                selected.push(*child);
            }
        }
    }
    Ok(selected)
}

pub(super) fn matches(
    source: &Document,
    node: NodeId,
    path: &VariableFilteredElementPath,
    variables: &BTreeMap<String, AtomicValue>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if source.kind(node) != NodeKind::Element {
        return Ok(false);
    }
    let Some(value) = variables.get(&path.variable) else {
        return Ok(false);
    };
    if !element_attribute_equals(source, node, path, value, request_id, control)? {
        return Ok(false);
    }
    let mut current = node;
    for required in path.parent_steps.iter().rev() {
        let Some(parent) = source.parent(current) else {
            return Ok(false);
        };
        charge_visit(control, request_id)?;
        if source.kind(parent) != NodeKind::Element || source.name(parent) != Some(required) {
            return Ok(false);
        }
        current = parent;
    }
    Ok(true)
}

fn element_attribute_equals(
    source: &Document,
    node: NodeId,
    path: &VariableFilteredElementPath,
    value: &AtomicValue,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    for attribute in source.attributes(node) {
        charge_visit(control, request_id)?;
        if source.name(*attribute) == Some(&path.attribute)
            && attribute_equals_atomic(&source.string_value(*attribute), value)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn charge_visit(control: &mut InvocationControl, request_id: &str) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, request_id))
}

pub(super) fn attribute_equals_atomic(attribute: &str, value: &AtomicValue) -> bool {
    if value.atomic_type() == BuiltinAtomicType::Integer {
        return attribute
            .parse::<i64>()
            .ok()
            .zip(value.lexical().parse::<i64>().ok())
            .is_some_and(|(left, right)| left == right);
    }
    attribute == value.lexical()
}
