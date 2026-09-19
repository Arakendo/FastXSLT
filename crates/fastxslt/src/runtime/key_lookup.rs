//! Charged XSLT 1.0 key lookup reference selection.

use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{KeyUseExpression, Xslt10KeyLookup};

use super::runtime_context::{SequenceInputs, required_source_context};
use super::template_selector::{TemplateSelectionContext, matches_pattern};
use super::value_evaluator::xslt10_number_lexical;
use super::{ExecutionFailure, FailureCategory, control_failure, failure_at};

pub(super) fn select(
    inputs: &SequenceInputs<'_>,
    lookup: &Xslt10KeyLookup,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let (source, _) = required_source_context(inputs, context)?;
    let definitions = inputs
        .program
        .key_definitions
        .iter()
        .enumerate()
        .filter(|(_, definition)| definition.name == lookup.name)
        .collect::<Vec<_>>();
    if definitions.is_empty() {
        return Err(failure_at(
            "XTDE1260",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            lookup.location.clone(),
            format!("key() refers to an undeclared key: {}", lookup.name.local),
        ));
    }

    let mut candidates = Vec::new();
    collect_source_nodes(source, source.document_node(), &mut candidates);
    let variables = BTreeMap::new();
    let mut selected = Vec::new();
    for candidate in candidates {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let mut matches = false;
        for (definition_index, definition) in &definitions {
            let selection = TemplateSelectionContext {
                source,
                node: candidate,
                mode: None,
                variables: &variables,
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
                inputs,
                source,
                candidate,
                &definition.use_expression,
                control,
            )?
            .into_iter()
            .any(|lexical| lexical == lookup.value)
            {
                matches = true;
                break;
            }
        }
        if matches {
            selected.push(candidate);
        }
    }

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

fn evaluate_use(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    candidate: NodeId,
    expression: &KeyUseExpression,
    control: &mut InvocationControl,
) -> Result<Vec<String>, ExecutionFailure> {
    match expression {
        KeyUseExpression::LiteralString(value) => Ok(vec![value.clone()]),
        KeyUseExpression::LocationPath(path) => {
            let selected = evaluate_location_path_controlled(source, candidate, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            selected
                .into_iter()
                .map(|node| {
                    source
                        .string_value_controlled(node, control)
                        .map_err(|failure| control_failure(failure, inputs.request_id))
                })
                .collect()
        }
        KeyUseExpression::NumberPath(path) => {
            let selected = evaluate_location_path_controlled(source, candidate, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let lexical = if let Some(node) = selected.first().copied() {
                source
                    .string_value_controlled(node, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?
            } else {
                String::new()
            };
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
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
