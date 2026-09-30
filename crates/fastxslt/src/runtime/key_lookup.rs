//! Charged XSLT 1.0 key lookup reference selection.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    KeyUseExpression, StylesheetProgram, Xslt10ConcatExpression, Xslt10ConcatPart, Xslt10KeyLookup,
    Xslt10KeyName, Xslt10KeyNodePredicate, Xslt10KeyValue,
};

use super::runtime_context::{RuntimeVariables, SequenceInputs, required_source_context};
use super::template_selector::{
    DocumentRootedMatchCache, TemplateSelectionContext, matches_pattern,
};
use super::value_evaluator::xslt10_number_lexical;
use super::{ExecutionFailure, FailureCategory, control_failure, failure_at};

#[derive(Debug, Default)]
pub(super) struct KeyIndexCache {
    indexes: HashMap<ExpandedName, BTreeMap<String, Vec<NodeId>>>,
}

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
    let mut selected = if control.key_index_cache_enabled() {
        ensure_index(inputs, source, &lookup_name, &lookup.location, control)?;
        let indexes = inputs.key_indexes.borrow();
        let index = indexes
            .indexes
            .get(lookup_name.as_ref())
            .expect("key index was inserted before lookup");
        let mut selected = lookup_values
            .iter()
            .filter_map(|value| index.get(value))
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        selected.sort_unstable_by_key(|node| source.document_order(*node));
        selected.dedup();
        selected
    } else {
        select_reference(
            inputs,
            source,
            &lookup_name,
            &lookup_values,
            &lookup.location,
            control,
        )?
    };

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

pub(super) fn retain_muenchian_first(
    inputs: &SequenceInputs<'_>,
    lookup: &Xslt10KeyLookup,
    candidates: Vec<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        super::failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "key() requires a source document",
        )
    })?;
    let lookup_name =
        resolve_lookup_name(inputs, &lookup.name, &lookup.location, variables, control)?;
    if !control.key_index_cache_enabled() {
        let mut grouped = Vec::new();
        for candidate in candidates {
            let first = select(inputs, lookup, Some(candidate), variables, control)?
                .first()
                .copied();
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if first.is_none_or(|node| node == candidate) {
                grouped.push(candidate);
            }
        }
        return Ok(grouped);
    }
    ensure_index(inputs, source, &lookup_name, &lookup.location, control)?;
    let indexes = inputs.key_indexes.borrow();
    let index = indexes
        .indexes
        .get(lookup_name.as_ref())
        .expect("key index was inserted before grouping lookup");

    let mut grouped = Vec::new();
    for candidate in candidates {
        let lookup_values =
            evaluate_lookup_value(inputs, source, candidate, &lookup.value, variables, control)?;
        let first = lookup_values
            .iter()
            .filter_map(|value| index.get(value).and_then(|nodes| nodes.first()).copied())
            .min_by_key(|node| source.document_order(*node));
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if first.is_none_or(|node| node == candidate) {
            grouped.push(candidate);
        }
    }
    Ok(grouped)
}

fn ensure_index(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    lookup_name: &ExpandedName,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if inputs
        .key_indexes
        .borrow()
        .indexes
        .contains_key(lookup_name)
    {
        control.observe_key_index_cache_hit();
        return Ok(());
    }
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
            location.clone(),
            format!("key() refers to an undeclared key: {}", lookup_name.local),
        ));
    }

    let mut candidates = Vec::new();
    collect_source_nodes(source, source.document_node(), &mut candidates);
    let match_variables = BTreeMap::new();
    let mut index: BTreeMap<String, Vec<NodeId>> = BTreeMap::new();
    for candidate in candidates {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
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
            if !matches_pattern(
                inputs.program.matched_templates.len() + *definition_index,
                &definition.match_pattern,
                &selection,
                control,
            )? {
                continue;
            }
            for value in evaluate_use(
                source,
                candidate,
                &definition.use_expression,
                inputs.request_id,
                control,
            )? {
                let nodes = index.entry(value).or_default();
                if nodes.last().copied() != Some(candidate) {
                    nodes.push(candidate);
                }
            }
        }
    }
    let retained_bytes = index.iter().fold(0usize, |bytes, (value, nodes)| {
        bytes
            .saturating_add(value.capacity())
            .saturating_add(nodes.capacity().saturating_mul(size_of::<NodeId>()))
    });
    inputs
        .key_indexes
        .borrow_mut()
        .indexes
        .insert(lookup_name.clone(), index);
    control.observe_key_index_cache_build(retained_bytes);
    Ok(())
}

fn select_reference(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    lookup_name: &ExpandedName,
    lookup_values: &[String],
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
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
            location.clone(),
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
            if matches_pattern(
                inputs.program.matched_templates.len() + *definition_index,
                &definition.match_pattern,
                &selection,
                control,
            )? && evaluate_use(
                source,
                candidate,
                &definition.use_expression,
                inputs.request_id,
                control,
            )?
            .iter()
            .any(|value| lookup_values.contains(value))
            {
                selected.push(candidate);
                break;
            }
        }
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
            if let Some(values) = variables.detached_source_node_strings(name) {
                return Ok(values.clone());
            }
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
        Xslt10KeyValue::Concat(expression) => super::value_evaluator::evaluate_xslt10_concat(
            inputs,
            Some(context),
            expression,
            variables,
            control,
        )
        .map(|value| vec![value]),
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
        KeyUseExpression::Xslt10Concat(expression) => {
            evaluate_key_concat(source, candidate, expression, request_id, control)
                .map(|value| vec![value])
        }
    }
}

fn evaluate_key_concat(
    source: &Document,
    candidate: NodeId,
    expression: &Xslt10ConcatExpression,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut value = String::new();
    for part in &expression.parts {
        match part {
            Xslt10ConcatPart::Literal(literal) => value.push_str(literal),
            Xslt10ConcatPart::Path(path) => {
                let selected = evaluate_location_path_controlled(source, candidate, path, control)
                    .map_err(|failure| control_failure(failure, request_id))?;
                if let Some(node) = selected.first().copied() {
                    value.push_str(
                        &source
                            .string_value_controlled(node, control)
                            .map_err(|failure| control_failure(failure, request_id))?,
                    );
                }
            }
            Xslt10ConcatPart::Variable(_)
            | Xslt10ConcatPart::VariablePosition { .. }
            | Xslt10ConcatPart::SumPath(_) => {
                unreachable!("key concat compiler admits only literal and path parts")
            }
        }
    }
    Ok(value)
}

fn collect_source_nodes(source: &Document, node: NodeId, nodes: &mut Vec<NodeId>) {
    nodes.push(node);
    nodes.extend_from_slice(source.attributes(node));
    for child in source.children(node) {
        collect_source_nodes(source, *child, nodes);
    }
}
