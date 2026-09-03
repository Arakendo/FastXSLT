use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::ResourceSnapshot;
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits};
use crate::xpath::castable_experiment::{CastEvaluationFailure, CastExpression, evaluate_cast};
use crate::xpath::for_distinct_values_experiment::{
    ForDistinctValuesExpression, evaluate as evaluate_for_distinct_values,
};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, BooleanExpression, ComputedAttribute, Instruction, NodeTest,
    OnMultipleMatchPolicy, OnNoMatchPolicy, SequenceItemExpression, SourceWhitespacePolicy,
    StylesheetProgram, TemplateArgument,
};

#[path = "atomic_template_executor.rs"]
mod atomic_template_executor;
#[cfg(test)]
#[path = "golden_runtime_experiment/byte_encoding.rs"]
mod byte_encoding;
#[path = "dynamic_document.rs"]
mod dynamic_document;
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
    InvocationParameter, RuntimeVariables, SequenceInputs, TemporaryNodeKind, TemporaryTree,
    bind_template_parameters, evaluate_template_arguments, materialize_global_defaults,
    materialize_temporary_tree, required_source_context,
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
    execute_program_with_parameters_and_resources(
        program,
        source,
        parameters,
        multiple_match_policy,
        request_id,
        None,
        None,
        control,
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_program_with_parameters_and_resources(
    program: &StylesheetProgram,
    source: &Document,
    parameters: &BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &str,
    resource_snapshot: Option<&ResourceSnapshot>,
    denied_resources: Option<&HashSet<String>>,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    execute_program_with_parameters_using(
        program,
        source,
        parameters,
        multiple_match_policy,
        request_id,
        WhitespaceRepresentation::VisibilityView,
        resource_snapshot,
        denied_resources,
        control,
    )
}

#[derive(Clone, Copy)]
enum WhitespaceRepresentation {
    VisibilityView,
    #[cfg(test)]
    CompleteReference,
}

fn validate_whitespace_source(
    policy: SourceWhitespacePolicy,
    source: &Document,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if policy == SourceWhitespacePolicy::StripAllElementWhitespace
        && source
            .has_xml_space_declaration(control)
            .map_err(|failure| control_failure(failure, request_id))?
    {
        return Err(failure(
            "FXRT1014",
            FailureCategory::Unsupported,
            Some(request_id),
            "xsl:strip-space over a source containing xml:space is outside the admitted whitespace profile",
        ));
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the test-only representation choice preserves the complete runtime invocation contract"
)]
fn execute_program_with_parameters_using(
    program: &StylesheetProgram,
    source: &Document,
    parameters: &BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &str,
    representation: WhitespaceRepresentation,
    resource_snapshot: Option<&ResourceSnapshot>,
    denied_resources: Option<&HashSet<String>>,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    if let Some(name) = program.default_initial_mode.as_deref() {
        return execute_initial_mode(
            InitialModeInvocation {
                program,
                source,
                initial_node: source.document_node(),
                name,
                parameters,
                multiple_match_policy,
                request_id,
            },
            control,
        );
    }
    validate_whitespace_source(program.source_whitespace, source, request_id, control)?;
    let effective_source = match (program.source_whitespace, representation) {
        (SourceWhitespacePolicy::Preserve, _) => None,
        (
            SourceWhitespacePolicy::StripAllElementWhitespace,
            WhitespaceRepresentation::VisibilityView,
        ) => Some(
            source
                .view_stripping_all_element_whitespace(control)
                .map_err(|failure| control_failure(failure, request_id))?,
        ),
        #[cfg(test)]
        (
            SourceWhitespacePolicy::StripAllElementWhitespace,
            WhitespaceRepresentation::CompleteReference,
        ) => Some(
            source
                .derive_stripping_all_element_whitespace(control)
                .map_err(|failure| control_failure(failure, request_id))?,
        ),
    };
    let source = effective_source.as_ref().unwrap_or(source);
    let globals =
        materialize_global_defaults(program, Some(source), parameters, request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source: Some(source),
        request_id,
        globals: &globals,
        multiple_match_policy,
        document_rooted_matches: RefCell::default(),
        complete_atomic_frame_clones: control.complete_atomic_frame_clones(),
        resource_snapshot,
        denied_resources,
        dynamic_documents: RefCell::default(),
    };
    let children = if let Some(root_template) = program
        .root_template
        .as_ref()
        .filter(|_| program.root_template_modes.is_empty())
    {
        let variables = bind_template_parameters(
            root_template,
            &BTreeMap::new(),
            &globals.atomics,
            inputs.complete_atomic_frame_clones,
            request_id,
        )?;
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
    if let Some(requirement) = program
        .typed_mode_requirements
        .iter()
        .find(|requirement| requirement.name == name)
    {
        return Err(failure_at(
            "XTTE3100",
            FailureCategory::Invalid,
            Some(request_id),
            requirement.location.clone(),
            "the requested typed mode cannot accept an untyped source node",
        ));
    }
    validate_whitespace_source(program.source_whitespace, source, request_id, control)?;
    let effective_source = match program.source_whitespace {
        SourceWhitespacePolicy::Preserve => None,
        SourceWhitespacePolicy::StripAllElementWhitespace => Some(
            source
                .view_stripping_all_element_whitespace(control)
                .map_err(|failure| control_failure(failure, request_id))?,
        ),
    };
    let source = effective_source.as_ref().unwrap_or(source);
    let globals =
        materialize_global_defaults(program, Some(source), parameters, request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source: Some(source),
        request_id,
        globals: &globals,
        multiple_match_policy,
        document_rooted_matches: RefCell::default(),
        complete_atomic_frame_clones: control.complete_atomic_frame_clones(),
        resource_snapshot: None,
        denied_resources: None,
        dynamic_documents: RefCell::default(),
    };
    let children = if initial_node == source.document_node()
        && program.root_template_modes.iter().any(|mode| mode == name)
    {
        let template = program
            .root_template
            .as_ref()
            .expect("a compiled root initial mode has a root template");
        let variables = bind_template_parameters(
            template,
            parameters,
            &globals.atomics,
            inputs.complete_atomic_frame_clones,
            request_id,
        )?;
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
            document_rooted_matches: &inputs.document_rooted_matches,
        },
        effective_multiple_match_policy(inputs, Some(mode)),
        control,
    )? {
        let variables = bind_template_parameters(
            &template.template,
            parameters,
            &inputs.globals.atomics,
            inputs.complete_atomic_frame_clones,
            inputs.request_id,
        )?;
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

fn program_has_mode(program: &StylesheetProgram, name: &str) -> bool {
    program.root_template_modes.iter().any(|mode| mode == name)
        || program
            .matched_templates
            .iter()
            .any(|template| template.modes.iter().any(|mode| mode == name))
        || program
            .typed_mode_requirements
            .iter()
            .any(|requirement| requirement.name == name)
        || program
            .mode_policies
            .iter()
            .any(|policy| policy.name.as_deref() == Some(name))
}

#[cfg(test)]
fn execute_initial_template(
    program: &StylesheetProgram,
    name: &str,
    parameters: &BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    execute_initial_template_with_optional_source(
        program,
        name,
        None,
        parameters,
        multiple_match_policy,
        request_id,
        control,
    )
}

#[cfg(test)]
fn execute_initial_template_with_source(
    program: &StylesheetProgram,
    name: &str,
    source: &Document,
    parameters: &BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    execute_initial_template_with_optional_source(
        program,
        name,
        Some(source),
        parameters,
        multiple_match_policy,
        request_id,
        control,
    )
}

#[cfg(test)]
fn execute_initial_template_with_optional_source(
    program: &StylesheetProgram,
    name: &str,
    source: Option<&Document>,
    parameters: &BTreeMap<String, InvocationParameter>,
    multiple_match_policy: MultipleMatchPolicy,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let template = program
        .named_templates
        .iter()
        .find(|template| template.name == name)
        .expect("initial-template entries are validated during request admission");
    let globals = materialize_global_defaults(program, source, parameters, request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source,
        request_id,
        globals: &globals,
        multiple_match_policy,
        document_rooted_matches: RefCell::default(),
        complete_atomic_frame_clones: control.complete_atomic_frame_clones(),
        resource_snapshot: None,
        denied_resources: None,
        dynamic_documents: RefCell::default(),
    };
    let variables = bind_template_parameters(
        &template.template,
        &BTreeMap::new(),
        &globals.atomics,
        inputs.complete_atomic_frame_clones,
        request_id,
    )?;
    let children = execute_sequence(
        &inputs,
        &template.template.body,
        SequenceContext::new(source.map(Document::document_node), None),
        &variables,
        control,
    )?;
    Ok(SemanticResult { children })
}

#[derive(Clone, Copy)]
struct SequenceContext<'a> {
    node: Option<NodeId>,
    temporary_focus: Option<TemporaryFocus<'a>>,
    atomic_focus: Option<i64>,
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
            atomic_focus: None,
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
        sequence_focus: SequenceFocus,
    ) -> Self {
        Self {
            temporary_focus: Some(focus),
            current_template_index: Some(index),
            focus_position: sequence_focus.position,
            focus_size: sequence_focus.size,
            ..Self::new(None, current_mode)
        }
    }

    fn for_atomic_template(
        value: i64,
        current_mode: Option<&'a str>,
        index: usize,
        focus: SequenceFocus,
    ) -> Self {
        Self {
            atomic_focus: Some(value),
            current_template_index: Some(index),
            focus_position: focus.position,
            focus_size: focus.size,
            ..Self::new(None, current_mode)
        }
    }
}

#[derive(Clone, Copy)]
struct SequenceFocus {
    position: usize,
    size: usize,
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
        Instruction::CommentNode { value, .. } => {
            result.push(construct_comment(value, inputs.request_id, control)?);
        }
        Instruction::Attribute { attribute, .. } => result.push(execute_attribute_instruction(
            inputs, attribute, execution, scope, control,
        )?),
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
        | Instruction::SourceNodeVariable { .. }
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
        Instruction::ForEachTemporaryRoot { .. }
        | Instruction::ForEachStaticIntegerRange { .. }
        | Instruction::ForEachNodes { .. }
        | Instruction::NextMatch { .. }
        | Instruction::ApplyImports { .. }
        | Instruction::If { .. }
        | Instruction::Choose { .. }
        | Instruction::CallTemplate { .. }
        | Instruction::CopyOfCurrent { .. }
        | Instruction::CopyOfChildElements { .. }
        | Instruction::CopyOfAncestorOrSelfElements { .. }
        | Instruction::Copy { .. } => result.extend(execute_result_instruction(
            inputs,
            instruction,
            execution,
            scope,
            control,
        )?),
    }
    Ok(())
}

fn execute_attribute_instruction(
    inputs: &SequenceInputs<'_>,
    attribute: &ComputedAttribute,
    execution: SequenceContext<'_>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    let mut materialized = materialize_computed_attributes(
        std::slice::from_ref(attribute),
        scope,
        execution.focus_position,
        execution.focus_size,
        execution_context_value(inputs, execution),
        inputs.request_id,
        control,
    )?;
    Ok(ResultNode::PendingAttribute(materialized.pop().expect(
        "one compiled attribute materializes one result attribute",
    )))
}

fn execute_result_instruction<'a>(
    inputs: &SequenceInputs<'a>,
    instruction: &Instruction,
    execution: SequenceContext<'a>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match instruction {
        Instruction::ForEachTemporaryRoot { .. }
        | Instruction::ForEachStaticIntegerRange { .. }
        | Instruction::ForEachNodes { .. } => {
            execute_for_each_instruction(inputs, instruction, execution, scope, control)
        }
        Instruction::NextMatch { .. } | Instruction::ApplyImports { .. } => {
            execute_continuation_instruction(inputs, instruction, execution, scope, control)
        }
        Instruction::If { test, body, .. } => {
            execute_if(inputs, test, body, execution, scope, control)
        }
        Instruction::Choose {
            branches,
            otherwise,
            ..
        } => execute_choose(inputs, branches, otherwise, execution, scope, control),
        Instruction::CallTemplate { .. } => {
            execute_call(inputs, instruction, execution, scope, control)
        }
        Instruction::CopyOfCurrent { .. } => {
            execute_copy_of_current(inputs, execution.node, control)
        }
        Instruction::CopyOfChildElements { .. } => {
            execute_copy_of_child_elements(inputs, execution.node, control)
        }
        Instruction::CopyOfAncestorOrSelfElements { location } => {
            execute_copy_of_ancestor_or_self(inputs, execution.node, location, control)
        }
        Instruction::Copy { .. } => execute_copy(inputs, instruction, execution, scope, control),
        _ => unreachable!("result dispatch receives only result-producing instructions"),
    }
}

fn execute_copy_of_current(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, node) = required_source_context(inputs, context)?;
    copy_source_node(source, inputs.request_id, node, control)
}

fn execute_copy_of_ancestor_or_self(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let Some(source) = inputs.source else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "ancestor-or-self::* requires a context item",
        ));
    };
    let Some(context) = context else {
        return Err(failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "ancestor-or-self::* requires a context item",
        ));
    };
    let mut result = Vec::new();
    let mut current = Some(context);
    while let Some(node) = current {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if source.kind(node) == NodeKind::Element {
            result.extend(copy_source_node(source, inputs.request_id, node, control)?);
        }
        current = source.parent(node);
    }
    Ok(result)
}

fn execute_copy_of_child_elements(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, node) = required_source_context(inputs, context)?;
    let mut copied = Vec::new();
    for child in source.children(node).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if source.kind(child) == NodeKind::Element {
            copied.extend(copy_source_node(source, inputs.request_id, child, control)?);
        }
    }
    Ok(copied)
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

fn execute_for_each_instruction<'a>(
    inputs: &SequenceInputs<'a>,
    instruction: &Instruction,
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match instruction {
        Instruction::ForEachTemporaryRoot { variable, body, .. } => {
            execute_for_each_temporary_root(inputs, variable, body, execution, variables, control)
        }
        Instruction::ForEachStaticIntegerRange {
            start, end, body, ..
        } => execute_for_each_static_integer_range(
            inputs, *start, *end, body, execution, variables, control,
        ),
        Instruction::ForEachNodes { select, body, .. } => {
            execute_for_each_nodes(inputs, select, body, execution, variables, control)
        }
        _ => unreachable!("for-each dispatch receives only for-each instructions"),
    }
}

fn execute_for_each_static_integer_range<'a>(
    inputs: &SequenceInputs<'a>,
    start: i64,
    end: i64,
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if start > end {
        return Ok(Vec::new());
    }
    let span = end
        .checked_sub(start)
        .and_then(|value| value.checked_add(1));
    let focus_size = span
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            failure(
                "FXRT0007",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                "integer range cannot be represented by this host",
            )
        })?;
    let mut result = Vec::new();
    for index in 0..focus_size {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        result.extend(execute_sequence(
            inputs,
            body,
            SequenceContext {
                node: None,
                temporary_focus: None,
                atomic_focus: None,
                focus_position: index + 1,
                focus_size,
                ..execution
            },
            variables,
            control,
        )?);
    }
    Ok(result)
}

fn execute_for_each_nodes<'a>(
    inputs: &SequenceInputs<'a>,
    select: &ApplySelection,
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (_, context) = required_source_context(inputs, execution.node)?;
    let selected = select_apply_nodes(inputs, Some(select), context, &variables.atomics, control)?;
    let focus_size = selected.len();
    let mut result = Vec::new();
    for (index, node) in selected.into_iter().enumerate() {
        result.extend(execute_sequence(
            inputs,
            body,
            SequenceContext {
                node: Some(node),
                temporary_focus: None,
                atomic_focus: None,
                focus_position: index + 1,
                focus_size,
                ..execution
            },
            variables,
            control,
        )?);
    }
    Ok(result)
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
            Arc::make_mut(&mut scope.atomics).insert(name.clone(), value);
        }
        Instruction::SourceNodeVariable { name, select, .. } => {
            let (source, context) = required_source_context(inputs, context)?;
            let nodes = evaluate_location_path_controlled(source, context, select, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            scope.source_nodes.insert(name.clone(), nodes);
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
    if execution.temporary_focus.is_some() {
        return temporary_tree_executor::execute_temporary_copy(
            inputs, attributes, body, execution, variables, control,
        );
    }
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
                namespaces: source.in_scope_namespaces(node),
                attributes: materialize_literal_attributes(
                    attributes,
                    variables,
                    execution.focus_position,
                    execution.focus_size,
                    source.name(node),
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
        execution_context_name(inputs, execution),
        inputs.request_id,
        control,
    )?;
    attributes.extend(materialize_computed_attributes(
        computed_attributes,
        variables,
        execution.focus_position,
        execution.focus_size,
        None,
        inputs.request_id,
        control,
    )?);
    let mut children = Vec::new();
    for item in execute_sequence(inputs, body, execution, variables, control)? {
        match item {
            ResultNode::PendingAttribute(attribute) if children.is_empty() => {
                if attributes
                    .iter()
                    .any(|existing| existing.name == attribute.name)
                {
                    return Err(failure(
                        "XTDE0410",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        "result element construction produced duplicate expanded attribute names",
                    ));
                }
                attributes.push(attribute);
            }
            ResultNode::PendingAttribute(_) => {
                return Err(failure(
                    "XTDE0410",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    "result attributes must be constructed before result child nodes",
                ));
            }
            child => children.push(child),
        }
    }
    Ok(ResultNode::Element {
        name: name.clone(),
        namespaces: namespaces.clone(),
        attributes,
        children,
    })
}

fn execution_context_name<'a>(
    inputs: &'a SequenceInputs<'a>,
    execution: SequenceContext<'a>,
) -> Option<&'a ExpandedName> {
    if let Some(TemporaryFocus::Node(tree, node)) = execution.temporary_focus {
        return match &tree.nodes[node].kind {
            TemporaryNodeKind::Element { name, .. } | TemporaryNodeKind::Attribute { name, .. } => {
                Some(name)
            }
            TemporaryNodeKind::Text(_)
            | TemporaryNodeKind::Comment(_)
            | TemporaryNodeKind::ProcessingInstruction { .. } => None,
        };
    }
    execution
        .node
        .and_then(|node| inputs.source.and_then(|source| source.name(node)))
}

fn execution_context_value<'a>(
    inputs: &'a SequenceInputs<'a>,
    execution: SequenceContext<'a>,
) -> Option<&'a str> {
    if let Some(TemporaryFocus::Node(tree, node)) = execution.temporary_focus {
        return match &tree.nodes[node].kind {
            TemporaryNodeKind::Attribute { value, .. }
            | TemporaryNodeKind::Text(value)
            | TemporaryNodeKind::Comment(value)
            | TemporaryNodeKind::ProcessingInstruction { value, .. } => Some(value),
            TemporaryNodeKind::Element { .. } => None,
        };
    }
    execution
        .node
        .and_then(|node| inputs.source.and_then(|source| source.value(node)))
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
    let parameters =
        evaluate_template_arguments(arguments, variables, inputs, execution.node, control)?;
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
    if let Some(ApplySelection::AtomicIntegerRange { start, end }) = select {
        return atomic_template_executor::apply_integer_range(
            inputs, *start, *end, mode, parameters, control,
        );
    }
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
        let focus_size = tree.roots.len();
        for (offset, node) in tree.roots.iter().enumerate() {
            result.extend(apply_temporary_template(
                inputs,
                tree,
                *node,
                mode,
                parameters,
                SequenceFocus {
                    position: offset + 1,
                    size: focus_size,
                },
                control,
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
    let parameters =
        evaluate_template_arguments(arguments, variables, inputs, execution.node, control)?;
    if let Some(focus) = execution.temporary_focus {
        return temporary_tree_executor::apply_temporary_next(
            inputs,
            focus,
            execution.current_mode,
            current_index,
            &parameters,
            SequenceFocus {
                position: execution.focus_position,
                size: execution.focus_size,
            },
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
            document_rooted_matches: &inputs.document_rooted_matches,
        },
        current_index,
        effective_multiple_match_policy(inputs, execution.current_mode),
        control,
    )? {
        let variables = bind_template_parameters(
            &template.template,
            &parameters,
            &inputs.globals.atomics,
            inputs.complete_atomic_frame_clones,
            inputs.request_id,
        )?;
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
    let parameters =
        evaluate_template_arguments(arguments, variables, inputs, execution.node, control)?;
    if let Some(value) = execution.atomic_focus {
        return atomic_template_executor::apply_imports(
            inputs,
            value,
            execution.current_mode,
            current_index,
            &parameters,
            SequenceFocus {
                position: execution.focus_position,
                size: execution.focus_size,
            },
            control,
        );
    }
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
            document_rooted_matches: &inputs.document_rooted_matches,
        },
        current_index,
        control,
    )? {
        let variables = bind_template_parameters(
            &template.template,
            &parameters,
            &inputs.globals.atomics,
            inputs.complete_atomic_frame_clones,
            inputs.request_id,
        )?;
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
    if evaluate_boolean(inputs, test, execution.node, variables, control)? {
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
        if evaluate_boolean(inputs, &branch.test, execution.node, variables, control)? {
            return execute_sequence(inputs, &branch.body, execution, variables, control);
        }
    }
    execute_sequence(inputs, otherwise, execution, variables, control)
}

fn evaluate_boolean(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    match expression {
        BooleanExpression::Constant(value) => Ok(*value),
        BooleanExpression::NodeExists(path) => {
            let (source, context) = required_source_context(inputs, context)?;
            evaluate_location_path_controlled(source, context, path, control)
                .map(|nodes| !nodes.is_empty())
                .map_err(|failure| control_failure(failure, inputs.request_id))
        }
        BooleanExpression::NodeStringEquals { path, value } => {
            let (source, context) = required_source_context(inputs, context)?;
            let nodes = evaluate_location_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            for node in nodes {
                let actual = source
                    .string_value_controlled(node, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if actual == *value {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        BooleanExpression::NodeIntegerLessThan { path, value } => {
            let (source, context) = required_source_context(inputs, context)?;
            let nodes = evaluate_location_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            for node in nodes {
                let actual = source
                    .string_value_controlled(node, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if actual
                    .trim()
                    .parse::<i64>()
                    .is_ok_and(|actual| actual < *value)
                {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        BooleanExpression::ContextStringEquals(expected) => {
            let (source, context) = required_source_context(inputs, context)?;
            source
                .string_value_controlled(context, control)
                .map(|actual| actual == *expected)
                .map_err(|failure| control_failure(failure, inputs.request_id))
        }
        BooleanExpression::Not(expression) => {
            evaluate_boolean(inputs, expression, context, variables, control).map(|value| !value)
        }
        BooleanExpression::RootIdentityEqualsVariable { path, variable } => {
            evaluate_root_identity_equals_variable(
                inputs, path, variable, variables, context, control,
            )
        }
        BooleanExpression::TemporaryRootIdentityEqual {
            variable,
            descendant_local,
        } => evaluate_temporary_root_identity_equal(
            inputs,
            variable,
            descendant_local,
            variables,
            control,
        ),
        BooleanExpression::DocumentRootIdentityEqual { left, right } => {
            let left = dynamic_document::document_root_identity(inputs, left, control)?;
            let right = dynamic_document::document_root_identity(inputs, right, control)?;
            Ok(left == right)
        }
        BooleanExpression::VariableEqualsInteger(test) => {
            let value = variables.atomics.get(&test.variable).ok_or_else(|| {
                failure(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    format!("unbound variable: ${}", test.variable),
                )
            })?;
            Ok(value.lexical().trim().parse::<i64>() == Ok(test.integer))
        }
    }
}

fn evaluate_temporary_root_identity_equal(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    descendant_local: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
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
    let document =
        runtime_context::temporary_document_identity(tree, None, inputs.request_id, control)?;
    let descendant = runtime_context::temporary_document_identity(
        tree,
        Some(descendant_local),
        inputs.request_id,
        control,
    )?;
    Ok(document.is_some() && document == descendant)
}

fn evaluate_root_identity_equals_variable(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    variable: &str,
    variables: &RuntimeVariables,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let nodes = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let [node] = nodes.as_slice() else {
        return Err(failure(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "root() requires a single node in this identity comparison",
        ));
    };
    let mut root = *node;
    loop {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let Some(parent) = source.parent(root) else {
            break;
        };
        root = parent;
    }
    let expected = variables.atomics.get(variable).ok_or_else(|| {
        failure(
            "FXRT0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            format!("unbound variable: ${variable}"),
        )
    })?;
    Ok(expected.lexical() == runtime_context::source_node_identity(root))
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
        NodeKind::Document => {
            let mut copied = Vec::new();
            for child in source.children(node).iter().copied() {
                copied.extend(copy_source_node(source, request_id, child, control)?);
            }
            Ok(copied)
        }
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
                namespaces: source.in_scope_namespaces(node),
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
        NodeKind::Attribute | NodeKind::Comment | NodeKind::ProcessingInstruction => Err(failure(
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
        | ApplySelection::TemporaryPath { .. }
        | ApplySelection::AtomicIntegerRange { .. } => {
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
    if inputs.complete_atomic_frame_clones {
        control.observe_global_atomic_frame_clone(inputs.globals.atomics.len());
    }
    let supplied =
        evaluate_template_arguments(arguments, variables, inputs, execution.node, control)?;
    let frame = bind_template_parameters(
        &target.template,
        &supplied,
        &inputs.globals.atomics,
        inputs.complete_atomic_frame_clones,
        inputs.request_id,
    )?;
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
            document_rooted_matches: &inputs.document_rooted_matches,
        },
        effective_multiple_match_policy(inputs, mode),
        control,
    )? {
        let variables = bind_template_parameters(
            &template.template,
            parameters,
            &inputs.globals.atomics,
            inputs.complete_atomic_frame_clones,
            inputs.request_id,
        )?;
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

fn effective_multiple_match_policy(
    inputs: &SequenceInputs<'_>,
    mode: Option<&str>,
) -> MultipleMatchPolicy {
    if inputs.program.mode_policies.iter().any(|policy| {
        policy.name.as_deref() == mode
            && policy.on_multiple_match == Some(OnMultipleMatchPolicy::Fail)
    }) {
        MultipleMatchPolicy::Error
    } else {
        inputs.multiple_match_policy
    }
}

fn apply_builtin_template(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some(mode_policy) = inputs
        .program
        .mode_policies
        .iter()
        .find(|policy| policy.name.as_deref() == mode && policy.on_no_match.is_some())
    {
        if let Some(on_no_match) = mode_policy.on_no_match {
            match on_no_match {
                OnNoMatchPolicy::Fail => {
                    return Err(failure_at(
                        "XTDE0555",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        mode_policy.location.clone(),
                        "the active mode's on-no-match='fail' policy rejected an unmatched node",
                    ));
                }
                OnNoMatchPolicy::ShallowCopy => {
                    return apply_shallow_copy_template(inputs, node, mode, parameters, control);
                }
                OnNoMatchPolicy::ShallowSkip => {
                    return apply_shallow_skip_template(inputs, node, mode, parameters, control);
                }
                OnNoMatchPolicy::TextOnlyCopy => {
                    return apply_text_only_copy_template(inputs, node, mode, parameters, control);
                }
            }
        }
    }
    apply_text_only_copy_template(inputs, node, mode, parameters, control)
}

fn apply_shallow_skip_template(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("shallow-skip built-in templates require a source document");
    match source.kind(node) {
        NodeKind::Document | NodeKind::Element => {
            apply_child_templates(inputs, node, mode, parameters, control)
        }
        NodeKind::Text
        | NodeKind::Attribute
        | NodeKind::Comment
        | NodeKind::ProcessingInstruction => Ok(Vec::new()),
    }
}

fn apply_text_only_copy_template(
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

fn apply_shallow_copy_template(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("shallow-copy built-in templates require a source document");
    match source.kind(node) {
        NodeKind::Document => apply_child_templates(inputs, node, mode, parameters, control),
        NodeKind::Element => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let attributes = source.attributes(node);
            let children = source.children(node);
            let focus_size = attributes.len() + children.len();
            let (attributes, mut generated_children) =
                shallow_copy_attributes(inputs, node, mode, parameters, focus_size, control)?;
            generated_children.extend(apply_child_templates_with_focus(
                inputs,
                node,
                mode,
                parameters,
                source.attributes(node).len(),
                focus_size,
                control,
            )?);
            Ok(vec![ResultNode::Element {
                name: source
                    .name(node)
                    .expect("source element has a name")
                    .clone(),
                namespaces: source.namespace_declarations(node).to_vec(),
                attributes,
                children: generated_children,
            }])
        }
        NodeKind::Text => {
            let mut result = Vec::new();
            append_text(
                &mut result,
                source.value(node).unwrap_or_default(),
                inputs.request_id,
                control,
            )?;
            Ok(result)
        }
        NodeKind::ProcessingInstruction => Ok(vec![construct_processing_instruction(
            &source
                .name(node)
                .expect("processing instruction has a target")
                .local,
            source.value(node).unwrap_or_default(),
            inputs.request_id,
            control,
        )?]),
        NodeKind::Attribute => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(vec![ResultNode::PendingAttribute(ResultAttribute {
                name: source
                    .name(node)
                    .expect("source attribute has a name")
                    .clone(),
                value: source.string_value(node),
            })])
        }
        NodeKind::Comment => Ok(vec![construct_comment(
            source.value(node).unwrap_or_default(),
            inputs.request_id,
            control,
        )?]),
    }
}

fn shallow_copy_attributes(
    inputs: &SequenceInputs<'_>,
    element: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    focus_size: usize,
    control: &mut InvocationControl,
) -> Result<(Vec<ResultAttribute>, Vec<ResultNode>), ExecutionFailure> {
    let source = inputs
        .source
        .expect("shallow-copy attributes require a source");
    let mut result_attributes = Vec::new();
    let mut generated_children = Vec::new();
    for (offset, attribute) in source.attributes(element).iter().copied().enumerate() {
        for item in apply_template_at(
            inputs,
            attribute,
            mode,
            parameters,
            offset + 1,
            focus_size,
            control,
        )? {
            match item {
                ResultNode::PendingAttribute(result_attribute) => {
                    if result_attributes
                        .iter()
                        .any(|existing: &ResultAttribute| existing.name == result_attribute.name)
                    {
                        return Err(failure_at(
                            "XTDE0410",
                            FailureCategory::Invalid,
                            Some(inputs.request_id),
                            source.location(attribute).clone(),
                            "shallow-copy attribute templates produced duplicate expanded names",
                        ));
                    }
                    result_attributes.push(result_attribute);
                }
                child => generated_children.push(child),
            }
        }
    }
    Ok((result_attributes, generated_children))
}

fn apply_child_templates(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let child_count = inputs
        .source
        .expect("built-in traversal requires a source")
        .children(node)
        .len();
    apply_child_templates_with_focus(inputs, node, mode, parameters, 0, child_count, control)
}

#[allow(clippy::too_many_arguments)]
fn apply_child_templates_with_focus(
    inputs: &SequenceInputs<'_>,
    node: NodeId,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    position_offset: usize,
    focus_size: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source = inputs.source.expect("built-in traversal requires a source");
    let mut result = Vec::new();
    let children = source.children(node);
    for (offset, child) in children.iter().copied().enumerate() {
        result.extend(apply_template_at(
            inputs,
            child,
            mode,
            parameters,
            position_offset + offset + 1,
            focus_size,
            control,
        )?);
    }
    Ok(result)
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

fn construct_comment(
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    control
        .charge(WorkDomain::ResultTextByte, value.len())
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(ResultNode::Comment(value.to_owned()))
}

#[cfg(test)]
#[path = "golden_runtime_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "golden_runtime_control_tests.rs"]
mod control_phase_tests;

#[cfg(test)]
#[path = "whitespace_view_measurement_tests.rs"]
mod whitespace_view_measurement_tests;

#[cfg(test)]
#[path = "whitespace_view_runtime_tests.rs"]
mod whitespace_view_runtime_tests;

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
#[path = "xslt30_strip_space_tests.rs"]
mod xslt30_strip_space_tests;

#[cfg(test)]
#[path = "xslt30_built_in_templates_tests.rs"]
mod xslt30_built_in_templates_tests;

#[cfg(test)]
#[path = "xslt30_deep_equal_inventory_tests.rs"]
mod xslt30_deep_equal_inventory_tests;

#[cfg(test)]
#[path = "xslt30_root_inventory_tests.rs"]
mod xslt30_root_inventory_tests;

#[cfg(test)]
#[path = "xslt30_apply_imports_inventory_tests.rs"]
mod xslt30_apply_imports_inventory_tests;

#[cfg(test)]
#[path = "xslt30_choose_inventory_tests.rs"]
mod xslt30_choose_inventory_tests;

#[cfg(test)]
#[path = "xslt30_call_template_inventory_tests.rs"]
mod xslt30_call_template_inventory_tests;
