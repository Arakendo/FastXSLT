//! Shared for-each dispatch and focus construction; execution stays in the parent evaluator.

use std::cell::RefCell;

#[cfg(test)]
mod tests;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xslt::golden_semantics_experiment::{ApplySelection, Instruction, SortKey};

use super::{
    ExecutionFailure, FailureCategory, LiteralDocumentTarget, ResultNode, RuntimeVariables,
    SequenceContext, SequenceInputs, TemporaryFocus, WhitespaceRepresentation, control_failure,
    derive_effective_source, dynamic_document, execute_sequence, failure, failure_at,
    literal_document_target, namespace_focus, required_source_context, select_apply_nodes,
    select_literal_document_nodes, select_source_variable_path, sort_selected_nodes,
    temporary_tree_executor, validate_literal_document_context,
};

fn execute_for_each_variable<'a>(
    inputs: &SequenceInputs<'a>,
    variable: &str,
    sorts: &[SortKey],
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some(tree) = variables.temporary_tree(inputs.globals, variable) {
        return execute_sequence(
            inputs,
            body,
            SequenceContext {
                node: None,
                temporary_focus: Some(TemporaryFocus::Document(tree)),
                namespace_focus: None,
                ..execution
            },
            variables,
            control,
        );
    }
    if let Some(nodes) = variables.source_nodes(inputs.globals, variable) {
        let nodes = sort_selected_nodes(inputs, nodes.clone(), sorts, variables, control)?;
        let focus_size = nodes.len();
        let mut result = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
            result.extend(execute_sequence(
                inputs,
                body,
                SequenceContext {
                    node: Some(node),
                    temporary_focus: None,
                    namespace_focus: None,
                    atomic_focus: None,
                    focus_position: index + 1,
                    focus_size,
                    ..execution
                },
                variables,
                control,
            )?);
        }
        return Ok(result);
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        format!("unbound or unsupported sequence variable: ${variable}"),
    ))
}

pub(super) fn execute<'a>(
    inputs: &SequenceInputs<'a>,
    instruction: &Instruction,
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match instruction {
        Instruction::ForEachVariable {
            variable,
            sorts,
            body,
            ..
        } => {
            execute_for_each_variable(inputs, variable, sorts, body, execution, variables, control)
        }
        Instruction::ForEachStaticIntegerRange {
            start, end, body, ..
        } => execute_for_each_static_integer_range(
            inputs, *start, *end, body, execution, variables, control,
        ),
        Instruction::ForEachNodes {
            select,
            sorts,
            body,
            ..
        } => execute_for_each_nodes(inputs, select, sorts, body, execution, variables, control),
        _ => unreachable!("for-each dispatch receives only for-each instructions"),
    }
}

fn execute_for_each_static_integer_range<'a>(
    inputs: &SequenceInputs<'a>,
    start: i64,
    end: i64,
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if start > end {
        return Ok(Vec::new());
    }
    let span = end
        .checked_sub(start)
        .and_then(|value| value.checked_add(1));
    let focus_size = span
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            failure(
                "FXRT0007",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                "integer range cannot be represented by this host",
            )
        })?;
    let mut result = Vec::new();
    for index in 0..focus_size {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        result.extend(execute_sequence(
            inputs,
            body,
            SequenceContext {
                node: None,
                temporary_focus: None,
                namespace_focus: None,
                atomic_focus: None,
                focus_position: index + 1,
                focus_size,
                ..execution
            },
            variables,
            control,
        )?);
    }
    Ok(result)
}

fn execute_for_each_nodes<'a>(
    inputs: &SequenceInputs<'a>,
    select: &ApplySelection,
    sorts: &[SortKey],
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let ApplySelection::QualifiedPath(path) = select {
        return namespace_focus::execute(inputs, path, sorts, body, execution, variables, control);
    }
    if let Some(target) = literal_document_target(Some(select)) {
        return execute_for_each_literal_document_root(
            inputs, target, sorts, body, execution, variables, control,
        );
    }
    if let ApplySelection::TemporaryPath { variable, steps } = select
        && variables.source_nodes(inputs.globals, variable).is_none()
    {
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
        if let Some(sort) = sorts.first() {
            return Err(failure_at(
                "FXRT0007",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                sort.location.clone(),
                "sorting a temporary-tree for-each path is not supported",
            ));
        }
        let selected = temporary_tree_executor::select_temporary_path(
            tree,
            steps,
            inputs.request_id,
            control,
        )?;
        let focus_size = selected.len();
        let mut result = Vec::new();
        for (index, node) in selected.into_iter().enumerate() {
            result.extend(execute_sequence(
                inputs,
                body,
                SequenceContext {
                    node: None,
                    temporary_focus: Some(TemporaryFocus::Node(tree, node)),
                    namespace_focus: None,
                    atomic_focus: None,
                    focus_position: index + 1,
                    focus_size,
                    ..execution
                },
                variables,
                control,
            )?);
        }
        return Ok(result);
    }
    let (_, context) = required_source_context(inputs, execution.node)?;
    let selected = if let ApplySelection::TemporaryPath { variable, steps } = select
        && let Some(selected) =
            select_source_variable_path(inputs, variables, variable, steps, control)
    {
        selected?
    } else {
        select_apply_nodes(inputs, Some(select), context, variables, control)?
    };
    let selected = sort_selected_nodes(inputs, selected, sorts, variables, control)?;
    let focus_size = selected.len();
    let mut result = Vec::new();
    for (index, node) in selected.into_iter().enumerate() {
        result.extend(execute_sequence(
            inputs,
            body,
            SequenceContext {
                node: Some(node),
                temporary_focus: None,
                namespace_focus: None,
                atomic_focus: None,
                focus_position: index + 1,
                focus_size,
                ..execution
            },
            variables,
            control,
        )?);
    }
    Ok(result)
}

fn execute_for_each_literal_document_root<'a>(
    inputs: &SequenceInputs<'a>,
    target: LiteralDocumentTarget<'_>,
    sorts: &[SortKey],
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let principal_source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "document-root execution requires a principal source context",
        )
    })?;
    let detached_variables = variables.detach_source_nodes_for_document_context(
        principal_source,
        inputs.request_id,
        control,
    )?;
    validate_literal_document_context(inputs, detached_variables.has_source_node_values())?;
    let dynamic = dynamic_document::prepare_document(inputs, target.reference(), control)?;
    let effective_document = derive_effective_source(
        &inputs.program.source_whitespace,
        dynamic.document.as_ref(),
        WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let document = effective_document
        .as_ref()
        .unwrap_or(dynamic.document.as_ref());
    let external_inputs = SequenceInputs {
        program: inputs.program,
        source: Some(document),
        request_id: inputs.request_id,
        globals: inputs.globals,
        multiple_match_policy: inputs.multiple_match_policy,
        document_rooted_matches: RefCell::default(),
        key_indexes: RefCell::default(),
        complete_atomic_frame_clones: inputs.complete_atomic_frame_clones,
        resource_snapshot: inputs.resource_snapshot,
        denied_resources: inputs.denied_resources,
        dynamic_documents: inputs.dynamic_documents,
    };
    let selected = select_literal_document_nodes(
        inputs,
        &external_inputs,
        document,
        target,
        variables,
        control,
    )?;
    let selected = sort_selected_nodes(
        &external_inputs,
        selected,
        sorts,
        &detached_variables,
        control,
    )?;
    let focus_size = selected.len();
    let mut result = Vec::new();
    for (index, node) in selected.into_iter().enumerate() {
        result.extend(execute_sequence(
            &external_inputs,
            body,
            SequenceContext {
                node: Some(node),
                temporary_focus: None,
                namespace_focus: None,
                atomic_focus: None,
                focus_position: index + 1,
                focus_size,
                ..execution
            },
            &detached_variables,
            control,
        )?);
    }
    Ok(result)
}
