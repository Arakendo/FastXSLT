//! Invocation-owned preparation for literal sealed-snapshot `document()` references.

use std::sync::Arc;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::{ResolutionFailure, ResolutionLimits, SnapshotResolver};
use crate::xdm::owned_tree_experiment::{BuildFailure, Document, NodeId};
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::DocumentRootReference;

use super::{ExecutionFailure, FailureCategory, SequenceInputs, control_failure, failure};

#[derive(Debug, Clone)]
pub(super) struct DynamicDocument {
    pub(super) identity: u64,
    pub(super) document: Arc<Document>,
}

pub(super) fn document_root_identity(
    inputs: &SequenceInputs<'_>,
    reference: &DocumentRootReference,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let dynamic = prepare_document(inputs, reference, control)?;
    if let Some(name) = reference.descendant_name.as_ref()
        && !contains_descendant_local(
            &dynamic.document,
            dynamic.document.document_node(),
            name,
            inputs.request_id,
            control,
        )?
    {
        return Ok(String::new());
    }
    Ok(format!("d{}", dynamic.identity))
}

pub(super) fn copy_document(
    inputs: &SequenceInputs<'_>,
    reference: &DocumentRootReference,
    path: Option<&LocationPath>,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<super::ResultNode>, ExecutionFailure> {
    let dynamic = prepare_document(inputs, reference, control)?;
    if let Some(path) = path {
        let effective_document = super::derive_effective_source(
            &inputs.program.source_whitespace,
            dynamic.document.as_ref(),
            super::WhitespaceRepresentation::VisibilityView,
            inputs.request_id,
            control,
        )?;
        let document = effective_document
            .as_ref()
            .unwrap_or(dynamic.document.as_ref());
        let selected =
            evaluate_location_path_controlled(document, document.document_node(), path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let mut copied = Vec::new();
        for node in selected {
            copied.extend(super::copy_source_node(
                document,
                inputs.request_id,
                node,
                recover_unattached_attributes,
                control,
            )?);
        }
        return Ok(copied);
    }
    let Some(descendant_name) = reference.descendant_name.as_ref() else {
        return super::copy_source_node(
            &dynamic.document,
            inputs.request_id,
            dynamic.document.document_node(),
            recover_unattached_attributes,
            control,
        );
    };
    copy_matching_descendants(
        &dynamic.document,
        dynamic.document.document_node(),
        descendant_name,
        inputs.request_id,
        recover_unattached_attributes,
        control,
    )
}

fn copy_matching_descendants(
    document: &Document,
    parent: NodeId,
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    request_id: &str,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<super::ResultNode>, ExecutionFailure> {
    let mut copied = Vec::new();
    for child in document.children(parent) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if document
            .name(*child)
            .is_some_and(|candidate| candidate == name)
        {
            copied.extend(super::copy_source_node(
                document,
                request_id,
                *child,
                recover_unattached_attributes,
                control,
            )?);
        }
        copied.extend(copy_matching_descendants(
            document,
            *child,
            name,
            request_id,
            recover_unattached_attributes,
            control,
        )?);
    }
    Ok(copied)
}

pub(super) fn prepare_document(
    inputs: &SequenceInputs<'_>,
    reference: &DocumentRootReference,
    control: &mut InvocationControl,
) -> Result<DynamicDocument, ExecutionFailure> {
    let snapshot = inputs.resource_snapshot.ok_or_else(|| {
        failure(
            "FXRT1015",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "document() requires an explicitly supplied sealed resource snapshot",
        )
    })?;
    let denied = inputs
        .denied_resources
        .into_iter()
        .flat_map(|denied| denied.iter().cloned());
    let mut resolver = SnapshotResolver::new(snapshot, denied, ResolutionLimits::new(1));
    let resource = resolver
        .resolve_from(&reference.base, &reference.reference)
        .map_err(|error| document_resolution_failure(&error, inputs.request_id))?;
    if resource.fragment.is_some() {
        return Err(failure(
            "FXRT1016",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "fragment-bearing document() references are outside the admitted runtime slice",
        ));
    }
    let identity_key = resource.identity;
    if !inputs
        .dynamic_documents
        .borrow()
        .contains_key(&identity_key)
    {
        let parsed = crate::xml::quick_xml_experiment::parse_document_controlled(
            &identity_key,
            resource.bytes,
            super::XML_LIMITS,
            control,
        )
        .map_err(|error| {
            error.control_failure().map_or_else(
                || {
                    failure(
                        "FXXM0002",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        format!("document() resource XML is invalid: {error:?}"),
                    )
                },
                |failure| control_failure(*failure, inputs.request_id),
            )
        })?;
        let document = Arc::new(Document::from_parsed_controlled(parsed, control).map_err(
            |error| match error {
                BuildFailure::Control(failure) => control_failure(failure, inputs.request_id),
                _ => failure(
                    "FXXD0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    format!("document() resource XDM construction failed: {error:?}"),
                ),
            },
        )?);
        let identity = control.allocate_temporary_tree_identity().ok_or_else(|| {
            failure(
                "FXRT0016",
                FailureCategory::Limit,
                Some(inputs.request_id),
                "invocation-local document identity space is exhausted",
            )
        })?;
        inputs
            .dynamic_documents
            .borrow_mut()
            .insert(identity_key.clone(), DynamicDocument { identity, document });
    }
    Ok(inputs
        .dynamic_documents
        .borrow()
        .get(&identity_key)
        .expect("resolved dynamic document was inserted")
        .clone())
}

fn contains_descendant_local(
    document: &Document,
    parent: NodeId,
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    for child in document.children(parent) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if document
            .name(*child)
            .is_some_and(|candidate| candidate == name)
            || contains_descendant_local(document, *child, name, request_id, control)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn document_resolution_failure(error: &ResolutionFailure, request_id: &str) -> ExecutionFailure {
    let (code, category) = match error {
        ResolutionFailure::Denied { .. } => ("FXRS0003", FailureCategory::Denied),
        ResolutionFailure::Missing { .. } => ("FXRS0001", FailureCategory::MissingResource),
        ResolutionFailure::AttemptLimit { .. } => ("FXRS0004", FailureCategory::Limit),
        ResolutionFailure::InvalidBase { .. }
        | ResolutionFailure::InvalidReference { .. }
        | ResolutionFailure::ResolutionFailed { .. } => ("FXRS0005", FailureCategory::Invalid),
    };
    failure(
        code,
        category,
        Some(request_id),
        format!("document() resource resolution failed: {error:?}"),
    )
}
