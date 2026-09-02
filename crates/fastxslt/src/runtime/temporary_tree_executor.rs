use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{
    Instruction, LiteralAttribute, MatchPattern, MatchedTemplate, OnNoMatchPolicy,
};

use super::result_tree::ResultNode;
use super::runtime_context::{
    InvocationParameter, RuntimeVariables, SequenceInputs, TemporaryNodeKind, TemporaryTree,
    bind_template_parameters,
};
use super::runtime_failure::{
    ExecutionFailure, FailureCategory, control_failure, failure, failure_at,
};
use super::template_selector::accepts_mode as template_accepts_mode;
use super::{
    SequenceContext, SequenceFocus, TemporaryFocus, charge_xslt_instruction, execute_sequence,
    materialize_literal_attributes,
};

pub(super) fn apply_temporary_template(
    inputs: &SequenceInputs<'_>,
    tree: &TemporaryTree,
    node: usize,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    sequence_focus: SequenceFocus,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    charge_xslt_instruction(control, inputs.request_id)?;
    let template = select_temporary_template(inputs, tree, node, mode, control)?;
    if let Some((template_index, template)) = template {
        return execute_selected_temporary_template(
            inputs,
            tree,
            node,
            mode,
            parameters,
            sequence_focus,
            template_index,
            template,
            control,
        );
    }
    apply_temporary_builtin(
        inputs,
        TemporaryFocus::Node(tree, node),
        mode,
        parameters,
        control,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_selected_temporary_template(
    inputs: &SequenceInputs<'_>,
    tree: &TemporaryTree,
    node: usize,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    sequence_focus: SequenceFocus,
    template_index: usize,
    template: &MatchedTemplate,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let variables = bind_template_parameters(
        &template.template,
        parameters,
        &inputs.globals.atomics,
        inputs.complete_atomic_frame_clones,
    );
    execute_sequence(
        inputs,
        &template.template.body,
        SequenceContext::for_temporary_template(
            TemporaryFocus::Node(tree, node),
            mode,
            template_index,
            sequence_focus,
        ),
        &variables,
        control,
    )
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
        control
            .charge_template_candidate()
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
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
    let mut document_template = None;
    for (template_index, template) in inputs.program.matched_templates.iter().enumerate().rev() {
        control
            .charge_template_candidate()
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if template_accepts_mode(&template.modes, mode)
            && template.pattern == MatchPattern::Document
        {
            document_template = Some((template_index, template));
            break;
        }
    }
    if let Some((template_index, template)) = document_template {
        let variables = bind_template_parameters(
            &template.template,
            parameters,
            &inputs.globals.atomics,
            inputs.complete_atomic_frame_clones,
        );
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext::for_temporary_template(
                TemporaryFocus::Document(tree),
                mode,
                template_index,
                SequenceFocus {
                    position: 1,
                    size: 1,
                },
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
    let focus_size = selected.len();
    for (offset, node) in selected.into_iter().enumerate() {
        result.extend(apply_temporary_template(
            inputs,
            tree,
            node,
            mode,
            parameters,
            SequenceFocus {
                position: offset + 1,
                size: focus_size,
            },
            control,
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
    sequence_focus: SequenceFocus,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some((next_index, template)) =
        select_next_temporary_template(inputs, focus, mode, current_index, control)?
    {
        let variables = bind_template_parameters(
            &template.template,
            parameters,
            &inputs.globals.atomics,
            inputs.complete_atomic_frame_clones,
        );
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext::for_temporary_template(focus, mode, next_index, sequence_focus),
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
        control
            .charge_template_candidate()
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
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
    let policy = inputs
        .program
        .mode_on_no_match
        .iter()
        .find(|policy| policy.name.as_deref() == mode);
    if let Some(policy) = policy {
        match policy.policy {
            OnNoMatchPolicy::Fail => {
                return Err(failure_at(
                    "XTDE0555",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    policy.location.clone(),
                    "the active mode's on-no-match='fail' policy rejected an unmatched temporary node",
                ));
            }
            OnNoMatchPolicy::ShallowCopy => {
                return copy_temporary_focus(inputs, focus, mode, parameters, control);
            }
            OnNoMatchPolicy::ShallowSkip => {
                return apply_temporary_descendants(inputs, focus, mode, parameters, true, control);
            }
            OnNoMatchPolicy::TextOnlyCopy => {}
        }
    }
    match focus {
        TemporaryFocus::Node(tree, node) => match &tree.nodes[node].kind {
            TemporaryNodeKind::Text(value) | TemporaryNodeKind::Attribute { value, .. } => {
                copy_temporary_text(value, inputs.request_id, control)
            }
            TemporaryNodeKind::Element { .. } => {
                apply_temporary_children(inputs, focus, mode, parameters, control)
            }
            TemporaryNodeKind::Comment(_) | TemporaryNodeKind::ProcessingInstruction { .. } => {
                Ok(Vec::new())
            }
        },
        TemporaryFocus::Document(_) => {
            apply_temporary_children(inputs, focus, mode, parameters, control)
        }
    }
}

fn apply_temporary_descendants(
    inputs: &SequenceInputs<'_>,
    focus: TemporaryFocus<'_>,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    include_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (tree, attributes, children) = match focus {
        TemporaryFocus::Document(tree) => (tree, &[][..], tree.roots.as_slice()),
        TemporaryFocus::Node(tree, node) => match &tree.nodes[node].kind {
            TemporaryNodeKind::Element { attributes, .. } => (
                tree,
                attributes.as_slice(),
                tree.nodes[node].children.as_slice(),
            ),
            _ => return Ok(Vec::new()),
        },
    };
    let attributes = if include_attributes { attributes } else { &[] };
    let focus_size = attributes.len() + children.len();
    let mut result = Vec::new();
    for (offset, selected) in attributes.iter().chain(children).copied().enumerate() {
        result.extend(apply_temporary_template(
            inputs,
            tree,
            selected,
            mode,
            parameters,
            SequenceFocus {
                position: offset + 1,
                size: focus_size,
            },
            control,
        )?);
    }
    Ok(result)
}

fn apply_temporary_children(
    inputs: &SequenceInputs<'_>,
    focus: TemporaryFocus<'_>,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let child_count = match focus {
        TemporaryFocus::Document(tree) => tree.roots.len(),
        TemporaryFocus::Node(tree, node) => tree.nodes[node].children.len(),
    };
    apply_temporary_children_with_focus(inputs, focus, mode, parameters, 0, child_count, control)
}

#[allow(clippy::too_many_arguments)]
fn apply_temporary_children_with_focus(
    inputs: &SequenceInputs<'_>,
    focus: TemporaryFocus<'_>,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    position_offset: usize,
    focus_size: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (tree, children) = match focus {
        TemporaryFocus::Document(tree) => (tree, tree.roots.as_slice()),
        TemporaryFocus::Node(tree, node) => (tree, tree.nodes[node].children.as_slice()),
    };
    let mut result = Vec::new();
    for (offset, child) in children.iter().copied().enumerate() {
        result.extend(apply_temporary_template(
            inputs,
            tree,
            child,
            mode,
            parameters,
            SequenceFocus {
                position: position_offset + offset + 1,
                size: focus_size,
            },
            control,
        )?);
    }
    Ok(result)
}

fn copy_temporary_focus(
    inputs: &SequenceInputs<'_>,
    focus: TemporaryFocus<'_>,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let TemporaryFocus::Node(tree, node) = focus else {
        return apply_temporary_children(inputs, focus, mode, parameters, control);
    };
    match &tree.nodes[node].kind {
        TemporaryNodeKind::Text(value) => copy_temporary_text(value, inputs.request_id, control),
        TemporaryNodeKind::Comment(value) => {
            copy_temporary_comment(value, inputs.request_id, control)
        }
        TemporaryNodeKind::ProcessingInstruction { target, value } => {
            copy_temporary_processing_instruction(target, value, inputs.request_id, control)
        }
        TemporaryNodeKind::Element {
            name,
            namespaces,
            attributes,
        } => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let (result_attributes, attribute_results) = shallow_copy_temporary_attributes(
                inputs,
                tree,
                attributes,
                mode,
                parameters,
                tree.nodes[node].children.len(),
                control,
            )?;
            let mut children = attribute_results;
            let focus_size = attributes.len() + tree.nodes[node].children.len();
            children.extend(apply_temporary_children_with_focus(
                inputs,
                focus,
                mode,
                parameters,
                attributes.len(),
                focus_size,
                control,
            )?);
            Ok(vec![ResultNode::Element {
                name: name.clone(),
                namespaces: namespaces.clone(),
                attributes: result_attributes,
                children,
            }])
        }
        TemporaryNodeKind::Attribute { name, value } => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(vec![ResultNode::PendingAttribute(
                super::result_tree::ResultAttribute {
                    name: name.clone(),
                    value: value.clone(),
                },
            )])
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn shallow_copy_temporary_attributes(
    inputs: &SequenceInputs<'_>,
    tree: &TemporaryTree,
    attributes: &[usize],
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    child_count: usize,
    control: &mut InvocationControl,
) -> Result<(Vec<super::result_tree::ResultAttribute>, Vec<ResultNode>), ExecutionFailure> {
    let mut copied = Vec::new();
    let mut generated = Vec::new();
    let focus_size = attributes.len() + child_count;
    for (offset, node) in attributes.iter().copied().enumerate() {
        charge_xslt_instruction(control, inputs.request_id)?;
        if let Some((template_index, template)) =
            select_temporary_template(inputs, tree, node, mode, control)?
        {
            generated.extend(execute_selected_temporary_template(
                inputs,
                tree,
                node,
                mode,
                parameters,
                SequenceFocus {
                    position: offset + 1,
                    size: focus_size,
                },
                template_index,
                template,
                control,
            )?);
            continue;
        }
        let TemporaryNodeKind::Attribute { name, value } = &tree.nodes[node].kind else {
            unreachable!("temporary element attribute indexes identify attributes")
        };
        control
            .charge(WorkDomain::ResultNode, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        copied.push(super::result_tree::ResultAttribute {
            name: name.clone(),
            value: value.clone(),
        });
    }
    Ok((copied, generated))
}

pub(super) fn execute_temporary_copy(
    inputs: &SequenceInputs<'_>,
    attributes: &[LiteralAttribute],
    body: &[Instruction],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let focus = execution
        .temporary_focus
        .expect("temporary xsl:copy has a temporary focus");
    let TemporaryFocus::Node(tree, node) = focus else {
        return execute_sequence(inputs, body, execution, variables, control);
    };
    let temporary = &tree.nodes[node];
    match &temporary.kind {
        TemporaryNodeKind::Text(value) => copy_temporary_text(value, inputs.request_id, control),
        TemporaryNodeKind::Comment(value) => {
            copy_temporary_comment(value, inputs.request_id, control)
        }
        TemporaryNodeKind::ProcessingInstruction { target, value } => {
            copy_temporary_processing_instruction(target, value, inputs.request_id, control)
        }
        TemporaryNodeKind::Element {
            name, namespaces, ..
        } => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(vec![ResultNode::Element {
                name: name.clone(),
                namespaces: namespaces.clone(),
                attributes: materialize_literal_attributes(
                    attributes,
                    variables,
                    execution.focus_position,
                    execution.focus_size,
                    inputs.request_id,
                    control,
                )?,
                children: execute_sequence(inputs, body, execution, variables, control)?,
            }])
        }
        TemporaryNodeKind::Attribute { .. } => Err(failure(
            "FXRT1012",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "xsl:copy of a temporary attribute is outside the private result-tree slice",
        )),
    }
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
        (TemporaryNodeKind::Element { name, .. }, MatchPattern::Element(expected))
        | (TemporaryNodeKind::Attribute { name, .. }, MatchPattern::Attribute(expected)) => {
            name == expected
        }
        (TemporaryNodeKind::Element { name, .. }, MatchPattern::ElementLocal(local)) => {
            name.local == *local
        }
        (
            TemporaryNodeKind::Attribute { .. },
            MatchPattern::AnyAttribute | MatchPattern::AnyNode,
        )
        | (TemporaryNodeKind::Element { .. }, MatchPattern::AnyElement | MatchPattern::AnyNode)
        | (TemporaryNodeKind::Text(_), MatchPattern::Text | MatchPattern::AnyNode)
        | (TemporaryNodeKind::Comment(_), MatchPattern::Comment | MatchPattern::AnyNode)
        | (
            TemporaryNodeKind::ProcessingInstruction { .. },
            MatchPattern::ProcessingInstruction | MatchPattern::AnyNode,
        ) => true,
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

fn copy_temporary_comment(
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
    Ok(vec![ResultNode::Comment(value.to_owned())])
}

fn copy_temporary_processing_instruction(
    target: &str,
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    control
        .charge(WorkDomain::ResultTextByte, target.len() + value.len())
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(vec![ResultNode::ProcessingInstruction {
        target: target.to_owned(),
        value: value.to_owned(),
    }])
}
