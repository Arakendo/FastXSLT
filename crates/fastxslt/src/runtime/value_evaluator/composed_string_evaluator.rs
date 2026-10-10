//! Shared lexical evaluation; no result construction, authority or retained state.

use super::{
    ExecutionFailure, InvocationControl, NodeId, RuntimeVariables, SequenceInputs, WorkDomain,
    control_failure,
};
use crate::xslt::golden_semantics_experiment::{Xslt10ComposedString, Xslt10StringBound};

pub(super) fn evaluate(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10ComposedString,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    match expression {
        Xslt10ComposedString::Path(path) => {
            super::xslt10_compatibility::first_path_string(inputs, context, path, control)
        }
        Xslt10ComposedString::After { source, delimiter } => {
            let value = evaluate(inputs, context, source, variables, control)?;
            control
                .charge(WorkDomain::XPathOperation, value.len())
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(value
                .split_once(delimiter)
                .map_or("", |pair| pair.1)
                .to_owned())
        }
        Xslt10ComposedString::Substring {
            source,
            start,
            length,
        } => {
            let value = evaluate(inputs, context, source, variables, control)?;
            let start = bound(inputs, context, start, variables, control)?;
            let length = length
                .as_ref()
                .map(|value| bound(inputs, context, value, variables, control))
                .transpose()?;
            control
                .charge(WorkDomain::XPathOperation, value.len())
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(crate::xpath::static_string_experiment::evaluate_substring(
                &value, start, length,
            ))
        }
    }
}

fn bound(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &Xslt10StringBound,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<f64, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    match expression {
        Xslt10StringBound::Literal(bits) => Ok(f64::from_bits(*bits)),
        Xslt10StringBound::Length(source) => {
            let value = evaluate(inputs, context, source, variables, control)?;
            // Precharge UTF-8 bytes before scanning character boundaries.
            control
                .charge(WorkDomain::XPathOperation, value.len())
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            #[allow(clippy::cast_precision_loss)]
            Ok(value.chars().count() as f64)
        }
        Xslt10StringBound::Boolean(value) => {
            super::super::evaluate_boolean(inputs, value, context.into(), None, variables, control)
                .map(|value| if value { 1.0 } else { 0.0 })
        }
        Xslt10StringBound::Multiply(left, right) => {
            let left = bound(inputs, context, left, variables, control)?;
            let right = bound(inputs, context, right, variables, control)?;
            Ok(left * right)
        }
    }
}
