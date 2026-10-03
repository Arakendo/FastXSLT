//! Bounded attribute-axis projection from invocation-owned temporary focus.

use super::super::TemporaryFocus;
use super::super::runtime_context::TemporaryNodeKind;
use super::super::{
    ExecutionFailure, FailureCategory, InvocationControl, ResultNode, SequenceInputs, WorkDomain,
    append_text, control_failure, failure_at,
};
use crate::xpath::path_experiment::{LocationPath, PathStep};

pub(super) fn append(
    inputs: &SequenceInputs<'_>,
    path: &LocationPath,
    separator: &str,
    first_only: bool,
    focus: TemporaryFocus<'_>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let step = path.plain_relative_attribute_step().ok_or_else(|| {
        failure_at(
            "FXRT0007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            path.location.clone(),
            "temporary-focus value paths currently support only a plain relative attribute axis",
        )
    })?;
    let TemporaryFocus::Node(tree, node) = focus else {
        return Ok(()); // A document node has no attributes.
    };
    let TemporaryNodeKind::Element { attributes, .. } = &tree.nodes[node].kind else {
        return Ok(());
    };
    // Select completely before reading values, matching the reference path's
    // traversal-before-string-value failure ordering even for XSLT 1.0.
    let mut selected = Vec::new();
    for attribute in attributes {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        let TemporaryNodeKind::Attribute { name, .. } = &tree.nodes[*attribute].kind else {
            unreachable!("temporary element attribute indexes identify attributes");
        };
        let matches = match step {
            PathStep::AttributeNamed(required) => {
                name.namespace.is_none() && name.local == *required
            }
            PathStep::AttributeExpandedName(required) => name == required,
            PathStep::AttributeNamespace(required) => name.namespace.as_deref() == Some(required),
            PathStep::AttributeAny => true,
            _ => unreachable!("eligibility admits only attribute name tests"),
        };
        if matches {
            selected.push(*attribute);
        }
    }
    if first_only {
        selected.truncate(1);
    }
    for (index, attribute) in selected.into_iter().enumerate() {
        if index > 0 {
            append_text(result, separator, inputs.request_id, control)?;
        }
        control
            .charge(WorkDomain::XdmStringValueNode, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        let TemporaryNodeKind::Attribute { value, .. } = &tree.nodes[attribute].kind else {
            unreachable!("selected temporary attributes retain attribute kind");
        };
        append_text(result, value, inputs.request_id, control)?;
    }
    Ok(())
}
