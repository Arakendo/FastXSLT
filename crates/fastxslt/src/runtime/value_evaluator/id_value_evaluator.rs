//! ID lookup scalar consumers reuse the shared charged selector and value semantics.

use super::super::id_lookup;
use super::{
    ExecutionFailure, InvocationControl, ResultNode, RuntimeVariables, SequenceContext,
    SequenceInputs, ValueExpression, append_context_node_local_name, append_context_node_name,
    append_context_node_namespace_uri, append_source_string_value, append_text,
    required_source_context, xslt10_id_variable_value,
};

pub(super) fn execute(
    inputs: &SequenceInputs<'_>,
    select: &ValueExpression,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let context = execution.node;
    match select {
        ValueExpression::Xslt10IdLookup(lookup) => {
            let (source, context) = required_source_context(inputs, context)?;
            let variable_value = xslt10_id_variable_value(inputs, lookup, variables, control)?;
            if let Some(node) = id_lookup::select(
                source,
                context,
                lookup,
                variable_value.as_deref(),
                inputs.request_id,
                control,
            )?
            .first()
            {
                append_source_string_value(inputs, *node, result, control)?;
            }
        }
        ValueExpression::Xslt10CountIdLookup(lookup) => {
            let (source, context) = required_source_context(inputs, context)?;
            let variable_value = xslt10_id_variable_value(inputs, lookup, variables, control)?;
            let count = id_lookup::select(
                source,
                context,
                lookup,
                variable_value.as_deref(),
                inputs.request_id,
                control,
            )?
            .len();
            append_text(result, &count.to_string(), inputs.request_id, control)?;
        }
        ValueExpression::Xslt10NameIdLookup(lookup)
        | ValueExpression::Xslt10LocalNameIdLookup(lookup)
        | ValueExpression::Xslt10NamespaceUriIdLookup(lookup) => {
            let (source, context) = required_source_context(inputs, context)?;
            let variable_value = xslt10_id_variable_value(inputs, lookup, variables, control)?;
            if let Some(node) = id_lookup::select(
                source,
                context,
                lookup,
                variable_value.as_deref(),
                inputs.request_id,
                control,
            )?
            .first()
            {
                let execution = SequenceContext {
                    node: Some(*node),
                    ..execution
                };
                match select {
                    ValueExpression::Xslt10NameIdLookup(_) => {
                        append_context_node_name(inputs, execution, result, control)?;
                    }
                    ValueExpression::Xslt10LocalNameIdLookup(_) => {
                        append_context_node_local_name(inputs, execution, result, control)?;
                    }
                    ValueExpression::Xslt10NamespaceUriIdLookup(_) => {
                        append_context_node_namespace_uri(inputs, execution, result, control)?;
                    }
                    _ => unreachable!("typed ID node-name consumer"),
                }
            }
        }
        _ => unreachable!("typed ID scalar evaluator receives only ID scalar consumers"),
    }
    Ok(())
}
