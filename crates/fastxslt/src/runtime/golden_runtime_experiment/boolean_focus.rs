//! Qualified node focus carried through the shared boolean evaluator.

use super::{
    ExecutionFailure, FailureCategory, InvocationControl, NodeId, SequenceContext, SequenceInputs,
    SourceLocation, WorkDomain, control_failure, failure_at, required_source_context,
};
use crate::xdm::qualified_nodes::NamespaceOccurrence;
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub(super) struct BooleanNodeFocus<'a> {
    pub(super) node: Option<NodeId>,
    namespace: Option<NamespaceOccurrence<'a>>,
    other_focus: bool,
}

impl<'a> From<SequenceContext<'a>> for BooleanNodeFocus<'a> {
    fn from(execution: SequenceContext<'a>) -> Self {
        Self {
            node: execution.node,
            namespace: execution.namespace_focus,
            other_focus: execution.temporary_focus.is_some() || execution.atomic_focus.is_some(),
        }
    }
}

impl From<Option<NodeId>> for BooleanNodeFocus<'_> {
    fn from(node: Option<NodeId>) -> Self {
        Self {
            node,
            namespace: None,
            other_focus: false,
        }
    }
}

pub(super) fn contains(
    inputs: &SequenceInputs<'_>,
    focus: BooleanNodeFocus<'_>,
    sought: &str,
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if focus.other_focus {
        return Err(failure_at(
            "FXRT0007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            location.clone(),
            "contains(.) on temporary or atomic focus is not yet admitted",
        ));
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let value = if let Some(namespace) = focus.namespace {
        control
            .charge(WorkDomain::XdmStringValueNode, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        Cow::Borrowed(namespace.string_value())
    } else {
        if focus.node.is_none() {
            return Err(failure_at(
                "XPDY0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                location.clone(),
                "contains(.) requires a node focus",
            ));
        }
        let (source, node) = required_source_context(inputs, focus.node)?;
        Cow::Owned(
            source
                .string_value_controlled(node, control)
                .map_err(|error| control_failure(error, inputs.request_id))?,
        )
    };
    // Abstract scan work is independent of the standard library search strategy.
    control
        .charge(
            WorkDomain::XPathNodeVisit,
            value.len().saturating_add(sought.len()),
        )
        .map_err(|error| control_failure(error, inputs.request_id))?;
    Ok(value.contains(sought))
}
