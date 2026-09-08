//! Private semantic result-tree representation and literal-attribute materialization.

use std::sync::Arc;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xslt::golden_semantics_experiment::{
    ComputedAttribute, LiteralAttribute, LiteralAttributeValue,
};

use super::runtime_context::{RuntimeVariables, SequenceInputs};
use super::value_evaluator::evaluate_xslt10_concat;
use super::{ExecutionFailure, FailureCategory, control_failure, failure_at};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ResultNode {
    Element {
        name: ExpandedName,
        namespaces: Arc<[NamespaceBinding]>,
        attributes: Vec<ResultAttribute>,
        children: Vec<ResultNode>,
    },
    PendingAttribute(ResultAttribute),
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

pub(super) fn literal_attributes_require_context_string(attributes: &[LiteralAttribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.value == LiteralAttributeValue::ContextStringValue)
}

struct AttributeContext<'a> {
    variables: &'a RuntimeVariables,
    focus_position: usize,
    focus_size: usize,
    context_name: Option<&'a ExpandedName>,
    context_value: Option<&'a str>,
    source_focus: Option<(&'a Document, NodeId)>,
    request_id: &'a str,
}

#[derive(Clone, Copy)]
pub(super) struct LiteralAttributeFocus<'a> {
    pub(super) position: usize,
    pub(super) size: usize,
    pub(super) name: Option<&'a ExpandedName>,
    pub(super) value: Option<&'a str>,
    pub(super) source: Option<(&'a Document, NodeId)>,
}

pub(super) fn materialize_literal_attributes(
    attributes: &[LiteralAttribute],
    variables: &RuntimeVariables,
    focus: LiteralAttributeFocus<'_>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultAttribute>, ExecutionFailure> {
    let context = AttributeContext {
        variables,
        focus_position: focus.position,
        focus_size: focus.size,
        context_name: focus.name,
        context_value: focus.value,
        source_focus: focus.source,
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
    inputs: &SequenceInputs<'_>,
    attributes: &[ComputedAttribute],
    variables: &RuntimeVariables,
    focus: LiteralAttributeFocus<'_>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultAttribute>, ExecutionFailure> {
    let context = AttributeContext {
        variables,
        focus_position: focus.position,
        focus_size: focus.size,
        context_name: focus.name,
        context_value: focus.value,
        source_focus: focus.source,
        request_id,
    };
    let mut materialized = Vec::with_capacity(attributes.len());
    for attribute in attributes {
        if let LiteralAttributeValue::Xslt10Concat(expression) = &attribute.value {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            materialized.push(ResultAttribute {
                name: attribute.name.clone(),
                value: evaluate_xslt10_concat(
                    inputs,
                    focus.source.map(|(_, node)| node),
                    expression,
                    variables,
                    control,
                )?,
            });
        } else {
            materialized.push(materialize_attribute(
                &attribute.name,
                &attribute.value,
                &attribute.location,
                &context,
                control,
            )?);
        }
    }
    Ok(materialized)
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
        LiteralAttributeValue::Xslt10Concat(_) => {
            unreachable!("dynamic computed-attribute values are materialized by their owner")
        }
        LiteralAttributeValue::SourceAttribute(name) => {
            let Some((source, node)) = context.source_focus else {
                return Err(failure_at(
                    "XPDY0002",
                    FailureCategory::Invalid,
                    Some(context.request_id),
                    location.clone(),
                    "the source-copy attribute expression requires a source-node context",
                ));
            };
            let mut value = String::new();
            for &attribute in source.attributes(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, context.request_id))?;
                if source.name(attribute) == Some(name) {
                    source
                        .value(attribute)
                        .unwrap_or_default()
                        .clone_into(&mut value);
                    break;
                }
            }
            value
        }
        LiteralAttributeValue::ContextPosition => context.focus_position.to_string(),
        LiteralAttributeValue::ContextSize => context.focus_size.to_string(),
        LiteralAttributeValue::ContextLocalName => context
            .context_name
            .map_or_else(String::new, |name| name.local.clone()),
        LiteralAttributeValue::ContextStringValue => context
            .context_value
            .ok_or_else(|| {
                failure_at(
                    "XPDY0002",
                    FailureCategory::Invalid,
                    Some(context.request_id),
                    location.clone(),
                    "the context item is absent for the attribute value template",
                )
            })?
            .to_owned(),
        LiteralAttributeValue::ContextIntegerIncrement(increment) => {
            let lexical = context.context_value.ok_or_else(|| {
                failure_at(
                    "XPTY0004",
                    FailureCategory::Invalid,
                    Some(context.request_id),
                    location.clone(),
                    "the numeric attribute expression requires an atomic context value",
                )
            })?;
            lexical
                .trim()
                .parse::<i64>()
                .ok()
                .and_then(|value| value.checked_add(*increment))
                .ok_or_else(|| {
                    failure_at(
                        "FORG0001",
                        FailureCategory::Invalid,
                        Some(context.request_id),
                        location.clone(),
                        format!("attribute value is not an admitted integer: {lexical}"),
                    )
                })?
                .to_string()
        }
    };
    Ok(ResultAttribute {
        name: name.clone(),
        value,
    })
}
