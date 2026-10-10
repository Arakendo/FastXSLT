//! Dispatch a bound sealed document through the ordinary template executor.
//! Retained principal selections use the same executor; other consumers stay guarded.

use super::{
    Arc, BTreeMap, Document, ExecutionFailure, InvocationControl, InvocationParameter,
    InvocationParameterValue, NodeId, ResultNode, SequenceInputs, WhitespaceRepresentation,
    apply_template_at, derive_effective_source, dynamic_document, external_document_inputs,
    validate_literal_document_context,
};

pub(super) fn require_empty_union_branch(
    inputs: &SequenceInputs<'_>,
    reference: &crate::xslt::golden_semantics_experiment::DocumentRootReference,
    path: &crate::xpath::path_experiment::LocationPath,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let dynamic = dynamic_document::prepare_document(inputs, reference, control)?;
    let effective = derive_effective_source(
        &inputs.program.source_whitespace,
        &dynamic.document,
        WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let source = effective.as_ref().unwrap_or(&dynamic.document);
    let nodes =
        super::evaluate_location_path_controlled(source, source.document_node(), path, control)
            .map_err(|failure| super::control_failure(failure, inputs.request_id))?;
    if !nodes.is_empty() {
        return Err(super::failure(
            "FXRT1017",
            super::FailureCategory::Unsupported,
            Some(inputs.request_id),
            "nonempty document union branches require additional owner-qualified union support",
        ));
    }
    Ok(())
}

pub(super) fn apply_retained_source(
    inputs: &SequenceInputs<'_>,
    owner: &Arc<Document>,
    nodes: &[NodeId],
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if inputs
        .source
        .is_some_and(|source| source.same_effective_storage(owner))
    {
        return apply_retained_nodes(inputs, nodes, mode, parameters, control);
    }
    let source_parameters = parameters.values().any(|parameter| {
        matches!(&parameter.value,
            InvocationParameterValue::SourceNodes(nodes) if !nodes.is_empty())
    });
    validate_literal_document_context(inputs, source_parameters)?;
    let effective_inputs = external_document_inputs(inputs, owner);
    apply_retained_nodes(&effective_inputs, nodes, mode, parameters, control)
}

fn apply_retained_nodes(
    inputs: &SequenceInputs<'_>,
    nodes: &[NodeId],
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let mut result = Vec::new();
    for (offset, node) in nodes.iter().copied().enumerate() {
        result.extend(apply_template_at(
            inputs,
            node,
            mode,
            parameters,
            offset + 1,
            nodes.len(),
            control,
        )?);
    }
    Ok(result)
}

pub(super) fn apply_document_root(
    inputs: &SequenceInputs<'_>,
    document: &dynamic_document::DynamicDocument,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source_parameters = parameters.values().any(|parameter| {
        matches!(&parameter.value,
            InvocationParameterValue::SourceNodes(nodes) if !nodes.is_empty())
    });
    validate_literal_document_context(inputs, source_parameters)?;
    let effective = derive_effective_source(
        &inputs.program.source_whitespace,
        document.document.as_ref(),
        WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    // The frame's immutable owner remains live throughout this borrowed focus.
    // Fresh caches prevent local-ID membership from crossing effective sources.
    let source = effective.as_ref().unwrap_or(document.document.as_ref());
    let external_inputs = external_document_inputs(inputs, source);
    apply_template_at(
        &external_inputs,
        source.document_node(),
        mode,
        parameters,
        1,
        1,
        control,
    )
}
