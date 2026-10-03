//! Qualified selection adapts focus; the existing sequence executor owns semantics.

use super::{
    ExecutionFailure, FailureCategory, Instruction, InvocationControl, ResultNode,
    RuntimeVariables, SequenceContext, SequenceInputs, WorkDomain, append_text, control_failure,
    execute_sequence, failure, failure_at, required_source_context,
};
use crate::xdm::qualified_nodes::{NamespaceFailure, QualifiedSourceNode};
use crate::xpath::path_experiment::qualified_nodes::{
    self, QualifiedLocationPath, QualifiedPathFailure,
};
use crate::xslt::golden_semantics_experiment::{NamespaceScalarKind, ValueExpression};

#[path = "namespace_sort.rs"]
mod namespace_sort;

#[cfg(test)]
#[path = "namespace_focus_tests.rs"]
mod tests;

pub(super) fn execute<'a>(
    inputs: &SequenceInputs<'a>,
    path: &QualifiedLocationPath,
    sorts: &[super::SortKey],
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let selected = select(inputs, path, execution, control)?;
    let selected = namespace_sort::sort(inputs, selected, sorts, variables, control)?;
    let focus_size = selected.len();
    let mut result = Vec::new();
    for (index, selected) in selected.into_iter().enumerate() {
        let (node, namespace_focus) = match selected {
            QualifiedSourceNode::Tree { id, .. } => (Some(id), None),
            QualifiedSourceNode::Namespace(namespace) => (None, Some(namespace)),
        };
        result.extend(execute_sequence(
            inputs,
            body,
            SequenceContext {
                node,
                namespace_focus,
                temporary_focus: None,
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

pub(super) fn generated_identity_equal<'a>(
    inputs: &SequenceInputs<'a>,
    left: &QualifiedLocationPath,
    right: &QualifiedLocationPath,
    first_node: bool,
    execution: SequenceContext<'a>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let mut operand = |path: &QualifiedLocationPath| {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        let selected = select(inputs, path, execution, control)?;
        if !first_node && selected.len() > 1 {
            return Err(failure_at(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                path.location().clone(),
                "generate-id requires zero or one qualified node",
            ));
        }
        Ok(selected.first().copied())
    };
    let left = operand(left)?;
    let right = operand(right)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .and_then(|()| control.charge(WorkDomain::XPathNodeVisit, 1))
        .map_err(|error| control_failure(error, inputs.request_id))?;
    Ok(match (left, right) {
        (Some(left), Some(right)) => left.same_node(right),
        (None, None) => true, // generate-id(()) is the empty string on both sides.
        _ => false,
    })
}

fn select<'a>(
    inputs: &SequenceInputs<'a>,
    path: &QualifiedLocationPath,
    execution: SequenceContext<'a>,
    control: &mut InvocationControl,
) -> Result<Vec<QualifiedSourceNode<'a>>, ExecutionFailure> {
    if execution.temporary_focus.is_some()
        || execution.atomic_focus.is_some()
        || execution.namespace_focus.is_some()
    {
        return Err(failure_at(
            "FXRT0007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            path.location().clone(),
            "this qualified namespace owner context is not yet admitted",
        ));
    }
    let (source, context) = required_source_context(inputs, execution.node)?;
    let selected =
        qualified_nodes::evaluate(source, context, path, usize::MAX, usize::MAX, control).map_err(
            |error| match error {
                QualifiedPathFailure::Control(error)
                | QualifiedPathFailure::Namespace(NamespaceFailure::Control(error)) => {
                    control_failure(error, inputs.request_id)
                }
                QualifiedPathFailure::Namespace(NamespaceFailure::BindingCapacity)
                | QualifiedPathFailure::SequenceCapacity => failure(
                    "FXCT0002",
                    FailureCategory::Limit,
                    Some(inputs.request_id),
                    "qualified namespace selection capacity exhausted",
                ),
            },
        )?;
    Ok(selected)
}

pub(super) fn count<'a>(
    inputs: &SequenceInputs<'a>,
    path: &QualifiedLocationPath,
    execution: SequenceContext<'a>,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|error| control_failure(error, inputs.request_id))?;
    select(inputs, path, execution, control).map(|selected| selected.len())
}

pub(super) fn execute_scalar<'a>(
    inputs: &SequenceInputs<'a>,
    expression: &ValueExpression,
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let ValueExpression::NamespacePathScalar {
        path,
        kind,
        first_node,
    } = expression
    else {
        unreachable!("qualified scalar adapter receives a typed namespace scalar");
    };
    if matches!(kind, NamespaceScalarKind::Count) {
        return append_text(
            result,
            &count(inputs, path, execution, control)?.to_string(),
            inputs.request_id,
            control,
        );
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let selected = select(inputs, path, execution, control)?;
    if !first_node && selected.len() > 1 {
        return Err(failure_at(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            path.location().clone(),
            "node-name functions require zero or one namespace node",
        ));
    }
    let Some(node) = selected.first() else {
        return Ok(());
    };
    let scalar = match kind {
        NamespaceScalarKind::Name => ValueExpression::ContextNodeName,
        NamespaceScalarKind::LocalName => ValueExpression::ContextNodeLocalName,
        NamespaceScalarKind::NamespaceUri => ValueExpression::ContextNodeNamespaceUri,
        NamespaceScalarKind::Count => unreachable!("count handled before scalar node selection"),
    };
    super::execute_value_of(
        inputs,
        &scalar,
        " ",
        SequenceContext {
            node: None,
            namespace_focus: node.namespace(),
            temporary_focus: None,
            atomic_focus: None,
            ..execution
        },
        variables,
        result,
        control,
    )
}

pub(super) fn parent_context<'a>(
    inputs: &SequenceInputs<'a>,
    path: &super::LocationPath,
    execution: SequenceContext<'a>,
    control: &mut InvocationControl,
) -> Result<SequenceContext<'a>, ExecutionFailure> {
    let namespace = execution
        .namespace_focus
        .expect("parent scalar requires namespace focus");
    let parent =
        qualified_nodes::parent_controlled(QualifiedSourceNode::Namespace(namespace), control)
            .map_err(|error| control_failure(error, inputs.request_id))?;
    let Some(QualifiedSourceNode::Tree { document, id }) = parent else {
        unreachable!("a source namespace occurrence always has its owning element as parent");
    };
    if !inputs
        .source
        .is_some_and(|source| std::ptr::eq(source, document))
    {
        return Err(failure_at(
            "FXRT0007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            path.location.clone(),
            "namespace parent handoff across effective source owners is not admitted",
        ));
    }
    Ok(SequenceContext {
        node: Some(id),
        namespace_focus: None,
        temporary_focus: None,
        atomic_focus: None,
        ..execution
    })
}
