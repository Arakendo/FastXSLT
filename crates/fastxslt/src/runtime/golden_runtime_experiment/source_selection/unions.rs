//! Single-effective-source key/path unions; foreign branches remain guarded.

use super::{
    ExecutionFailure, InvocationControl, NodeId, RuntimeVariables, SequenceInputs, control_failure,
    document_variable_dispatch, evaluate_location_path_controlled, id_lookup, key_lookup,
    source_variable_nodes, value_evaluator,
};
use crate::xslt::golden_semantics_experiment::{Xslt10ApplyUnionPart, Xslt10KeyLookup};

pub(super) fn select_keys(
    inputs: &SequenceInputs<'_>,
    lookups: &[Xslt10KeyLookup],
    context: NodeId,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs.source.expect("key selection requires a source");
    let mut selected = Vec::new();
    for lookup in lookups {
        selected.extend(key_lookup::select(
            inputs,
            lookup,
            Some(context),
            variables,
            control,
        )?);
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    Ok(selected)
}

pub(super) fn select_mixed(
    inputs: &SequenceInputs<'_>,
    alternatives: &[Xslt10ApplyUnionPart],
    context: NodeId,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("mixed union selection requires a source");
    let mut selected = Vec::new();
    for alternative in alternatives {
        match alternative {
            Xslt10ApplyUnionPart::Path(path) => selected.extend(
                evaluate_location_path_controlled(source, context, path, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?,
            ),
            Xslt10ApplyUnionPart::Id(lookup) => {
                let variable_value =
                    if let crate::xslt::golden_semantics_experiment::Xslt10IdArgument::Variable(
                        name,
                    ) = &lookup.argument
                    {
                        Some(value_evaluator::xslt10_variable_string_value(
                            inputs, name, variables, control,
                        )?)
                    } else {
                        None
                    };
                selected.extend(id_lookup::select(
                    source,
                    context,
                    lookup,
                    variable_value.as_deref(),
                    inputs.request_id,
                    control,
                )?);
            }
            Xslt10ApplyUnionPart::Key(lookup) => selected.extend(key_lookup::select(
                inputs,
                lookup,
                Some(context),
                variables,
                control,
            )?),
            Xslt10ApplyUnionPart::Variable(name) => {
                selected.extend(source_variable_nodes(inputs, name, variables)?);
            }
            Xslt10ApplyUnionPart::LiteralDocumentPath { reference, path } => {
                document_variable_dispatch::require_empty_union_branch(
                    inputs, reference, path, control,
                )?;
            }
        }
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    Ok(selected)
}
