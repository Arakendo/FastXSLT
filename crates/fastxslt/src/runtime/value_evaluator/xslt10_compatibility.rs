//! Private `XPath` 1.0 value-conversion compatibility operations.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::{
    Xslt10ConcatExpression, Xslt10ConcatPart, Xslt10PathStringFunction,
    Xslt10PathStringFunctionKind, Xslt10PathSubstring, Xslt10PathTranslate,
};

use super::super::{
    ExecutionFailure, FailureCategory, ResultNode, RuntimeVariables, SequenceInputs,
    control_failure, failure, required_source_context, runtime_context,
};
use super::{append_boolean, append_source_string_value, append_text};

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
    let Some(node) = selected.get(position.saturating_sub(1)).copied() else {
        return Ok(());
    };
    let value = source
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    append_text(result, &value, inputs.request_id, control)
}

fn f64_lexical(value: f64) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

fn first_path_string(
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
    let mut sum = 0.0;
    for node in selected {
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

pub(super) fn append_concat(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10ConcatExpression,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    for part in &expression.parts {
        match part {
            Xslt10ConcatPart::Literal(value) => {
                append_text(result, value, inputs.request_id, control)?;
            }
            Xslt10ConcatPart::Variable(name) => {
                let value = variable_string_value(inputs, name, variables, control)?;
                append_text(result, &value, inputs.request_id, control)?;
            }
            Xslt10ConcatPart::Path(path) => {
                let value = first_path_string(inputs, context, path, control)?;
                append_text(result, &value, inputs.request_id, control)?;
            }
            Xslt10ConcatPart::SumPath(path) => {
                let value = sum_path_lexical(inputs, context, path, control)?;
                append_text(result, &value, inputs.request_id, control)?;
            }
        }
    }
    Ok(())
}
