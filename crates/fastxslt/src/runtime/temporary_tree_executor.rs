use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{Instruction, MatchPattern, MatchedTemplate};

use super::result_tree::ResultNode;
use super::runtime_context::{
    InvocationParameter, SequenceInputs, TemporaryNodeKind, TemporaryTree, bind_template_parameters,
};
use super::runtime_failure::{ExecutionFailure, FailureCategory, control_failure, failure_at};
use super::template_selector::accepts_mode as template_accepts_mode;
use super::{SequenceContext, TemporaryFocus, charge_xslt_instruction, execute_sequence};

pub(super) fn apply_temporary_template(
    inputs: &SequenceInputs<'_>,
    tree: &TemporaryTree,
    node: usize,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    charge_xslt_instruction(control, inputs.request_id)?;
    let temporary = &tree.nodes[node];
    let template = select_temporary_template(inputs, tree, node, mode, control)?;
    if let Some((template_index, template)) = template {
        if matches!(
            template.template.body.as_slice(),
            [Instruction::Copy { .. }]
        ) {
            return Ok(vec![copy_temporary_node(
                tree,
                node,
                inputs.request_id,
                control,
            )?]);
        }
        let variables =
            bind_template_parameters(&template.template, parameters, &inputs.globals.atomics);
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext::for_temporary_template(
                TemporaryFocus::Node(tree, node),
                mode,
                template_index,
            ),
            &variables,
            control,
        );
    }
    if let TemporaryNodeKind::Text(value) = &temporary.kind {
        return copy_temporary_text(value, inputs.request_id, control);
    }
    let mut result = Vec::new();
    for child in &temporary.children {
        result.extend(apply_temporary_template(
            inputs, tree, *child, mode, parameters, control,
        )?);
    }
    Ok(result)
}

fn select_temporary_template<'a>(
    inputs: &'a SequenceInputs<'_>,
    tree: &TemporaryTree,
    node: usize,
    mode: Option<&str>,
    control: &mut InvocationControl,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let mut selected = None;
    let mut selected_rank = None;
    let mut ambiguous = false;
    for (index, candidate) in inputs.program.matched_templates.iter().enumerate() {
        if !template_accepts_mode(&candidate.modes, mode)
            || !temporary_matches(tree, node, &candidate.pattern, inputs.request_id, control)?
        {
            continue;
        }
        let rank = (candidate.import_precedence, candidate.priority);
        if selected_rank.is_none_or(|current| rank > current) {
            selected = Some((index, candidate));
            selected_rank = Some(rank);
            ambiguous = false;
        } else if selected_rank == Some(rank) {
            selected = Some((index, candidate));
            ambiguous = true;
        }
    }
    if inputs.multiple_match_policy == super::MultipleMatchPolicy::Error && ambiguous {
        let (_, selected) = selected.expect("an ambiguous temporary rank has a template");
        return Err(failure_at(
            "XTDE0540",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            selected.template.location.clone(),
            "more than one temporary-tree template rule matches at the highest import precedence and priority",
        ));
    }
    Ok(selected)
}

pub(super) fn apply_temporary_roots(
    inputs: &SequenceInputs<'_>,
    tree: &TemporaryTree,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    charge_xslt_instruction(control, inputs.request_id)?;
    if let Some((template_index, template)) = inputs
        .program
        .matched_templates
        .iter()
        .enumerate()
        .rev()
        .find(|(_, template)| {
            template_accepts_mode(&template.modes, mode)
                && template.pattern == MatchPattern::Document
        })
    {
        let variables =
            bind_template_parameters(&template.template, parameters, &inputs.globals.atomics);
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext::for_temporary_template(
                TemporaryFocus::Document(tree),
                mode,
                template_index,
            ),
            &variables,
            control,
        );
    }
    apply_temporary_builtin(
        inputs,
        TemporaryFocus::Document(tree),
        mode,
        parameters,
        control,
    )
}

pub(super) fn apply_temporary_path(
    inputs: &SequenceInputs<'_>,
    tree: &TemporaryTree,
    steps: &[ExpandedName],
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (first, remaining) = steps
        .split_first()
        .expect("compiled temporary paths contain at least one step");
    let mut selected = Vec::new();
    for root in &tree.roots {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if matches!(
            &tree.nodes[*root].kind,
            TemporaryNodeKind::Element { name, .. } if name == first
        ) {
            selected.push(*root);
        }
    }
    for step in remaining {
        let mut next = Vec::new();
        for parent in selected {
            for child in &tree.nodes[parent].children {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if matches!(
                    &tree.nodes[*child].kind,
                    TemporaryNodeKind::Element { name, .. } if name == step
                ) {
                    next.push(*child);
                }
            }
        }
        selected = next;
    }
    let mut result = Vec::new();
    for node in selected {
        result.extend(apply_temporary_template(
            inputs, tree, node, mode, parameters, control,
        )?);
    }
    Ok(result)
}

pub(super) fn apply_temporary_next(
    inputs: &SequenceInputs<'_>,
    focus: TemporaryFocus<'_>,
    mode: Option<&str>,
    current_index: usize,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some((next_index, template)) =
        select_next_temporary_template(inputs, focus, mode, current_index, control)?
    {
        let variables =
            bind_template_parameters(&template.template, parameters, &inputs.globals.atomics);
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext::for_temporary_template(focus, mode, next_index),
            &variables,
            control,
        );
    }
    apply_temporary_builtin(inputs, focus, mode, parameters, control)
}

fn select_next_temporary_template<'a>(
    inputs: &'a SequenceInputs<'_>,
    focus: TemporaryFocus<'_>,
    mode: Option<&str>,
    current_index: usize,
    control: &mut InvocationControl,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let current = &inputs.program.matched_templates[current_index];
    let current_rank = (current.import_precedence, current.priority, current_index);
    let mut selected = None;
    let mut selected_rank = None;
    let mut ambiguous = false;
    for (index, candidate) in inputs.program.matched_templates.iter().enumerate() {
        let rank = (candidate.import_precedence, candidate.priority, index);
        if rank >= current_rank
            || !template_accepts_mode(&candidate.modes, mode)
            || !temporary_focus_matches(focus, &candidate.pattern, inputs.request_id, control)?
        {
            continue;
        }
        let semantic_rank = (candidate.import_precedence, candidate.priority);
        if selected_rank.is_none_or(|current| semantic_rank > current) {
            selected = Some((index, candidate));
            selected_rank = Some(semantic_rank);
            ambiguous = false;
        } else if selected_rank == Some(semantic_rank) {
            selected = Some((index, candidate));
            ambiguous = true;
        }
    }
    if inputs.multiple_match_policy == super::MultipleMatchPolicy::Error && ambiguous {
        let (_, selected) = selected.expect("an ambiguous temporary next rank has a template");
        return Err(failure_at(
            "XTDE0540",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            selected.template.location.clone(),
            "more than one temporary-tree next-match rule has the highest eligible rank",
        ));
    }
    Ok(selected)
}

fn temporary_focus_matches(
    focus: TemporaryFocus<'_>,
    pattern: &MatchPattern,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    match focus {
        TemporaryFocus::Document(_) => Ok(pattern == &MatchPattern::Document),
        TemporaryFocus::Node(tree, node) => {
            temporary_matches(tree, node, pattern, request_id, control)
        }
    }
}

pub(super) fn apply_temporary_builtin(
    inputs: &SequenceInputs<'_>,
    focus: TemporaryFocus<'_>,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let mut result = Vec::new();
    let (tree, children) = match focus {
        TemporaryFocus::Document(tree) => (tree, tree.roots.as_slice()),
        TemporaryFocus::Node(tree, node) => (tree, tree.nodes[node].children.as_slice()),
    };
    for root in children {
        result.extend(apply_temporary_template(
            inputs, tree, *root, mode, parameters, control,
        )?);
    }
    Ok(result)
}

fn copy_temporary_node(
    tree: &TemporaryTree,
    node: usize,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let node = &tree.nodes[node];
    if let TemporaryNodeKind::Text(value) = &node.kind {
        control
            .charge(WorkDomain::ResultTextByte, value.len())
            .map_err(|failure| control_failure(failure, request_id))?;
        return Ok(ResultNode::Text(value.clone()));
    }
    let TemporaryNodeKind::Element { name, namespaces } = &node.kind else {
        unreachable!("temporary node kinds are exhausted")
    };
    let children = node
        .children
        .iter()
        .map(|child| copy_temporary_node(tree, *child, request_id, control))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResultNode::Element {
        name: name.clone(),
        namespaces: namespaces.clone(),
        attributes: Vec::new(),
        children,
    })
}

fn temporary_matches(
    tree: &TemporaryTree,
    node: usize,
    pattern: &MatchPattern,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let kind = &tree.nodes[node].kind;
    let matched = match (kind, pattern) {
        (TemporaryNodeKind::Element { name, .. }, MatchPattern::Element(expected)) => {
            name == expected
        }
        (TemporaryNodeKind::Element { name, .. }, MatchPattern::ElementLocal(local)) => {
            name.local == *local
        }
        (TemporaryNodeKind::Element { .. }, MatchPattern::AnyElement | MatchPattern::AnyNode)
        | (TemporaryNodeKind::Text(_), MatchPattern::Text | MatchPattern::AnyNode) => true,
        (_, MatchPattern::QualifiedElementPathAlternatives(alternatives)) => {
            return matches_temporary_path_alternatives(
                tree,
                node,
                alternatives,
                request_id,
                control,
            );
        }
        _ => false,
    };
    Ok(matched)
}

fn matches_temporary_path_alternatives(
    tree: &TemporaryTree,
    node: usize,
    alternatives: &[Vec<ExpandedName>],
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    for path in alternatives {
        let mut current = Some(node);
        let mut matches = true;
        for expected in path.iter().rev() {
            let Some(candidate) = current else {
                matches = false;
                break;
            };
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            if !matches!(
                &tree.nodes[candidate].kind,
                TemporaryNodeKind::Element { name, .. } if name == expected
            ) {
                matches = false;
                break;
            }
            current = tree.nodes[candidate].parent;
        }
        if matches {
            return Ok(true);
        }
    }
    Ok(false)
}

fn copy_temporary_text(
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    control
        .charge(WorkDomain::ResultTextByte, value.len())
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(vec![ResultNode::Text(value.to_owned())])
}
