//! Private semantic result-tree representation and literal-attribute materialization.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xslt::golden_semantics_experiment::{
    ComputedAttribute, LiteralAttribute, LiteralAttributeValue,
};

use super::runtime_context::RuntimeVariables;
use super::{ExecutionFailure, FailureCategory, control_failure, failure_at};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ResultNode {
    Element {
        name: ExpandedName,
        namespaces: Vec<NamespaceBinding>,
        attributes: Vec<ResultAttribute>,
        children: Vec<ResultNode>,
    },
    Text(String),
    ProcessingInstruction {
        target: String,
        value: String,
    },
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResultAttribute {
    pub(super) name: ExpandedName,
    pub(super) value: String,
}

struct AttributeContext<'a> {
    variables: &'a RuntimeVariables,
    focus_position: usize,
    focus_size: usize,
    request_id: &'a str,
}

pub(super) fn materialize_literal_attributes(
    attributes: &[LiteralAttribute],
    variables: &RuntimeVariables,
    context_position: usize,
    context_size: usize,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultAttribute>, ExecutionFailure> {
    let context = AttributeContext {
        variables,
        focus_position: context_position,
        focus_size: context_size,
        request_id,
    };
    attributes
        .iter()
        .map(|attribute| {
            materialize_attribute(
                &attribute.name,
                &attribute.value,
                &attribute.location,
                &context,
                control,
            )
        })
        .collect()
}

pub(super) fn materialize_computed_attributes(
    attributes: &[ComputedAttribute],
    variables: &RuntimeVariables,
    context_position: usize,
    context_size: usize,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultAttribute>, ExecutionFailure> {
    let context = AttributeContext {
        variables,
        focus_position: context_position,
        focus_size: context_size,
        request_id,
    };
    attributes
        .iter()
        .map(|attribute| {
            materialize_attribute(
                &attribute.name,
                &attribute.value,
                &attribute.location,
                &context,
                control,
            )
        })
        .collect()
}

fn materialize_attribute(
    name: &ExpandedName,
    value: &LiteralAttributeValue,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let value = match value {
        LiteralAttributeValue::Text(value) => value.clone(),
        LiteralAttributeValue::Variable(variable) => context
            .variables
            .atomics
            .get(variable)
            .ok_or_else(|| {
                failure_at(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(context.request_id),
                    location.clone(),
                    format!("unbound variable in result attribute: ${variable}"),
                )
            })?
            .lexical()
            .to_owned(),
        LiteralAttributeValue::ContextPosition => context.focus_position.to_string(),
        LiteralAttributeValue::ContextSize => context.focus_size.to_string(),
    };
    Ok(ResultAttribute {
        name: name.clone(),
        value,
    })
}
