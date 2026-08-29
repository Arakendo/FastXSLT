use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xslt::golden_semantics_experiment::{Instruction, MatchPattern};

use super::result_tree::ResultNode;
use super::runtime_context::{
    InvocationParameter, SequenceInputs, TemporaryTree, bind_template_parameters,
};
use super::runtime_failure::{ExecutionFailure, control_failure};
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
    let template = inputs
        .program
        .matched_templates
        .iter()
        .enumerate()
        .rev()
        .find(|(_, template)| {
            template_accepts_mode(&template.modes, mode)
                && match &template.pattern {
                    MatchPattern::Element(name) => &temporary.name == name,
                    MatchPattern::ElementLocal(local) => temporary.name.local == *local,
                    MatchPattern::AnyElement | MatchPattern::AnyNode => true,
                    _ => false,
                }
        });
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
                TemporaryFocus::Element(tree, node),
                mode,
                template_index,
            ),
            &variables,
            control,
        );
    }
    let mut result = Vec::new();
    for child in &temporary.children {
        result.extend(apply_temporary_template(
            inputs, tree, *child, mode, parameters, control,
        )?);
    }
    Ok(result)
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
        TemporaryFocus::Element(tree, node) => (tree, tree.nodes[node].children.as_slice()),
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
    let children = node
        .children
        .iter()
        .map(|child| copy_temporary_node(tree, *child, request_id, control))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResultNode::Element {
        name: node.name.clone(),
        namespaces: node.namespaces.clone(),
        attributes: Vec::new(),
        children,
    })
}
