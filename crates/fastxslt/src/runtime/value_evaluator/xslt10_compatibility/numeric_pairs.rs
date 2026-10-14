//! Bounded numeric operands for `XPath` 1.0 variable-to-variable relations.

use super::{
    ExecutionFailure, InvocationControl, RuntimeVariables, SequenceInputs, WorkDomain,
    control_failure,
};
use crate::xslt::golden_semantics_experiment::FocusComparison;

enum Operand {
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
        control
            .charge(WorkDomain::XdmNode, nodes.len())
            .map_err(|error| control_failure(error, inputs.request_id))?;
        let mut values = Vec::with_capacity(nodes.len());
        for node in nodes {
            let value = source
                .string_value_controlled(*node, control)
                .map_err(|error| control_failure(error, inputs.request_id))?;
            values.push(number(&value));
        }
        return Ok(Operand::Nodes(values));
    }
    if let Some(value) = variables.atomics.get(name).or_else(|| {
        variables
            .allows_global_fallback(name)
            .then(|| inputs.globals.atomics.get(name))
            .flatten()
    }) {
        use super::BuiltinAtomicType;
        match value.atomic_type() {
            BuiltinAtomicType::Boolean => {
                return Ok(Operand::Boolean(f64::from(u8::from(matches!(
                    value.lexical(),
                    "true" | "1"
                )))));
            }
            BuiltinAtomicType::Integer
            | BuiltinAtomicType::Decimal
            | BuiltinAtomicType::Float
            | BuiltinAtomicType::Double => {
                return Ok(Operand::Scalar(match value.lexical() {
                    "INF" | "Infinity" => f64::INFINITY,
                    "-INF" | "-Infinity" => f64::NEG_INFINITY,
                    text => text.parse().unwrap_or(f64::NAN),
                }));
            }
            _ => {}
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
