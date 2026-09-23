use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::ResourceSnapshot;
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{
    Document, NodeId, NodeKind, SourceLocation, StringValueVisitFailure,
};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding, ParseLimits};
use crate::xpath::castable_experiment::{CastEvaluationFailure, CastExpression, evaluate_cast};
use crate::xpath::for_distinct_values_experiment::{
    ForDistinctValuesExpression, evaluate as evaluate_for_distinct_values,
};
use crate::xpath::path_experiment::{
    evaluate_location_path_controlled, evaluate_location_path_union_controlled,
};
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, BooleanExpression, ComputedAttribute, FocusComparison, FocusEqualityOperand,
    Instruction, LiteralAttribute, NodeTest, OnMultipleMatchPolicy, OnNoMatchPolicy,
    SequenceItemExpression, SortDataType, SortKey, SortOrder, SortSelect, SourceWhitespacePolicy,
    StringComparison, StylesheetProgram, TemplateArgument, Xslt10AncestorFilter,
    Xslt10ApplyUnionPart, Xslt10KeyLookup,
};

#[path = "atomic_template_executor.rs"]
mod atomic_template_executor;
#[cfg(any(test, feature = "workbench"))]
#[path = "golden_runtime_experiment/byte_encoding.rs"]
mod byte_encoding;
#[path = "dynamic_attribute_name.rs"]
mod dynamic_attribute_name;
#[path = "dynamic_document.rs"]
mod dynamic_document;
#[path = "dynamic_element_name.rs"]
mod dynamic_element_name;
#[path = "key_lookup.rs"]
mod key_lookup;
#[path = "match_sequence_predicate.rs"]
mod match_sequence_predicate;
#[path = "number_executor.rs"]
mod number_executor;
#[cfg(test)]
#[path = "preparation_pipeline_controller_tests.rs"]
mod preparation_pipeline_controller_tests;
#[cfg(test)]
#[path = "preparation_pipeline_experiment.rs"]
mod preparation_pipeline_experiment;
#[cfg(test)]
#[path = "preparation_pipeline_topology_tests.rs"]
mod preparation_pipeline_topology_tests;
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
#[path = "xslt10_current_name.rs"]
mod xslt10_current_name;

use dynamic_element_name::{
    DynamicElementNameRequest, resolve_dynamic_element_name, resolve_dynamic_element_namespace,
};
use number_executor::execute as execute_number_instruction;
#[cfg(test)]
pub(super) use resource_compiler::compile_resource;
pub(super) use resource_compiler::compile_resource_with_denied;
use result_tree::{
    LiteralAttributeFocus, ResultAttribute, ResultNode, literal_attributes_require_context_string,
    materialize_computed_attributes, materialize_literal_attributes,
};
use runtime_context::{
    InvocationParameter, RuntimeVariables, SequenceInputs, TemporaryNodeKind, TemporaryTree,
    bind_template_parameters, evaluate_template_arguments, materialize_global_defaults,
    materialize_result_nodes, materialize_temporary_tree, required_source_context,
};
pub(super) use runtime_failure::ExecutionFailure;
use runtime_failure::{FailureCategory, control_failure, failure, failure_at};
pub(super) use serialization::serialize_xml;
#[cfg(any(test, feature = "workbench"))]
pub(super) use serialization::serialize_xml_bytes;
#[cfg(test)]
pub(super) use serialization::serialize_xml_complete_namespace_reference;
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

#[cfg(test)]
pub(crate) fn execute_compiled_stylesheet_for_test(
    program: &StylesheetProgram,
    source: &Document,
    request_id: &str,
) -> Result<String, String> {
    let mut control = InvocationControl::unbounded();
    let result = execute_program(program, source, request_id, &mut control)
        .map_err(|failure| format!("{failure:?}"))?;
    serialize_xml(
        &result,
        &program.output,
        request_id,
        64 * 1024,
        &mut control,
    )
    .map_err(|failure| format!("{failure:?}"))
}

#[cfg(test)]
pub(crate) fn execute_compiled_initial_template_without_source_for_test(
    program: &StylesheetProgram,
    name: &str,
    request_id: &str,
) -> Result<String, String> {
    let mut control = InvocationControl::unbounded();
    let result = execute_initial_template(
        program,
        name,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        request_id,
        &mut control,
    )
    .map_err(|failure| format!("{failure:?}"))?;
    serialize_xml(
        &result,
        &program.output,
        request_id,
        64 * 1024,
        &mut control,
    )
    .map_err(|failure| format!("{failure:?}"))
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
            &inputs,
            Some(source.document_node()),
            control,
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
        let variables =
            bind_template_parameters(template, parameters, &inputs, Some(initial_node), control)?;
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
        let variables =
            bind_template_parameters(&template.template, parameters, inputs, Some(node), control)?;
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
        &inputs,
        source.map(Document::document_node),
        control,
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

    fn sequence_focus(self) -> Option<SequenceFocus> {
        (self.node.is_some() || self.temporary_focus.is_some() || self.atomic_focus.is_some())
            .then_some(SequenceFocus {
                position: self.focus_position,
                size: self.focus_size,
            })
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
    #[cfg(test)]
    {
        let (atomic_sequences, source_nodes, temporary_trees, local_bindings) =
            variables.clone_population();
        control.observe_sequence_frame_clone(
            atomic_sequences,
            source_nodes,
            temporary_trees,
            local_bindings,
        );
    }
    let mut result = Vec::new();
    let mut scope = variables.clone();
    #[cfg(test)]
    if control.complete_sequence_frame_clones() {
        scope = variables.clone_complete_for_sequence();
    }
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
        Instruction::LiteralElement { .. }
        | Instruction::ContextNameElement { .. }
        | Instruction::DynamicNameElement { .. } => result.push(execute_literal_element(
            inputs,
            instruction,
            execution,
            scope,
            control,
        )?),
        Instruction::Text { value, .. } | Instruction::CopyOfStaticAtomicText { value, .. } => {
            append_text(result, value, inputs.request_id, control)?;
        }
        Instruction::ProcessingInstructionNode { .. }
        | Instruction::Xslt10ProcessingInstructionNode { .. }
        | Instruction::CommentNode { .. }
        | Instruction::Xslt10CommentNode { .. }
        | Instruction::Attribute { .. } => result.push(execute_node_constructor(
            inputs,
            instruction,
            execution,
            scope,
            control,
        )?),
        Instruction::ValueOf {
            select, separator, ..
        } => {
            execute_value_of(inputs, select, separator, execution, scope, result, control)?;
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
        | Instruction::StaticAtomicVariable { .. }
        | Instruction::VariableAlias { .. }
        | Instruction::ContextPositionVariable { .. }
        | Instruction::ContextNodeNameVariable { .. }
        | Instruction::ContextCountPathVariable { .. }
        | Instruction::Xslt10BinaryNumericVariable { .. }
        | Instruction::SourceNodeVariable { .. }
        | Instruction::SourceVariablePathVariable { .. }
        | Instruction::SourceNodeUnionVariable { .. }
        | Instruction::IntegerRangeVariable { .. }
        | Instruction::TemporaryTreeVariable { .. }
        | Instruction::Xslt10TextTreeVariable { .. }
        | Instruction::Xslt10ValueOfTreeVariable { .. }
        | Instruction::Xslt10ForEachTextTreeVariable { .. }
        | Instruction::Xslt10SequenceTreeVariable { .. } => {
            execute_binding(inputs, instruction, execution, scope, control)?;
        }
        Instruction::ApplyTemplates { .. } => result.extend(execute_apply_instruction(
            inputs,
            instruction,
            execution,
            scope,
            control,
        )?),
        Instruction::Number { .. }
        | Instruction::ForEachVariable { .. }
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
        | Instruction::CopyOfLocationPath { .. }
        | Instruction::CopyOfXslt10KeyLookup { .. }
        | Instruction::CopyOfPathUnion { .. }
        | Instruction::CopyOfVariable { .. }
        | Instruction::CopyOfAtomicValue { .. }
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

fn execute_node_constructor(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    match instruction {
        Instruction::ProcessingInstructionNode { target, value, .. } => {
            construct_processing_instruction(target, value, inputs.request_id, control)
        }
        Instruction::Xslt10ProcessingInstructionNode { target, body, .. } => {
            execute_xslt10_processing_instruction(inputs, target, body, execution, scope, control)
        }
        Instruction::CommentNode { value, .. } => {
            construct_comment(value, inputs.request_id, control)
        }
        Instruction::Xslt10CommentNode { body, .. } => {
            let value =
                execute_xslt10_text_constructor_value(inputs, body, execution, scope, control)?;
            let value = recover_xslt10_comment_content(&value);
            construct_comment(&value, inputs.request_id, control)
        }
        Instruction::Attribute {
            attribute,
            recover_unattached,
            ..
        } => execute_attribute_instruction(
            inputs,
            attribute,
            *recover_unattached,
            execution,
            scope,
            control,
        ),
        _ => unreachable!("only node constructors are delegated here"),
    }
}

fn execute_xslt10_processing_instruction(
    inputs: &SequenceInputs<'_>,
    target: &str,
    body: &[Instruction],
    execution: SequenceContext<'_>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    let value = execute_xslt10_text_constructor_value(inputs, body, execution, scope, control)?;
    let value = value.replace("?>", "? >");
    construct_processing_instruction(target, &value, inputs.request_id, control)
}

fn execute_xslt10_text_constructor_value(
    inputs: &SequenceInputs<'_>,
    body: &[Instruction],
    execution: SequenceContext<'_>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let nodes = execute_sequence(inputs, body, execution, scope, control)?;
    Ok(nodes
        .into_iter()
        .filter_map(|node| match node {
            ResultNode::Text(text) => Some(text),
            _ => None,
        })
        .collect())
}

fn recover_xslt10_comment_content(value: &str) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    let mut recovered = String::with_capacity(value.len() + 1);
    for (index, character) in characters.iter().copied().enumerate() {
        recovered.push(character);
        if character == '-'
            && (characters.get(index + 1) == Some(&'-') || index + 1 == characters.len())
        {
            recovered.push(' ');
        }
    }
    recovered
}

fn execute_attribute_instruction(
    inputs: &SequenceInputs<'_>,
    attribute: &ComputedAttribute,
    recover_unattached: bool,
    execution: SequenceContext<'_>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    let context_string =
        result_tree::computed_attributes_require_context_string(std::slice::from_ref(attribute))
            .then(|| execution_context_string_value(inputs, execution, control))
            .transpose()?
            .flatten();
    let mut materialized = materialize_computed_attributes(
        inputs,
        std::slice::from_ref(attribute),
        scope,
        execution,
        LiteralAttributeFocus {
            position: execution.focus_position,
            size: execution.focus_size,
            name: execution_context_name(inputs, execution),
            value: context_string
                .as_deref()
                .or_else(|| execution_context_value(inputs, execution)),
            source: execution_source_focus(inputs, execution),
        },
        inputs.request_id,
        control,
    )?;
    Ok(result_tree::pending_attribute(
        materialized
            .pop()
            .expect("one compiled attribute materializes one result attribute"),
        recover_unattached,
    ))
}

fn execute_result_instruction<'a>(
    inputs: &SequenceInputs<'a>,
    instruction: &Instruction,
    execution: SequenceContext<'a>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match instruction {
        Instruction::Number { .. } => {
            let mut result = Vec::new();
            execute_number_instruction(
                inputs,
                instruction,
                execution,
                scope,
                &mut result,
                control,
            )?;
            Ok(result)
        }
        Instruction::ForEachVariable { .. }
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
        Instruction::CopyOfCurrent { .. }
        | Instruction::CopyOfChildElements { .. }
        | Instruction::CopyOfAncestorOrSelfElements { .. }
        | Instruction::CopyOfLocationPath { .. }
        | Instruction::CopyOfXslt10KeyLookup { .. }
        | Instruction::CopyOfPathUnion { .. }
        | Instruction::CopyOfVariable { .. } => {
            execute_copy_of_instruction(inputs, instruction, execution, scope, control)
        }
        Instruction::CopyOfAtomicValue { select, .. } => {
            let value = value_evaluator::evaluate_binary_numeric_value(
                inputs,
                execution.node,
                execution.sequence_focus(),
                select,
                scope,
                control,
            )?;
            let mut result = Vec::new();
            append_text(&mut result, &value, inputs.request_id, control)?;
            Ok(result)
        }
        Instruction::Copy { .. } => execute_copy(inputs, instruction, execution, scope, control),
        _ => unreachable!("result dispatch receives only result-producing instructions"),
    }
}

fn execute_copy_of_instruction<'a>(
    inputs: &SequenceInputs<'a>,
    instruction: &Instruction,
    execution: SequenceContext<'a>,
    scope: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match instruction {
        Instruction::CopyOfCurrent {
            recover_unattached_attributes,
            ..
        } => execute_copy_of_current(
            inputs,
            execution.node,
            *recover_unattached_attributes,
            control,
        ),
        Instruction::CopyOfChildElements {
            recover_unattached_attributes,
            ..
        } => execute_copy_of_child_elements(
            inputs,
            execution.node,
            *recover_unattached_attributes,
            control,
        ),
        Instruction::CopyOfAncestorOrSelfElements {
            recover_unattached_attributes,
            location,
        } => execute_copy_of_ancestor_or_self(
            inputs,
            execution.node,
            location,
            *recover_unattached_attributes,
            control,
        ),
        Instruction::CopyOfLocationPath {
            select,
            recover_unattached_attributes,
            ..
        } => execute_copy_of_location_path(
            inputs,
            execution.node,
            select,
            *recover_unattached_attributes,
            control,
        ),
        Instruction::CopyOfXslt10KeyLookup {
            select,
            recover_unattached_attributes,
            ..
        } => execute_copy_of_xslt10_key_lookup(
            inputs,
            execution.node,
            select,
            scope,
            *recover_unattached_attributes,
            control,
        ),
        Instruction::CopyOfPathUnion {
            alternatives,
            recover_unattached_attributes,
            ..
        } => execute_copy_of_path_union(
            inputs,
            execution.node,
            alternatives,
            *recover_unattached_attributes,
            control,
        ),
        Instruction::CopyOfVariable {
            variable,
            recover_unattached_attributes,
            location,
        } => execute_copy_of_variable(
            inputs,
            variable,
            location,
            scope,
            *recover_unattached_attributes,
            control,
        ),
        _ => unreachable!("copy-of dispatch receives only copy-of instructions"),
    }
}

fn execute_copy_of_xslt10_key_lookup(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    select: &crate::xslt::golden_semantics_experiment::Xslt10KeyLookup,
    variables: &RuntimeVariables,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let source = inputs.source.expect("key lookup requires a source");
    let selected = key_lookup::select(inputs, select, context, variables, control)?;
    let mut copied = Vec::new();
    for node in selected {
        copied.extend(copy_source_node(
            source,
            inputs.request_id,
            node,
            recover_unattached_attributes,
            control,
        )?);
    }
    Ok(copied)
}

fn execute_copy_of_path_union(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    alternatives: &[crate::xpath::path_experiment::LocationPath],
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_source_path_union(inputs, source, context, alternatives, control)?;

    let mut copied = Vec::new();
    for node in selected {
        copied.extend(copy_source_node(
            source,
            inputs.request_id,
            node,
            recover_unattached_attributes,
            control,
        )?);
    }
    Ok(copied)
}

fn evaluate_source_path_union(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    context: NodeId,
    alternatives: &[crate::xpath::path_experiment::LocationPath],
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    evaluate_location_path_union_controlled(source, context, alternatives, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))
}

fn execute_copy_of_variable(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    variables: &RuntimeVariables,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;

    let mut copied = Vec::new();
    if let Some(value) = variables.atomics.get(variable) {
        append_text(&mut copied, value.lexical(), inputs.request_id, control)?;
        return Ok(copied);
    }
    if let Some(values) = variables.atomic_sequences.get(variable) {
        for (index, value) in values.iter().enumerate() {
            if index != 0 {
                append_text(&mut copied, " ", inputs.request_id, control)?;
            }
            append_text(&mut copied, value.lexical(), inputs.request_id, control)?;
        }
        return Ok(copied);
    }
    if let Some(nodes) = variables.source_nodes(inputs.globals, variable) {
        let source = inputs.source.ok_or_else(|| {
            failure(
                "XPDY0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("source-node variable ${variable} requires a source document"),
            )
        })?;
        for node in nodes.iter().copied() {
            copied.extend(copy_source_node(
                source,
                inputs.request_id,
                node,
                recover_unattached_attributes,
                control,
            )?);
        }
        return Ok(copied);
    }
    if let Some(tree) = variables.temporary_tree(inputs.globals, variable) {
        return temporary_tree_executor::copy_temporary_tree(inputs, tree, control);
    }
    if variables.allows_global_fallback(variable)
        && inputs.globals.empty_sequences.contains(variable)
    {
        return Ok(copied);
    }
    Err(failure_at(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        location.clone(),
        format!("unbound variable: ${variable}"),
    ))
}

fn execute_copy_of_location_path(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    select: &crate::xpath::path_experiment::LocationPath,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, select, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut copied = Vec::new();
    for node in selected {
        copied.extend(copy_source_node(
            source,
            inputs.request_id,
            node,
            recover_unattached_attributes,
            control,
        )?);
    }
    Ok(copied)
}

fn execute_copy_of_current(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, node) = required_source_context(inputs, context)?;
    copy_source_node(
        source,
        inputs.request_id,
        node,
        recover_unattached_attributes,
        control,
    )
}

fn execute_copy_of_ancestor_or_self(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    recover_unattached_attributes: bool,
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
            result.extend(copy_source_node(
                source,
                inputs.request_id,
                node,
                recover_unattached_attributes,
                control,
            )?);
        }
        current = source.parent(node);
    }
    Ok(result)
}

fn execute_copy_of_child_elements(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, node) = required_source_context(inputs, context)?;
    let mut copied = Vec::new();
    for child in source.children(node).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if source.kind(child) == NodeKind::Element {
            copied.extend(copy_source_node(
                source,
                inputs.request_id,
                child,
                recover_unattached_attributes,
                control,
            )?);
        }
    }
    Ok(copied)
}

fn execute_for_each_variable<'a>(
    inputs: &SequenceInputs<'a>,
    variable: &str,
    sorts: &[SortKey],
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some(tree) = variables.temporary_tree(inputs.globals, variable) {
        return execute_sequence(
            inputs,
            body,
            SequenceContext {
                node: None,
                temporary_focus: Some(TemporaryFocus::Document(tree)),
                ..execution
            },
            variables,
            control,
        );
    }
    if let Some(nodes) = variables.source_nodes(inputs.globals, variable) {
        let nodes = sort_selected_nodes(inputs, nodes.clone(), sorts, variables, control)?;
        let focus_size = nodes.len();
        let mut result = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
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
        return Ok(result);
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        format!("unbound or unsupported sequence variable: ${variable}"),
    ))
}

fn execute_for_each_instruction<'a>(
    inputs: &SequenceInputs<'a>,
    instruction: &Instruction,
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match instruction {
        Instruction::ForEachVariable {
            variable,
            sorts,
            body,
            ..
        } => {
            execute_for_each_variable(inputs, variable, sorts, body, execution, variables, control)
        }
        Instruction::ForEachStaticIntegerRange {
            start, end, body, ..
        } => execute_for_each_static_integer_range(
            inputs, *start, *end, body, execution, variables, control,
        ),
        Instruction::ForEachNodes {
            select,
            sorts,
            body,
            ..
        } => execute_for_each_nodes(inputs, select, sorts, body, execution, variables, control),
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
    sorts: &[SortKey],
    body: &[Instruction],
    execution: SequenceContext<'a>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (_, context) = required_source_context(inputs, execution.node)?;
    let selected = if let ApplySelection::TemporaryPath { variable, steps } = select
        && let Some(selected) =
            select_source_variable_path(inputs, variables, variable, steps, control)
    {
        selected?
    } else {
        select_apply_nodes(inputs, Some(select), context, variables, control)?
    };
    let selected = sort_selected_nodes(inputs, selected, sorts, variables, control)?;
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

#[derive(Debug)]
enum EvaluatedSortKey {
    Text(String),
    Number(Option<f64>),
}

#[derive(Debug, Clone, Copy)]
struct EvaluatedSortControl {
    data_type: EvaluatedSortDataType,
    order: EvaluatedSortOrder,
}

#[derive(Debug, Clone, Copy)]
enum EvaluatedSortDataType {
    Text,
    Number,
}

#[derive(Debug, Clone, Copy)]
enum EvaluatedSortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy)]
struct SortFocus<'a> {
    source: &'a Document,
    node: NodeId,
    position: usize,
    size: usize,
}

fn sort_selected_nodes(
    inputs: &SequenceInputs<'_>,
    selected: Vec<NodeId>,
    sorts: &[SortKey],
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    if sorts.is_empty() {
        return Ok(selected);
    }
    let sort_controls = sorts
        .iter()
        .map(|sort| evaluate_sort_control(inputs, sort, variables, control))
        .collect::<Result<Vec<_>, ExecutionFailure>>()?;
    if selected.len() < 2 {
        return Ok(selected);
    }
    let source = inputs
        .source
        .expect("source-node sorting requires a source document");
    let mut keyed = Vec::with_capacity(selected.len());
    let focus_size = selected.len();
    for (offset, node) in selected.into_iter().enumerate() {
        let mut values = Vec::with_capacity(sorts.len());
        for (sort, sort_control) in sorts.iter().zip(&sort_controls) {
            let value = evaluate_sort_key_value(
                inputs,
                SortFocus {
                    source,
                    node,
                    position: offset + 1,
                    size: focus_size,
                },
                sort,
                variables,
                control,
            )?;
            values.push(typed_sort_key(
                sort_control.data_type,
                sort.xslt10_numeric_conversion,
                value,
            ));
        }
        keyed.push((node, values));
    }
    let comparison_charge = keyed
        .len()
        .saturating_mul(keyed.len().ilog2() as usize + 1)
        .saturating_mul(sorts.len());
    control
        .charge(WorkDomain::XPathOperation, comparison_charge)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    keyed.sort_by(|left, right| {
        for (index, sort_control) in sort_controls.iter().enumerate() {
            let ordering = compare_sort_keys(&left.1[index], &right.1[index]);
            let ordering = match sort_control.order {
                EvaluatedSortOrder::Ascending => ordering,
                EvaluatedSortOrder::Descending => ordering.reverse(),
            };
            if !ordering.is_eq() {
                return ordering;
            }
        }
        std::cmp::Ordering::Equal
    });
    Ok(keyed.into_iter().map(|(node, _)| node).collect())
}

fn evaluate_sort_key_value(
    inputs: &SequenceInputs<'_>,
    focus: SortFocus<'_>,
    sort: &SortKey,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let source = focus.source;
    let node = focus.node;
    match &sort.select {
        SortSelect::LocationPath(path) => {
            let nodes = evaluate_location_path_controlled(source, node, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(nodes
                .first()
                .map_or_else(String::new, |selected| source.string_value(*selected)))
        }
        SortSelect::Xslt10VariablePositionPath {
            path,
            variable,
            explicit_position_comparison,
        } => {
            let nodes = evaluate_location_path_controlled(source, node, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if !explicit_position_comparison
                && variables.temporary_tree(inputs.globals, variable).is_some()
            {
                control
                    .charge(WorkDomain::XPathOperation, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                return Ok(nodes
                    .first()
                    .map_or_else(String::new, |selected| source.string_value(*selected)));
            }
            let position = sort_variable(inputs, variable, variables, control)?;
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let selected =
                crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&position)
                    .filter(|position| {
                        position.is_finite() && *position >= 1.0 && position.fract() == 0.0
                    })
                    .and_then(|position| position.to_string().parse::<usize>().ok())
                    .and_then(|position| nodes.get(position.saturating_sub(1)).copied());
            Ok(selected.map_or_else(String::new, |selected| source.string_value(selected)))
        }
        SortSelect::Xslt10ChildNameEqualsVariable { variable } => {
            evaluate_xslt10_child_name_variable_sort(
                inputs, source, node, variable, variables, control,
            )
        }
        SortSelect::Xslt10KeyLookup(lookup) => {
            Ok(
                key_lookup::select(inputs, lookup, Some(node), variables, control)?
                    .first()
                    .map_or_else(String::new, |selected| source.string_value(*selected)),
            )
        }
        SortSelect::PathUnion(alternatives) => {
            Ok(
                evaluate_source_path_union(inputs, source, node, alternatives, control)?
                    .first()
                    .map_or_else(String::new, |selected| source.string_value(*selected)),
            )
        }
        SortSelect::Literal(value) => Ok(value.clone()),
        SortSelect::Variable(name) => sort_variable(inputs, name, variables, control),
        SortSelect::ContextPosition => Ok(focus.position.to_string()),
        SortSelect::ContextSize => Ok(focus.size.to_string()),
        SortSelect::ContextNodeName => {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(source.name(node).map_or_else(String::new, |name| {
                source.prefix(node).map_or_else(
                    || name.local.clone(),
                    |prefix| format!("{prefix}:{}", name.local),
                )
            }))
        }
        SortSelect::ContextStringLength => Ok(evaluate_sort_context_string_length(
            source,
            node,
            inputs.request_id,
            &sort.location,
            control,
        )?
        .to_string()),
        SortSelect::CountPath(path) => {
            let count = evaluate_location_path_controlled(source, node, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?
                .len();
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(count.to_string())
        }
        SortSelect::NumberPath(path) => evaluate_sort_number_path(
            source,
            node,
            path,
            sort.xslt10_numeric_conversion,
            inputs.request_id,
            control,
        ),
    }
}

fn evaluate_xslt10_child_name_variable_sort(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    node: NodeId,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let expected = sort_variable(inputs, variable, variables, control)?;
    for child in source.children(node) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if source.kind(*child) != NodeKind::Element {
            continue;
        }
        let matches = source.name(*child).is_some_and(|name| {
            source.prefix(*child).map_or_else(
                || name.local == expected,
                |prefix| format!("{prefix}:{}", name.local) == expected,
            )
        });
        if matches {
            return Ok(source.string_value(*child));
        }
    }
    Ok(String::new())
}

fn evaluate_sort_control(
    inputs: &SequenceInputs<'_>,
    sort: &SortKey,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<EvaluatedSortControl, ExecutionFailure> {
    let data_type = match &sort.data_type {
        SortDataType::Text => EvaluatedSortDataType::Text,
        SortDataType::Number => EvaluatedSortDataType::Number,
        SortDataType::Variable(variable) => {
            match sort_variable(inputs, variable, variables, control)?.as_str() {
                "text" => EvaluatedSortDataType::Text,
                "number" => EvaluatedSortDataType::Number,
                value => {
                    return Err(failure_at(
                        "FXST1044",
                        FailureCategory::Unsupported,
                        Some(inputs.request_id),
                        sort.location.clone(),
                        format!("unsupported dynamic xsl:sort data-type: {value}"),
                    ));
                }
            }
        }
    };
    let order = match &sort.order {
        SortOrder::Ascending => EvaluatedSortOrder::Ascending,
        SortOrder::Descending => EvaluatedSortOrder::Descending,
        SortOrder::Variable(variable) => {
            match sort_variable(inputs, variable, variables, control)?.as_str() {
                "ascending" => EvaluatedSortOrder::Ascending,
                "descending" => EvaluatedSortOrder::Descending,
                value => {
                    return Err(failure_at(
                        "XTDE0030",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        sort.location.clone(),
                        format!("invalid dynamic xsl:sort order: {value}"),
                    ));
                }
            }
        }
    };
    Ok(EvaluatedSortControl { data_type, order })
}

fn typed_sort_key(
    data_type: EvaluatedSortDataType,
    xslt10_numeric_conversion: bool,
    value: String,
) -> EvaluatedSortKey {
    match data_type {
        EvaluatedSortDataType::Text => EvaluatedSortKey::Text(value),
        EvaluatedSortDataType::Number => EvaluatedSortKey::Number(if xslt10_numeric_conversion {
            crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&value)
        } else {
            value.trim().parse().ok()
        }),
    }
}

fn evaluate_sort_number_path(
    source: &Document,
    context: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    xslt10_compatibility: bool,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    if !xslt10_compatibility && selected.len() > 1 {
        return Err(failure_at(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(request_id),
            path.location.clone(),
            "fn:number requires a zero-or-one item argument",
        ));
    }
    let Some(node) = selected.first().copied() else {
        return Ok("NaN".to_owned());
    };
    let lexical = source
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    crate::xpath::constant_numeric_experiment::evaluate_number_lexical(&lexical).map_err(
        |numeric_failure| match numeric_failure {
            crate::xpath::constant_numeric_experiment::ConstantNumericFailure::Invalid => {
                failure_at(
                    "FORG0001",
                    FailureCategory::Invalid,
                    Some(request_id),
                    path.location.clone(),
                    format!("value is not a valid finite numeric lexical: {lexical}"),
                )
            }
            crate::xpath::constant_numeric_experiment::ConstantNumericFailure::Unsupported => {
                failure_at(
                    "FXXP1020",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    path.location.clone(),
                    format!(
                        "numeric lexical is outside the admitted finite-decimal slice: {lexical}"
                    ),
                )
            }
        },
    )
}

fn evaluate_sort_context_string_length(
    source: &Document,
    node: NodeId,
    request_id: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    let mut length = 0_usize;
    source
        .visit_string_value_controlled(node, control, &mut |part, control| {
            for _ in part.chars() {
                control
                    .charge(WorkDomain::XPathOperation, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                length = length.checked_add(1).ok_or_else(|| {
                    failure_at(
                        "FOAR0002",
                        FailureCategory::Invalid,
                        Some(request_id),
                        location.clone(),
                        "sort-key string length exceeds the supported integer range",
                    )
                })?;
            }
            Ok(())
        })
        .map_err(|failure| match failure {
            StringValueVisitFailure::Control(failure) => control_failure(failure, request_id),
            StringValueVisitFailure::Sink(failure) => failure,
        })?;
    Ok(length)
}

fn sort_variable(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    value_evaluator::xslt10_variable_string_value(inputs, variable, variables, control)
}

fn compare_sort_keys(left: &EvaluatedSortKey, right: &EvaluatedSortKey) -> std::cmp::Ordering {
    match (left, right) {
        (EvaluatedSortKey::Text(left), EvaluatedSortKey::Text(right)) => {
            left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase())
        }
        (EvaluatedSortKey::Number(left), EvaluatedSortKey::Number(right)) => match (left, right) {
            (Some(left), Some(right)) => left
                .partial_cmp(right)
                .expect("evaluated numeric sort keys exclude NaN"),
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
        },
        _ => unreachable!("a compiled sort key has one stable data type"),
    }
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
    execution: SequenceContext<'_>,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    match instruction {
        Instruction::Variable { name, select, .. } => {
            let value = execute_variable_binding(inputs, name, select, execution.node, control)?;
            scope.bind_atomic(name.clone(), value);
        }
        Instruction::StaticAtomicVariable { name, value, .. } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            scope.bind_atomic(name.clone(), value.clone());
        }
        Instruction::VariableAlias {
            name,
            source,
            location,
        } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if !scope.bind_alias(name.clone(), source, inputs.globals) {
                return Err(failure_at(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    location.clone(),
                    format!("unbound local variable alias: ${source}"),
                ));
            }
        }
        Instruction::ContextPositionVariable { name, offset, .. } => {
            bind_context_position(inputs, execution, name, *offset, scope, control)?;
        }
        Instruction::ContextNodeNameVariable { name, location } => {
            bind_context_node_name(inputs, execution, name, location, scope, control)?;
        }
        Instruction::ContextCountPathVariable { name, select, .. } => {
            bind_context_count_path(inputs, execution, name, select, scope, control)?;
        }
        Instruction::Xslt10BinaryNumericVariable { name, select, .. } => {
            bind_binary_numeric_variable(inputs, execution, name, select, scope, control)?;
        }
        Instruction::SourceNodeVariable { name, select, .. } => {
            bind_source_node_variable(inputs, execution, name, select, scope, control)?;
        }
        instruction @ Instruction::SourceVariablePathVariable { .. } => {
            bind_source_variable_path_variable(inputs, instruction, scope, control)?;
        }
        instruction @ Instruction::SourceNodeUnionVariable { .. } => {
            bind_source_node_union(inputs, instruction, scope, control)?;
        }
        Instruction::IntegerRangeVariable {
            name, start, end, ..
        } => {
            let values = materialize_integer_range(*start, *end, inputs.request_id, control)?;
            scope.bind_atomic_sequence(name.clone(), values);
        }
        Instruction::TemporaryTreeVariable { name, elements, .. } => {
            let tree = materialize_temporary_tree(elements, inputs.request_id, control)?;
            scope.bind_temporary_tree(name.clone(), tree);
        }
        Instruction::Xslt10TextTreeVariable { name, value, .. } => {
            bind_xslt10_text_tree(scope, name, value, inputs.request_id, control)?;
        }
        Instruction::Xslt10ValueOfTreeVariable { name, select, .. } => {
            bind_xslt10_value_of_tree(inputs, execution, scope, name, select, control)?;
        }
        Instruction::Xslt10ForEachTextTreeVariable { name, select, .. } => {
            bind_xslt10_for_each_text_tree(inputs, scope, name, select, execution.node, control)?;
        }
        Instruction::Xslt10SequenceTreeVariable { name, body, .. } => {
            let nodes = execute_sequence(inputs, body, execution, scope, control)?;
            let tree = materialize_result_nodes(&nodes, inputs.request_id, control)?;
            scope.bind_temporary_tree(name.clone(), tree);
        }
        _ => unreachable!("execute_binding receives a variable instruction"),
    }
    Ok(())
}

fn bind_source_node_union(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let Instruction::SourceNodeUnionVariable { name, sources, .. } = instruction else {
        unreachable!("source-node union binder receives one union binding")
    };
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "source-node union variable requires a source document",
        )
    })?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut nodes = Vec::new();
    for source_name in sources {
        let selected = scope
            .source_nodes(inputs.globals, source_name)
            .ok_or_else(|| {
                failure(
                    "XPTY0004",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    format!("node-set union requires a source-node sequence: ${source_name}"),
                )
            })?;
        nodes.extend(selected.iter().copied());
    }
    nodes.sort_unstable_by_key(|node| source.document_order(*node));
    nodes.dedup();
    scope.bind_source_nodes(name.clone(), nodes);
    Ok(())
}

fn bind_source_node_variable(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    name: &str,
    select: &crate::xpath::path_experiment::LocationPath,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, execution.node)?;
    let nodes = evaluate_location_path_controlled(source, context, select, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    scope.bind_source_nodes(name.to_owned(), nodes);
    Ok(())
}

fn bind_source_variable_path_variable(
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
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "source-variable path binding requires a principal source",
        )
    })?;
    let selected =
        evaluate_source_variable_path(inputs, source, source_name, select, scope, control)?;
    scope.bind_source_nodes(name.to_owned(), selected);
    Ok(())
}

fn bind_binary_numeric_variable(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    name: &str,
    select: &crate::xpath::binary_numeric_experiment::BinaryNumericExpression,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = value_evaluator::evaluate_binary_numeric_value(
        inputs,
        execution.node,
        execution.sequence_focus(),
        select,
        scope,
        control,
    )?;
    scope.bind_atomic(
        name.to_owned(),
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Double, value),
    );
    Ok(())
}

fn bind_context_position(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    name: &str,
    offset: usize,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let position = execution
        .focus_position
        .checked_add(offset)
        .ok_or_else(|| {
            failure(
                "FOAR0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                "context position offset exceeds the supported integer range",
            )
        })?;
    let value =
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, position.to_string());
    scope.bind_atomic(name.to_owned(), value);
    Ok(())
}

fn bind_context_node_name(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    name: &str,
    location: &SourceLocation,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let value = execution_context_lexical_name(inputs, execution, location, control)?;
    scope.bind_atomic(name.to_owned(), AtomicValue::string(value));
    Ok(())
}

fn bind_context_count_path(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    name: &str,
    select: &crate::xpath::path_experiment::LocationPath,
    scope: &mut RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, execution.node)?;
    let count = evaluate_location_path_controlled(source, context, select, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?
        .len();
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    scope.bind_atomic(
        name.to_owned(),
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, count.to_string()),
    );
    Ok(())
}

fn bind_xslt10_value_of_tree(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    scope: &mut RuntimeVariables,
    name: &str,
    select: &crate::xslt::golden_semantics_experiment::ValueExpression,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let value = value_evaluator::evaluate_as_temporary_text(
        inputs,
        select,
        execution.node,
        execution.focus_position,
        execution.focus_size,
        scope,
        control,
    )?;
    let tree = runtime_context::materialize_parentless_temporary_node(
        TemporaryNodeKind::Text(value),
        inputs.request_id,
        control,
    )?;
    scope.bind_temporary_tree(name.to_owned(), tree);
    Ok(())
}

fn bind_xslt10_text_tree(
    scope: &mut RuntimeVariables,
    name: &str,
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let tree = runtime_context::materialize_parentless_temporary_node(
        TemporaryNodeKind::Text(value.to_owned()),
        request_id,
        control,
    )?;
    scope.bind_temporary_tree(name.to_owned(), tree);
    Ok(())
}

fn bind_xslt10_for_each_text_tree(
    inputs: &SequenceInputs<'_>,
    scope: &mut RuntimeVariables,
    name: &str,
    select: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let tree = runtime_context::materialize_xslt10_for_each_text_tree(
        source,
        context,
        select,
        inputs.request_id,
        control,
    )?;
    scope.bind_temporary_tree(name.to_owned(), tree);
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
        attributes,
        body,
        recover_unattached_attributes,
        ..
    } = instruction
    else {
        unreachable!("execute_copy_instruction receives xsl:copy")
    };
    if execution.temporary_focus.is_some() {
        return temporary_tree_executor::execute_temporary_copy(
            inputs,
            attributes,
            body,
            *recover_unattached_attributes,
            execution,
            variables,
            control,
        );
    }
    execute_source_element_copy(
        inputs,
        attributes,
        body,
        *recover_unattached_attributes,
        execution,
        variables,
        control,
    )
}

fn execute_source_element_copy(
    inputs: &SequenceInputs<'_>,
    attributes: &[crate::xslt::golden_semantics_experiment::LiteralAttribute],
    body: &[Instruction],
    recover_unattached_attributes: bool,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, node) = required_source_context(inputs, execution.node)?;
    match source.kind(node) {
        NodeKind::Document => {
            let context_string = literal_attributes_require_context_string(attributes)
                .then(|| source.string_value_controlled(node, control))
                .transpose()
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let mut copied = materialize_literal_attributes(
                inputs,
                attributes,
                variables,
                LiteralAttributeFocus {
                    position: execution.focus_position,
                    size: execution.focus_size,
                    name: None,
                    value: context_string.as_deref(),
                    source: Some((source, node)),
                },
                inputs.request_id,
                control,
            )?
            .into_iter()
            .map(|attribute| {
                result_tree::pending_attribute(attribute, recover_unattached_attributes)
            })
            .collect::<Vec<_>>();
            copied.extend(execute_sequence(
                inputs, body, execution, variables, control,
            )?);
            Ok(copied)
        }
        NodeKind::Element => execute_source_element_copy_element(
            inputs, source, node, attributes, body, execution, variables, control,
        ),
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
        NodeKind::Comment => Ok(vec![construct_comment(
            source.value(node).unwrap_or_default(),
            inputs.request_id,
            control,
        )?]),
        NodeKind::Attribute => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(vec![result_tree::pending_attribute(
                ResultAttribute {
                    name: source
                        .name(node)
                        .expect("source attribute has a name")
                        .clone(),
                    value: source.string_value(node),
                },
                recover_unattached_attributes,
            )])
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_source_element_copy_element(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    node: NodeId,
    attributes: &[crate::xslt::golden_semantics_experiment::LiteralAttribute],
    body: &[Instruction],
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let context_string = literal_attributes_require_context_string(attributes)
        .then(|| source.string_value_controlled(node, control))
        .transpose()
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut result_attributes = materialize_literal_attributes(
        inputs,
        attributes,
        variables,
        LiteralAttributeFocus {
            position: execution.focus_position,
            size: execution.focus_size,
            name: source.name(node),
            value: context_string.as_deref(),
            source: Some((source, node)),
        },
        inputs.request_id,
        control,
    )?;
    let body = execute_sequence(inputs, body, execution, variables, control)?;
    let children =
        result_tree::assemble_element_content(&mut result_attributes, body, inputs.request_id)?;
    Ok(vec![ResultNode::Element {
        name: source
            .name(node)
            .expect("element context has a name")
            .clone(),
        namespaces: source.in_scope_namespaces(node).into(),
        attributes: result_attributes,
        children,
    }])
}

fn execute_literal_element(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    let ElementExecutionParts {
        name,
        namespaces,
        attributes,
        computed_attributes,
        body,
    } = prepare_element_execution(inputs, instruction, execution, variables, control)?;
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let context_string = (literal_attributes_require_context_string(attributes)
        || result_tree::computed_attributes_require_context_string(computed_attributes))
    .then(|| execution_context_string_value(inputs, execution, control))
    .transpose()?
    .flatten();
    let mut attributes = materialize_literal_attributes(
        inputs,
        attributes,
        variables,
        LiteralAttributeFocus {
            position: execution.focus_position,
            size: execution.focus_size,
            name: execution_context_name(inputs, execution),
            value: context_string.as_deref(),
            source: execution_source_focus(inputs, execution),
        },
        inputs.request_id,
        control,
    )?;
    let materialized_computed_attributes = materialize_computed_attributes(
        inputs,
        computed_attributes,
        variables,
        execution,
        LiteralAttributeFocus {
            position: execution.focus_position,
            size: execution.focus_size,
            name: execution_context_name(inputs, execution),
            value: context_string.as_deref(),
            source: execution_source_focus(inputs, execution),
        },
        inputs.request_id,
        control,
    )?;
    for (definition, attribute) in computed_attributes
        .iter()
        .zip(materialized_computed_attributes)
    {
        if definition.recover_duplicate {
            attributes.retain(|existing| existing.name != attribute.name);
        }
        attributes.push(attribute);
    }
    let body = execute_sequence(inputs, body, execution, variables, control)?;
    let children = result_tree::assemble_element_content(&mut attributes, body, inputs.request_id)?;
    let result_namespaces =
        result_tree::retain_dynamic_attribute_namespace_bindings(namespaces, &attributes);
    #[cfg(test)]
    let result_namespaces = if control.complete_result_namespace_clones() {
        Arc::from(result_namespaces.as_ref())
    } else {
        result_namespaces
    };
    Ok(ResultNode::Element {
        name,
        namespaces: result_namespaces,
        attributes,
        children,
    })
}

struct ElementExecutionParts<'a> {
    name: ExpandedName,
    namespaces: Arc<[NamespaceBinding]>,
    attributes: &'a [LiteralAttribute],
    computed_attributes: &'a [ComputedAttribute],
    body: &'a [Instruction],
}

fn prepare_element_execution<'a>(
    inputs: &SequenceInputs<'_>,
    instruction: &'a Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<ElementExecutionParts<'a>, ExecutionFailure> {
    match instruction {
        Instruction::LiteralElement {
            name,
            namespaces,
            attributes,
            computed_attributes,
            body,
            ..
        } => Ok(ElementExecutionParts {
            name: name.clone(),
            namespaces: Arc::clone(namespaces),
            attributes,
            computed_attributes,
            body,
        }),
        Instruction::ContextNameElement {
            namespace_override,
            static_namespaces,
            computed_attributes,
            body,
            location,
        } => {
            let namespace_override = resolve_dynamic_element_namespace(
                inputs,
                execution,
                namespace_override.as_ref(),
                control,
            )?;
            let (name, namespaces) = resolve_context_element_name(
                inputs,
                execution,
                namespace_override.as_deref(),
                static_namespaces,
                location,
                control,
            )?;
            Ok(ElementExecutionParts {
                name,
                namespaces,
                attributes: &[],
                computed_attributes,
                body,
            })
        }
        Instruction::DynamicNameElement {
            name,
            namespace_override,
            static_namespaces,
            computed_attributes,
            body,
            location,
        } => {
            let (name, namespaces) = resolve_dynamic_element_name(
                inputs,
                execution,
                DynamicElementNameRequest {
                    name,
                    variables,
                    namespace_override: namespace_override.as_ref(),
                    static_namespaces,
                    location,
                },
                control,
            )?;
            Ok(ElementExecutionParts {
                name,
                namespaces,
                attributes: &[],
                computed_attributes,
                body,
            })
        }
        _ => unreachable!("execute_literal_element receives an element instruction"),
    }
}

fn resolve_context_element_name(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    namespace_override: Option<&str>,
    static_namespaces: &[crate::xml::quick_xml_experiment::NamespaceBinding],
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<
    (
        crate::xml::quick_xml_experiment::ExpandedName,
        Arc<[crate::xml::quick_xml_experiment::NamespaceBinding]>,
    ),
    ExecutionFailure,
> {
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let context_name = execution_context_name(inputs, execution).ok_or_else(|| {
        failure_at(
            "XTDE0820",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "xsl:element name AVT evaluated to an empty lexical QName",
        )
    })?;
    let prefix = execution_context_prefix(inputs, execution);
    let namespace = match namespace_override {
        Some("") => None,
        Some(namespace) => Some(namespace.to_owned()),
        None => static_namespaces
            .iter()
            .find(|binding| binding.prefix.as_deref() == prefix)
            .map(|binding| binding.namespace.clone()),
    };
    if prefix.is_some() && namespace.is_none() {
        return Err(failure_at(
            "XTDE0830",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "xsl:element dynamic name uses a prefix not bound by its static namespace context",
        ));
    }
    let namespaces = match (prefix, namespace.as_deref()) {
        (Some(prefix), Some(namespace)) => {
            Arc::from([crate::xml::quick_xml_experiment::NamespaceBinding {
                prefix: Some(prefix.to_owned()),
                namespace: namespace.to_owned(),
            }])
        }
        (None, Some(namespace)) => {
            Arc::from([crate::xml::quick_xml_experiment::NamespaceBinding {
                prefix: None,
                namespace: namespace.to_owned(),
            }])
        }
        _ => Arc::from([]),
    };
    Ok((
        crate::xml::quick_xml_experiment::ExpandedName {
            namespace,
            local: context_name.local.clone(),
        },
        namespaces,
    ))
}

fn execution_source_focus<'a>(
    inputs: &'a SequenceInputs<'a>,
    execution: SequenceContext<'a>,
) -> Option<(&'a Document, NodeId)> {
    if execution.temporary_focus.is_some() || execution.atomic_focus.is_some() {
        return None;
    }
    inputs.source.zip(execution.node)
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

fn execution_context_prefix<'a>(
    inputs: &'a SequenceInputs<'a>,
    execution: SequenceContext<'a>,
) -> Option<&'a str> {
    if execution.temporary_focus.is_some() || execution.atomic_focus.is_some() {
        return None;
    }
    execution
        .node
        .and_then(|node| inputs.source.and_then(|source| source.prefix(node)))
}

fn execution_context_lexical_name(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    if let Some(TemporaryFocus::Node(tree, node)) = execution.temporary_focus {
        return Ok(match &tree.nodes[node].kind {
            TemporaryNodeKind::Element { name, .. } | TemporaryNodeKind::Attribute { name, .. } => {
                name.local.clone()
            }
            TemporaryNodeKind::Text(_)
            | TemporaryNodeKind::Comment(_)
            | TemporaryNodeKind::ProcessingInstruction { .. } => String::new(),
        });
    }
    if execution.atomic_focus.is_some() {
        return Err(failure_at(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "fn:name requires a node context item",
        ));
    }
    let (source, node) = execution_source_focus(inputs, execution).ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "fn:name requires a context node",
        )
    })?;
    let Some(name) = source.name(node) else {
        return Ok(String::new());
    };
    Ok(source.prefix(node).map_or_else(
        || name.local.clone(),
        |prefix| format!("{prefix}:{}", name.local),
    ))
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

fn execution_context_string_value(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
    if let Some(focus) = execution.temporary_focus {
        return match focus {
            TemporaryFocus::Document(tree) => {
                runtime_context::temporary_tree_string_value(tree, inputs.request_id, control)
                    .map(Some)
            }
            TemporaryFocus::Node(tree, node) => {
                runtime_context::temporary_node_string_value(tree, node, inputs.request_id, control)
                    .map(Some)
            }
        };
    }
    if let Some(value) = execution.atomic_focus {
        return Ok(Some(value.to_string()));
    }
    execution
        .node
        .map(|node| {
            inputs
                .source
                .expect("source focus requires a source document")
                .string_value_controlled(node, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))
        })
        .transpose()
}

fn execute_apply_instruction(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let Instruction::ApplyTemplates {
        select,
        sorts,
        mode,
        arguments,
        ..
    } = instruction
    else {
        unreachable!("apply instruction dispatch receives only xsl:apply-templates")
    };
    let requested_mode = match mode {
        Some(mode) if mode == "#current" => execution.current_mode,
        Some(mode) if mode == "#default" => None,
        Some(mode) => Some(mode.as_str()),
        None => None,
    };
    let parameters = evaluate_template_arguments(arguments, variables, inputs, execution, control)?;
    execute_apply_templates(
        inputs,
        ApplyExecutionPlan {
            select: select.as_ref(),
            sorts,
            mode: requested_mode,
        },
        execution,
        &parameters,
        variables,
        control,
    )
}

#[derive(Clone, Copy)]
struct ApplyExecutionPlan<'a> {
    select: Option<&'a ApplySelection>,
    sorts: &'a [SortKey],
    mode: Option<&'a str>,
}

fn execute_apply_templates(
    inputs: &SequenceInputs<'_>,
    plan: ApplyExecutionPlan<'_>,
    execution: SequenceContext<'_>,
    parameters: &BTreeMap<String, InvocationParameter>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let ApplyExecutionPlan {
        select,
        sorts,
        mode,
    } = plan;
    if sorts.is_empty()
        && let Some(result) = execute_special_apply_selection(
            inputs, select, mode, execution, parameters, variables, control,
        )?
    {
        return Ok(result);
    }
    let (_, context) = required_source_context(inputs, execution.node)?;
    let selected = select_apply_nodes(inputs, select, context, variables, control)?;
    let selected = sort_selected_nodes(inputs, selected, sorts, variables, control)?;
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

fn apply_variable_sequence(
    inputs: &SequenceInputs<'_>,
    name: &str,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some(nodes) = variables.source_nodes(inputs.globals, name) {
        let mut result = Vec::new();
        let focus_size = nodes.len();
        for (offset, node) in nodes.iter().copied().enumerate() {
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
        return Ok(result);
    }
    if let Some(tree) = variables.temporary_tree(inputs.globals, name) {
        return apply_temporary_roots(inputs, tree, mode, parameters, control);
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        format!("unbound or unsupported sequence variable: ${name}"),
    ))
}

fn execute_special_apply_selection(
    inputs: &SequenceInputs<'_>,
    select: Option<&ApplySelection>,
    mode: Option<&str>,
    execution: SequenceContext<'_>,
    parameters: &BTreeMap<String, InvocationParameter>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Option<Vec<ResultNode>>, ExecutionFailure> {
    if let Some(ApplySelection::AtomicIntegerRange { start, end }) = select {
        return atomic_template_executor::apply_integer_range(
            inputs, *start, *end, mode, parameters, control,
        )
        .map(Some);
    }
    if let Some(ApplySelection::VariableSequence(name)) = select {
        return apply_variable_sequence(inputs, name, mode, parameters, variables, control)
            .map(Some);
    }
    if let Some(ApplySelection::TemporaryPath { variable, steps }) = select {
        if let Some(selected) =
            select_source_variable_path(inputs, variables, variable, steps, control)
        {
            let selected = selected?;
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
            return Ok(Some(result));
        }
        let tree = variables
            .temporary_tree(inputs.globals, variable)
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
        )
        .map(Some);
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
        return Ok(Some(result));
    }
    if select.is_none()
        && let Some(focus) = execution.temporary_focus
    {
        return temporary_tree_executor::apply_temporary_builtin(
            inputs, focus, mode, parameters, control,
        )
        .map(Some);
    }
    Ok(None)
}

fn select_source_variable_path(
    inputs: &SequenceInputs<'_>,
    variables: &RuntimeVariables,
    variable: &str,
    steps: &[ExpandedName],
    control: &mut InvocationControl,
) -> Option<Result<Vec<NodeId>, ExecutionFailure>> {
    let source = inputs.source?;
    let mut selected = variables.source_nodes(inputs.globals, variable)?.clone();
    for step in steps {
        let mut next = Vec::new();
        for node in selected {
            for child in source.children(node).iter().copied() {
                if let Err(charge_failure) = control.charge(WorkDomain::XPathNodeVisit, 1) {
                    return Some(Err(control_failure(charge_failure, inputs.request_id)));
                }
                if source.kind(child) == NodeKind::Element && source.name(child) == Some(step) {
                    next.push(child);
                }
            }
        }
        next.sort_unstable_by_key(|node| source.document_order(*node));
        next.dedup();
        selected = next;
    }
    Some(Ok(selected))
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
    let parameters = evaluate_template_arguments(arguments, variables, inputs, execution, control)?;
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
            inputs,
            execution.node,
            control,
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
    let parameters = evaluate_template_arguments(arguments, variables, inputs, execution, control)?;
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
            inputs,
            execution.node,
            control,
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
        result.extend(copy_source_node(
            source,
            inputs.request_id,
            node,
            false,
            control,
        )?);
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
                        result.extend(copy_source_node(
                            source,
                            inputs.request_id,
                            child,
                            false,
                            control,
                        )?);
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
    if evaluate_boolean(
        inputs,
        test,
        execution.node,
        execution.sequence_focus(),
        variables,
        control,
    )? {
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
        if evaluate_boolean(
            inputs,
            &branch.test,
            execution.node,
            execution.sequence_focus(),
            variables,
            control,
        )? {
            return execute_sequence(inputs, &branch.body, execution, variables, control);
        }
    }
    execute_sequence(inputs, otherwise, execution, variables, control)
}

fn evaluate_xslt10_template_parameter_text_choice(
    inputs: &SequenceInputs<'_>,
    branches: &[crate::xslt::golden_semantics_experiment::Xslt10TextChoiceBranch],
    otherwise: &str,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    for branch in branches {
        if evaluate_boolean(inputs, &branch.test, context, None, variables, control)? {
            return Ok(branch.value.clone());
        }
    }
    Ok(otherwise.to_owned())
}

fn evaluate_boolean(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    context: Option<NodeId>,
    focus: Option<SequenceFocus>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if let Some(identity) =
        evaluate_identity_boolean(inputs, expression, context, variables, control)?
    {
        return Ok(identity);
    }
    if let BooleanExpression::Xslt10VariableStringLengthComparison {
        string_variable,
        operator,
        numeric_variable,
    } = expression
    {
        return evaluate_xslt10_variable_string_length_comparison(
            inputs,
            string_variable,
            *operator,
            numeric_variable,
            variables,
            control,
        );
    }
    if let BooleanExpression::Xslt10VariableNumericComparison {
        left,
        operator,
        right,
    } = expression
    {
        return evaluate_xslt10_variable_numeric_comparison(
            inputs, left, *operator, right, variables, control,
        );
    }
    if let BooleanExpression::Xslt10VariableLessThanNodeCount {
        numeric_variable,
        nodes_variable,
    } = expression
    {
        return evaluate_xslt10_variable_less_than_node_count(
            inputs,
            numeric_variable,
            nodes_variable,
            variables,
            control,
        );
    }
    if let BooleanExpression::Xslt10ContextTranslateStartsWith(expression) = expression {
        return evaluate_xslt10_context_translate_starts_with(inputs, context, expression, control);
    }
    if let BooleanExpression::Xslt10ContextPositionModuloVariable { divisor, location } = expression
    {
        return evaluate_xslt10_context_position_modulo_variable(
            inputs, focus, divisor, location, variables, control,
        );
    }
    if let BooleanExpression::Xslt10VariableStringLength(variable) = expression {
        return value_evaluator::xslt10_variable_string_length(
            inputs, variable, variables, control,
        )
        .map(|length| length != 0);
    }
    if let BooleanExpression::Xslt10ChildAttributeVariableEquals {
        child,
        attribute,
        variable,
    } = expression
    {
        return evaluate_xslt10_child_attribute_variable_equals(
            inputs, context, child, attribute, variable, variables, control,
        );
    }
    if let BooleanExpression::Xslt10ContextNodeSetEqualsVariable { variable, location } = expression
    {
        return evaluate_xslt10_context_node_set_equals_variable(
            inputs, context, variable, location, variables, control,
        );
    }
    if let BooleanExpression::Xslt10AncestorFilter(filter) = expression {
        return evaluate_xslt10_ancestor_filter(inputs, context, filter, control);
    }
    if let BooleanExpression::Xslt10KeyLookupEffectiveBooleanValue(lookup) = expression {
        return key_lookup::select(inputs, lookup, context, variables, control)
            .map(|selected| !selected.is_empty());
    }
    if let Some(value) =
        evaluate_xslt10_current_name_boolean(inputs, expression, context, variables, control)
    {
        return value;
    }
    if let BooleanExpression::ContextStringEquals(expected) = expression {
        return evaluate_context_string_equals(inputs, context, expected, control);
    }
    if let BooleanExpression::Xslt10ContextNumberIsNaN = expression {
        return evaluate_xslt10_context_number_is_nan(inputs, context, control);
    }
    evaluate_ordinary_boolean(inputs, expression, context, focus, variables, control)
}

fn evaluate_xslt10_current_name_boolean(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Option<Result<bool, ExecutionFailure>> {
    match expression {
        BooleanExpression::Xslt10DescendantOrFollowingSameNameAsCurrent => {
            let (source, context) = match required_source_context(inputs, context) {
                Ok(value) => value,
                Err(failure) => return Some(Err(failure)),
            };
            Some(xslt10_current_name::has_descendant_or_following(
                source,
                context,
                inputs.request_id,
                control,
            ))
        }
        BooleanExpression::Xslt10PriorDescendantSameNameAsCurrent(position_variable) => {
            Some(evaluate_xslt10_prior_descendant_same_name(
                inputs,
                context,
                position_variable,
                variables,
                control,
            ))
        }
        _ => None,
    }
}

fn evaluate_xslt10_prior_descendant_same_name(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    position_variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let position =
        value_evaluator::xslt10_variable_number(inputs, position_variable, variables, control)?;
    xslt10_current_name::has_prior_descendant_same_name(
        source,
        context,
        position,
        inputs.request_id,
        control,
    )
}

fn evaluate_xslt10_variable_less_than_node_count(
    inputs: &SequenceInputs<'_>,
    numeric_variable: &str,
    nodes_variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let left =
        value_evaluator::xslt10_variable_number(inputs, numeric_variable, variables, control)?;
    let count = variables
        .source_nodes(inputs.globals, nodes_variable)
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("count() requires a source-node sequence: ${nodes_variable}"),
            )
        })?
        .len();
    let right = count
        .to_string()
        .parse::<f64>()
        .expect("a usize count always has a finite XPath number representation");
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(left < right)
}

fn evaluate_ordinary_boolean(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    context: Option<NodeId>,
    focus: Option<SequenceFocus>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    match expression {
        BooleanExpression::Constant(value) => Ok(*value),
        BooleanExpression::NodeExists(path) => node_exists(inputs, path, context, control),
        BooleanExpression::NodeStringEquals { path, value } => {
            evaluate_node_string_equals(inputs, path, value, context, control)
        }
        BooleanExpression::NodeIntegerLessThan { path, value } => {
            evaluate_node_integer_less_than(inputs, path, *value, context, control)
        }
        BooleanExpression::CountPathEquals { path, expected } => {
            count_path_eq(inputs, path, *expected, context, control)
        }
        BooleanExpression::UnqualifiedNodeNameEquals {
            path,
            local,
            comparison,
        } => node_name_path_equals(inputs, path, local, *comparison, context, control),
        BooleanExpression::ContextNodeNameEquals {
            lexical,
            comparison,
        } => evaluate_context_node_name_equals(inputs, context, lexical, *comparison, control),
        BooleanExpression::ContextStringLengthEquals(expected) => {
            evaluate_context_string_length(inputs, context, *expected, control)
        }
        position @ (BooleanExpression::ContextPositionNotEqualSize(_)
        | BooleanExpression::ContextPositionModuloEquals { .. }) => {
            evaluate_context_position_boolean(inputs, position, focus, control)
        }
        BooleanExpression::ContextFocusEquals {
            left,
            right,
            location,
        } => evaluate_context_focus_equality(inputs, focus, *left, *right, location, control),
        BooleanExpression::ContextFocusCompares {
            left,
            operator,
            right,
            location,
        } => evaluate_context_focus_comparison(
            inputs, focus, *left, *operator, *right, location, control,
        ),
        BooleanExpression::ContextLanguageMatches(language) => {
            evaluate_context_language_matches(inputs, context, language, control)
        }
        composition @ (BooleanExpression::Or { .. } | BooleanExpression::And { .. }) => {
            evaluate_boolean_composition(inputs, composition, context, focus, variables, control)
        }
        BooleanExpression::Not(expression) => {
            evaluate_boolean(inputs, expression, context, focus, variables, control)
                .map(|value| !value)
        }
        BooleanExpression::NodeIdentityEqual { .. }
        | BooleanExpression::RootIdentityEqualsVariable { .. }
        | BooleanExpression::TemporaryRootIdentityEqual { .. }
        | BooleanExpression::DocumentRootIdentityEqual { .. }
        | BooleanExpression::Xslt10VariableStringLengthComparison { .. }
        | BooleanExpression::Xslt10VariableNumericComparison { .. }
        | BooleanExpression::Xslt10VariableLessThanNodeCount { .. }
        | BooleanExpression::Xslt10ContextTranslateStartsWith(_)
        | BooleanExpression::Xslt10ContextPositionModuloVariable { .. }
        | BooleanExpression::Xslt10VariableStringLength(_)
        | BooleanExpression::Xslt10ChildAttributeVariableEquals { .. }
        | BooleanExpression::Xslt10ContextNodeSetEqualsVariable { .. }
        | BooleanExpression::Xslt10AncestorFilter(_)
        | BooleanExpression::Xslt10KeyLookupEffectiveBooleanValue(_)
        | BooleanExpression::Xslt10DescendantOrFollowingSameNameAsCurrent
        | BooleanExpression::Xslt10PriorDescendantSameNameAsCurrent(_)
        | BooleanExpression::ContextStringEquals(_)
        | BooleanExpression::Xslt10ContextNumberIsNaN => {
            unreachable!("specialized expressions return before ordinary boolean dispatch")
        }
        variable @ (BooleanExpression::VariableEqualsInteger(_)
        | BooleanExpression::VariableEqualsEmptySequence(_)
        | BooleanExpression::VariableEffectiveBooleanValue(_)
        | BooleanExpression::VariableStringEquals { .. }
        | BooleanExpression::Xslt10VariableStringLiteralEquals { .. }) => {
            evaluate_variable_boolean(inputs, variable, variables, control)
        }
        BooleanExpression::Xslt10SourcePathStringComparison { left, right, equal } => {
            runtime_context::evaluate_source_path_string_comparison(
                inputs, context, left, right, *equal, control,
            )
        }
        BooleanExpression::ConditionalInteger(expression) => {
            value_evaluator::evaluate_conditional_integer(inputs, expression, context, control)
                .map(|value| value != 0)
        }
    }
}

fn evaluate_variable_boolean(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    match expression {
        BooleanExpression::VariableEqualsInteger(test) => {
            evaluate_variable_integer_equality(inputs, test, variables, control)
        }
        BooleanExpression::VariableEqualsEmptySequence(variable) => {
            ensure_variable_is_bound(inputs, variable, variables).map(|()| false)
        }
        BooleanExpression::VariableEffectiveBooleanValue(variable) => {
            evaluate_variable_effective_boolean_value(inputs, variable, variables, control)
        }
        BooleanExpression::VariableStringEquals {
            left,
            right,
            comparison,
        } => {
            evaluate_variable_string_equality(inputs, left, right, *comparison, variables, control)
        }
        BooleanExpression::Xslt10VariableStringLiteralEquals {
            variable,
            literal,
            equal,
        } => value_evaluator::evaluate_xslt10_variable_string_comparison(
            inputs, variable, literal, *equal, variables, control,
        ),
        _ => unreachable!("variable boolean dispatcher receives only variable plans"),
    }
}

fn evaluate_xslt10_context_node_set_equals_variable(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    variable: &str,
    location: &SourceLocation,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let nodes = variables
        .source_nodes(inputs.globals, variable)
        .ok_or_else(|| {
            failure_at(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                location.clone(),
                format!("node-set comparison requires a source-node variable: ${variable}"),
            )
        })?;
    let context_value = source
        .string_value_controlled(context, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    for node in nodes {
        let value = source
            .string_value_controlled(*node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if value == context_value {
            return Ok(true);
        }
    }
    Ok(false)
}

fn evaluate_xslt10_ancestor_filter(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    filter: &Xslt10AncestorFilter,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let mut current = source.parent(context);
    let mut element_position = 0_usize;
    while let Some(ancestor) = current {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        current = source.parent(ancestor);
        if source.kind(ancestor) != NodeKind::Element {
            continue;
        }
        element_position += 1;
        if filter
            .position
            .is_some_and(|required| required != element_position)
        {
            continue;
        }
        let mut attribute_matches = false;
        for attribute in source.attributes(ancestor) {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if source
                .name(*attribute)
                .is_some_and(|name| name.namespace.is_none() && name.local == filter.attribute)
                && filter
                    .value
                    .as_deref()
                    .is_none_or(|value| source.value(*attribute) == Some(value))
            {
                attribute_matches = true;
                break;
            }
        }
        if attribute_matches
            && (!filter.require_absent_text_child
                || !ancestor_has_text_child(source, ancestor, inputs.request_id, control)?)
        {
            return Ok(true);
        }
        if filter.position.is_some() {
            return Ok(false);
        }
    }
    Ok(false)
}

fn ancestor_has_text_child(
    source: &Document,
    ancestor: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    for child in source.children(ancestor) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.kind(*child) == NodeKind::Text {
            return Ok(true);
        }
    }
    Ok(false)
}

fn evaluate_xslt10_child_attribute_variable_equals(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    child_name: &ExpandedName,
    attribute_name: &ExpandedName,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let expected =
        value_evaluator::xslt10_variable_string_value(inputs, variable, variables, control)?;
    let (source, context) = required_source_context(inputs, context)?;
    for child in source.children(context) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if source.kind(*child) != NodeKind::Element || source.name(*child) != Some(child_name) {
            continue;
        }
        for attribute in source.attributes(*child) {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if source.name(*attribute) == Some(attribute_name)
                && source.string_value(*attribute) == expected
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn evaluate_xslt10_context_position_modulo_variable(
    inputs: &SequenceInputs<'_>,
    focus: Option<SequenceFocus>,
    divisor: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let focus = focus.ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "position() requires a dynamic focus",
        )
    })?;
    let divisor = value_evaluator::xslt10_variable_number(inputs, divisor, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let position = focus
        .position
        .to_string()
        .parse::<f64>()
        .expect("a usize lexical always converts to f64");
    let remainder = position % divisor;
    Ok(remainder != 0.0 && !remainder.is_nan())
}

fn evaluate_xslt10_context_translate_starts_with(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expression: &crate::xslt::golden_semantics_experiment::Xslt10ContextTranslateStartsWith,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let value = source
        .string_value_controlled(context, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    control
        .charge(WorkDomain::XPathOperation, 2)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let translated = crate::xpath::static_string_experiment::evaluate_translate(
        &value,
        &expression.search,
        &expression.replacement,
    );
    Ok(translated.starts_with(&expression.prefix))
}

fn evaluate_xslt10_variable_string_length_comparison(
    inputs: &SequenceInputs<'_>,
    string_variable: &str,
    operator: FocusComparison,
    numeric_variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let length = value_evaluator::xslt10_variable_string_length(
        inputs,
        string_variable,
        variables,
        control,
    )?;
    let numeric =
        value_evaluator::xslt10_variable_number(inputs, numeric_variable, variables, control)?;
    let length = length
        .to_string()
        .parse::<f64>()
        .expect("usize lexical values are valid XPath doubles");
    Ok(compare_xpath_numbers(length, operator, numeric))
}

fn compare_xpath_numbers(left: f64, operator: FocusComparison, right: f64) -> bool {
    let ordering = left.partial_cmp(&right);
    match operator {
        FocusComparison::NotEqual => ordering != Some(std::cmp::Ordering::Equal),
        FocusComparison::LessThan => ordering == Some(std::cmp::Ordering::Less),
        FocusComparison::LessThanOrEqual => matches!(
            ordering,
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        ),
        FocusComparison::GreaterThan => ordering == Some(std::cmp::Ordering::Greater),
        FocusComparison::GreaterThanOrEqual => matches!(
            ordering,
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        ),
    }
}

fn evaluate_xslt10_variable_numeric_comparison(
    inputs: &SequenceInputs<'_>,
    left: &str,
    operator: FocusComparison,
    right: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let left = value_evaluator::xslt10_variable_number(inputs, left, variables, control)?;
    let right = value_evaluator::xslt10_variable_number(inputs, right, variables, control)?;
    Ok(compare_xpath_numbers(left, operator, right))
}

fn evaluate_identity_boolean(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Option<bool>, ExecutionFailure> {
    let value = match expression {
        BooleanExpression::NodeIdentityEqual { left, right } => {
            evaluate_node_identity_equal(inputs, left, right, context, control)?
        }
        BooleanExpression::RootIdentityEqualsVariable { path, variable } => {
            evaluate_root_identity_equals_variable(
                inputs, path, variable, variables, context, control,
            )?
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
        )?,
        BooleanExpression::DocumentRootIdentityEqual { left, right } => {
            let left = dynamic_document::document_root_identity(inputs, left, control)?;
            let right = dynamic_document::document_root_identity(inputs, right, control)?;
            left == right
        }
        _ => return Ok(None),
    };
    Ok(Some(value))
}

fn evaluate_boolean_composition(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    context: Option<NodeId>,
    focus: Option<SequenceFocus>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (left, right, conjunction) = match expression {
        BooleanExpression::And { left, right } => (left.as_ref(), right.as_ref(), true),
        BooleanExpression::Or { left, right } => (left.as_ref(), right.as_ref(), false),
        _ => unreachable!("boolean composition helper receives only and/or expressions"),
    };
    let left = evaluate_boolean(inputs, left, context, focus, variables, control)?;
    match (conjunction, left) {
        (true, false) => Ok(false),
        (false, true) => Ok(true),
        _ => evaluate_boolean(inputs, right, context, focus, variables, control),
    }
}

fn node_exists(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    evaluate_location_path_controlled(source, context, path, control)
        .map(|nodes| !nodes.is_empty())
        .map_err(|failure| control_failure(failure, inputs.request_id))
}

fn evaluate_context_position_not_equal_size(
    inputs: &SequenceInputs<'_>,
    focus: Option<SequenceFocus>,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let focus = focus.ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "position() and last() require a dynamic focus",
        )
    })?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(focus.position != focus.size)
}

fn evaluate_context_focus_equality(
    inputs: &SequenceInputs<'_>,
    focus: Option<SequenceFocus>,
    left: FocusEqualityOperand,
    right: FocusEqualityOperand,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let focus = focus.ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "position() and last() require a dynamic focus",
        )
    })?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(focus_operand_value(focus, left) == focus_operand_value(focus, right))
}

fn evaluate_context_focus_comparison(
    inputs: &SequenceInputs<'_>,
    focus: Option<SequenceFocus>,
    left: FocusEqualityOperand,
    operator: FocusComparison,
    right: FocusEqualityOperand,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let focus = focus.ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "position() and last() require a dynamic focus",
        )
    })?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let left = focus_operand_value(focus, left);
    let right = focus_operand_value(focus, right);
    Ok(match operator {
        FocusComparison::NotEqual => left != right,
        FocusComparison::LessThan => left < right,
        FocusComparison::LessThanOrEqual => left <= right,
        FocusComparison::GreaterThan => left > right,
        FocusComparison::GreaterThanOrEqual => left >= right,
    })
}

fn focus_operand_value(focus: SequenceFocus, operand: FocusEqualityOperand) -> usize {
    match operand {
        FocusEqualityOperand::Position => focus.position,
        FocusEqualityOperand::Size => focus.size,
        FocusEqualityOperand::CeilingHalfSize => focus.size / 2 + focus.size % 2,
        FocusEqualityOperand::Static(value) => value,
    }
}

fn evaluate_context_position_boolean(
    inputs: &SequenceInputs<'_>,
    expression: &BooleanExpression,
    focus: Option<SequenceFocus>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    match expression {
        BooleanExpression::ContextPositionNotEqualSize(location) => {
            evaluate_context_position_not_equal_size(inputs, focus, location, control)
        }
        BooleanExpression::ContextPositionModuloEquals {
            divisor,
            remainder,
            location,
        } => evaluate_context_position_modulo_equality(
            inputs, focus, *divisor, *remainder, location, control,
        ),
        _ => unreachable!("position expressions are selected before this helper"),
    }
}

fn evaluate_context_position_modulo_equality(
    inputs: &SequenceInputs<'_>,
    focus: Option<SequenceFocus>,
    divisor: usize,
    remainder: usize,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let focus = focus.ok_or_else(|| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "position() requires a dynamic focus",
        )
    })?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(focus.position % divisor == remainder)
}

fn evaluate_node_identity_equal(
    inputs: &SequenceInputs<'_>,
    left: &crate::xpath::path_experiment::LocationPath,
    right: &crate::xpath::path_experiment::LocationPath,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    runtime_context::source_node_identities_equal(
        source,
        context,
        left,
        right,
        inputs.request_id,
        control,
    )
}

fn evaluate_context_language_matches(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    language: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    crate::xpath::language_experiment::evaluate(source, context, language, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))
}

fn evaluate_node_integer_less_than(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    expected: i64,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
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
            .is_ok_and(|actual| actual < expected)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn count_path_eq(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    expected: usize,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let count = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?
        .len();
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(count == expected)
}

fn evaluate_node_string_equals(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    expected: &str,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let nodes = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    for node in nodes {
        let actual = source
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if actual == expected {
            return Ok(true);
        }
    }
    Ok(false)
}

fn evaluate_variable_string_equality(
    inputs: &SequenceInputs<'_>,
    left: &str,
    right: &str,
    comparison: StringComparison,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let left = variable_string_value(inputs, left, variables, control)?;
    let right = variable_string_value(inputs, right, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(match comparison {
        StringComparison::Codepoint => left == right,
        StringComparison::HtmlAsciiCaseInsensitive => left.eq_ignore_ascii_case(&right),
    })
}

fn variable_string_value(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    if let Some(value) = variables.atomics.get(variable) {
        return Ok(value.lexical().to_owned());
    }
    if variables.allows_global_fallback(variable)
        && let Some(value) = inputs.globals.atomics.get(variable)
    {
        return Ok(value.lexical().to_owned());
    }
    let tree = variables
        .temporary_tree(inputs.globals, variable)
        .ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound string-compatible variable: ${variable}"),
            )
        })?;
    runtime_context::temporary_tree_string_value(tree, inputs.request_id, control)
}

fn node_name_path_equals(
    inputs: &SequenceInputs<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    local: &str,
    comparison: StringComparison,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let nodes = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(nodes.into_iter().any(|node| {
        source.name(node).is_some_and(|name| {
            name.namespace.is_none()
                && match comparison {
                    StringComparison::Codepoint => name.local == local,
                    StringComparison::HtmlAsciiCaseInsensitive => {
                        name.local.eq_ignore_ascii_case(local)
                    }
                }
        })
    }))
}

fn evaluate_context_node_name_equals(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expected: &str,
    comparison: StringComparison,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let actual = source.name(context).map_or_else(String::new, |name| {
        source.prefix(context).map_or_else(
            || name.local.clone(),
            |prefix| format!("{prefix}:{}", name.local),
        )
    });
    Ok(match comparison {
        StringComparison::Codepoint => actual == expected,
        StringComparison::HtmlAsciiCaseInsensitive => actual.eq_ignore_ascii_case(expected),
    })
}

fn evaluate_variable_integer_equality(
    inputs: &SequenceInputs<'_>,
    test: &crate::xslt::golden_semantics_experiment::EqualityTest,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if test.xslt10_compatibility {
        let actual =
            value_evaluator::xslt10_variable_number(inputs, &test.variable, variables, control)?;
        let expected = test
            .integer
            .to_string()
            .parse::<f64>()
            .expect("i64 lexical values are valid XPath doubles");
        return Ok(actual.partial_cmp(&expected) == Some(std::cmp::Ordering::Equal));
    }
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

fn evaluate_variable_effective_boolean_value(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    if let Some(value) = variables.atomics.get(variable) {
        return atomic_effective_boolean_value(value, inputs.request_id);
    }
    if variables.allows_global_fallback(variable)
        && let Some(value) = inputs.globals.atomics.get(variable)
    {
        return atomic_effective_boolean_value(value, inputs.request_id);
    }
    if let Some(nodes) = variables.source_nodes.get(variable) {
        return Ok(!nodes.is_empty());
    }
    if variables.allows_global_fallback(variable)
        && let Some(nodes) = inputs.globals.nodes.get(variable)
    {
        return Ok(!nodes.is_empty());
    }
    if variables.temporary_trees.contains_key(variable) {
        return Ok(true);
    }
    if variables.allows_global_fallback(variable)
        && inputs.globals.temporary_trees.contains_key(variable)
    {
        return Ok(true);
    }
    if variables.allows_global_fallback(variable)
        && inputs.globals.empty_sequences.contains(variable)
    {
        return Ok(false);
    }
    if let Some(values) = variables.atomic_sequences.get(variable) {
        return match values.as_slice() {
            [] => Ok(false),
            [value] => atomic_effective_boolean_value(value, inputs.request_id),
            _ => Err(invalid_effective_boolean_value(inputs.request_id)),
        };
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        format!("unbound variable: ${variable}"),
    ))
}

fn ensure_variable_is_bound(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    variables: &RuntimeVariables,
) -> Result<(), ExecutionFailure> {
    let is_bound = variables.atomics.contains_key(variable)
        || variables.atomic_sequences.contains_key(variable)
        || variables.source_nodes.contains_key(variable)
        || variables.temporary_trees.contains_key(variable)
        || inputs.globals.atomics.contains_key(variable)
        || inputs.globals.empty_sequences.contains(variable)
        || inputs.globals.nodes.contains_key(variable)
        || inputs.globals.temporary_trees.contains_key(variable);
    if is_bound {
        return Ok(());
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        format!("unbound variable: ${variable}"),
    ))
}

fn atomic_effective_boolean_value(
    value: &AtomicValue,
    request_id: &str,
) -> Result<bool, ExecutionFailure> {
    match value.atomic_type() {
        BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic => {
            Ok(!value.lexical().is_empty())
        }
        BuiltinAtomicType::Boolean => Ok(matches!(value.lexical(), "true" | "1")),
        BuiltinAtomicType::Integer
        | BuiltinAtomicType::Decimal
        | BuiltinAtomicType::Float
        | BuiltinAtomicType::Double => value
            .lexical()
            .parse::<f64>()
            .map(|number| number != 0.0 && !number.is_nan())
            .map_err(|_| invalid_effective_boolean_value(request_id)),
        _ => Err(invalid_effective_boolean_value(request_id)),
    }
}

fn invalid_effective_boolean_value(request_id: &str) -> ExecutionFailure {
    failure(
        "FORG0006",
        FailureCategory::Invalid,
        Some(request_id),
        "the variable value has no effective boolean value",
    )
}

fn evaluate_context_string_equals(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expected: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    source
        .string_value_controlled(context, control)
        .map(|actual| actual == expected)
        .map_err(|failure| control_failure(failure, inputs.request_id))
}

fn evaluate_xslt10_context_number_is_nan(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let lexical = source
        .string_value_controlled(context, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&lexical).is_none())
}

fn evaluate_context_string_length(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    expected: usize,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let actual = source
        .string_value_controlled(context, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let length = actual.chars().count();
    control
        .charge(WorkDomain::XPathOperation, length.max(1))
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(length == expected)
}

fn evaluate_temporary_root_identity_equal(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    descendant_local: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let tree = variables
        .temporary_tree(inputs.globals, variable)
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
    recover_unattached_attributes: bool,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match source.kind(node) {
        NodeKind::Document => {
            let mut copied = Vec::new();
            for child in source.children(node).iter().copied() {
                copied.extend(copy_source_node(
                    source,
                    request_id,
                    child,
                    recover_unattached_attributes,
                    control,
                )?);
            }
            Ok(copied)
        }
        NodeKind::Element => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let mut children = Vec::new();
            for child in source.children(node).iter().copied() {
                children.extend(copy_source_node(
                    source,
                    request_id,
                    child,
                    recover_unattached_attributes,
                    control,
                )?);
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
                namespaces: source.in_scope_namespaces(node).into(),
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
        NodeKind::Attribute => {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            Ok(vec![result_tree::pending_attribute(
                ResultAttribute {
                    name: source
                        .name(node)
                        .expect("source attribute has a name")
                        .clone(),
                    value: source.string_value(node),
                },
                recover_unattached_attributes,
            )])
        }
        NodeKind::Comment => Ok(vec![construct_comment(
            source.value(node).unwrap_or_default(),
            request_id,
            control,
        )?]),
        NodeKind::ProcessingInstruction => Ok(vec![construct_processing_instruction(
            &source
                .name(node)
                .expect("source processing instruction has a target")
                .local,
            source.value(node).unwrap_or_default(),
            request_id,
            control,
        )?]),
    }
}

fn select_apply_nodes(
    inputs: &SequenceInputs<'_>,
    select: Option<&ApplySelection>,
    context: NodeId,
    variables: &RuntimeVariables,
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
        ApplySelection::Xslt10KeyLookup(lookup) => {
            key_lookup::select(inputs, lookup, Some(context), variables, control)
        }
        ApplySelection::Xslt10KeyUnion(lookups) => {
            select_xslt10_key_union(inputs, lookups, context, variables, control)
        }
        ApplySelection::Xslt10MixedUnion(alternatives) => {
            select_xslt10_mixed_union(inputs, alternatives, context, variables, control)
        }
        ApplySelection::PathUnion(alternatives) => {
            evaluate_source_path_union(inputs, source, context, alternatives, control)
        }
        ApplySelection::Xslt10PathUnionPosition {
            alternatives,
            position,
        } => select_xslt10_path_union_position(
            inputs,
            source,
            context,
            alternatives,
            *position,
            control,
        ),
        ApplySelection::VariablePathUnion {
            variable,
            alternatives,
        } => evaluate_variable_path_union(
            inputs,
            source,
            context,
            variable,
            alternatives,
            variables,
            control,
        ),
        ApplySelection::SourceVariablePath { variable, path } => {
            evaluate_source_variable_path(inputs, source, variable, path, variables, control)
        }
        ApplySelection::Xslt10VariableNodeSetComparisonPath {
            selection,
            variable,
            comparison,
        } => select_xslt10_variable_node_set_comparison(
            inputs, context, selection, variable, comparison, variables, control,
        ),
        ApplySelection::ChildElement(name) => {
            select_child_elements(source, context, name, inputs.request_id, control)
        }
        ApplySelection::DescendantElement(name) => {
            select_descendant_apply_nodes(inputs, source, context, name, control)
        }
        ApplySelection::ChildNodes(node_test) => {
            select_child_nodes(source, context, *node_test, inputs.request_id, control)
        }
        ApplySelection::Attribute(name) => {
            select_attribute(source, context, name, inputs.request_id, control)
        }
        ApplySelection::VariableFilteredElementPath(path) => variable_filtered_path::select(
            source,
            context,
            path,
            &variables.atomics,
            inputs.request_id,
            control,
        ),
        ApplySelection::Xslt10ChildrenOfSameNameElementsAsCurrent => {
            xslt10_current_name::select_children_of_same_name_elements(
                source,
                context,
                inputs.request_id,
                control,
            )
        }
        ApplySelection::VariableSequence(name) => source_variable_nodes(inputs, name, variables),
        selection @ (ApplySelection::Xslt10VariablePosition { .. }
        | ApplySelection::Xslt10VariableNodePosition { .. }
        | ApplySelection::Xslt10VariableUnionPosition { .. }) => {
            select_xslt10_variable_position_nodes(inputs, source, selection, variables, control)
        }
        ApplySelection::GlobalTemporaryChildren(_)
        | ApplySelection::TemporaryPath { .. }
        | ApplySelection::AtomicIntegerRange { .. } => {
            unreachable!("temporary-tree selection is dispatched before source selection")
        }
    }
}

fn select_descendant_apply_nodes(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    context: NodeId,
    name: &ExpandedName,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
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

fn select_xslt10_path_union_position(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    context: NodeId,
    alternatives: &[crate::xpath::path_experiment::LocationPath],
    position: crate::xslt::golden_semantics_experiment::Xslt10NodePosition,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let selected = evaluate_source_path_union(inputs, source, context, alternatives, control)?;
    Ok(select_xslt10_node_position(&selected, position)
        .into_iter()
        .collect())
}

fn select_xslt10_node_position(
    nodes: &[NodeId],
    position: crate::xslt::golden_semantics_experiment::Xslt10NodePosition,
) -> Option<NodeId> {
    match position {
        crate::xslt::golden_semantics_experiment::Xslt10NodePosition::Index(position) => {
            nodes.get(position.saturating_sub(1)).copied()
        }
        crate::xslt::golden_semantics_experiment::Xslt10NodePosition::Last => nodes.last().copied(),
        crate::xslt::golden_semantics_experiment::Xslt10NodePosition::LastMinus(offset) => nodes
            .len()
            .checked_sub(offset.saturating_add(1))
            .and_then(|index| nodes.get(index).copied()),
    }
}

fn select_attribute(
    source: &Document,
    context: NodeId,
    name: &ExpandedName,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut selected = Vec::new();
    for attribute in source.attributes(context).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.name(attribute) == Some(name) {
            selected.push(attribute);
        }
    }
    Ok(selected)
}

fn select_xslt10_variable_position_nodes(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    selection: &ApplySelection,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    match selection {
        ApplySelection::Xslt10VariablePosition {
            variable,
            position_variable,
        } => {
            select_xslt10_variable_position(inputs, variable, position_variable, variables, control)
        }
        ApplySelection::Xslt10VariableNodePosition { variable, position } => {
            select_xslt10_variable_node_position(inputs, variable, *position, variables, control)
        }
        ApplySelection::Xslt10VariableUnionPosition {
            variables: names,
            position,
        } => select_xslt10_variable_union_position(
            inputs, source, names, *position, variables, control,
        ),
        _ => unreachable!("variable-position dispatch receives only typed position selections"),
    }
}

fn select_xslt10_variable_position(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    position_variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    Ok(value_evaluator::xslt10_variable_position_source_node(
        inputs,
        variable,
        position_variable,
        variables,
        control,
    )?
    .into_iter()
    .collect())
}

fn select_xslt10_variable_node_position(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    position: crate::xslt::golden_semantics_experiment::Xslt10NodePosition,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    Ok(value_evaluator::xslt10_variable_node_position_source_node(
        inputs, variable, position, variables, control,
    )?
    .into_iter()
    .collect())
}

fn select_xslt10_variable_union_position(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    names: &[String],
    position: crate::xslt::golden_semantics_experiment::Xslt10NodePosition,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut selected = Vec::new();
    for name in names {
        selected.extend(source_variable_nodes(inputs, name, variables)?);
    }
    control
        .charge(WorkDomain::XPathOperation, selected.len().saturating_add(1))
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    let selected = select_xslt10_node_position(&selected, position);
    Ok(selected.into_iter().collect())
}

fn select_child_nodes(
    source: &Document,
    context: NodeId,
    node_test: NodeTest,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut selected = Vec::new();
    for child in source.children(context).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
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

fn select_child_elements(
    source: &Document,
    context: NodeId,
    name: &ExpandedName,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut selected = Vec::new();
    for child in source.children(context).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.kind(child) == NodeKind::Element && source.name(child) == Some(name) {
            selected.push(child);
        }
    }
    Ok(selected)
}

fn select_xslt10_key_union(
    inputs: &SequenceInputs<'_>,
    lookups: &[Xslt10KeyLookup],
    context: NodeId,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs.source.expect("key selection requires a source");
    let mut selected = Vec::new();
    for lookup in lookups {
        selected.extend(key_lookup::select(
            inputs,
            lookup,
            Some(context),
            variables,
            control,
        )?);
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    Ok(selected)
}

fn select_xslt10_mixed_union(
    inputs: &SequenceInputs<'_>,
    alternatives: &[Xslt10ApplyUnionPart],
    context: NodeId,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("mixed union selection requires a source");
    let mut selected = Vec::new();
    for alternative in alternatives {
        match alternative {
            Xslt10ApplyUnionPart::Path(path) => selected.extend(
                evaluate_location_path_controlled(source, context, path, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?,
            ),
            Xslt10ApplyUnionPart::Key(lookup) => selected.extend(key_lookup::select(
                inputs,
                lookup,
                Some(context),
                variables,
                control,
            )?),
            Xslt10ApplyUnionPart::Variable(name) => {
                selected.extend(source_variable_nodes(inputs, name, variables)?);
            }
        }
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    Ok(selected)
}

fn source_variable_nodes(
    inputs: &SequenceInputs<'_>,
    name: &str,
    variables: &RuntimeVariables,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    variables
        .source_nodes(inputs.globals, name)
        .cloned()
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("sorted variable selection requires source nodes: ${name}"),
            )
        })
}

fn evaluate_source_variable_path(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    variable: &str,
    path: &crate::xpath::path_experiment::LocationPath,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let roots = variables
        .source_nodes(inputs.globals, variable)
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

fn select_xslt10_variable_node_set_comparison(
    inputs: &SequenceInputs<'_>,
    context: NodeId,
    selection: &crate::xpath::path_experiment::LocationPath,
    variable: &str,
    comparison: &crate::xpath::path_experiment::LocationPath,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs
        .source
        .expect("node-set comparison selection requires a source");
    let candidates = evaluate_location_path_controlled(source, context, selection, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let variable_nodes = variables
        .source_nodes(inputs.globals, variable)
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("node-set comparison requires source nodes: ${variable}"),
            )
        })?;
    let mut selected = Vec::new();
    for candidate in candidates {
        let compared = evaluate_location_path_controlled(source, candidate, comparison, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if source_node_sets_have_equal_string_value(
            source,
            variable_nodes,
            &compared,
            inputs.request_id,
            control,
        )? {
            selected.push(candidate);
        }
    }
    Ok(selected)
}

fn source_node_sets_have_equal_string_value(
    source: &Document,
    left: &[NodeId],
    right: &[NodeId],
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    for left_node in left {
        let left_value = source
            .string_value_controlled(*left_node, control)
            .map_err(|failure| control_failure(failure, request_id))?;
        for right_node in right {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let right_value = source
                .string_value_controlled(*right_node, control)
                .map_err(|failure| control_failure(failure, request_id))?;
            if left_value == right_value {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn evaluate_variable_path_union(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    context: NodeId,
    variable: &str,
    alternatives: &[crate::xpath::path_experiment::LocationPath],
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let mut selected = variables
        .source_nodes(inputs.globals, variable)
        .cloned()
        .ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("path union requires a source-node sequence: ${variable}"),
            )
        })?;
    for alternative in alternatives {
        selected.extend(
            evaluate_location_path_controlled(source, context, alternative, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?,
        );
    }
    selected.sort_unstable_by_key(|node| source.document_order(*node));
    selected.dedup();
    Ok(selected)
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
    let supplied = evaluate_template_arguments(arguments, variables, inputs, execution, control)?;
    let frame =
        bind_template_parameters(&target.template, &supplied, inputs, execution.node, control)?;
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
        let variables =
            bind_template_parameters(&template.template, parameters, inputs, Some(node), control)?;
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
                namespaces: source.namespace_declarations(node).to_vec().into(),
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
#[path = "computed_attribute_namespace_tests.rs"]
mod computed_attribute_namespace_tests;

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
