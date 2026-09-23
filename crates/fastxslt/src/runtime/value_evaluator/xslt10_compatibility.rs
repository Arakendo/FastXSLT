//! Private `XPath` 1.0 value-conversion compatibility operations.

use std::borrow::Cow;
use std::collections::HashMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::{
    Xslt10ComposedPathTranslate, Xslt10ConcatExpression, Xslt10ConcatPart,
    Xslt10NormalizedVariableTranslate, Xslt10PathStringFunction, Xslt10PathStringFunctionKind,
    Xslt10PathSubstring, Xslt10PathTranslate, Xslt10StringOperand, Xslt10TranslateOperand,
    Xslt10VariableStringFunction,
};

use super::super::{
    ExecutionFailure, FailureCategory, ResultNode, RuntimeVariables, SequenceInputs,
    control_failure, failure, required_source_context, runtime_context,
};
use super::{append_boolean, append_source_string_value, append_text, normalized_node_string};

pub(super) fn variable_string_value(
    inputs: &SequenceInputs<'_>,
    name: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    if let Some(value) = variables.atomics.get(name) {
        return Ok(value.lexical().to_owned());
    }
    if variables.allows_global_fallback(name)
        && let Some(value) = inputs.globals.atomics.get(name)
    {
        return Ok(value.lexical().to_owned());
    }
    if let Some(values) = variables.atomic_sequences.get(name) {
        return Ok(values.first().map_or("", AtomicValue::lexical).to_owned());
    }
    if let Some(nodes) = variables.source_nodes(inputs.globals, name) {
        let Some(node) = nodes.first() else {
            return Ok(String::new());
        };
        let source = inputs.source.ok_or_else(|| {
            failure(
                "FXRT1004",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                "an XSLT 1.0 source-node conversion requires a principal source",
            )
        })?;
        return source
            .string_value_controlled(*node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id));
    }
    if let Some(tree) = variables.temporary_tree(inputs.globals, name) {
        return runtime_context::temporary_tree_string_value(tree, inputs.request_id, control);
    }
    if variables.allows_global_fallback(name) && inputs.globals.empty_sequences.contains(name) {
        return Ok(String::new());
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        format!("unbound variable: ${name}"),
    ))
}

pub(super) fn variable_numeric_lexical_value(
    inputs: &SequenceInputs<'_>,
    name: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let atomic = variables.atomics.get(name).or_else(|| {
        variables
            .allows_global_fallback(name)
            .then(|| inputs.globals.atomics.get(name))
            .flatten()
    });
    if let Some(value) = atomic {
        return Ok(xslt10_atomic_number_lexical(value));
    }
    if let Some(value) = variables
        .atomic_sequences
        .get(name)
        .and_then(|values| values.first())
    {
        return Ok(xslt10_atomic_number_lexical(value));
    }
    variable_string_value(inputs, name, variables, control)
}

fn xslt10_atomic_number_lexical(value: &AtomicValue) -> String {
    if value.atomic_type() == BuiltinAtomicType::Boolean {
        if value.lexical() == "true" {
            "1".to_owned()
        } else {
            "0".to_owned()
        }
    } else {
        value.lexical().to_owned()
    }
}

pub(super) fn variable_string_length(
    inputs: &SequenceInputs<'_>,
    name: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    let value = variable_string_value(inputs, name, variables, control)?;
    let mut length = 0_usize;
    for _ in value.chars() {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        length = length.checked_add(1).ok_or_else(|| {
            failure(
                "FOAR0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                "variable string length exceeds the supported integer range",
            )
        })?;
    }
    Ok(length)
}

pub(super) fn append_variable_path(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    path: &LocationPath,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let source = required_comparison_source(inputs)?;
    let roots = variables
        .source_nodes(inputs.globals, variable)
        .ok_or_else(|| {
            failure(
                "XPTY0019",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("variable path requires a source-node sequence: ${variable}"),
            )
        })?;
    let mut selected = Vec::new();
    for root in roots {
        selected.extend(
            evaluate_location_path_controlled(source, *root, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?,
        );
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    if let Some(node) = selected.first() {
        append_source_string_value(inputs, *node, result, control)?;
    }
    Ok(())
}

pub(super) fn variable_string_comparison(
    inputs: &SequenceInputs<'_>,
    name: &str,
    expected: &str,
    equal: bool,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if let Some(nodes) = variables.source_nodes(inputs.globals, name) {
        let source = required_comparison_source(inputs)?;
        for node in nodes {
            let actual = source
                .string_value_controlled(*node, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if (actual == expected) == equal {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    let actual = variable_string_value(inputs, name, variables, control)?;
    Ok((actual == expected) == equal)
}

pub(super) fn variable_number_comparison(
    inputs: &SequenceInputs<'_>,
    name: &str,
    expected: i32,
    equal: bool,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if let Some(nodes) = variables.source_nodes(inputs.globals, name) {
        let source = required_comparison_source(inputs)?;
        for node in nodes {
            let actual = source
                .string_value_controlled(*node, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if number_equals(&actual, expected) == equal {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    let actual = variable_string_value(inputs, name, variables, control)?;
    Ok(number_equals(&actual, expected) == equal)
}

fn required_comparison_source<'a>(
    inputs: &SequenceInputs<'a>,
) -> Result<&'a Document, ExecutionFailure> {
    inputs.source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "an XSLT 1.0 source-node comparison requires a principal source",
        )
    })
}

fn number_equals(actual: &str, expected: i32) -> bool {
    crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(actual)
        .and_then(|actual| actual.partial_cmp(&f64::from(expected)))
        == Some(std::cmp::Ordering::Equal)
}

pub(super) fn number_lexical(value: &str) -> String {
    crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(value)
        .map_or_else(|| "NaN".to_owned(), f64_lexical)
}

pub(super) fn append_variable_division_string(
    inputs: &SequenceInputs<'_>,
    numerator: &str,
    denominator: &str,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let numerator = variable_numeric_lexical_value(inputs, numerator, variables, control)?;
    let denominator = variable_numeric_lexical_value(inputs, denominator, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let numerator =
        crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&numerator)
            .unwrap_or(f64::NAN);
    let denominator =
        crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&denominator)
            .unwrap_or(f64::NAN);
    append_text(
        result,
        &f64_lexical(numerator / denominator),
        inputs.request_id,
        control,
    )
}

pub(super) fn append_variable_position_path(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    selection: (&LocationPath, &str, bool),
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (path, variable, explicit_position_comparison) = selection;
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    if !explicit_position_comparison && variables.temporary_tree(inputs.globals, variable).is_some()
    {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let Some(node) = selected.first().copied() else {
            return Ok(());
        };
        let value = source
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        return append_text(result, &value, inputs.request_id, control);
    }
    let position = variable_string_value(inputs, variable, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(position) =
        crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&position)
    else {
        return Ok(());
    };
    if !position.is_finite() || position < 1.0 || position.fract() != 0.0 {
        return Ok(());
    }
    let Ok(position) = position.to_string().parse::<usize>() else {
        return Ok(());
    };
    let index = position.saturating_sub(1);
    let node = if path.single_step_predicate_axis_is_reverse() {
        selected.iter().rev().nth(index).copied()
    } else {
        selected.get(index).copied()
    };
    let Some(node) = node else {
        return Ok(());
    };
    let value = source
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    append_text(result, &value, inputs.request_id, control)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_descendant_child_variable_position_path(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    path: &LocationPath,
    variable: &str,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let position = variable_string_value(inputs, variable, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(position) =
        crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&position)
    else {
        return Ok(());
    };
    if !position.is_finite() || position < 1.0 || position.fract() != 0.0 {
        return Ok(());
    }
    let Ok(position) = position.to_string().parse::<usize>() else {
        return Ok(());
    };
    let mut parent_positions = HashMap::<NodeId, usize>::new();
    for node in selected {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let Some(parent) = source.parent(node) else {
            continue;
        };
        let current = parent_positions.entry(parent).or_default();
        *current += 1;
        if *current == position {
            let value = source
                .string_value_controlled(node, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            return append_text(result, &value, inputs.request_id, control);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_grouped_variable_position_path(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    selection: &LocationPath,
    variable: &str,
    suffix: &LocationPath,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, selection, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let position = variable_string_value(inputs, variable, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(position) =
        crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&position)
    else {
        return Ok(());
    };
    if !position.is_finite() || position < 1.0 || position.fract() != 0.0 {
        return Ok(());
    }
    let Ok(position) = position.to_string().parse::<usize>() else {
        return Ok(());
    };
    let Some(node) = selected.get(position.saturating_sub(1)).copied() else {
        return Ok(());
    };
    let selected = evaluate_location_path_controlled(source, node, suffix, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(node) = selected.first().copied() else {
        return Ok(());
    };
    let value = source
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    append_text(result, &value, inputs.request_id, control)
}

fn f64_lexical(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value == f64::INFINITY {
        "Infinity".to_owned()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".to_owned()
    } else if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

pub(super) fn first_path_string(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    path: &LocationPath,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(node) = selected.first().copied() else {
        return Ok(String::new());
    };
    source
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))
}

pub(super) fn append_path_string_function(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10PathStringFunction,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = first_path_string(inputs, context, &expression.path, control)?;
    let path_operand;
    let operand = match &expression.operand {
        Xslt10StringOperand::Literal(value) => value.as_str(),
        Xslt10StringOperand::Path(path) => {
            path_operand = first_path_string(inputs, context, path, control)?;
            &path_operand
        }
    };
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    match expression.kind {
        Xslt10PathStringFunctionKind::Contains => {
            append_boolean(inputs, value.contains(operand), result, control)
        }
        Xslt10PathStringFunctionKind::StartsWith => {
            append_boolean(inputs, value.starts_with(operand), result, control)
        }
        Xslt10PathStringFunctionKind::SubstringBefore => append_text(
            result,
            value.split_once(operand).map_or("", |pair| pair.0),
            inputs.request_id,
            control,
        ),
        Xslt10PathStringFunctionKind::SubstringAfter => append_text(
            result,
            value.split_once(operand).map_or("", |pair| pair.1),
            inputs.request_id,
            control,
        ),
    }
}

pub(super) fn append_variable_string_function(
    inputs: &SequenceInputs<'_>,
    expression: &Xslt10VariableStringFunction,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = variable_string_value(inputs, &expression.variable, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    match expression.kind {
        Xslt10PathStringFunctionKind::Contains => {
            append_boolean(inputs, value.contains(&expression.operand), result, control)
        }
        Xslt10PathStringFunctionKind::StartsWith => append_boolean(
            inputs,
            value.starts_with(&expression.operand),
            result,
            control,
        ),
        Xslt10PathStringFunctionKind::SubstringBefore => append_text(
            result,
            value
                .split_once(&expression.operand)
                .map_or("", |pair| pair.0),
            inputs.request_id,
            control,
        ),
        Xslt10PathStringFunctionKind::SubstringAfter => append_text(
            result,
            value
                .split_once(&expression.operand)
                .map_or("", |pair| pair.1),
            inputs.request_id,
            control,
        ),
    }
}

pub(super) fn append_sum_path(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    path: &LocationPath,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = sum_path_lexical(inputs, context, path, control)?;
    append_text(result, &value, inputs.request_id, control)
}

pub(super) fn sum_path_lexical(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    path: &LocationPath,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    sum_nodes_lexical(inputs, source, &selected, control)
}

pub(super) fn append_variable_sum(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "an XSLT 1.0 source-node sum requires a principal source",
        )
    })?;
    let selected = variables
        .source_nodes(inputs.globals, variable)
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("sum requires a source-node sequence: ${variable}"),
            )
        })?;
    let value = sum_nodes_lexical(inputs, source, selected, control)?;
    append_text(result, &value, inputs.request_id, control)
}

fn sum_nodes_lexical(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    selected: &[NodeId],
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let mut sum = 0.0;
    for &node in selected {
        let value = source
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let Some(value) =
            crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&value)
        else {
            return Ok("NaN".to_owned());
        };
        sum += value;
    }
    Ok(f64_lexical(sum))
}

pub(super) fn append_path_substring(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10PathSubstring,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = first_path_string(inputs, context, &expression.path, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let value = crate::xpath::static_string_experiment::evaluate_substring(
        &value,
        f64::from_bits(expression.start_bits),
        expression.length_bits.map(f64::from_bits),
    );
    append_text(result, &value, inputs.request_id, control)
}

pub(super) fn append_path_translate(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10PathTranslate,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = first_path_string(inputs, context, &expression.path, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let value = crate::xpath::static_string_experiment::evaluate_translate(
        &value,
        &expression.search,
        &expression.replacement,
    );
    append_text(result, &value, inputs.request_id, control)
}

pub(super) fn append_normalized_variable_translate(
    inputs: &SequenceInputs<'_>,
    expression: &Xslt10NormalizedVariableTranslate,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "source-node variable translation requires a source document",
        )
    })?;
    let nodes = variables
        .source_nodes(inputs.globals, &expression.variable)
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!(
                    "normalize-space requires a source-node sequence: ${}",
                    expression.variable
                ),
            )
        })?;
    let value = match nodes.first().copied() {
        Some(node) => normalized_node_string(source, node, inputs.request_id, control)?,
        None => String::new(),
    };
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let value = crate::xpath::static_string_experiment::evaluate_translate(
        &value,
        &expression.search,
        &expression.replacement,
    );
    append_text(result, &value, inputs.request_id, control)
}

pub(super) fn append_composed_path_translate(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10ComposedPathTranslate,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = first_path_string(inputs, context, &expression.path, control)?;
    let search = translate_operand_value(inputs, context, &expression.search, variables, control)?;
    let replacement =
        translate_operand_value(inputs, context, &expression.replacement, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let value =
        crate::xpath::static_string_experiment::evaluate_translate(&value, &search, &replacement);
    append_text(result, &value, inputs.request_id, control)
}

fn translate_operand_value<'a>(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    operand: &'a Xslt10TranslateOperand,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Cow<'a, str>, ExecutionFailure> {
    match operand {
        Xslt10TranslateOperand::Literal(value) => Ok(Cow::Borrowed(value)),
        Xslt10TranslateOperand::Concat(expression) => {
            concat_value(inputs, context, expression, variables, control).map(Cow::Owned)
        }
    }
}

pub(super) fn append_concat(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10ConcatExpression,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = concat_value(inputs, context, expression, variables, control)?;
    append_text(result, &value, inputs.request_id, control)
}

pub(super) fn concat_value(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10ConcatExpression,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut result = String::new();
    for part in &expression.parts {
        match part {
            Xslt10ConcatPart::Literal(value) => result.push_str(value),
            Xslt10ConcatPart::Variable(name) => {
                let value = variable_string_value(inputs, name, variables, control)?;
                result.push_str(&value);
            }
            Xslt10ConcatPart::VariablePosition {
                variable,
                position_variable,
            } => {
                let Some(node) = variable_position_source_node(
                    inputs,
                    variable,
                    position_variable,
                    variables,
                    control,
                )?
                else {
                    continue;
                };
                let source = inputs
                    .source
                    .expect("source-node variable requires a source");
                let value = source
                    .string_value_controlled(node, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                result.push_str(&value);
            }
            Xslt10ConcatPart::Path(path) => {
                let value = first_path_string(inputs, context, path, control)?;
                result.push_str(&value);
            }
            Xslt10ConcatPart::SumPath(path) => {
                let value = sum_path_lexical(inputs, context, path, control)?;
                result.push_str(&value);
            }
        }
    }
    Ok(result)
}

pub(super) fn variable_position_source_node(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    position_variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Option<NodeId>, ExecutionFailure> {
    let nodes = variables
        .source_nodes(inputs.globals, variable)
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("positional variable selection requires source nodes: ${variable}"),
            )
        })?;
    let position = variable_string_value(inputs, position_variable, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, nodes.len().saturating_add(1))
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(
        crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&position)
            .filter(|position| position.is_finite() && *position >= 1.0 && position.fract() == 0.0)
            .and_then(|position| position.to_string().parse::<usize>().ok())
            .and_then(|position| nodes.get(position.saturating_sub(1)).copied()),
    )
}

pub(super) fn variable_node_position_source_node(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    position: crate::xslt::golden_semantics_experiment::Xslt10NodePosition,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Option<NodeId>, ExecutionFailure> {
    let nodes = variables
        .source_nodes(inputs.globals, variable)
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("positional variable selection requires source nodes: ${variable}"),
            )
        })?;
    control
        .charge(WorkDomain::XPathOperation, nodes.len().saturating_add(1))
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let index = match position {
        crate::xslt::golden_semantics_experiment::Xslt10NodePosition::Index(position) => {
            position.saturating_sub(1)
        }
        crate::xslt::golden_semantics_experiment::Xslt10NodePosition::Last => {
            return Ok(nodes.last().copied());
        }
        crate::xslt::golden_semantics_experiment::Xslt10NodePosition::LastMinus(offset) => {
            let Some(index) = nodes.len().checked_sub(offset.saturating_add(1)) else {
                return Ok(None);
            };
            index
        }
    };
    Ok(nodes.get(index).copied())
}

pub(super) fn append_variable_node_position(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    position: crate::xslt::golden_semantics_experiment::Xslt10NodePosition,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let Some(node) =
        variable_node_position_source_node(inputs, variable, position, variables, control)?
    else {
        return Ok(());
    };
    let source = inputs
        .source
        .expect("source-node variable requires a source");
    let value = source
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    append_text(result, &value, inputs.request_id, control)
}
