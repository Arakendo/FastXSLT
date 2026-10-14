//! `XPath` 1.0 typed variable equality with independent existential comparisons.

use super::{
    AtomicValue, BuiltinAtomicType, ExecutionFailure, FailureCategory, InvocationControl,
    RuntimeVariables, SequenceInputs, WorkDomain, control_failure, failure,
};

enum Operand {
    Boolean(bool),
    Number(f64),
    String(String),
    Temporary(String),
    Nodes(Vec<String>),
}

impl Operand {
    fn boolean(&self) -> bool {
        match self {
            Self::Boolean(value) => *value,
            Self::Number(value) => *value != 0.0 && !value.is_nan(),
            Self::String(value) => !value.is_empty(),
            // An XSLT 1.0 result tree fragment has a root even with no text.
            Self::Temporary(_) => true,
            Self::Nodes(values) => !values.is_empty(),
        }
    }

    fn number(&self) -> f64 {
        match self {
            Self::Boolean(value) => f64::from(u8::from(*value)),
            Self::Number(value) => *value,
            Self::String(value) | Self::Temporary(value) => {
                crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(value)
                    .unwrap_or(f64::NAN)
            }
            Self::Nodes(_) => unreachable!("node-set numeric comparison is existential"),
        }
    }

    fn text(&self) -> &str {
        match self {
            Self::String(value) | Self::Temporary(value) => value,
            _ => unreachable!("boolean and numeric precedence is handled before string equality"),
        }
    }
}

pub(in super::super) fn compare(
    inputs: &SequenceInputs<'_>,
    operands: [(&str, bool); 2],
    equal: bool,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let left = operand(inputs, operands[0], variables, control)?;
    let right = operand(inputs, operands[1], variables, control)?;
    match (&left, &right) {
        (Operand::Nodes(left), Operand::Nodes(right)) => {
            for left in left {
                for right in right {
                    control
                        .charge(WorkDomain::XPathOperation, 1)
                        .map_err(|error| control_failure(error, inputs.request_id))?;
                    if (left == right) == equal {
                        return Ok(true);
                    }
                }
            }
            return Ok(false);
        }
        (Operand::Nodes(nodes), scalar) | (scalar, Operand::Nodes(nodes)) => {
            return compare_nodes_scalar(inputs, nodes, scalar, equal, control);
        }
        _ => {}
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let matches = if matches!(left, Operand::Boolean(_)) || matches!(right, Operand::Boolean(_)) {
        left.boolean() == right.boolean()
    } else if matches!(left, Operand::Number(_)) || matches!(right, Operand::Number(_)) {
        left.number().partial_cmp(&right.number()) == Some(std::cmp::Ordering::Equal)
    } else {
        left.text() == right.text()
    };
    Ok(matches == equal)
}

fn compare_nodes_scalar(
    inputs: &SequenceInputs<'_>,
    nodes: &[String],
    scalar: &Operand,
    equal: bool,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if let Operand::Boolean(value) = scalar {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        return Ok((nodes.is_empty() != *value) == equal);
    }
    for text in nodes {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        let matches = if let Operand::Number(number) = scalar {
            let node_number =
                crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(text)
                    .unwrap_or(f64::NAN);
            node_number.partial_cmp(number) == Some(std::cmp::Ordering::Equal)
        } else {
            text == scalar.text()
        };
        if matches == equal {
            return Ok(true);
        }
    }
    Ok(false)
}

fn source_strings(
    inputs: &SequenceInputs<'_>,
    name: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Option<Vec<String>>, ExecutionFailure> {
    #[cfg(test)]
    if variables.allows_global_fallback(name)
        && let Some(reader) = &inputs.globals.demand_source_reference
        && let Some((owner, nodes)) = reader.read(name, control)?
    {
        let mut frame = variables.clone();
        frame.bind_retained_source_nodes(name.to_owned(), owner, nodes);
        return source_strings(inputs, name, &frame, control);
    }
    if let Some(values) = variables.detached_source_node_strings(name) {
        control
            .charge(WorkDomain::XdmNode, values.len())
            .map_err(|error| control_failure(error, inputs.request_id))?;
        let mut retained = Vec::with_capacity(values.len());
        for value in values {
            control
                .charge(WorkDomain::ResultTextByte, value.len())
                .map_err(|error| control_failure(error, inputs.request_id))?;
            retained.push(value.clone());
        }
        return Ok(Some(retained));
    }
    let Some(nodes) = variables.source_nodes_in_context(inputs, name)? else {
        return Ok(None);
    };
    let source = super::required_comparison_source(inputs)?;
    control
        .charge(WorkDomain::XdmNode, nodes.len())
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let mut values = Vec::with_capacity(nodes.len());
    for node in nodes {
        let value = source
            .string_value_controlled(*node, control)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        control
            .charge(WorkDomain::ResultTextByte, value.len())
            .map_err(|error| control_failure(error, inputs.request_id))?;
        values.push(value);
    }
    Ok(Some(values))
}

fn operand(
    inputs: &SequenceInputs<'_>,
    (name, explicit_string): (&str, bool),
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Operand, ExecutionFailure> {
    if explicit_string {
        return super::variable_string_value(inputs, name, variables, control).map(Operand::String);
    }
    if let Some(values) = source_strings(inputs, name, variables, control)? {
        return Ok(Operand::Nodes(values));
    }
    if let Some(value) = variables.atomics.get(name).or_else(|| {
        variables
            .allows_global_fallback(name)
            .then(|| inputs.globals.atomics.get(name))
            .flatten()
    }) {
        return atomic(inputs, value);
    }
    // The existing guarded reader accepts temporary trees, including demand
    // reference values, but never scalarizes unresolved source selections.
    super::super::super::variable_string_value(inputs, name, variables, control)
        .map(Operand::Temporary)
}

fn atomic(inputs: &SequenceInputs<'_>, value: &AtomicValue) -> Result<Operand, ExecutionFailure> {
    Ok(match value.atomic_type() {
        BuiltinAtomicType::Boolean => Operand::Boolean(matches!(value.lexical(), "true" | "1")),
        BuiltinAtomicType::Integer
        | BuiltinAtomicType::Decimal
        | BuiltinAtomicType::Float
        | BuiltinAtomicType::Double => Operand::Number(match value.lexical() {
            "INF" | "Infinity" => f64::INFINITY,
            "-INF" | "-Infinity" => f64::NEG_INFINITY,
            text => text.parse().unwrap_or(f64::NAN),
        }),
        BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic => {
            Operand::String(value.lexical().to_owned())
        }
        _ => {
            return Err(failure(
                "FXRT1015",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                "extended atomic types are outside legacy scalar equality",
            ));
        }
    })
}
