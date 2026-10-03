//! Namespace sort keys adapt qualified focus to the shared stable comparator.

use super::super::{
    ExecutionFailure, InvocationControl, RuntimeVariables, SequenceInputs, SortKey, SortSelect,
    WorkDomain, control_failure, evaluate_sort_control, sort_keyed_items, typed_sort_key,
};
use crate::xdm::qualified_nodes::QualifiedSourceNode;

pub(super) fn sort<'a>(
    inputs: &SequenceInputs<'_>,
    selected: Vec<QualifiedSourceNode<'a>>,
    sorts: &[SortKey],
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<QualifiedSourceNode<'a>>, ExecutionFailure> {
    if sorts.is_empty() {
        return Ok(selected);
    }
    control
        .charge(WorkDomain::XPathOperation, sorts.len())
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let controls = sorts
        .iter()
        .map(|sort| evaluate_sort_control(inputs, sort, variables, control))
        .collect::<Result<Vec<_>, _>>()?;
    if selected.len() < 2 {
        return Ok(selected);
    }
    let size = selected.len();
    control
        .charge(
            WorkDomain::XPathOperation,
            size.saturating_mul(sorts.len().saturating_add(1)),
        )
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let mut keyed = Vec::with_capacity(size);
    for (offset, node) in selected.into_iter().enumerate() {
        let namespace = node.namespace().expect("terminal namespace selection");
        let mut values = Vec::with_capacity(sorts.len());
        for (sort, policy) in sorts.iter().zip(&controls) {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|error| control_failure(error, inputs.request_id))?;
            let value = match &sort.select {
                SortSelect::ContextNodeName | SortSelect::ContextNodeLocalName => {
                    namespace.prefix().to_owned()
                }
                SortSelect::ContextPosition => (offset + 1).to_string(),
                SortSelect::ContextSize => size.to_string(),
                SortSelect::Literal(value) => value.clone(),
                SortSelect::LocationPath(path) if path.is_bare_context_item() => {
                    control
                        .charge(WorkDomain::XdmStringValueNode, 1)
                        .map_err(|error| control_failure(error, inputs.request_id))?;
                    namespace.string_value().to_owned()
                }
                _ => unreachable!("compiler admits only qualified namespace scalar sort keys"),
            };
            values.push(typed_sort_key(
                policy.data_type,
                sort.xslt10_numeric_conversion,
                value,
            ));
        }
        keyed.push((node, values));
    }
    sort_keyed_items(inputs.request_id, keyed, &controls, control)
}
