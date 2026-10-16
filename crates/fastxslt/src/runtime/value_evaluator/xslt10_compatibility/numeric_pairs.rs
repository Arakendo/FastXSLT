//! Bounded numeric operands for `XPath` 1.0 variable-to-variable relations.

use super::{
    ExecutionFailure, InvocationControl, RuntimeVariables, SequenceInputs, WorkDomain,
    control_failure,
};
use crate::xslt::golden_semantics_experiment::FocusComparison;

pub(in crate::runtime) enum Operand {
    Scalar(f64),
    Boolean(f64),
    Nodes(Vec<f64>),
}

impl Operand {
    fn values(&self) -> &[f64] {
        match self {
            Self::Scalar(value) | Self::Boolean(value) => std::slice::from_ref(value),
            Self::Nodes(values) => values,
        }
    }
}

pub(in super::super) fn compare(
    inputs: &SequenceInputs<'_>,
    left: &str,
    right: &str,
    operator: FocusComparison,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let left = operand(inputs, left, variables, control)?;
    let right = operand(inputs, right, variables, control)?;
    // XPath 1.0 applies whole-set boolean conversion for every comparison
    // operator when the other operand is boolean, including relations.
    let boolean_pair = match (&left, &right) {
        (Operand::Nodes(nodes), Operand::Boolean(value)) => {
            Some((f64::from(u8::from(!nodes.is_empty())), *value))
        }
        (Operand::Boolean(value), Operand::Nodes(nodes)) => {
            Some((*value, f64::from(u8::from(!nodes.is_empty()))))
        }
        _ => None,
    };
    if let Some((left, right)) = boolean_pair {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        return Ok(super::super::super::compare_xpath_numbers(
            left, operator, right,
        ));
    }
    for left in left.values() {
        for right in right.values() {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|error| control_failure(error, inputs.request_id))?;
            if super::super::super::compare_xpath_numbers(*left, operator, *right) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn number(value: &str) -> f64 {
    crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(value).unwrap_or(f64::NAN)
}

fn operand(
    inputs: &SequenceInputs<'_>,
    name: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Operand, ExecutionFailure> {
    if variables.allows_global_fallback(name)
        && let Some(owner) = inputs.initializing_globals
    {
        return owner.read_numeric_pair_operand(name, control);
    }
    #[cfg(test)]
    if variables.allows_global_fallback(name)
        && let Some(reader) = &inputs.globals.demand_source_reference
        && let Some((owner, nodes)) = reader.read(name, control)?
    {
        let mut frame = variables.clone();
        frame.bind_retained_source_nodes(name.to_owned(), owner, nodes);
        return operand(inputs, name, &frame, control);
    }
    if let Some(values) = variables.detached_source_node_strings(name) {
        control
            .charge(WorkDomain::XdmNode, values.len())
            .map_err(|error| control_failure(error, inputs.request_id))?;
        return Ok(Operand::Nodes(
            values.iter().map(|value| number(value)).collect(),
        ));
    }
    if let Some(nodes) = variables.source_nodes_in_context(inputs, name)? {
        let source = super::required_comparison_source(inputs)?;
        return source_operand(source, nodes, inputs.request_id, control);
    }
    if let Some(value) = variables.atomics.get(name).or_else(|| {
        variables
            .allows_global_fallback(name)
            .then(|| inputs.globals.atomics.get(name))
            .flatten()
    }) {
        if let Some(value) = atomic_operand(value) {
            return Ok(value);
        }
    }
    let value = super::variable_string_value_for(
        inputs,
        name,
        variables,
        control,
        #[cfg(test)]
        None,
        #[cfg(test)]
        true,
    )?;
    Ok(Operand::Scalar(number(&value)))
}

fn atomic_operand(value: &super::AtomicValue) -> Option<Operand> {
    use super::BuiltinAtomicType;
    match value.atomic_type() {
        BuiltinAtomicType::Boolean => Some(Operand::Boolean(f64::from(u8::from(matches!(
            value.lexical(),
            "true" | "1"
        ))))),
        BuiltinAtomicType::Integer
        | BuiltinAtomicType::Decimal
        | BuiltinAtomicType::Float
        | BuiltinAtomicType::Double => Some(Operand::Scalar(match value.lexical() {
            "INF" | "Infinity" => f64::INFINITY,
            "-INF" | "-Infinity" => f64::NEG_INFINITY,
            text => text.parse().unwrap_or(f64::NAN),
        })),
        BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic => {
            Some(Operand::Scalar(number(value.lexical())))
        }
        _ => None,
    }
}

fn source_operand(
    source: &crate::xdm::owned_tree_experiment::Document,
    nodes: &[crate::xdm::owned_tree_experiment::NodeId],
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Operand, ExecutionFailure> {
    control
        .charge(WorkDomain::XdmNode, nodes.len())
        .map_err(|error| control_failure(error, request_id))?;
    let mut values = Vec::with_capacity(nodes.len());
    for node in nodes {
        let value = source
            .string_value_controlled(*node, control)
            .map_err(|error| control_failure(error, request_id))?;
        values.push(number(&value));
    }
    Ok(Operand::Nodes(values))
}

/// Owned operand preparation shared with the initialization resolver. No
/// completed-store borrow escapes into evaluation of the other operand.
pub(in crate::runtime) fn completed_operand(
    globals: &super::runtime_context::RuntimeGlobals,
    source: Option<&crate::xdm::owned_tree_experiment::Document>,
    name: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Operand, ExecutionFailure> {
    use super::{FailureCategory, failure};
    if let Some(nodes) = globals.nodes.get(name) {
        let source = source.ok_or_else(|| {
            failure(
                "FXRT1004",
                FailureCategory::Unsupported,
                Some(request_id),
                "global numeric relation requires a principal source",
            )
        })?;
        return source_operand(source, nodes, request_id, control);
    }
    if let Some(value) = globals.atomics.get(name) {
        return atomic_operand(value).ok_or_else(|| {
            failure(
                "FXRT1015",
                FailureCategory::Unsupported,
                Some(request_id),
                "extended atomic types are outside admitted constructor relations",
            )
        });
    }
    if let Some(tree) = globals.temporary_trees.get(name) {
        let value = super::runtime_context::temporary_tree_string_value(tree, request_id, control)?;
        return Ok(Operand::Scalar(number(&value)));
    }
    if globals.literal_documents.contains_key(name) || globals.empty_sequences.contains(name) {
        return Err(failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "this global value kind is outside admitted constructor relations",
        ));
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(request_id),
        format!("unbound global relational operand: ${name}"),
    ))
}
