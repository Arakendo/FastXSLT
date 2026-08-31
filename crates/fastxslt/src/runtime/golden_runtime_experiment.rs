use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits};
use crate::xpath::castable_experiment::{CastEvaluationFailure, CastExpression, evaluate_cast};
use crate::xpath::for_distinct_values_experiment::{
    ForDistinctValuesExpression, evaluate as evaluate_for_distinct_values,
};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, BooleanExpression, Instruction, NodeTest, SequenceItemExpression,
    StylesheetProgram, TemplateArgument,
};

#[path = "resource_compiler.rs"]
mod resource_compiler;
#[path = "result_tree.rs"]
mod result_tree;
#[path = "runtime_context.rs"]
mod runtime_context;
#[path = "runtime_failure.rs"]
mod runtime_failure;
mod serialization;
#[path = "stylesheet_dependency_loader.rs"]
mod stylesheet_dependency_loader;
#[path = "template_selector.rs"]
mod template_selector;
#[path = "temporary_tree_executor.rs"]
mod temporary_tree_executor;
#[cfg(test)]
#[path = "transform_set_experiment.rs"]
mod transform_set_experiment;
#[path = "value_evaluator.rs"]
mod value_evaluator;
#[path = "variable_filtered_path.rs"]
mod variable_filtered_path;

#[cfg(test)]
pub(super) use resource_compiler::compile_resource;
pub(super) use resource_compiler::compile_resource_with_denied;
use result_tree::{
    ResultAttribute, ResultNode, materialize_computed_attributes, materialize_literal_attributes,
};
use runtime_context::{
    InvocationParameter, RuntimeVariables, SequenceInputs, TemporaryTree, bind_template_parameters,
    evaluate_template_arguments, materialize_global_defaults, materialize_temporary_tree,
    required_source_context,
};
pub(super) use runtime_failure::ExecutionFailure;
use runtime_failure::{FailureCategory, control_failure, failure, failure_at};
pub(super) use serialization::serialize_xml;
#[cfg(test)]
pub(super) use serialization::serialize_xml_bytes;
use template_selector::{
    TemplateSelectionContext, select_imported_template, select_next_template,
    select_template_with_index,
};
use temporary_tree_executor::{apply_temporary_roots, apply_temporary_template};
#[cfg(test)]
use transform_set_experiment::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, execute_transform_set,
};
use value_evaluator::execute_value_of;

const XML_LIMITS: ParseLimits = ParseLimits {
    max_events: 1_024,
    max_depth: 64,
};
const MAX_NAMED_TEMPLATE_CALL_DEPTH: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MultipleMatchPolicy {
    UseLast,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticResult {
    children: Vec<ResultNode>,
}

pub(super) fn execute_program(
    program: &StylesheetProgram,
    source: &Document,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    execute_program_with_parameters(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        request_id,
        control,
    )
}

fn execute_program_with_parameters(
    program: &StylesheetProgram,
    source: &Document,
    parameters: &BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let globals =
        materialize_global_defaults(program, Some(source), parameters, request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source: Some(source),
        request_id,
        globals: &globals,
        multiple_match_policy,
    };
    let children = if let Some(root_template) = program
        .root_template
        .as_ref()
        .filter(|_| program.root_template_modes.is_empty())
    {
        let variables = bind_template_parameters(root_template, &BTreeMap::new(), &globals.atomics);
        execute_sequence(
            &inputs,
            &root_template.body,
            SequenceContext::new(Some(source.document_node()), None),
            &variables,
            control,
        )?
    } else {
        apply_template(
            &inputs,
            source.document_node(),
            None,
            &BTreeMap::new(),
            control,
        )?
    };
    Ok(SemanticResult { children })
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct InitialModeInvocation<'a> {
    program: &'a StylesheetProgram,
    source: &'a Document,
    initial_node: NodeId,
    name: &'a str,
    parameters: &'a BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &'a str,
}

#[cfg(test)]
fn execute_initial_mode(
    invocation: InitialModeInvocation<'_>,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let InitialModeInvocation {
        program,
        source,
        initial_node,
        name,
        parameters,
        multiple_match_policy,
        request_id,
    } = invocation;
    if !program_has_mode(program, name) {
        return Err(failure(
            "XTDE0045",
            FailureCategory::Invalid,
            Some(request_id),
            format!("unknown initial mode: {name}"),
        ));
    }
    let globals =
        materialize_global_defaults(program, Some(source), parameters, request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source: Some(source),
        request_id,
        globals: &globals,
        multiple_match_policy,
    };
    let children = if initial_node == source.document_node()
        && program.root_template_modes.iter().any(|mode| mode == name)
    {
        let template = program
            .root_template
            .as_ref()
            .expect("a compiled root initial mode has a root template");
        let variables = bind_template_parameters(template, parameters, &globals.atomics);
        execute_sequence(
            &inputs,
            &template.body,
            SequenceContext::new(Some(initial_node), Some(name)),
            &variables,
            control,
        )?
    } else {
        apply_initial_mode_template(&inputs, initial_node, name, parameters, control)?
    };
    Ok(SemanticResult { children })
}

#[cfg(test)]
fn apply_initial_mode_template(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: &str,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("initial-mode template dispatch requires a source document");
    charge_xslt_instruction(control, inputs.request_id)?;
    if let Some((template_index, template)) = select_template_with_index(
        inputs.program,
        &TemplateSelectionContext {
            source,
            node,
            mode: Some(mode),
            variables: &inputs.globals.atomics,
            request_id: inputs.request_id,
        },
        inputs.multiple_match_policy,
        control,
    )? {
        let variables =
            bind_template_parameters(&template.template, parameters, &inputs.globals.atomics);
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext::for_template(Some(node), Some(mode), template_index),
            &variables,
            control,
        );
    }
    apply_builtin_template(inputs, node, Some(mode), parameters, control)
}

#[cfg(test)]
fn program_has_mode(program: &StylesheetProgram, name: &str) -> bool {
    program.root_template_modes.iter().any(|mode| mode == name)
        || program
            .matched_templates
            .iter()
            .any(|template| template.modes.iter().any(|mode| mode == name))
}

#[cfg(test)]
fn execute_initial_template(
    program: &StylesheetProgram,
    name: &str,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let template = program
        .named_templates
        .iter()
        .find(|template| template.name == name)
        .expect("initial-template entries are validated during request admission");
    if !template.parameters.is_empty() {
        return Err(failure(
            "FXRT1003",
            FailureCategory::Unsupported,
            Some(request_id),
            "initial-template parameters are outside the private invocation-entry slice",
        ));
    }
    let globals =
        materialize_global_defaults(program, None, &BTreeMap::new(), request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source: None,
        request_id,
        globals: &globals,
        multiple_match_policy,
    };
    let children = execute_sequence(
        &inputs,
        &template.template.body,
        SequenceContext::new(None, None),
        &RuntimeVariables::from_atomics(&globals.atomics),
        control,
    )?;
    Ok(SemanticResult { children })
}

#[derive(Clone, Copy)]
struct SequenceContext<'a> {
    node: Option<NodeId>,
    temporary_focus: Option<TemporaryFocus<'a>>,
    current_mode: Option<&'a str>,
    current_template_index: Option<usize>,
    focus_position: usize,
    focus_size: usize,
    call_depth: usize,
}

impl<'a> SequenceContext<'a> {
    fn new(node: Option<NodeId>, current_mode: Option<&'a str>) -> Self {
        Self {
            node,
            temporary_focus: None,
            current_mode,
            current_template_index: None,
            focus_position: 1,
            focus_size: 1,
            call_depth: 0,
        }
    }

    fn for_template(node: Option<NodeId>, current_mode: Option<&'a str>, index: usize) -> Self {
        Self {
            current_template_index: Some(index),
            ..Self::new(node, current_mode)
        }
    }

    fn for_template_at(
        node: NodeId,
        current_mode: Option<&'a str>,
        index: usize,
        focus_position: usize,
        focus_size: usize,
    ) -> Self {
        Self {
            focus_position,
            focus_size,
            ..Self::for_template(Some(node), current_mode, index)
        }
    }

    fn for_temporary_template(
        focus: TemporaryFocus<'a>,
        current_mode: Option<&'a str>,
        index: usize,
    ) -> Self {
        Self {
            temporary_focus: Some(focus),
            current_template_index: Some(index),
            ..Self::new(None, current_mode)
        }
    }
}

#[derive(Clone, Copy)]
enum TemporaryFocus<'a> {
    Document(&'a TemporaryTree),
    Node(&'a TemporaryTree, usize),
}

fn execute_sequence(
    inputs: &SequenceInputs<'_>,
    instructions: &[Instruction],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (mut result, mut scope) = (Vec::new(), variables.clone());
    for instruction in instructions {
        charge_xslt_instruction(control, inputs.request_id)?;
        execute_instruction(
            inputs,
            instruction,
            execution,
            &mut scope,
            &mut result,
            control,
        )?;
    }
    Ok(result)
}

fn execute_instruction(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    scope: &mut RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    match instruction {
        Instruction::LiteralElement { .. } => result.push(execute_literal_element(
            inputs,
            instruction,
            execution,
            scope,
            control,
        )?),
        Instruction::Text { value, .. } => append_text(result, value, inputs.request_id, control)?,
        Instruction::ProcessingInstructionNode { target, value, .. } => result.push(
            construct_processing_instruction(target, value, inputs.request_id, control)?,
        ),
        Instruction::ValueOf {
            select, separator, ..
        } => {
            execute_value_of(
                inputs,
                select,
                separator,
                execution.node,
                scope,
                result,
                control,
            )?;
        }
        Instruction::SequenceNodes { select, .. } => {
            result.extend(execute_sequence_nodes(
                inputs,
                select,
                execution.node,
                control,
            )?);
        }
        Instruction::SequenceItems { select, .. } => {
            result.extend(execute_sequence_items(
                inputs,
                select,
                execution.node,
                scope,
                control,
            )?);
        }
        Instruction::Variable { .. }
        | Instruction::IntegerRangeVariable { .. }
        | Instruction::TemporaryTreeVariable { .. } => {
            execute_binding(inputs, instruction, execution.node, scope, control)?;
        }
        Instruction::ApplyTemplates {
            select,
            mode,
            arguments,
            ..
        } => {
            result.extend(execute_apply_instruction(
                inputs,
                select.as_ref(),
                mode.as_deref(),
                arguments,
                execution,
                scope,
                control,
            )?);
        }
        Instruction::ForEachTemporaryRoot { variable, body, .. } => result.extend(
            execute_for_each_temporary_root(inputs, variable, body, execution, scope, control)?,
        ),
        Instruction::NextMatch { .. } | Instruction::ApplyImports { .. } => result.extend(
            execute_continuation_instruction(inputs, instruction, execution, scope, control)?,
        ),
        Instruction::If { test, body, .. } => {
            result.extend(execute_if(inputs, test, body, execution, scope, control)?);
        }
        Instruction::Choose {
            branches,
            otherwise,
            ..
        } => {
            result.extend(execute_choose(
                inputs, branches, otherwise, execution, scope, control,
            )?);
        }
        Instruction::CallTemplate { .. } => result.extend(execute_call(
            inputs,
            instruction,
            execution,
            scope,
            control,
        )?),
        Instruction::CopyOfCurrent { .. } => {
            result.extend(execute_copy_of_current(inputs, execution.node, control)?);
        }
        Instruction::Copy { .. } => result.extend(execute_copy(
            inputs,
            instruction,
            execution,
            scope,
            control,
        )?),
    }
    Ok(())
}

fn execute_copy_of_current(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, node) = required_source_context(inputs, context)?;
    copy_source_node(source, inputs.request_id, node, control)
}

fn execute_for_each_temporary_root<'a>(
    inputs: &SequenceInputs<'a>,
    variable: &str,
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let tree = variables
        .temporary_trees
        .get(variable)
        .or_else(|| inputs.globals.temporary_trees.get(variable))
        .ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound temporary tree: ${variable}"),
            )
        })?;
    execute_sequence(
        inputs,
        body,
        SequenceContext {
            node: None,
            temporary_focus: Some(TemporaryFocus::Document(tree)),
            ..execution
        },
        variables,
        control,
    )
}

fn execute_continuation_instruction(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match instruction {
        Instruction::NextMatch { arguments, .. } => {
            execute_next_match(inputs, arguments, execution, variables, control)
        }
        Instruction::ApplyImports { arguments, .. } => {
            execute_apply_imports(inputs, arguments, execution, variables, control)
        }
        _ => unreachable!("continuation dispatch receives only continuation instructions"),
    }
}

fn execute_binding(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    context: Option<NodeId>,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    match instruction {
        Instruction::Variable { name, select, .. } => {
            let value = execute_variable_binding(inputs, name, select, context, control)?;
            scope.atomics.insert(name.clone(), value);
        }
        Instruction::IntegerRangeVariable {
            name, start, end, ..
        } => {
            let values = materialize_integer_range(*start, *end, inputs.request_id, control)?;
            scope.atomic_sequences.insert(name.clone(), values);
        }
        Instruction::TemporaryTreeVariable { name, elements, .. } => {
            let tree = materialize_temporary_tree(elements, inputs.request_id, control)?;
            scope.temporary_trees.insert(name.clone(), tree);
        }
        _ => unreachable!("execute_binding receives a variable instruction"),
    }
    Ok(())
}

fn execute_call(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let Instruction::CallTemplate {
        name, arguments, ..
    } = instruction
    else {
        unreachable!("execute_call_instruction receives xsl:call-template")
    };
    execute_named_call(inputs, name, arguments, execution, variables, control)
}

fn execute_copy(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let Instruction::Copy {
        attributes, body, ..
    } = instruction
    else {
        unreachable!("execute_copy_instruction receives xsl:copy")
    };
    execute_source_element_copy(inputs, attributes, body, execution, variables, control)
}

fn execute_source_element_copy(
    inputs: &SequenceInputs<'_>,
    attributes: &[crate::xslt::golden_semantics_experiment::LiteralAttribute],
    body: &[Instruction],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, node) = required_source_context(inputs, execution.node)?;
    match source.kind(node) {
        NodeKind::Document => execute_sequence(inputs, body, execution, variables, control),
        NodeKind::Element => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(vec![ResultNode::Element {
                name: source
                    .name(node)
                    .expect("element context has a name")
                    .clone(),
                namespaces: source.namespace_declarations(node).to_vec(),
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
        NodeKind::Text => {
            let mut copied = Vec::new();
            append_text(
                &mut copied,
                source.value(node).unwrap_or_default(),
                inputs.request_id,
                control,
            )?;
            Ok(copied)
        }
        NodeKind::ProcessingInstruction => Ok(vec![construct_processing_instruction(
            &source
                .name(node)
                .expect("processing-instruction context has a target")
                .local,
            source.value(node).unwrap_or_default(),
            inputs.request_id,
            control,
        )?]),
        NodeKind::Attribute | NodeKind::Comment => Err(failure(
            "FXRT1007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "the selected source node kind is outside the private xsl:copy slice",
        )),
    }
}

fn execute_literal_element(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    let Instruction::LiteralElement {
        name,
        namespaces,
        attributes,
        computed_attributes,
        body,
        ..
    } = instruction
    else {
        unreachable!("execute_literal_element receives a literal element instruction")
    };
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut attributes = materialize_literal_attributes(
        attributes,
        variables,
        execution.focus_position,
        execution.focus_size,
        inputs.request_id,
        control,
    )?;
    attributes.extend(materialize_computed_attributes(
        computed_attributes,
        variables,
        execution.focus_position,
        execution.focus_size,
        inputs.request_id,
        control,
    )?);
    Ok(ResultNode::Element {
        name: name.clone(),
        namespaces: namespaces.clone(),
        attributes,
        children: execute_sequence(inputs, body, execution, variables, control)?,
    })
}

fn execute_apply_instruction(
    inputs: &SequenceInputs<'_>,
    select: Option<&ApplySelection>,
    mode: Option<&str>,
    arguments: &[TemplateArgument],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let requested_mode = match mode {
        Some("#current") => execution.current_mode,
        Some("#default") => None,
        mode => mode,
    };
    let parameters = evaluate_template_arguments(arguments, variables, inputs.request_id)?;
    execute_apply_templates(
        inputs,
        select,
        requested_mode,
        execution,
        &parameters,
        variables,
        control,
    )
}

fn execute_apply_templates(
    inputs: &SequenceInputs<'_>,
    select: Option<&ApplySelection>,
    mode: Option<&str>,
    execution: SequenceContext<'_>,
    parameters: &BTreeMap<String, InvocationParameter>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some(ApplySelection::TemporaryRoot(name)) = select {
        let tree = variables
            .temporary_trees
            .get(name)
            .or_else(|| inputs.globals.temporary_trees.get(name))
            .ok_or_else(|| {
                failure(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    format!("unbound temporary tree: ${name}"),
                )
            })?;
        return apply_temporary_roots(inputs, tree, mode, parameters, control);
    }
    if let Some(ApplySelection::TemporaryPath { variable, steps }) = select {
        let tree = variables
            .temporary_trees
            .get(variable)
            .or_else(|| inputs.globals.temporary_trees.get(variable))
            .ok_or_else(|| {
                failure(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    format!("unbound temporary tree: ${variable}"),
                )
            })?;
        return temporary_tree_executor::apply_temporary_path(
            inputs, tree, steps, mode, parameters, control,
        );
    }
    if let Some(ApplySelection::GlobalTemporaryChildren(name)) = select {
        let tree = inputs.globals.temporary_trees.get(name).ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound temporary tree: ${name}"),
            )
        })?;
        let mut result = Vec::new();
        for node in &tree.roots {
            result.extend(apply_temporary_template(
                inputs, tree, *node, mode, parameters, control,
            )?);
        }
        return Ok(result);
    }
    if select.is_none()
        && let Some(focus) = execution.temporary_focus
    {
        return temporary_tree_executor::apply_temporary_builtin(
            inputs, focus, mode, parameters, control,
        );
    }
    let (_, context) = required_source_context(inputs, execution.node)?;
    let selected = select_apply_nodes(inputs, select, context, &variables.atomics, control)?;
    let mut result = Vec::new();
    let focus_size = selected.len();
    for (offset, node) in selected.into_iter().enumerate() {
        result.extend(apply_template_at(
            inputs,
            node,
            mode,
            parameters,
            offset + 1,
            focus_size,
            control,
        )?);
    }
    Ok(result)
}

fn execute_next_match(
    inputs: &SequenceInputs<'_>,
    arguments: &[TemplateArgument],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let current_index = execution.current_template_index.ok_or_else(|| {
        failure(
            "XTDE0560",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "xsl:next-match requires a current matched template rule",
        )
    })?;
    let parameters = evaluate_template_arguments(arguments, variables, inputs.request_id)?;
    if let Some(focus) = execution.temporary_focus {
        return temporary_tree_executor::apply_temporary_next(
            inputs,
            focus,
            execution.current_mode,
            current_index,
            &parameters,
            control,
        );
    }
    let (source, node) = required_source_context(inputs, execution.node)?;
    if let Some((next_index, template)) = select_next_template(
        inputs.program,
        &TemplateSelectionContext {
            source,
            node,
            mode: execution.current_mode,
            variables: &inputs.globals.atomics,
            request_id: inputs.request_id,
        },
        current_index,
        inputs.multiple_match_policy,
        control,
    )? {
        let variables =
            bind_template_parameters(&template.template, &parameters, &inputs.globals.atomics);
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext {
                current_template_index: Some(next_index),
                ..execution
            },
            &variables,
            control,
        );
    }
    apply_builtin_template(inputs, node, execution.current_mode, &parameters, control)
}

fn execute_apply_imports(
    inputs: &SequenceInputs<'_>,
    arguments: &[TemplateArgument],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let current_index = execution.current_template_index.ok_or_else(|| {
        failure(
            "XTDE0560",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "xsl:apply-imports requires a current matched template rule",
        )
    })?;
    let parameters = evaluate_template_arguments(arguments, variables, inputs.request_id)?;
    if let Some(focus) = execution.temporary_focus {
        return temporary_tree_executor::apply_temporary_builtin(
            inputs,
            focus,
            execution.current_mode,
            &parameters,
            control,
        );
    }
    let (source, node) = required_source_context(inputs, execution.node)?;
    if let Some((next_index, template)) = select_imported_template(
        inputs.program,
        &TemplateSelectionContext {
            source,
            node,
            mode: execution.current_mode,
            variables: &inputs.globals.atomics,
            request_id: inputs.request_id,
        },
        current_index,
        control,
    )? {
        let variables =
            bind_template_parameters(&template.template, &parameters, &inputs.globals.atomics);
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext {
                current_template_index: Some(next_index),
                ..execution
            },
            &variables,
            control,
        );
    }
    apply_builtin_template(inputs, node, execution.current_mode, &parameters, control)
}

fn charge_xslt_instruction(
    control: &mut InvocationControl,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, request_id))
}

fn execute_sequence_nodes(
    inputs: &SequenceInputs<'_>,
    select: &ForDistinctValuesExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, _) = required_source_context(inputs, context)?;
    let selected = evaluate_for_distinct_values(select, source, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut result = Vec::new();
    for node in selected {
        result.extend(copy_source_node(source, inputs.request_id, node, control)?);
    }
    Ok(result)
}

fn execute_sequence_items(
    inputs: &SequenceInputs<'_>,
    select: &[SequenceItemExpression],
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let mut result = Vec::new();
    let mut previous_was_atomic = false;
    for item in select {
        match item {
            SequenceItemExpression::ChildElements => {
                let (source, context) = required_source_context(inputs, context)?;
                for child in source.children(context).iter().copied() {
                    control
                        .charge(WorkDomain::XPathNodeVisit, 1)
                        .map_err(|failure| control_failure(failure, inputs.request_id))?;
                    if source.kind(child) == NodeKind::Element {
                        result.extend(copy_source_node(source, inputs.request_id, child, control)?);
                    }
                }
                previous_was_atomic = false;
            }
            SequenceItemExpression::Variable(name) => {
                let values = variable_atomic_values(variables, name, inputs.request_id)?;
                for value in values {
                    if previous_was_atomic {
                        append_text(&mut result, " ", inputs.request_id, control)?;
                    }
                    append_text(&mut result, value.lexical(), inputs.request_id, control)?;
                    previous_was_atomic = true;
                }
            }
        }
    }
    Ok(result)
}

fn execute_if(
    inputs: &SequenceInputs<'_>,
    test: &BooleanExpression,
    body: &[Instruction],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if evaluate_boolean(test, variables, inputs.request_id)? {
        execute_sequence(inputs, body, execution, variables, control)
    } else {
        Ok(Vec::new())
    }
}

fn execute_choose(
    inputs: &SequenceInputs<'_>,
    branches: &[crate::xslt::golden_semantics_experiment::ChooseBranch],
    otherwise: &[Instruction],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    for branch in branches {
        if evaluate_boolean(&branch.test, variables, inputs.request_id)? {
            return execute_sequence(inputs, &branch.body, execution, variables, control);
        }
    }
    execute_sequence(inputs, otherwise, execution, variables, control)
}

fn evaluate_boolean(
    expression: &BooleanExpression,
    variables: &RuntimeVariables,
    request_id: &str,
) -> Result<bool, ExecutionFailure> {
    match expression {
        BooleanExpression::Constant(value) => Ok(*value),
        BooleanExpression::VariableEqualsInteger(test) => {
            let value = variables.atomics.get(&test.variable).ok_or_else(|| {
                failure(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(request_id),
                    format!("unbound variable: ${}", test.variable),
                )
            })?;
            Ok(value.lexical().trim().parse::<i64>() == Ok(test.integer))
        }
    }
}

fn execute_variable_binding(
    inputs: &SequenceInputs<'_>,
    name: &str,
    select: &CastExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<AtomicValue, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    evaluate_cast(select, source, context, control).map_err(|evaluation_failure| {
        match evaluation_failure {
            CastEvaluationFailure::Control(control) => control_failure(control, inputs.request_id),
            CastEvaluationFailure::InvalidValue => failure(
                "FXRT0006",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("the value selected for ${name} cannot be cast to its target type"),
            ),
        }
    })
}

fn materialize_integer_range(
    start: i64,
    end: i64,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<AtomicValue>, ExecutionFailure> {
    let mut values = Vec::new();
    if start > end {
        return Ok(values);
    }
    let mut value = start;
    loop {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        values.push(AtomicValue::from_validated_lexical(
            BuiltinAtomicType::Integer,
            value.to_string(),
        ));
        if value == end {
            break;
        }
        value = value.checked_add(1).ok_or_else(|| {
            failure(
                "FXRT0007",
                FailureCategory::Invalid,
                Some(request_id),
                "integer range overflowed during materialization",
            )
        })?;
    }
    Ok(values)
}

fn variable_atomic_values<'a>(
    variables: &'a RuntimeVariables,
    name: &str,
    request_id: &str,
) -> Result<Vec<&'a AtomicValue>, ExecutionFailure> {
    if let Some(value) = variables.atomics.get(name) {
        return Ok(vec![value]);
    }
    if let Some(values) = variables.atomic_sequences.get(name) {
        return Ok(values.iter().collect());
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(request_id),
        format!("unbound variable: ${name}"),
    ))
}

fn copy_source_node(
    source: &Document,
    request_id: &str,
    node: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match source.kind(node) {
        NodeKind::Element => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let mut children = Vec::new();
            for child in source.children(node).iter().copied() {
                children.extend(copy_source_node(source, request_id, child, control)?);
            }
            let attributes = source
                .attributes(node)
                .iter()
                .map(|attribute| {
                    control
                        .charge(WorkDomain::ResultNode, 1)
                        .map_err(|failure| control_failure(failure, request_id))?;
                    Ok(ResultAttribute {
                        name: source
                            .name(*attribute)
                            .expect("source attribute has a name")
                            .clone(),
                        value: source.string_value(*attribute),
                    })
                })
                .collect::<Result<Vec<_>, ExecutionFailure>>()?;
            Ok(vec![ResultNode::Element {
                name: source
                    .name(node)
                    .expect("source element nodes have names")
                    .clone(),
                namespaces: source.namespace_declarations(node).to_vec(),
                attributes,
                children,
            }])
        }
        NodeKind::Text => {
            let mut copied = Vec::new();
            append_text(
                &mut copied,
                source.value(node).unwrap_or_default(),
                request_id,
                control,
            )?;
            Ok(copied)
        }
        NodeKind::Document
        | NodeKind::Attribute
        | NodeKind::Comment
        | NodeKind::ProcessingInstruction => Err(failure(
            "FXRT1002",
            FailureCategory::Unsupported,
            Some(request_id),
            "the selected source node kind is outside the private xsl:sequence copy slice",
        )),
    }
}

fn select_apply_nodes(
    inputs: &SequenceInputs<'_>,
    select: Option<&ApplySelection>,
    context: NodeId,
    variables: &BTreeMap<String, AtomicValue>,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs.source.expect("apply selection requires a source");
    let Some(select) = select else {
        return Ok(source.children(context).to_vec());
    };
    match select {
        ApplySelection::LocationPath(path) => {
            evaluate_location_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))
        }
        ApplySelection::ChildElement(name) => {
            let mut selected = Vec::new();
            for child in source.children(context).iter().copied() {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if source.kind(child) == NodeKind::Element && source.name(child) == Some(name) {
                    selected.push(child);
                }
            }
            Ok(selected)
        }
        ApplySelection::DescendantElement(name) => {
            let mut selected = Vec::new();
            select_descendant_elements(
                source,
                context,
                name,
                inputs.request_id,
                control,
                &mut selected,
            )?;
            Ok(selected)
        }
        ApplySelection::ChildNodes(node_test) => {
            let mut selected = Vec::new();
            for child in source.children(context).iter().copied() {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                let matches = match node_test {
                    NodeTest::Comment => source.kind(child) == NodeKind::Comment,
                    NodeTest::ProcessingInstruction => {
                        source.kind(child) == NodeKind::ProcessingInstruction
                    }
                    NodeTest::AnyNode => true,
                };
                if matches {
                    selected.push(child);
                }
            }
            Ok(selected)
        }
        ApplySelection::Attribute(name) => {
            let mut selected = Vec::new();
            for attribute in source.attributes(context).iter().copied() {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if source.name(attribute) == Some(name) {
                    selected.push(attribute);
                }
            }
            Ok(selected)
        }
        ApplySelection::VariableFilteredElementPath(path) => variable_filtered_path::select(
            source,
            context,
            path,
            variables,
            inputs.request_id,
            control,
        ),
        ApplySelection::GlobalTemporaryChildren(_)
        | ApplySelection::TemporaryRoot(_)
        | ApplySelection::TemporaryPath { .. } => {
            unreachable!("temporary-tree selection is dispatched before source selection")
        }
    }
}

fn select_descendant_elements(
    source: &Document,
    parent: NodeId,
    name: &ExpandedName,
    request_id: &str,
    control: &mut InvocationControl,
    selected: &mut Vec<NodeId>,
) -> Result<(), ExecutionFailure> {
    for child in source.children(parent).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.kind(child) == NodeKind::Element && source.name(child) == Some(name) {
            selected.push(child);
        }
        select_descendant_elements(source, child, name, request_id, control, selected)?;
    }
    Ok(())
}

fn execute_named_call(
    inputs: &SequenceInputs<'_>,
    name: &str,
    arguments: &[TemplateArgument],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if execution.call_depth >= MAX_NAMED_TEMPLATE_CALL_DEPTH {
        return Err(failure(
            "FXRT0003",
            FailureCategory::Limit,
            Some(inputs.request_id),
            format!(
                "named-template call depth exceeds private limit {MAX_NAMED_TEMPLATE_CALL_DEPTH}"
            ),
        ));
    }
    let target = inputs
        .program
        .named_templates
        .iter()
        .find(|template| template.name == name)
        .expect("named-template references were validated during compilation");
    let mut frame = RuntimeVariables::from_atomics(&inputs.globals.atomics);
    frame.atomics.extend(
        target
            .parameters
            .iter()
            .map(|parameter| (parameter.clone(), AtomicValue::string(""))),
    );
    for (name, parameter) in evaluate_template_arguments(arguments, variables, inputs.request_id)? {
        frame.atomics.insert(name, parameter.value);
    }
    execute_sequence(
        inputs,
        &target.template.body,
        SequenceContext {
            call_depth: execution.call_depth + 1,
            ..execution
        },
        &frame,
        control,
    )
}

fn apply_template(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    apply_template_at(inputs, node, mode, parameters, 1, 1, control)
}

fn apply_template_at(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    focus_position: usize,
    focus_size: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("matched and built-in source templates require a source document");
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    if let Some((template_index, template)) = select_template_with_index(
        inputs.program,
        &TemplateSelectionContext {
            source,
            node,
            mode,
            variables: &inputs.globals.atomics,
            request_id: inputs.request_id,
        },
        inputs.multiple_match_policy,
        control,
    )? {
        let variables =
            bind_template_parameters(&template.template, parameters, &inputs.globals.atomics);
        return execute_sequence(
            inputs,
            &template.template.body,
            SequenceContext::for_template_at(
                node,
                mode,
                template_index,
                focus_position,
                focus_size,
            ),
            &variables,
            control,
        );
    }

    apply_builtin_template(inputs, node, mode, parameters, control)
}

fn apply_builtin_template(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("built-in source templates require a source document");
    match source.kind(node) {
        NodeKind::Document | NodeKind::Element => {
            let mut result = Vec::new();
            let children = source.children(node);
            let focus_size = children.len();
            for (offset, child) in children.iter().copied().enumerate() {
                result.extend(apply_template_at(
                    inputs,
                    child,
                    mode,
                    parameters,
                    offset + 1,
                    focus_size,
                    control,
                )?);
            }
            Ok(result)
        }
        NodeKind::Text | NodeKind::Attribute => {
            let mut result = Vec::new();
            append_text(
                &mut result,
                source.value(node).unwrap_or_default(),
                inputs.request_id,
                control,
            )?;
            Ok(result)
        }
        NodeKind::Comment | NodeKind::ProcessingInstruction => Ok(Vec::new()),
    }
}

fn append_text(
    nodes: &mut Vec<ResultNode>,
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if value.is_empty() {
        return Ok(());
    }
    if !matches!(nodes.last(), Some(ResultNode::Text(_))) {
        control
            .charge(WorkDomain::ResultNode, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
    }
    control
        .charge(WorkDomain::ResultTextByte, value.len())
        .map_err(|failure| control_failure(failure, request_id))?;
    if let Some(ResultNode::Text(existing)) = nodes.last_mut() {
        existing.push_str(value);
    } else {
        nodes.push(ResultNode::Text(value.to_owned()));
    }
    Ok(())
}

fn construct_processing_instruction(
    target: &str,
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    control
        .charge(WorkDomain::ResultTextByte, target.len() + value.len())
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(ResultNode::ProcessingInstruction {
        target: target.to_owned(),
        value: value.to_owned(),
    })
}

#[cfg(test)]
#[path = "golden_runtime_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "golden_runtime_control_tests.rs"]
mod control_phase_tests;

#[cfg(test)]
#[path = "golden_runtime_workflow_tests.rs"]
mod workflow_tests;

#[cfg(test)]
#[path = "xslt30_template_dispatch_tests.rs"]
mod xslt30_template_dispatch_tests;

#[cfg(test)]
#[path = "xslt30_path_tests.rs"]
mod xslt30_path_tests;

#[cfg(test)]
#[path = "xslt30_apply_templates_inventory_tests.rs"]
mod xslt30_apply_templates_inventory_tests;

#[cfg(test)]
#[path = "xslt30_for_inventory_tests.rs"]
mod xslt30_for_inventory_tests;

#[cfg(test)]
#[path = "xslt30_castable_inventory_tests.rs"]
mod xslt30_castable_inventory_tests;
#[cfg(test)]
#[path = "xslt30_data_manipulation_inventory_tests.rs"]
mod xslt30_data_manipulation_inventory_tests;
#[cfg(test)]
#[path = "xslt30_initial_mode_inventory_tests.rs"]
mod xslt30_initial_mode_inventory_tests;

#[cfg(test)]
#[path = "xslt30_output_inventory_tests.rs"]
mod xslt30_output_inventory_tests;

#[cfg(test)]
#[path = "xslt30_include_inventory_tests.rs"]
mod xslt30_include_inventory_tests;

#[cfg(test)]
#[path = "xslt30_mode_qname_tests.rs"]
mod xslt30_mode_qname_tests;

#[cfg(test)]
#[path = "xslt30_deep_equal_inventory_tests.rs"]
mod xslt30_deep_equal_inventory_tests;
