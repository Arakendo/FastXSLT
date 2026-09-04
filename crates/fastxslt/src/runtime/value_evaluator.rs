//! Private dynamic value evaluation for admitted `xsl:value-of` expressions.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{NodeId, StringValueVisitFailure};
use crate::xpath::castable_experiment::{
    CastableExpression, evaluate as evaluate_castable, evaluate_value as evaluate_castable_value,
    variable_name as castable_variable_name,
};
use crate::xpath::decimal_sum_for_experiment::{
    DecimalSumEvaluationFailure, evaluate as evaluate_decimal_sum_for,
};
use crate::xpath::deep_equal_boolean_experiment::evaluate as evaluate_deep_equal;
use crate::xpath::deep_equal_experiment::DeepEqualEvaluationFailure;
use crate::xpath::default_collation_experiment::{
    DefaultCollationValue, evaluate as evaluate_default_collation,
};
use crate::xpath::focus_sum_for_experiment::{
    FocusSumEvaluationFailure, evaluate as evaluate_focus_sum_for,
};
use crate::xpath::format_number_experiment::{
    FormatNumberEvaluationFailure, evaluate as evaluate_format_number,
};
use crate::xpath::integer_for_experiment::evaluate as evaluate_integer_for;
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ConditionalIntegerBranch, ConditionalIntegerCondition, ConditionalIntegerExpression,
    ConditionalPathBranch, ConditionalPathExpression, IntegerComparisonOperator, ValueExpression,
};

use super::{
    ExecutionFailure, FailureCategory, ResultNode, RuntimeVariables, SequenceInputs, append_text,
    control_failure, failure, required_source_context, runtime_context,
};

pub(super) fn execute_value_of(
    inputs: &SequenceInputs<'_>,
    select: &ValueExpression,
    separator: &str,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    match select {
        ValueExpression::LiteralString(value) => {
            append_text(result, value, inputs.request_id, control)?;
        }
        ValueExpression::LocationPath(path) => {
            append_location_path_string(inputs, path, context, result, control)?;
        }
        ValueExpression::CountLocationPath(path) => {
            append_location_path_count(inputs, path, context, result, control)?;
        }
        ValueExpression::RootPath(path) => {
            append_root_path_string(inputs, path, context, result, control)?;
        }
        ValueExpression::RootVariable(name) => {
            append_root_variable_string(inputs, name, variables, result, control)?;
        }
        ValueExpression::GeneratedRootIdentity(path) => {
            append_generated_root_identity(inputs, path, context, result, control)?;
        }
        ValueExpression::GeneratedTemporaryRootIdentity {
            variable,
            descendant_local,
        } => append_generated_temporary_root_identity(
            inputs,
            variable,
            descendant_local.as_deref(),
            variables,
            result,
            control,
        )?,
        ValueExpression::GeneratedDocumentRootIdentity(reference) => {
            append_generated_document_root_identity(inputs, reference, result, control)?;
        }
        ValueExpression::ContextNodeName => {
            append_context_node_name(inputs, context, result, control)?;
        }
        ValueExpression::UpperCaseContextString => {
            append_upper_case_context_string(inputs, context, result, control)?;
        }
        ValueExpression::Variable(name) => {
            append_variable_value(inputs, name, separator, variables, result, control)?;
        }
        ValueExpression::LiteralVariableConcat { literal, variable } => {
            append_literal_variable_concat(inputs, literal, variable, variables, result, control)?;
        }
        ValueExpression::IntegerFor(expression) => {
            append_integer_for(inputs, expression, separator, result, control)?;
        }
        ValueExpression::FocusSumFor(expression) => {
            let (source, context) = required_source_context(inputs, context)?;
            let value = evaluate_focus_sum_for(expression, source, context, control).map_err(
                |evaluation_failure| match evaluation_failure {
                    FocusSumEvaluationFailure::Control(failure) => {
                        control_failure(failure, inputs.request_id)
                    }
                    FocusSumEvaluationFailure::Unsupported => failure(
                        "FXRT1005",
                        FailureCategory::Unsupported,
                        Some(inputs.request_id),
                        "non-empty numeric multiplication is outside the private focus-preserving sum slice",
                    ),
                },
            )?;
            append_text(result, &value.to_string(), inputs.request_id, control)?;
        }
        ValueExpression::DecimalSumFor(expression) => {
            let value = execute_decimal_sum(inputs, expression, context, control)?;
            append_text(result, &value, inputs.request_id, control)?;
        }
        ValueExpression::FormatNumber(expression) => {
            append_format_number(inputs, expression, variables, result, control)?;
        }
        ValueExpression::Castable(expression) => {
            let value =
                execute_castable_expression(inputs, expression, context, variables, control)?;
            append_boolean(inputs, value, result, control)?;
        }
        ValueExpression::DeepEqual(expression) => {
            let value = execute_deep_equal(inputs, expression, context, control)?;
            append_boolean(inputs, value, result, control)?;
        }
        ValueExpression::DefaultCollation(expression) => {
            append_default_collation(inputs, expression, result, control)?;
        }
        ValueExpression::ConditionalInteger(expression) => {
            let value = evaluate_conditional_integer(inputs, expression, context, control)?;
            append_text(result, &value.to_string(), inputs.request_id, control)?;
        }
        ValueExpression::ConditionalPath(expression) => {
            append_conditional_path(inputs, expression, context, result, control)?;
        }
    }
    Ok(())
}

fn append_boolean(
    inputs: &SequenceInputs<'_>,
    value: bool,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    append_text(
        result,
        if value { "true" } else { "false" },
        inputs.request_id,
        control,
    )
}

fn append_default_collation(
    inputs: &SequenceInputs<'_>,
    expression: &crate::xpath::default_collation_experiment::DefaultCollationExpression,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = evaluate_default_collation(expression, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let value = match value {
        DefaultCollationValue::Boolean(value) => {
            if value {
                "true".to_owned()
            } else {
                "false".to_owned()
            }
        }
        DefaultCollationValue::Integer(value) => value.to_string(),
        DefaultCollationValue::String(value) => value,
    };
    append_text(result, &value, inputs.request_id, control)
}

fn append_conditional_path(
    inputs: &SequenceInputs<'_>,
    expression: &ConditionalPathExpression,
    context: Option<NodeId>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = evaluate_conditional_path(inputs, expression, context, control)?;
    append_text(result, &value, inputs.request_id, control)
}

fn evaluate_conditional_path(
    inputs: &SequenceInputs<'_>,
    expression: &ConditionalPathExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let left = evaluate_integer_path(inputs, &expression.condition.left, context, control)?;
    let right = evaluate_integer_path(inputs, &expression.condition.right, context, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let condition = match expression.condition.operator {
        IntegerComparisonOperator::Equal => left == right,
        IntegerComparisonOperator::GreaterThan => left > right,
    };
    evaluate_conditional_path_branch(
        inputs,
        if condition {
            &expression.when_true
        } else {
            &expression.when_false
        },
        context,
        control,
    )
}

fn evaluate_conditional_path_branch(
    inputs: &SequenceInputs<'_>,
    branch: &ConditionalPathBranch,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    match branch {
        ConditionalPathBranch::Path(path) => evaluate_path_string(inputs, path, context, control),
        ConditionalPathBranch::Division {
            numerator,
            denominator,
        } => {
            let numerator = evaluate_integer_path(inputs, numerator, context, control)?;
            let denominator = evaluate_integer_path(inputs, denominator, context, control)?;
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if denominator == 0 {
                return Err(failure(
                    "FOAR0001",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    "integer division by zero in selected conditional branch",
                ));
            }
            if numerator % denominator != 0 {
                return Err(failure(
                    "FXRT1007",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    "a selected non-integral division branch exceeds the admitted conditional slice",
                ));
            }
            Ok((numerator / denominator).to_string())
        }
        ConditionalPathBranch::Conditional(expression) => {
            evaluate_conditional_path(inputs, expression, context, control)
        }
    }
}

fn evaluate_integer_path(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<i64, ExecutionFailure> {
    let value = evaluate_path_string(inputs, path, context, control)?;
    value.trim().parse::<i64>().map_err(|_| {
        failure(
            "FORG0001",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            format!("value is not a valid xs:integer lexical: {value}"),
        )
    })
}

fn evaluate_path_string(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let node = selected.first().ok_or_else(|| {
        failure(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "empty sequence cannot supply the required singleton value",
        )
    })?;
    source
        .string_value_controlled(*node, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))
}

pub(super) fn evaluate_conditional_integer(
    inputs: &SequenceInputs<'_>,
    expression: &ConditionalIntegerExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<i64, ExecutionFailure> {
    let condition = match &expression.condition {
        ConditionalIntegerCondition::Constant(value) => *value,
        ConditionalIntegerCondition::Contains { path, needle } => {
            let (source, context) = required_source_context(inputs, context)?;
            let selected = evaluate_location_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let value = if let Some(node) = selected.first() {
                source
                    .string_value_controlled(*node, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?
            } else {
                String::new()
            };
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            value.contains(needle)
        }
    };
    evaluate_conditional_integer_branch(
        inputs,
        if condition {
            &expression.when_true
        } else {
            &expression.when_false
        },
        context,
        control,
    )
}

fn evaluate_conditional_integer_branch(
    inputs: &SequenceInputs<'_>,
    branch: &ConditionalIntegerBranch,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<i64, ExecutionFailure> {
    match branch {
        ConditionalIntegerBranch::Integer(value) => Ok(*value),
        ConditionalIntegerBranch::Conditional(expression) => {
            evaluate_conditional_integer(inputs, expression, context, control)
        }
    }
}

fn append_location_path_count(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    append_text(
        result,
        &selected.len().to_string(),
        inputs.request_id,
        control,
    )
}

fn append_literal_variable_concat(
    inputs: &SequenceInputs<'_>,
    literal: &str,
    variable: &str,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    append_text(result, literal, inputs.request_id, control)?;
    let value = variables.atomics.get(variable).ok_or_else(|| {
        failure(
            "FXRT0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            format!("unbound atomic variable: ${variable}"),
        )
    })?;
    append_text(result, value.lexical(), inputs.request_id, control)
}

fn append_format_number(
    inputs: &SequenceInputs<'_>,
    expression: &crate::xpath::format_number_experiment::FormatNumberExpression,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let mut atomic_values = variables.atomics.as_ref().clone();
    for name in expression.variable_names() {
        if !atomic_values.contains_key(name) {
            if let Some(tree) = variables.temporary_tree(inputs.globals, name) {
                let value =
                    runtime_context::temporary_tree_string_value(tree, inputs.request_id, control)?;
                atomic_values.insert(name.to_owned(), AtomicValue::untyped(value));
            }
        }
    }
    let formatted =
        evaluate_format_number(expression, &atomic_values).map_err(|error| match error {
            FormatNumberEvaluationFailure::UnboundVariable(name) => failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound variable: ${name}"),
            ),
            FormatNumberEvaluationFailure::Unsupported => failure(
                "FXRT1007",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                "dynamic value or picture exceeds the admitted format-number slice",
            ),
        })?;
    append_text(result, &formatted, inputs.request_id, control)
}

fn append_generated_document_root_identity(
    inputs: &SequenceInputs<'_>,
    reference: &crate::xslt::golden_semantics_experiment::DocumentRootReference,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let identity = super::dynamic_document::document_root_identity(inputs, reference, control)?;
    append_text(result, &identity, inputs.request_id, control)
}

fn append_integer_for(
    inputs: &SequenceInputs<'_>,
    expression: &crate::xpath::integer_for_experiment::IntegerForExpression,
    separator: &str,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let values = evaluate_integer_for(expression, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            append_text(result, separator, inputs.request_id, control)?;
        }
        append_text(result, &value.to_string(), inputs.request_id, control)?;
    }
    Ok(())
}

fn append_generated_temporary_root_identity(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    descendant_local: Option<&str>,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let tree = variables
        .temporary_tree(inputs.globals, variable)
        .ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound temporary tree: ${variable}"),
            )
        })?;
    let Some(identity) = runtime_context::temporary_document_identity(
        tree,
        descendant_local,
        inputs.request_id,
        control,
    )?
    else {
        return Ok(());
    };
    append_text(
        result,
        &format!("fastxslt-temporary-d{identity}"),
        inputs.request_id,
        control,
    )
}

fn append_location_path_string(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    if selected.len() > 1 {
        return Err(failure(
            "FXRT1001",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "the private value-of slice does not define multi-node conversion",
        ));
    }
    if let Some(node) = selected.first() {
        append_source_string_value(inputs, *node, result, control)?;
    }
    Ok(())
}

fn append_root_path_string(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    append_root_nodes_string(inputs, &selected, result, control)
}

fn append_root_variable_string(
    inputs: &SequenceInputs<'_>,
    name: &str,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let nodes = variables
        .source_nodes(inputs.globals, name)
        .ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound source-node variable: ${name}"),
            )
        })?;
    append_root_nodes_string(inputs, nodes, result, control)
}

fn append_root_nodes_string(
    inputs: &SequenceInputs<'_>,
    selected: &[NodeId],
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let Some(root) = root_node(inputs, selected, control)? else {
        return Ok(());
    };
    append_source_string_value(inputs, root, result, control)
}

fn append_generated_root_identity(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(root) = root_node(inputs, &selected, control)? else {
        return Ok(());
    };
    append_text(
        result,
        &super::runtime_context::source_node_identity(root),
        inputs.request_id,
        control,
    )
}

fn root_node(
    inputs: &SequenceInputs<'_>,
    selected: &[NodeId],
    control: &mut InvocationControl,
) -> Result<Option<NodeId>, ExecutionFailure> {
    if selected.len() > 1 {
        return Err(failure(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "root() requires a zero-or-one node argument",
        ));
    }
    let source = inputs.source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "a source-node variable requires its principal source",
        )
    })?;
    let Some(mut root) = selected.first().copied() else {
        return Ok(None);
    };
    loop {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let Some(parent) = source.parent(root) else {
            break;
        };
        root = parent;
    }
    Ok(Some(root))
}

fn append_upper_case_context_string(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    source
        .visit_string_value_controlled(context, control, &mut |part, control| {
            let mut upper = String::with_capacity(part.len());
            for character in part.chars() {
                control
                    .charge(WorkDomain::XPathOperation, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                upper.extend(character.to_uppercase());
            }
            append_text(result, &upper, inputs.request_id, control)
        })
        .map_err(|failure| match failure {
            StringValueVisitFailure::Control(failure) => {
                control_failure(failure, inputs.request_id)
            }
            StringValueVisitFailure::Sink(failure) => failure,
        })
}

fn append_context_node_name(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(name) = source.name(context) else {
        return Ok(());
    };
    if name.namespace.is_some() {
        return Err(failure(
            "FXRT1008",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "name(.) for namespaced nodes is outside the prefix-preserving private slice",
        ));
    }
    append_text(result, &name.local, inputs.request_id, control)
}

fn execute_deep_equal(
    inputs: &SequenceInputs<'_>,
    expression: &crate::xpath::deep_equal_boolean_experiment::DeepEqualBooleanExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, _) = required_source_context(inputs, context)?;
    evaluate_deep_equal(expression, Some(source), control).map_err(|evaluation_failure| {
        match evaluation_failure {
            DeepEqualEvaluationFailure::Control(control) => {
                control_failure(control, inputs.request_id)
            }
            DeepEqualEvaluationFailure::MissingNodeContext => failure(
                "FXRT1004",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                "node deep-equal requires a principal source",
            ),
        }
    })
}

fn append_variable_value(
    inputs: &SequenceInputs<'_>,
    name: &str,
    separator: &str,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if let Some(value) = variables.atomics.get(name) {
        return append_text(result, value.lexical(), inputs.request_id, control);
    }
    if let Some(values) = variables.atomic_sequences.get(name) {
        for (index, value) in values.iter().enumerate() {
            if index > 0 {
                append_text(result, separator, inputs.request_id, control)?;
            }
            append_text(result, value.lexical(), inputs.request_id, control)?;
        }
        return Ok(());
    }
    if let Some(nodes) = variables.source_nodes(inputs.globals, name) {
        for (index, node) in nodes.iter().enumerate() {
            if index > 0 {
                append_text(result, separator, inputs.request_id, control)?;
            }
            append_source_string_value(inputs, *node, result, control)?;
        }
        return Ok(());
    }
    if let Some(tree) = variables.temporary_tree(inputs.globals, name) {
        let value = runtime_context::temporary_tree_string_value(tree, inputs.request_id, control)?;
        return append_text(result, &value, inputs.request_id, control);
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        format!("unbound variable: ${name}"),
    ))
}

fn append_source_string_value(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "a source-derived value requires its principal source",
        )
    })?;
    source
        .visit_string_value_controlled(node, control, &mut |part, control| {
            append_text(result, part, inputs.request_id, control)
        })
        .map_err(|failure| match failure {
            StringValueVisitFailure::Control(failure) => {
                control_failure(failure, inputs.request_id)
            }
            StringValueVisitFailure::Sink(failure) => failure,
        })
}

fn execute_decimal_sum(
    inputs: &SequenceInputs<'_>,
    expression: &crate::xpath::decimal_sum_for_experiment::DecimalSumForExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    evaluate_decimal_sum_for(expression, source, context, control).map_err(|evaluation_failure| {
        match evaluation_failure {
            DecimalSumEvaluationFailure::Control(control) => {
                control_failure(control, inputs.request_id)
            }
            DecimalSumEvaluationFailure::InvalidValue => failure(
                "FXRT0005",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                "an exact-decimal operand has an invalid lexical value",
            ),
            DecimalSumEvaluationFailure::Unsupported => failure(
                "FXRT1006",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                "decimal overflow or rounding is outside the private exact-decimal sum slice",
            ),
        }
    })
}

fn execute_castable_expression(
    inputs: &SequenceInputs<'_>,
    expression: &CastableExpression,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if let Some(name) = castable_variable_name(expression) {
        let value = variables.atomics.get(name).ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound variable: ${name}"),
            )
        })?;
        return evaluate_castable_value(expression, value, control)
            .map_err(|control| control_failure(control, inputs.request_id));
    }
    let (source, context) = required_source_context(inputs, context)?;
    evaluate_castable(expression, source, context, control)
        .map_err(|control| control_failure(control, inputs.request_id))
}
