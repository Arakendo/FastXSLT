//! Ordinary-source and source-union identity consumers; normalization stays in `XPath`.

use super::{
    ExecutionFailure, FailureCategory, InvocationControl, NodeId, RuntimeVariables, SequenceInputs,
    SourceLocation, WorkDomain, control_failure, failure_at, required_source_context,
};
use crate::xdm::atomic_value_experiment::BuiltinAtomicType;
use crate::xdm::qualified_nodes::QualifiedSourceNode;
use crate::xpath::path_experiment::evaluate_location_path_union_controlled;
use crate::xslt::golden_semantics_experiment::Xslt10GeneratedIdentitySelection;

enum Selected<'a> {
    Source(Vec<NodeId>),
    Qualified(Vec<QualifiedSourceNode<'a>>),
}

pub(in crate::runtime::golden_runtime_experiment) fn compare(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    left: &crate::xslt::golden_semantics_experiment::GeneratedSourceIdentity,
    right: &crate::xslt::golden_semantics_experiment::GeneratedSourceIdentity,
    equal: bool,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let left = source_value(inputs, context, left, variables, control)?;
    let right = source_value(inputs, context, right, variables, control)?;
    Ok((left == right) == equal)
}

pub(in crate::runtime::golden_runtime_experiment) fn source_value(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    operand: &crate::xslt::golden_semantics_experiment::GeneratedSourceIdentity,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    use super::super::{bound_variable_path, dynamic_document, runtime_context};
    use crate::xslt::golden_semantics_experiment::GeneratedSourceSelection as Selection;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let (owner, nodes) = match &operand.selection {
        Selection::Path(path) => {
            let (source, context) = required_source_context(inputs, context)?;
            let nodes = crate::xpath::path_experiment::evaluate_location_path_controlled(
                source, context, path, control,
            )
            .map_err(|error| control_failure(error, inputs.request_id))?;
            return nodes.first().map_or_else(
                || Ok(String::new()),
                |node| {
                    runtime_context::source_node_identity_in_context(inputs, source, *node, control)
                },
            );
        }
        Selection::Variable(name) => {
            super::super::ensure_variable_is_bound(inputs, name, variables)?;
            if let Some((owner, nodes)) = variables.retained_source(inputs.globals, name) {
                return nodes.first().map_or_else(
                    || Ok(String::new()),
                    |node| {
                        runtime_context::source_node_identity_in_context(
                            inputs, owner, *node, control,
                        )
                    },
                );
            }
            if let Some(nodes) = variables.source_nodes_in_context(inputs, name)? {
                let source = inputs.source.ok_or_else(|| {
                    failure_at(
                        "XPDY0002",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        operand.location.clone(),
                        "generated identity requires a source owner",
                    )
                })?;
                return nodes.first().map_or_else(
                    || Ok(String::new()),
                    |node| {
                        runtime_context::source_node_identity_in_context(
                            inputs, source, *node, control,
                        )
                    },
                );
            }
            return Err(failure_at(
                "FXRT0007",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                operand.location.clone(),
                "generated identity requires an admitted ordinary source-node variable",
            ));
        }
        Selection::LiteralDocument { reference, path } => {
            let dynamic = dynamic_document::prepare_document(inputs, reference, control)?;
            bound_variable_path::select_document_path(inputs, &dynamic.document, path, control)?
        }
        Selection::ReferencedDocuments { references, path } => {
            let documents =
                dynamic_document::prepare_source_documents(inputs, context, references, control)?;
            if documents.len() > 1 {
                return Err(failure_at(
                    "FXRT0007",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    operand.location.clone(),
                    "generated identity across distinct documents requires cross-document ordering",
                ));
            }
            let Some(dynamic) = documents.first() else {
                return Ok(String::new());
            };
            bound_variable_path::select_document_path(inputs, &dynamic.document, path, control)?
        }
    };
    nodes.first().map_or_else(
        || Ok(String::new()),
        |node| runtime_context::source_node_identity_in_context(inputs, &owner, *node, control),
    )
}

pub(super) fn union_position_value(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    selection: &Xslt10GeneratedIdentitySelection,
    variable: &str,
    location: &SourceLocation,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|error| control_failure(error, inputs.request_id))?;
    super::super::runtime_context::ensure_principal_identity_context(
        inputs, context, location, control,
    )?;
    let selected = match selection {
        Xslt10GeneratedIdentitySelection::SourcePaths(paths) => {
            let (source, context) = required_source_context(inputs, context)?;
            Selected::Source(
                evaluate_location_path_union_controlled(source, context, paths, control)
                    .map_err(|error| control_failure(error, inputs.request_id))?,
            )
        }
        Xslt10GeneratedIdentitySelection::Qualified(path) => {
            Selected::Qualified(super::super::namespace_focus::select(
                inputs,
                path,
                super::super::SequenceContext::new(context, None),
                control,
            )?)
        }
    };
    let value = variables.atomics.get(variable).or_else(|| {
        variables
            .allows_global_fallback(variable)
            .then(|| inputs.globals.atomics.get(variable))
            .flatten()
    });
    let Some(value) = value else {
        // Check kinds without converting/copying an unsupported value.
        let bound = variables.atomic_sequences.contains_key(variable)
            || variables.detached_source_node_strings(variable).is_some()
            || variables
                .source_nodes_in_context(inputs, variable)?
                .is_some()
            || variables.temporary_tree(inputs.globals, variable).is_some()
            || variables
                .literal_document(inputs.globals, variable)
                .is_some()
            || (variables.allows_global_fallback(variable)
                && inputs.globals.empty_sequences.contains(variable));
        if !bound {
            return Err(failure_at(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                location.clone(),
                format!("unbound variable: ${variable}"),
            ));
        }
        return Err(failure_at(
            "FXRT0007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            location.clone(),
            "filtered-union identity requires a scalar atomic position variable",
        ));
    };
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|error| control_failure(error, inputs.request_id))?;
    let position = match value.atomic_type() {
        // Equality with a boolean converts position() to boolean. Every
        // nonempty position is true; generate-id then takes the first match.
        BuiltinAtomicType::Boolean => matches!(value.lexical(), "true" | "1").then_some(1),
        BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic => numeric_position(
            crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(value.lexical()),
        ),
        BuiltinAtomicType::Integer
        | BuiltinAtomicType::Decimal
        | BuiltinAtomicType::Float
        | BuiltinAtomicType::Double => numeric_position(value.lexical().parse::<f64>().ok()),
        _ => {
            return Err(failure_at(
                "FXRT0007",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                location.clone(),
                "filtered-union identity position type is outside XPath 1.0 conversion",
            ));
        }
    };
    if let Some(position) = position {
        match selected {
            Selected::Source(nodes) => {
                if let Some(node) = nodes.get(position - 1) {
                    return Ok(super::super::runtime_context::source_node_identity(*node));
                }
            }
            Selected::Qualified(nodes) => {
                if let Some(node) = nodes.get(position - 1) {
                    return super::super::namespace_focus::node_identity(inputs, *node, control);
                }
            }
        }
    }
    Ok(String::new())
}

fn numeric_position(number: Option<f64>) -> Option<usize> {
    number
        .filter(|number| number.is_finite() && *number >= 1.0 && number.fract() == 0.0)
        .and_then(|number| number.to_string().parse().ok())
}
