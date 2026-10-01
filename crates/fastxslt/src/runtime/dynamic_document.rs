//! Invocation-owned preparation for literal sealed-snapshot `document()` references.

use std::collections::HashSet;
use std::sync::Arc;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::{ResolutionFailure, ResolutionLimits, SnapshotResolver};
use crate::xdm::owned_tree_experiment::{BuildFailure, Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::{
    DocumentBaseReference, DocumentRootReference, NestedDocumentReferences,
};

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
    context: Option<NodeId>,
    reference: &DocumentRootReference,
    base: Option<&DocumentBaseReference>,
    path: Option<&LocationPath>,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<super::ResultNode>, ExecutionFailure> {
    let Some(base) = effective_document_base(inputs, context, &reference.base, base, control)?
    else {
        return Ok(Vec::new());
    };
    let effective_reference = DocumentRootReference {
        base,
        reference: reference.reference.clone(),
        descendant_name: reference.descendant_name.clone(),
    };
    let dynamic = prepare_document(inputs, &effective_reference, control)?;
    if let Some(descendant_name) = reference.descendant_name.as_ref() {
        return copy_matching_descendants(
            &dynamic.document,
            dynamic.document.document_node(),
            descendant_name,
            inputs.request_id,
            recover_unattached_attributes,
            control,
        );
    }
    copy_prepared_document(
        inputs,
        &dynamic,
        path,
        recover_unattached_attributes,
        control,
    )
}

fn effective_document_base(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    default_base: &str,
    base: Option<&DocumentBaseReference>,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
    let Some(base) = base else {
        return Ok(Some(default_base.to_owned()));
    };
    if let DocumentBaseReference::LiteralDocument(reference) = base {
        let dynamic = prepare_document(inputs, reference, control)?;
        return Ok(Some(
            dynamic
                .document
                .location(dynamic.document.document_node())
                .resource
                .clone(),
        ));
    }
    let DocumentBaseReference::SourcePath(base_path) = base else {
        unreachable!("document base reference variants are exhaustively handled");
    };
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "document() base node-set requires a source document",
        )
    })?;
    let context = context.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "document() base node-set requires a source-node focus",
        )
    })?;
    let effective_source = super::derive_effective_source(
        &inputs.program.source_whitespace,
        source,
        super::WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let source = effective_source.as_ref().unwrap_or(source);
    let selected = evaluate_location_path_controlled(source, context, base_path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(selected
        .first()
        .map(|node| source.location(*node).resource.clone()))
}

pub(super) fn copy_source_documents(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    references: &LocationPath,
    base: Option<&DocumentBaseReference>,
    path: Option<&LocationPath>,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<super::ResultNode>, ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "document() node-set argument requires a source document",
        )
    })?;
    let context = context.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "document() node-set argument requires a source-node focus",
        )
    })?;
    let effective_source = super::derive_effective_source(
        &inputs.program.source_whitespace,
        source,
        super::WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let source = effective_source.as_ref().unwrap_or(source);
    let selected = evaluate_location_path_controlled(source, context, references, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let explicit_base = if base.is_some() {
        effective_document_base(inputs, Some(context), "", base, control)?
    } else {
        None
    };
    let mut copied = Vec::new();
    let mut copied_identities = HashSet::new();
    for node in selected {
        let reference = source
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let dynamic = prepare_document(
            inputs,
            &DocumentRootReference {
                base: explicit_base
                    .clone()
                    .unwrap_or_else(|| source.location(node).resource.clone()),
                reference,
                descendant_name: None,
            },
            control,
        )?;
        if copied_identities.insert(dynamic.identity) {
            copied.extend(copy_prepared_document(
                inputs,
                &dynamic,
                path,
                recover_unattached_attributes,
                control,
            )?);
        }
    }
    Ok(copied)
}

pub(super) fn prepare_source_documents(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    references: &LocationPath,
    control: &mut InvocationControl,
) -> Result<Vec<DynamicDocument>, ExecutionFailure> {
    prepare_source_documents_with_base(inputs, context, references, None, control)
}

pub(super) fn prepare_source_documents_with_base(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    references: &LocationPath,
    base: Option<&DocumentBaseReference>,
    control: &mut InvocationControl,
) -> Result<Vec<DynamicDocument>, ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "document() node-set argument requires a source document",
        )
    })?;
    let context = context.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "document() node-set argument requires a source-node focus",
        )
    })?;
    let effective_source = super::derive_effective_source(
        &inputs.program.source_whitespace,
        source,
        super::WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let source = effective_source.as_ref().unwrap_or(source);
    let selected = evaluate_location_path_controlled(source, context, references, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let explicit_base = if base.is_some() {
        effective_document_base(inputs, Some(context), "", base, control)?
    } else {
        None
    };
    let mut prepared = Vec::new();
    let mut prepared_identities = HashSet::new();
    for node in selected {
        let reference = source
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let dynamic = prepare_document(
            inputs,
            &DocumentRootReference {
                base: explicit_base
                    .clone()
                    .unwrap_or_else(|| source.location(node).resource.clone()),
                reference,
                descendant_name: None,
            },
            control,
        )?;
        if prepared_identities.insert(dynamic.identity) {
            prepared.push(dynamic);
        }
    }
    Ok(prepared)
}

pub(super) fn count_source_document_descendants(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    references: &LocationPath,
    name: &ExpandedName,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    let prepared = prepare_source_documents(inputs, context, references, control)?;
    let mut count = 0_usize;
    for dynamic in prepared {
        let document = dynamic.document.as_ref();
        let mut pending = document
            .children(document.document_node())
            .iter()
            .rev()
            .copied()
            .collect::<Vec<_>>();
        while let Some(node) = pending.pop() {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if document.kind(node) == NodeKind::Element && document.name(node) == Some(name) {
                count = count.checked_add(1).ok_or_else(|| {
                    failure(
                        "FODC0002",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        "document() descendant count exceeds the supported platform size",
                    )
                })?;
            }
            pending.extend(document.children(node).iter().rev().copied());
        }
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(count)
}

pub(super) fn copy_nested_documents(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    references: &NestedDocumentReferences,
    path: Option<&LocationPath>,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<super::ResultNode>, ExecutionFailure> {
    let mut copied = Vec::new();
    let mut copied_identities = HashSet::new();
    match references {
        NestedDocumentReferences::Literal {
            reference,
            path: reference_path,
        } => {
            let dynamic = prepare_document(inputs, reference, control)?;
            copy_nested_document_references(
                inputs,
                &dynamic,
                reference_path.as_ref(),
                path,
                recover_unattached_attributes,
                &mut copied_identities,
                &mut copied,
                control,
            )?;
        }
        NestedDocumentReferences::Source {
            references,
            path: reference_path,
        } => {
            let source = inputs.source.ok_or_else(|| {
                failure(
                    "XPDY0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    "nested document() source argument requires a source document",
                )
            })?;
            let context = context.ok_or_else(|| {
                failure(
                    "XPDY0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    "nested document() source argument requires a source-node focus",
                )
            })?;
            let effective_source = super::derive_effective_source(
                &inputs.program.source_whitespace,
                source,
                super::WhitespaceRepresentation::VisibilityView,
                inputs.request_id,
                control,
            )?;
            let source = effective_source.as_ref().unwrap_or(source);
            let selected = evaluate_location_path_controlled(source, context, references, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let mut prepared_identities = HashSet::new();
            for node in selected {
                let reference = source
                    .string_value_controlled(node, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                let dynamic = prepare_document(
                    inputs,
                    &DocumentRootReference {
                        base: source.location(node).resource.clone(),
                        reference,
                        descendant_name: None,
                    },
                    control,
                )?;
                if prepared_identities.insert(dynamic.identity) {
                    copy_nested_document_references(
                        inputs,
                        &dynamic,
                        reference_path.as_ref(),
                        path,
                        recover_unattached_attributes,
                        &mut copied_identities,
                        &mut copied,
                        control,
                    )?;
                }
            }
        }
    }
    Ok(copied)
}

#[allow(clippy::too_many_arguments)]
fn copy_nested_document_references(
    inputs: &SequenceInputs<'_>,
    references: &DynamicDocument,
    reference_path: Option<&LocationPath>,
    result_path: Option<&LocationPath>,
    recover_unattached_attributes: bool,
    copied_identities: &mut HashSet<u64>,
    copied: &mut Vec<super::ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let effective_document = super::derive_effective_source(
        &inputs.program.source_whitespace,
        references.document.as_ref(),
        super::WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    let document = effective_document
        .as_ref()
        .unwrap_or(references.document.as_ref());
    let selected = if let Some(reference_path) = reference_path {
        evaluate_location_path_controlled(
            document,
            document.document_node(),
            reference_path,
            control,
        )
        .map_err(|failure| control_failure(failure, inputs.request_id))?
    } else {
        vec![document.document_node()]
    };
    for node in selected {
        let reference = document
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let dynamic = prepare_document(
            inputs,
            &DocumentRootReference {
                base: document.location(node).resource.clone(),
                reference,
                descendant_name: None,
            },
            control,
        )?;
        if copied_identities.insert(dynamic.identity) {
            copied.extend(copy_prepared_document(
                inputs,
                &dynamic,
                result_path,
                recover_unattached_attributes,
                control,
            )?);
        }
    }
    Ok(())
}

fn copy_prepared_document(
    inputs: &SequenceInputs<'_>,
    dynamic: &DynamicDocument,
    path: Option<&LocationPath>,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<super::ResultNode>, ExecutionFailure> {
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
    super::copy_source_node(
        &dynamic.document,
        inputs.request_id,
        dynamic.document.document_node(),
        recover_unattached_attributes,
        control,
    )
}

pub(super) fn copy_prepared_variable_document(
    inputs: &SequenceInputs<'_>,
    dynamic: &DynamicDocument,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<super::ResultNode>, ExecutionFailure> {
    copy_prepared_document(
        inputs,
        dynamic,
        None,
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
    prepare_document_from_snapshot(
        inputs.resource_snapshot,
        inputs.denied_resources,
        inputs.dynamic_documents,
        inputs.request_id,
        reference,
        control,
    )
}

pub(super) fn prepare_document_from_snapshot(
    resource_snapshot: Option<&crate::resources::ResourceSnapshot>,
    denied_resources: Option<&HashSet<String>>,
    dynamic_documents: &std::cell::RefCell<std::collections::BTreeMap<String, DynamicDocument>>,
    request_id: &str,
    reference: &DocumentRootReference,
    control: &mut InvocationControl,
) -> Result<DynamicDocument, ExecutionFailure> {
    let snapshot = resource_snapshot.ok_or_else(|| {
        failure(
            "FXRT1015",
            FailureCategory::Unsupported,
            Some(request_id),
            "document() requires an explicitly supplied sealed resource snapshot",
        )
    })?;
    let denied = denied_resources
        .into_iter()
        .flat_map(|denied| denied.iter().cloned());
    let mut resolver = SnapshotResolver::new(snapshot, denied, ResolutionLimits::new(1));
    let resource = resolver
        .resolve_from(&reference.base, &reference.reference)
        .map_err(|error| document_resolution_failure(&error, request_id))?;
    if resource.fragment.is_some() {
        return Err(failure(
            "FXRT1016",
            FailureCategory::Unsupported,
            Some(request_id),
            "fragment-bearing document() references are outside the admitted runtime slice",
        ));
    }
    let identity_key = resource.identity;
    if !dynamic_documents.borrow().contains_key(&identity_key) {
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
                        Some(request_id),
                        format!("document() resource XML is invalid: {error:?}"),
                    )
                },
                |failure| control_failure(*failure, request_id),
            )
        })?;
        let document = Arc::new(Document::from_parsed_controlled(parsed, control).map_err(
            |error| match error {
                BuildFailure::Control(failure) => control_failure(failure, request_id),
                _ => failure(
                    "FXXD0002",
                    FailureCategory::Invalid,
                    Some(request_id),
                    format!("document() resource XDM construction failed: {error:?}"),
                ),
            },
        )?);
        let identity = control.allocate_temporary_tree_identity().ok_or_else(|| {
            failure(
                "FXRT0016",
                FailureCategory::Limit,
                Some(request_id),
                "invocation-local document identity space is exhausted",
            )
        })?;
        dynamic_documents
            .borrow_mut()
            .insert(identity_key.clone(), DynamicDocument { identity, document });
    }
    Ok(dynamic_documents
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
