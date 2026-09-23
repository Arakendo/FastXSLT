//! Charged XSLT 1.0 key lookup reference selection.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    KeyUseExpression, StylesheetProgram, Xslt10KeyLookup, Xslt10KeyName, Xslt10KeyNodePredicate,
    Xslt10KeyValue,
};

use super::runtime_context::{RuntimeVariables, SequenceInputs, required_source_context};
use super::template_selector::{
    DocumentRootedMatchCache, TemplateSelectionContext, matches_pattern,
};
use super::value_evaluator::xslt10_number_lexical;
use super::{ExecutionFailure, FailureCategory, control_failure, failure_at};

pub(super) fn select(
    inputs: &SequenceInputs<'_>,
    lookup: &Xslt10KeyLookup,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let lookup_values =
        evaluate_lookup_value(inputs, source, context, &lookup.value, variables, control)?;
    let lookup_name =
        resolve_lookup_name(inputs, &lookup.name, &lookup.location, variables, control)?;
    let definitions = inputs
        .program
        .key_definitions
        .iter()
        .enumerate()
        .filter(|(_, definition)| definition.name == *lookup_name)
        .collect::<Vec<_>>();
    if definitions.is_empty() {
        return Err(failure_at(
            "XTDE1260",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            lookup.location.clone(),
            format!("key() refers to an undeclared key: {}", lookup_name.local),
        ));
    }

    let mut candidates = Vec::new();
    collect_source_nodes(source, source.document_node(), &mut candidates);
    let match_variables = BTreeMap::new();
    let mut selected = Vec::new();
    for candidate in candidates {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let mut matches = false;
        for (definition_index, definition) in &definitions {
            let selection = TemplateSelectionContext {
                program: inputs.program,
                source,
                node: candidate,
                mode: None,
                variables: &match_variables,
                request_id: inputs.request_id,
                document_rooted_matches: &inputs.document_rooted_matches,
            };
            let pattern_index = inputs.program.matched_templates.len() + *definition_index;
            if !matches_pattern(
                pattern_index,
                &definition.match_pattern,
                &selection,
                control,
            )? {
                continue;
            }
            if evaluate_use(
                source,
                candidate,
                &definition.use_expression,
                inputs.request_id,
                control,
            )?
            .into_iter()
            .any(|lexical| lookup_values.contains(&lexical))
            {
                matches = true;
                break;
            }
        }
        if matches {
            selected.push(candidate);
        }
    }

    selected = apply_predicate(inputs, source, selected, lookup.predicate.as_ref(), control)?;

    if let Some(tail) = &lookup.tail {
        let mut tailed = Vec::new();
        for node in selected {
            tailed.extend(
                evaluate_location_path_controlled(source, node, tail, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?,
            );
        }
        tailed.sort_unstable_by_key(|node| source.document_order(*node));
        tailed.dedup();
        selected = tailed;
    }
    Ok(selected)
}

pub(super) fn select_static_pattern(
    program: &StylesheetProgram,
    source: &Document,
    lookup: &Xslt10KeyLookup,
    request_id: &str,
    document_rooted_matches: &std::cell::RefCell<DocumentRootedMatchCache>,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let (Xslt10KeyName::Static(lookup_name), Xslt10KeyValue::Static(lookup_value)) =
        (&lookup.name, &lookup.value)
    else {
        return Err(failure_at(
            "FXIN0001",
            FailureCategory::Invalid,
            Some(request_id),
            lookup.location.clone(),
            "compiled key() match pattern did not retain static arguments",
        ));
    };
    let definitions = program
        .key_definitions
        .iter()
        .enumerate()
        .filter(|(_, definition)| definition.name == *lookup_name)
        .collect::<Vec<_>>();
    if definitions.is_empty() {
        return Err(failure_at(
            "XTDE1260",
            FailureCategory::Invalid,
            Some(request_id),
            lookup.location.clone(),
            format!("key() refers to an undeclared key: {}", lookup_name.local),
        ));
    }

    let mut candidates = Vec::new();
    collect_source_nodes(source, source.document_node(), &mut candidates);
    let variables = BTreeMap::new();
    let mut selected = Vec::new();
    for candidate in candidates {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        for (definition_index, definition) in &definitions {
            let selection = TemplateSelectionContext {
                program,
                source,
                node: candidate,
                mode: None,
                variables: &variables,
                request_id,
                document_rooted_matches,
            };
            if !matches_pattern(
                program.matched_templates.len() + *definition_index,
                &definition.match_pattern,
                &selection,
                control,
            )? {
                continue;
            }
            if evaluate_use(
                source,
                candidate,
                &definition.use_expression,
                request_id,
                control,
            )?
            .iter()
            .any(|value| value == lookup_value)
            {
                selected.push(candidate);
                break;
            }
        }
    }

    if let Some(tail) = &lookup.tail {
        let mut tailed = Vec::new();
        for node in selected {
            tailed.extend(
                evaluate_location_path_controlled(source, node, tail, control)
                    .map_err(|failure| control_failure(failure, request_id))?,
            );
        }
        tailed.sort_unstable_by_key(|node| source.document_order(*node));
        tailed.dedup();
        selected = tailed;
    }
    Ok(selected)
}

fn resolve_lookup_name<'a>(
    inputs: &SequenceInputs<'_>,
    name: &'a Xslt10KeyName,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Cow<'a, ExpandedName>, ExecutionFailure> {
    let (lexical, static_namespaces) = match name {
        Xslt10KeyName::Static(name) => return Ok(Cow::Borrowed(name)),
        Xslt10KeyName::Variable {
            name,
            static_namespaces,
        } => (
            super::value_evaluator::xslt10_variable_string_value(inputs, name, variables, control)?,
            static_namespaces,
        ),
        Xslt10KeyName::Concat {
            expression,
            static_namespaces,
        } => (
            super::value_evaluator::evaluate_xslt10_concat(
                inputs, None, expression, variables, control,
            )?,
            static_namespaces,
        ),
    };
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    resolve_lexical_key_name(&lexical, static_namespaces)
        .map(Cow::Owned)
        .ok_or_else(|| {
            failure_at(
                "XTDE1260",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                location.clone(),
                format!("key() name is not a bound lexical QName: {lexical}"),
            )
        })
}

fn resolve_lexical_key_name(
    lexical: &str,
    static_namespaces: &[NamespaceBinding],
) -> Option<ExpandedName> {
    let (prefix, local) = lexical
        .split_once(':')
        .map_or((None, lexical), |(prefix, local)| (Some(prefix), local));
    if !is_ascii_ncname(local)
        || prefix.is_some_and(|prefix| !is_ascii_ncname(prefix))
        || local.contains(':')
    {
        return None;
    }
    let namespace = match prefix {
        Some(prefix) => Some(
            static_namespaces
                .iter()
                .find(|binding| binding.prefix.as_deref() == Some(prefix))?
                .namespace
                .clone(),
        ),
        None => None,
    };
    Some(ExpandedName {
        namespace,
        local: local.to_owned(),
    })
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| {
            character == '_'
                || character == '-'
                || character == '.'
                || character.is_ascii_alphanumeric()
        })
}

fn apply_predicate(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    selected: Vec<NodeId>,
    predicate: Option<&Xslt10KeyNodePredicate>,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let Some(predicate) = predicate else {
        return Ok(selected);
    };
    match predicate {
        Xslt10KeyNodePredicate::Position(position) => Ok(position
            .checked_sub(1)
            .and_then(|index| selected.get(index).copied())
            .into_iter()
            .collect()),
        Xslt10KeyNodePredicate::Last => Ok(selected.last().copied().into_iter().collect()),
        Xslt10KeyNodePredicate::AttributeEquals { name, value } => {
            let mut filtered = Vec::new();
            for node in selected {
                let mut matches = false;
                for attribute in source.attributes(node) {
                    control
                        .charge(WorkDomain::XPathNodeVisit, 1)
                        .map_err(|failure| control_failure(failure, inputs.request_id))?;
                    if source.name(*attribute) == Some(name)
                        && source.value(*attribute) == Some(value.as_str())
                    {
                        matches = true;
                        break;
                    }
                }
                if matches {
                    filtered.push(node);
                }
            }
            Ok(filtered)
        }
    }
}

fn evaluate_lookup_value(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    context: NodeId,
    value: &Xslt10KeyValue,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<String>, ExecutionFailure> {
    match value {
        Xslt10KeyValue::Static(value) => Ok(vec![value.clone()]),
        Xslt10KeyValue::Variable(name) => {
            if let Some(nodes) = variables.source_nodes(inputs.globals, name) {
                let source = inputs.source.ok_or_else(|| {
                    super::failure(
                        "XPDY0002",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        format!("key() variable ${name} requires a source document"),
                    )
                })?;
                return nodes
                    .iter()
                    .map(|node| {
                        source
                            .string_value_controlled(*node, control)
                            .map_err(|failure| control_failure(failure, inputs.request_id))
                    })
                    .collect();
            }
            super::value_evaluator::xslt10_variable_string_value(inputs, name, variables, control)
                .map(|value| vec![value])
        }
        Xslt10KeyValue::ContextPath(path) => {
            evaluate_location_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?
                .into_iter()
                .map(|node| {
                    source
                        .string_value_controlled(node, control)
                        .map_err(|failure| control_failure(failure, inputs.request_id))
                })
                .collect()
        }
        Xslt10KeyValue::NestedLookup(lookup) => {
            select(inputs, lookup, Some(context), variables, control)?
                .into_iter()
                .map(|node| {
                    source
                        .string_value_controlled(node, control)
                        .map_err(|failure| control_failure(failure, inputs.request_id))
                })
                .collect()
        }
    }
}

fn evaluate_use(
    source: &Document,
    candidate: NodeId,
    expression: &KeyUseExpression,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<String>, ExecutionFailure> {
    match expression {
        KeyUseExpression::LiteralString(value) => Ok(vec![value.clone()]),
        KeyUseExpression::LocationPath(path) => {
            let selected = evaluate_location_path_controlled(source, candidate, path, control)
                .map_err(|failure| control_failure(failure, request_id))?;
            selected
                .into_iter()
                .map(|node| {
                    source
                        .string_value_controlled(node, control)
                        .map_err(|failure| control_failure(failure, request_id))
                })
                .collect()
        }
        KeyUseExpression::PathUnion(alternatives) => {
            let mut selected = Vec::new();
            for path in alternatives {
                selected.extend(
                    evaluate_location_path_controlled(source, candidate, path, control)
                        .map_err(|failure| control_failure(failure, request_id))?,
                );
            }
            selected.sort_unstable_by_key(|node| source.document_order(*node));
            selected.dedup();
            selected
                .into_iter()
                .map(|node| {
                    source
                        .string_value_controlled(node, control)
                        .map_err(|failure| control_failure(failure, request_id))
                })
                .collect()
        }
        KeyUseExpression::NumberPath(path) => {
            let selected = evaluate_location_path_controlled(source, candidate, path, control)
                .map_err(|failure| control_failure(failure, request_id))?;
            let lexical = if let Some(node) = selected.first().copied() {
                source
                    .string_value_controlled(node, control)
                    .map_err(|failure| control_failure(failure, request_id))?
            } else {
                String::new()
            };
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            Ok(vec![xslt10_number_lexical(&lexical)])
        }
    }
}

fn collect_source_nodes(source: &Document, node: NodeId, nodes: &mut Vec<NodeId>) {
    nodes.push(node);
    nodes.extend_from_slice(source.attributes(node));
    for child in source.children(node) {
        collect_source_nodes(source, *child, nodes);
    }
}
