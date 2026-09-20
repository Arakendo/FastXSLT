//! Private semantic result-tree representation and literal-attribute materialization.

use std::sync::Arc;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::{
    LocationPath, evaluate_location_path_controlled, evaluate_location_path_union_controlled,
};
use crate::xslt::golden_semantics_experiment::{
    ComputedAttribute, LiteralAttribute, LiteralAttributeValue, Xslt10AvtExpression, Xslt10AvtPart,
    Xslt10ConcatExpression,
};

use super::runtime_context::{RuntimeVariables, SequenceInputs};
use super::value_evaluator::{
    evaluate_binary_numeric_value, evaluate_xslt10_concat, normalized_node_string,
    normalized_node_string_length, xslt10_variable_string_value,
};
use super::{ExecutionFailure, FailureCategory, SequenceContext, control_failure, failure_at};

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

pub(super) fn assemble_element_content(
    attributes: &mut Vec<ResultAttribute>,
    items: Vec<ResultNode>,
    request_id: &str,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let mut children = Vec::new();
    for item in items {
        match item {
            ResultNode::PendingAttribute(attribute) if children.is_empty() => {
                if attributes
                    .iter()
                    .any(|existing| existing.name == attribute.name)
                {
                    return Err(super::failure(
                        "XTDE0410",
                        FailureCategory::Invalid,
                        Some(request_id),
                        "result element construction produced duplicate expanded attribute names",
                    ));
                }
                attributes.push(attribute);
            }
            ResultNode::PendingAttribute(_) => {
                return Err(super::failure(
                    "XTDE0410",
                    FailureCategory::Invalid,
                    Some(request_id),
                    "result attributes must be constructed before result child nodes",
                ));
            }
            child => children.push(child),
        }
    }
    Ok(children)
}

pub(super) fn literal_attributes_require_context_string(attributes: &[LiteralAttribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.value == LiteralAttributeValue::ContextStringValue)
}

pub(super) fn computed_attributes_require_context_string(attributes: &[ComputedAttribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.value == LiteralAttributeValue::ContextStringValue)
}

struct AttributeContext<'a> {
    inputs: &'a SequenceInputs<'a>,
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
    inputs: &SequenceInputs<'_>,
    attributes: &[LiteralAttribute],
    variables: &RuntimeVariables,
    focus: LiteralAttributeFocus<'_>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultAttribute>, ExecutionFailure> {
    let context = AttributeContext {
        inputs,
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
    execution: SequenceContext<'_>,
    focus: LiteralAttributeFocus<'_>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultAttribute>, ExecutionFailure> {
    let context = AttributeContext {
        inputs,
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
        let mut result =
            materialize_computed_attribute_value(attribute, &context, execution, focus, control)?;
        if let Some(name) = &attribute.dynamic_name {
            result.name = super::dynamic_attribute_name::resolve(
                inputs,
                focus.source.map(|(_, node)| node),
                name,
                variables,
                &attribute.location,
                control,
            )?;
        }
        materialized.push(result);
    }
    Ok(materialized)
}

fn materialize_computed_attribute_value(
    attribute: &ComputedAttribute,
    context: &AttributeContext<'_>,
    execution: SequenceContext<'_>,
    focus: LiteralAttributeFocus<'_>,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    match &attribute.value {
        LiteralAttributeValue::Number(instruction) => materialize_number_attribute(
            context.inputs,
            attribute,
            instruction,
            focus,
            context.request_id,
            control,
        ),
        LiteralAttributeValue::Xslt10ForEachPathStringValue(path) => {
            materialize_for_each_attribute(attribute, path, context, control)
        }
        LiteralAttributeValue::Xslt10CopyOfPathAttributeValue(path) => {
            materialize_xslt10_copy_of_attribute(attribute, path, context, control)
        }
        LiteralAttributeValue::Xslt10SequenceConstructor(instructions) => {
            materialize_xslt10_sequence_attribute(
                context.inputs,
                attribute,
                instructions,
                execution,
                context.variables,
                control,
            )
        }
        LiteralAttributeValue::Xslt10Concat(expression) => materialize_concat_attribute(
            context.inputs,
            attribute,
            expression,
            context.variables,
            focus,
            context.request_id,
            control,
        ),
        LiteralAttributeValue::CountSourceNodeVariable(variable) => {
            materialize_source_node_variable_count(
                context.inputs,
                attribute,
                variable,
                context.variables,
                context.request_id,
                control,
            )
        }
        LiteralAttributeValue::CountSourcePath(path)
        | LiteralAttributeValue::Xslt10LocalSourcePathCount(path) => materialize_source_path_count(
            attribute,
            std::slice::from_ref(path),
            focus.source,
            context.request_id,
            control,
        ),
        LiteralAttributeValue::CountSourcePathUnion(alternatives) => materialize_source_path_count(
            attribute,
            alternatives,
            focus.source,
            context.request_id,
            control,
        ),
        LiteralAttributeValue::ContextNormalizedStringLength => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, context.request_id))?;
            let node = focus.source.map(|(_, node)| node).ok_or_else(|| {
                failure_at(
                    "XPDY0002",
                    FailureCategory::Invalid,
                    Some(context.request_id),
                    attribute.location.clone(),
                    "normalized string length requires a source context item",
                )
            })?;
            Ok(ResultAttribute {
                name: attribute.name.clone(),
                value: normalized_node_string_length(context.inputs, node, control)?.to_string(),
            })
        }
        _ => materialize_attribute(
            &attribute.name,
            &attribute.value,
            &attribute.location,
            context,
            control,
        ),
    }
}

fn materialize_xslt10_sequence_attribute(
    inputs: &SequenceInputs<'_>,
    attribute: &ComputedAttribute,
    instructions: &[crate::xslt::golden_semantics_experiment::Instruction],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    let nodes = super::execute_sequence(inputs, instructions, execution, variables, control)?;
    let mut value = String::new();
    for node in nodes {
        if let ResultNode::Text(text) = node {
            value.push_str(&text);
        }
    }
    Ok(ResultAttribute {
        name: attribute.name.clone(),
        value,
    })
}

fn materialize_for_each_attribute(
    attribute: &ComputedAttribute,
    path: &LocationPath,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    charge_result_node(control, context.request_id)?;
    Ok(ResultAttribute {
        name: attribute.name.clone(),
        value: materialize_for_each_path_string_value(path, &attribute.location, context, control)?,
    })
}

fn materialize_xslt10_copy_of_attribute(
    attribute: &ComputedAttribute,
    path: &LocationPath,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    charge_result_node(control, context.request_id)?;
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            attribute.location.clone(),
            "xsl:copy-of in an attribute constructor requires a source context item",
        ));
    };
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let selected = evaluate_location_path_controlled(source, node, path, control)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let mut value = String::new();
    for node in selected {
        if matches!(
            source.kind(node),
            crate::xdm::owned_tree_experiment::NodeKind::Text
                | crate::xdm::owned_tree_experiment::NodeKind::Attribute
        ) {
            value.push_str(source.value(node).unwrap_or_default());
        }
    }
    Ok(ResultAttribute {
        name: attribute.name.clone(),
        value,
    })
}

fn materialize_source_node_variable_count(
    inputs: &SequenceInputs<'_>,
    attribute: &ComputedAttribute,
    variable: &str,
    variables: &RuntimeVariables,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .and_then(|()| control.charge(WorkDomain::XPathOperation, 1))
        .map_err(|failure| control_failure(failure, request_id))?;
    let nodes = variables
        .source_nodes(inputs.globals, variable)
        .ok_or_else(|| {
            failure_at(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(request_id),
                attribute.location.clone(),
                format!("computed-attribute count requires a source-node variable: ${variable}"),
            )
        })?;
    Ok(ResultAttribute {
        name: attribute.name.clone(),
        value: nodes.len().to_string(),
    })
}

fn materialize_number_attribute(
    inputs: &SequenceInputs<'_>,
    attribute: &ComputedAttribute,
    instruction: &crate::xslt::golden_semantics_experiment::Instruction,
    focus: LiteralAttributeFocus<'_>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let execution = SequenceContext {
        node: focus.source.map(|(_, node)| node),
        focus_position: focus.position,
        focus_size: focus.size,
        ..SequenceContext::new(None, None)
    };
    Ok(ResultAttribute {
        name: attribute.name.clone(),
        value: super::number_executor::evaluate(inputs, instruction, execution, control)?
            .unwrap_or_default(),
    })
}

fn materialize_concat_attribute(
    inputs: &SequenceInputs<'_>,
    attribute: &ComputedAttribute,
    expression: &Xslt10ConcatExpression,
    variables: &RuntimeVariables,
    focus: LiteralAttributeFocus<'_>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(ResultAttribute {
        name: attribute.name.clone(),
        value: evaluate_xslt10_concat(
            inputs,
            focus.source.map(|(_, node)| node),
            expression,
            variables,
            control,
        )?,
    })
}

fn materialize_source_path_count(
    attribute: &ComputedAttribute,
    alternatives: &[LocationPath],
    source_focus: Option<(&Document, NodeId)>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .and_then(|()| control.charge(WorkDomain::XPathOperation, 1))
        .map_err(|failure| control_failure(failure, request_id))?;
    let (source, node) = source_focus.ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(request_id),
            attribute.location.clone(),
            "computed-attribute count requires a source context item",
        )
    })?;
    let nodes = evaluate_location_path_union_controlled(source, node, alternatives, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(ResultAttribute {
        name: attribute.name.clone(),
        value: nodes.len().to_string(),
    })
}

fn materialize_attribute(
    name: &ExpandedName,
    value: &LiteralAttributeValue,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<ResultAttribute, ExecutionFailure> {
    charge_result_node(control, context.request_id)?;
    let value = match value {
        LiteralAttributeValue::Text(value) => value.clone(),
        LiteralAttributeValue::Variable(variable) => {
            attribute_variable_string(variable, location, context, control)?
        }
        LiteralAttributeValue::GlobalVariable(variable) => {
            attribute_global_variable_string(variable, location, context)?
        }
        LiteralAttributeValue::Number(_)
        | LiteralAttributeValue::CountSourceNodeVariable(_)
        | LiteralAttributeValue::CountSourcePath(_)
        | LiteralAttributeValue::CountSourcePathUnion(_)
        | LiteralAttributeValue::Xslt10LocalSourcePathCount(_)
        | LiteralAttributeValue::ContextNormalizedStringLength
        | LiteralAttributeValue::Xslt10ForEachPathStringValue(_)
        | LiteralAttributeValue::Xslt10CopyOfPathAttributeValue(_)
        | LiteralAttributeValue::Xslt10SequenceConstructor(_)
        | LiteralAttributeValue::Xslt10Concat(_) => {
            unreachable!("specialized computed-attribute value is materialized by its owner")
        }
        LiteralAttributeValue::Xslt10MultiPathAvt(expression) => {
            multi_path_avt(expression, location, context, control)?
        }
        LiteralAttributeValue::Xslt10PathStringLiteralComparison { path, value, equal } => {
            materialize_path_string_literal_comparison(
                path, value, *equal, location, context, control,
            )?
        }
        value @ (LiteralAttributeValue::Xslt10TextAndPath { .. }
        | LiteralAttributeValue::Xslt10TextAndNormalizedPath { .. }
        | LiteralAttributeValue::Xslt10TextAndGeneratedKeyIdentity { .. }
        | LiteralAttributeValue::Xslt10TextAndAttributeIntegerOffset { .. }
        | LiteralAttributeValue::Xslt10TextAndSourceAttributeConcat { .. }
        | LiteralAttributeValue::Xslt10TextAndSourceAttributeStartsWith { .. }
        | LiteralAttributeValue::Xslt10TextAndLiteralVariableConcat { .. }) => {
            materialize_text_avt(value, location, context, control)?
        }
        LiteralAttributeValue::Xslt10VariableAndPath {
            variable,
            separator,
            path,
        } => materialize_variable_and_path_avt(
            variable, separator, path, location, context, control,
        )?,
        LiteralAttributeValue::SourceAttribute(name) => {
            materialize_source_attribute(name, location, context, control)?
        }
        LiteralAttributeValue::ContextPosition => context.focus_position.to_string(),
        LiteralAttributeValue::ContextSize => context.focus_size.to_string(),
        LiteralAttributeValue::ContextLocalName => context_local_name(context),
        LiteralAttributeValue::ContextLexicalName => context_lexical_name(context, control)?,
        LiteralAttributeValue::ContextStringValue => context_string_attribute(location, context)?,
        LiteralAttributeValue::ContextIntegerIncrement(increment) => {
            materialize_context_integer_increment(*increment, location, context)?
        }
    };
    Ok(ResultAttribute {
        name: name.clone(),
        value,
    })
}

fn materialize_text_avt(
    value: &LiteralAttributeValue,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    match value {
        LiteralAttributeValue::Xslt10TextAndPath {
            prefix,
            path,
            suffix,
        } => materialize_source_path_avt(prefix, path, suffix, location, context, control),
        LiteralAttributeValue::Xslt10TextAndNormalizedPath {
            prefix,
            path,
            suffix,
        } => {
            materialize_normalized_source_path_avt(prefix, path, suffix, location, context, control)
        }
        LiteralAttributeValue::Xslt10TextAndGeneratedKeyIdentity {
            prefix,
            lookup,
            suffix,
        } => materialize_generated_key_identity_avt(
            prefix, lookup, suffix, location, context, control,
        ),
        LiteralAttributeValue::Xslt10TextAndAttributeIntegerOffset {
            prefix,
            name,
            offset,
            suffix,
        } => materialize_source_attribute_integer_offset_avt(
            prefix, name, *offset, suffix, location, context, control,
        ),
        LiteralAttributeValue::Xslt10TextAndSourceAttributeConcat {
            prefix,
            left,
            right,
            suffix,
        } => materialize_source_attribute_concat_avt(
            prefix, left, right, suffix, location, context, control,
        ),
        LiteralAttributeValue::Xslt10TextAndSourceAttributeStartsWith {
            prefix,
            value,
            prefix_attribute,
            suffix,
        } => materialize_source_attribute_starts_with_avt(
            prefix,
            value,
            prefix_attribute,
            suffix,
            location,
            context,
            control,
        ),
        LiteralAttributeValue::Xslt10TextAndLiteralVariableConcat {
            prefix,
            literal,
            variable,
            suffix,
        } => materialize_literal_variable_concat_avt(
            prefix, literal, variable, suffix, location, context, control,
        ),
        _ => unreachable!("text AVT dispatch admits only text-composition values"),
    }
}

fn materialize_generated_key_identity_avt(
    prefix: &str,
    lookup: &crate::xslt::golden_semantics_experiment::Xslt10KeyLookup,
    suffix: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let node = context.source_focus.map(|(_, node)| node).ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "generate-id(key()) requires a source context item",
        )
    })?;
    let selected = super::key_lookup::select(
        context.inputs,
        lookup,
        Some(node),
        context.variables,
        control,
    )?;
    let mut value = String::with_capacity(prefix.len() + suffix.len() + 24);
    value.push_str(prefix);
    if let Some(node) = selected.first() {
        value.push_str(&super::runtime_context::source_node_identity(*node));
    }
    value.push_str(suffix);
    Ok(value)
}

fn attribute_global_variable_string(
    variable: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
) -> Result<String, ExecutionFailure> {
    context
        .inputs
        .globals
        .atomics
        .get(variable)
        .map(|value| value.lexical().to_owned())
        .ok_or_else(|| {
            failure_at(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(context.request_id),
                location.clone(),
                format!("unbound global variable in attribute-set value: ${variable}"),
            )
        })
}

fn materialize_for_each_path_string_value(
    path: &LocationPath,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the computed-attribute for-each requires a source-node context",
        ));
    };
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let selected = evaluate_location_path_controlled(source, node, path, control)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let mut value = String::new();
    for selected in selected {
        control
            .charge(WorkDomain::XsltInstruction, 1)
            .map_err(|failure| control_failure(failure, context.request_id))?;
        value.push_str(
            &source
                .string_value_controlled(selected, control)
                .map_err(|failure| control_failure(failure, context.request_id))?,
        );
    }
    Ok(value)
}

fn charge_result_node(
    control: &mut InvocationControl,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))
}

fn materialize_path_string_literal_comparison(
    path: &LocationPath,
    expected: &str,
    equal: bool,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the path comparison attribute expression requires a source-node context",
        ));
    };
    let selected = evaluate_location_path_controlled(source, node, path, control)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    for selected in selected {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, context.request_id))?;
        if (source.string_value(selected) == expected) == equal {
            return Ok("true".to_owned());
        }
    }
    Ok("false".to_owned())
}

fn multi_path_avt(
    expression: &Xslt10AvtExpression,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let mut value = String::new();
    for part in &expression.parts {
        match part {
            Xslt10AvtPart::Text(text) => value.push_str(text),
            Xslt10AvtPart::Path(path) => value.push_str(&materialize_source_path_avt(
                "", path, "", location, context, control,
            )?),
            Xslt10AvtPart::PathUnion(alternatives) => value.push_str(
                &materialize_source_path_union_avt(alternatives, location, context, control)?,
            ),
            Xslt10AvtPart::Numeric(expression) => {
                let node = context.source_focus.map(|(_, node)| node);
                value.push_str(&evaluate_binary_numeric_value(
                    context.inputs,
                    node,
                    expression,
                    context.variables,
                    control,
                )?);
            }
        }
    }
    Ok(value)
}

fn materialize_source_path_union_avt(
    alternatives: &[LocationPath],
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the source-path union attribute expression requires a source-node context",
        ));
    };
    let selected = evaluate_location_path_union_controlled(source, node, alternatives, control)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    Ok(selected
        .first()
        .map_or_else(String::new, |selected| source.string_value(*selected)))
}

fn context_local_name(context: &AttributeContext<'_>) -> String {
    context
        .context_name
        .map_or_else(String::new, |name| name.local.clone())
}

fn context_lexical_name(
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let Some((source, node)) = context.source_focus else {
        return Ok(context
            .context_name
            .map_or_else(String::new, |name| name.local.clone()));
    };
    let Some(name) = source.name(node) else {
        return Ok(String::new());
    };
    Ok(source.prefix(node).map_or_else(
        || name.local.clone(),
        |prefix| format!("{prefix}:{}", name.local),
    ))
}

fn materialize_normalized_source_path_avt(
    prefix: &str,
    path: &LocationPath,
    suffix: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, node) = context.source_focus.ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "normalized path attribute requires a source context item",
        )
    })?;
    let selected = evaluate_location_path_controlled(source, node, path, control)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let normalized = if let Some(node) = selected.first().copied() {
        normalized_node_string(source, node, context.request_id, control)?
    } else {
        String::new()
    };
    let mut value = String::with_capacity(prefix.len() + normalized.len() + suffix.len());
    value.push_str(prefix);
    value.push_str(&normalized);
    value.push_str(suffix);
    Ok(value)
}

fn context_string_attribute(
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
) -> Result<String, ExecutionFailure> {
    context.context_value.map(str::to_owned).ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the context item is absent for the attribute value template",
        )
    })
}

fn materialize_source_attribute(
    name: &ExpandedName,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the source-copy attribute expression requires a source-node context",
        ));
    };
    Ok(
        source_attribute_value(source, node, name, context.request_id, control)?
            .unwrap_or_default()
            .to_owned(),
    )
}

fn materialize_literal_variable_concat_avt(
    prefix: &str,
    literal: &str,
    variable: &str,
    suffix: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let variable_value = attribute_variable_string(variable, location, context, control)?;
    let mut result =
        String::with_capacity(prefix.len() + literal.len() + variable_value.len() + suffix.len());
    result.push_str(prefix);
    result.push_str(literal);
    result.push_str(&variable_value);
    result.push_str(suffix);
    Ok(result)
}

fn materialize_variable_and_path_avt(
    variable: &str,
    separator: &str,
    path: &LocationPath,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let variable = attribute_variable_string(variable, location, context, control)?;
    let mut prefix = String::with_capacity(variable.len() + separator.len());
    prefix.push_str(&variable);
    prefix.push_str(separator);
    materialize_source_path_avt(&prefix, path, "", location, context, control)
}

fn attribute_variable_string(
    variable: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    xslt10_variable_string_value(context.inputs, variable, context.variables, control).map_err(
        |mut failure| {
            if failure.location.is_none() {
                failure.location = Some(location.clone());
            }
            failure
        },
    )
}

fn materialize_context_integer_increment(
    increment: i64,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
) -> Result<String, ExecutionFailure> {
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
        .and_then(|value| value.checked_add(increment))
        .ok_or_else(|| {
            failure_at(
                "FORG0001",
                FailureCategory::Invalid,
                Some(context.request_id),
                location.clone(),
                format!("attribute value is not an admitted integer: {lexical}"),
            )
        })
        .map(|value| value.to_string())
}

fn materialize_source_path_avt(
    prefix: &str,
    path: &LocationPath,
    suffix: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the source-path attribute expression requires a source-node context",
        ));
    };
    let selected = evaluate_location_path_controlled(source, node, path, control)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let selected_value = selected
        .first()
        .map_or_else(String::new, |selected| source.string_value(*selected));
    let mut result = String::with_capacity(prefix.len() + selected_value.len() + suffix.len());
    result.push_str(prefix);
    result.push_str(&selected_value);
    result.push_str(suffix);
    Ok(result)
}

fn materialize_source_attribute_integer_offset_avt(
    prefix: &str,
    name: &ExpandedName,
    offset: i64,
    suffix: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the source-attribute arithmetic AVT requires a source-node context",
        ));
    };
    let lexical = source_attribute_value(source, node, name, context.request_id, control)?
        .ok_or_else(|| {
            failure_at(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(context.request_id),
                location.clone(),
                format!(
                    "source attribute is absent for numeric AVT: @{}",
                    name.local
                ),
            )
        })?;
    let adjusted = lexical
        .trim()
        .parse::<i64>()
        .ok()
        .and_then(|value| value.checked_add(offset))
        .ok_or_else(|| {
            failure_at(
                "FORG0001",
                FailureCategory::Invalid,
                Some(context.request_id),
                location.clone(),
                format!("source attribute is not an admitted integer: {lexical}"),
            )
        })?;
    let mut result = String::with_capacity(prefix.len() + 20 + suffix.len());
    result.push_str(prefix);
    result.push_str(&adjusted.to_string());
    result.push_str(suffix);
    Ok(result)
}

fn materialize_source_attribute_concat_avt(
    prefix: &str,
    left: &ExpandedName,
    right: &ExpandedName,
    suffix: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the source-attribute concat AVT requires a source-node context",
        ));
    };
    let left = source_attribute_value(source, node, left, context.request_id, control)?
        .unwrap_or_default();
    let right = source_attribute_value(source, node, right, context.request_id, control)?
        .unwrap_or_default();
    let mut result = String::with_capacity(prefix.len() + left.len() + right.len() + suffix.len());
    result.push_str(prefix);
    result.push_str(left);
    result.push_str(right);
    result.push_str(suffix);
    Ok(result)
}

fn materialize_source_attribute_starts_with_avt(
    prefix: &str,
    value: &ExpandedName,
    prefix_attribute: &ExpandedName,
    suffix: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    context: &AttributeContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let Some((source, node)) = context.source_focus else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(context.request_id),
            location.clone(),
            "the source-attribute starts-with AVT requires a source-node context",
        ));
    };
    let value = source_attribute_value(source, node, value, context.request_id, control)?
        .unwrap_or_default();
    let prefix_value =
        source_attribute_value(source, node, prefix_attribute, context.request_id, control)?
            .unwrap_or_default();
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, context.request_id))?;
    let boolean = if value.starts_with(prefix_value) {
        "true"
    } else {
        "false"
    };
    let mut result = String::with_capacity(prefix.len() + boolean.len() + suffix.len());
    result.push_str(prefix);
    result.push_str(boolean);
    result.push_str(suffix);
    Ok(result)
}

fn source_attribute_value<'a>(
    source: &'a Document,
    node: NodeId,
    name: &ExpandedName,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<&'a str>, ExecutionFailure> {
    for attribute in source.attributes(node) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.name(*attribute) == Some(name) {
            return Ok(Some(source.value(*attribute).unwrap_or_default()));
        }
    }
    Ok(None)
}
