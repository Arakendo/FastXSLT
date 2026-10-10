//! Private scalar evaluation of sealed literal-document lookups.
//! Outer focus is scalarized before secondary-document navigation.

use super::{
    ExecutionFailure, InvocationControl, NodeId, ResultNode, SequenceInputs, WorkDomain,
    append_text, control_failure, evaluate_location_path_controlled, required_source_context,
};

pub(super) fn document_string_value(
    inputs: &SequenceInputs<'_>,
    dynamic: &super::super::dynamic_document::DynamicDocument,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let effective = super::super::derive_effective_source(
        &inputs.program.source_whitespace,
        dynamic.document.as_ref(),
        super::super::WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let document = effective.as_ref().unwrap_or(dynamic.document.as_ref());
    document
        .string_value_controlled(document.document_node(), control)
        .map_err(|error| control_failure(error, inputs.request_id))
}

pub(super) fn append_xslt10_literal_document_path(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    reference: &crate::xslt::golden_semantics_experiment::DocumentRootReference,
    path: &crate::xpath::path_experiment::LocationPath,
    outer_attribute_filter: Option<
        &crate::xslt::golden_semantics_experiment::Xslt10OuterAttributeFilter,
    >,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    // Scalarize while the original source still owns the current() focus.
    // No principal NodeId enters the secondary document's evaluator.
    let mut outer_value = None;
    if let Some(filter) = outer_attribute_filter {
        let (source, context) = required_source_context(inputs, context)?;
        for attribute in source.attributes(context) {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if source.name(*attribute) == Some(&filter.outer) {
                outer_value = Some(
                    source
                        .string_value_controlled(*attribute, control)
                        .map_err(|failure| control_failure(failure, inputs.request_id))?,
                );
                break;
            }
        }
    }
    let dynamic = super::super::dynamic_document::prepare_document(inputs, reference, control)?;
    let effective_document = super::super::derive_effective_source(
        &inputs.program.source_whitespace,
        dynamic.document.as_ref(),
        super::super::WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let document = effective_document
        .as_ref()
        .unwrap_or(dynamic.document.as_ref());
    let selected =
        evaluate_location_path_controlled(document, document.document_node(), path, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut first = None;
    for node in selected {
        if let Some(filter) = outer_attribute_filter {
            for attribute in document.attributes(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if document.name(*attribute) == Some(&filter.candidate) {
                    let candidate = document
                        .string_value_controlled(*attribute, control)
                        .map_err(|failure| control_failure(failure, inputs.request_id))?;
                    if outer_value.as_deref() == Some(candidate.as_str()) {
                        first = Some(node);
                        break;
                    }
                }
            }
            if first.is_some() {
                break;
            }
        } else {
            first = Some(node);
            break;
        }
    }
    let Some(node) = first else {
        return Ok(());
    };
    let value = document
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    append_text(result, &value, inputs.request_id, control)
}
