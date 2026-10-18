//! Owner-preserving variable-path selection through the shared charged `XPath` evaluator.

use super::{
    Arc, Document, ExecutionFailure, FailureCategory, InvocationControl, NodeId, RuntimeVariables,
    SequenceInputs, WhitespaceRepresentation, WorkDomain, control_failure, derive_effective_source,
    evaluate_location_path_controlled, failure,
};
use crate::xdm::atomic_value_experiment::BuiltinAtomicType;
use crate::xpath::path_experiment::LocationPath;
use crate::xslt::golden_semantics_experiment::BoundVariablePath;
use crate::xslt::golden_semantics_experiment::Instruction;

type RetainedPathSelection = (Arc<Document>, Vec<NodeId>);

pub(super) fn current_lookup_value(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    lookup: &crate::xslt::golden_semantics_experiment::CurrentVariableLookup,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, current) = super::required_source_context(inputs, context)?;
    let outer =
        evaluate_location_path_controlled(source, current, &lookup.outer_attribute, control)
            .map_err(|error| control_failure(error, inputs.request_id))?;
    let outer_value = outer
        .first()
        .map(|node| source.string_value_controlled(*node, control))
        .transpose()
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let Some((owner, candidates)) = select(
        inputs,
        &lookup.variable,
        &BoundVariablePath::Path(lookup.candidates.clone()),
        variables,
        control,
    )?
    else {
        return Err(failure(
            "FXRT1017",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "current-node lookup requires a retained source selection",
        ));
    };
    for candidate in candidates {
        let attributes = evaluate_location_path_controlled(
            &owner,
            candidate,
            &lookup.candidate_attribute,
            control,
        )
        .map_err(|error| control_failure(error, inputs.request_id))?;
        let value = attributes
            .first()
            .map(|node| owner.string_value_controlled(*node, control))
            .transpose()
            .map_err(|error| control_failure(error, inputs.request_id))?;
        if outer_value.is_none() || value != outer_value {
            continue;
        }
        let result =
            evaluate_location_path_controlled(&owner, candidate, &lookup.result_attribute, control)
                .map_err(|error| control_failure(error, inputs.request_id))?;
        if let Some(node) = result.first() {
            return owner
                .string_value_controlled(*node, control)
                .map_err(|error| control_failure(error, inputs.request_id));
        }
    }
    Ok(String::new())
}

pub(super) fn select_document_path(
    inputs: &SequenceInputs<'_>,
    document: &Arc<Document>,
    path: &LocationPath,
    control: &mut InvocationControl,
) -> Result<RetainedPathSelection, ExecutionFailure> {
    let owner = effective_document_owner(inputs, document, control)?;
    let nodes = select_path(inputs, &owner, &[owner.document_node()], path, control)?;
    Ok((owner, nodes))
}

fn effective_document_owner(
    inputs: &SequenceInputs<'_>,
    document: &Arc<Document>,
    control: &mut InvocationControl,
) -> Result<Arc<Document>, ExecutionFailure> {
    let effective = derive_effective_source(
        &inputs.program.source_whitespace,
        document,
        WhitespaceRepresentation::VisibilityView,
        inputs.request_id,
        control,
    )?;
    if let Some(effective) = effective {
        effective
            .retain_effective_owner(control)
            .map_err(|failure| control_failure(failure, inputs.request_id))
    } else {
        Ok(document.clone())
    }
}

pub(super) fn bind(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let Instruction::SourceVariablePathVariable {
        name,
        source: source_name,
        select,
        ..
    } = instruction
    else {
        unreachable!("source-variable path binder receives one path binding")
    };
    if let Some((owner, selected)) = self::select(inputs, source_name, select, scope, control)? {
        scope.bind_retained_source_nodes(name.to_owned(), owner, selected);
        return Ok(());
    }
    let crate::xslt::golden_semantics_experiment::BoundVariablePath::Path(select) = select else {
        return Err(failure(
            "FXRT1017",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "filtered variable paths require an admitted source owner",
        ));
    };
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "source-variable path binding requires a principal source",
        )
    })?;
    let selected = evaluate_eligible_path(inputs, source, source_name, select, scope, control)?;
    scope.bind_source_nodes(name.to_owned(), selected);
    Ok(())
}

pub(super) fn evaluate_eligible_path(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    variable: &str,
    path: &crate::xpath::path_experiment::LocationPath,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let roots = variables
        .source_nodes_in_context(inputs, variable)?
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("relative path requires a source-node sequence: ${variable}"),
            )
        })?;
    let mut selected = Vec::new();
    for root in roots {
        selected.extend(
            evaluate_location_path_controlled(source, *root, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?,
        );
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    Ok(selected)
}

pub(super) fn select(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    plan: &BoundVariablePath,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Option<RetainedPathSelection>, ExecutionFailure> {
    let (owner, mut roots) =
        if let Some((owner, nodes)) = variables.retained_source(inputs.globals, variable) {
            charge_retention(inputs, nodes.len(), control)?;
            (owner.clone(), nodes.clone())
        } else if let Some(dynamic) = variables.literal_document(inputs.globals, variable) {
            let owner = effective_document_owner(inputs, &dynamic.document, control)?;
            charge_retention(inputs, 1, control)?;
            let root = owner.document_node();
            (owner, vec![root])
        } else if matches!(plan, BoundVariablePath::FirstFilteredChildren(_)) {
            let Some(nodes) = variables.source_nodes_in_context(inputs, variable)? else {
                super::ensure_variable_is_bound(inputs, variable, variables)?;
                return Ok(None);
            };
            let Some(source) = inputs.source else {
                return Ok(None);
            };
            let owner = source
                .retain_effective_owner(control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            charge_retention(inputs, nodes.len(), control)?;
            (owner, nodes.clone())
        } else {
            return Ok(None);
        };
    match plan {
        BoundVariablePath::Path(path) => {
            roots = select_path(inputs, &owner, &roots, path, control)?;
        }
        BoundVariablePath::FirstFilteredChildren(steps) => {
            for step in steps {
                roots = select_path(inputs, &owner, &roots, &step.path, control)?;
                if let Some((attribute, variable)) = &step.filter {
                    roots = filter_attribute(
                        inputs, &owner, roots, attribute, variable, variables, control,
                    )?;
                }
            }
            roots.truncate(1);
        }
    }
    Ok(Some((owner, roots)))
}

fn filter_attribute(
    inputs: &SequenceInputs<'_>,
    owner: &Document,
    nodes: Vec<NodeId>,
    attribute: &LocationPath,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    super::ensure_variable_is_bound(inputs, variable, variables)?;
    let value = variables
        .atomics
        .get(variable)
        .or_else(|| {
            variables
                .allows_global_fallback(variable)
                .then(|| inputs.globals.atomics.get(variable))
                .flatten()
        })
        .ok_or_else(|| {
            failure(
                "FXRT1017",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                format!("child filter requires a bound atomic string: ${variable}"),
            )
        })?;
    if value.atomic_type() != BuiltinAtomicType::String {
        return Err(failure(
            "FXRT1017",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "this child filter admits only atomic string comparison",
        ));
    }
    let mut filtered = Vec::new();
    for node in nodes {
        let attributes = evaluate_location_path_controlled(owner, node, attribute, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if let Some(attribute) = attributes.first() {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let actual = owner
                .string_value_controlled(*attribute, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if actual == value.lexical() {
                charge_retention(inputs, 1, control)?;
                filtered.push(node);
            }
        }
    }
    Ok(filtered)
}

fn select_path(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    roots: &[NodeId],
    path: &LocationPath,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut selected = Vec::new();
    for root in roots {
        let nodes = evaluate_location_path_controlled(source, *root, path, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        charge_retention(inputs, nodes.len(), control)?;
        selected.extend(nodes);
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    Ok(selected)
}

fn charge_retention(
    inputs: &SequenceInputs<'_>,
    count: usize,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XdmNode, count)
        .map_err(|failure| control_failure(failure, inputs.request_id))
}
