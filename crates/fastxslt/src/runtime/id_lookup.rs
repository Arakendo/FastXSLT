//! Shared charged XSLT 1.0 typed-ID selection.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{Xslt10IdArgument, Xslt10IdLookup};

use super::runtime_failure::{ExecutionFailure, control_failure};

pub(super) fn select(
    source: &Document,
    context: NodeId,
    lookup: &Xslt10IdLookup,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut values = Vec::new();
    match &lookup.argument {
        Xslt10IdArgument::Literal(value) => values.push(value.clone()),
        Xslt10IdArgument::Path(argument_path) => {
            for node in evaluate_location_path_controlled(source, context, argument_path, control)
                .map_err(|failure| control_failure(failure, request_id))?
            {
                values.push(
                    source
                        .string_value_controlled(node, control)
                        .map_err(|failure| control_failure(failure, request_id))?,
                );
            }
        }
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut selected = Vec::new();
    for value in values {
        for token in value
            .split([' ', '\t', '\r', '\n'])
            .filter(|part| !part.is_empty())
        {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            selected.extend_from_slice(source.elements_with_id(token));
        }
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();

    let Some(relative_path) = &lookup.relative_path else {
        return Ok(selected);
    };
    let mut relative = Vec::new();
    for node in selected {
        relative.extend(
            evaluate_location_path_controlled(source, node, relative_path, control)
                .map_err(|failure| control_failure(failure, request_id))?,
        );
    }
    relative.sort_unstable_by_key(|node| source.document_order(*node));
    relative.dedup();
    Ok(relative)
}
