//! Invocation-local globals, variable frames, and temporary-tree preparation.

#[path = "runtime_context/global_initialization.rs"]
mod global_initialization;
pub(super) use global_initialization::GlobalInitialization;
pub(super) use global_initialization::materialize_global_defaults;

#[path = "runtime_context/retained_source_values.rs"]
mod retained_source_values;
use retained_source_values::RetainedSourceSelection;

#[cfg(test)]
#[path = "runtime_context/retained_node_frames_reference.rs"]
mod retained_node_frames_reference;

#[cfg(test)]
#[path = "runtime_context/global_demand_reference.rs"]
mod global_demand_reference;

#[cfg(test)]
#[path = "golden_runtime_experiment/namespace_nodes_reference_tests.rs"]
mod namespace_nodes_reference_tests;

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::resources::ResourceSnapshot;
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::{
    evaluate_location_path_controlled, evaluate_location_path_union_controlled,
};
use crate::xslt::golden_semantics_experiment::{
    ConstructedElement, ConstructedNode, StylesheetProgram, Template, TemplateArgument,
    TemplateArgumentValue, TemplateParameterDefault, Xslt10ContentArgument,
};

use super::dynamic_document::DynamicDocument;
use super::key_lookup::KeyIndexCache;
use super::result_tree::ResultNode;
use super::template_selector::DocumentRootedMatchCache;
use super::value_evaluator::{
    evaluate_binary_numeric_value, evaluate_xslt10_sum_path, xslt10_concat_value,
    xslt10_variable_string_value,
};
use super::{
    ExecutionFailure, FailureCategory, MultipleMatchPolicy, control_failure,
    evaluate_xslt10_template_parameter_text_choice, failure, failure_at,
};

pub(super) struct SequenceInputs<'a> {
    pub(super) initializing_globals: Option<&'a GlobalInitialization<'a>>,
    pub(super) program: &'a StylesheetProgram,
    pub(super) source: Option<&'a Document>,
    pub(super) request_id: &'a str,
    pub(super) globals: &'a RuntimeGlobals,
    pub(super) multiple_match_policy: MultipleMatchPolicy,
    pub(super) document_rooted_matches: RefCell<DocumentRootedMatchCache>,
    pub(super) key_indexes: RefCell<KeyIndexCache>,
    pub(super) complete_atomic_frame_clones: bool,
    pub(super) resource_snapshot: Option<&'a ResourceSnapshot>,
    pub(super) denied_resources: Option<&'a HashSet<String>>,
    pub(super) dynamic_documents: &'a RefCell<BTreeMap<String, DynamicDocument>>,
}

pub(super) struct GlobalDocumentContext<'a> {
    pub(super) multiple_match_policy: MultipleMatchPolicy,
    pub(super) resource_snapshot: Option<&'a ResourceSnapshot>,
    pub(super) denied_resources: Option<&'a HashSet<String>>,
    pub(super) dynamic_documents: &'a RefCell<BTreeMap<String, DynamicDocument>>,
}

/// Completed-value semantics shared by ordinary reads and initialization reads.
/// This cannot trigger evaluation or retain a borrow across execution.
pub(super) fn completed_global_boolean(
    globals: &RuntimeGlobals,
    name: &str,
    request_id: &str,
) -> Result<Option<bool>, ExecutionFailure> {
    if let Some(value) = globals.atomics.get(name) {
        return super::atomic_effective_boolean_value(value, request_id).map(Some);
    }
    if let Some(nodes) = globals.nodes.get(name) {
        return Ok(Some(!nodes.is_empty()));
    }
    if let Some(selection) = globals.retained_sources.get(name) {
        return Ok(Some(!selection.nodes.is_empty()));
    }
    if globals.temporary_trees.contains_key(name) {
        return Ok(Some(true));
    }
    if globals.empty_sequences.contains(name) {
        return Ok(Some(false));
    }
    Ok(None)
}

#[derive(Debug, Default)]
pub(super) struct RuntimeGlobals {
    #[cfg(test)]
    pub(super) demand_string_reference: Option<global_demand_reference::StringReadHook>,
    #[cfg(test)]
    pub(super) demand_temporary_reference: Option<global_demand_reference::TemporaryReadHook>,
    #[cfg(test)]
    pub(super) demand_source_reference: Option<global_demand_reference::SourceReadHook>,
    #[cfg(test)]
    pub(super) demand_boolean_reference: Option<global_demand_reference::BooleanReadHook>,
    pub(super) principal_owner: Option<Arc<Document>>,
    #[cfg(test)]
    retained_nodes_reference:
        BTreeMap<String, Vec<crate::xdm::retained_nodes_reference::RetainedNode>>,
    pub(super) atomics: Arc<BTreeMap<String, AtomicValue>>,
    pub(super) empty_sequences: HashSet<String>,
    pub(super) nodes: BTreeMap<String, Vec<NodeId>>,
    pub(super) temporary_trees: BTreeMap<String, TemporaryTree>,
    pub(super) literal_documents: BTreeMap<String, DynamicDocument>,
    retained_sources: BTreeMap<String, RetainedSourceSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TemporaryTree {
    pub(super) identity: u64,
    pub(super) roots: Vec<usize>,
    pub(super) nodes: Vec<TemporaryNode>,
    pub(super) xslt10_disable_output_escaping_text: HashSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TemporaryNode {
    pub(super) kind: TemporaryNodeKind,
    pub(super) parent: Option<usize>,
    pub(super) children: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TemporaryNodeKind {
    Element {
        name: ExpandedName,
        namespaces: Vec<NamespaceBinding>,
        attributes: Vec<usize>,
    },
    Attribute {
        name: ExpandedName,
        value: String,
    },
    Text(String),
    Comment(String),
    ProcessingInstruction {
        target: String,
        value: String,
    },
}

#[derive(Debug, Clone, Default)]
pub(super) struct RuntimeVariables {
    retained_sources: Option<Arc<BTreeMap<String, RetainedSourceSelection>>>,
    #[cfg(test)]
    retained_nodes_reference:
        Arc<BTreeMap<String, Vec<crate::xdm::retained_nodes_reference::RetainedNode>>>,
    pub(super) atomics: Arc<BTreeMap<String, AtomicValue>>,
    pub(super) atomic_sequences: Arc<BTreeMap<String, Vec<AtomicValue>>>,
    pub(super) source_nodes: Arc<BTreeMap<String, Vec<NodeId>>>,
    pub(super) detached_source_node_strings: Arc<BTreeMap<String, Vec<String>>>,
    pub(super) temporary_trees: Arc<BTreeMap<String, TemporaryTree>>,
    literal_documents: Arc<BTreeMap<String, super::dynamic_document::DynamicDocument>>,
    local_bindings: Arc<HashSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InvocationParameter {
    pub(super) value: InvocationParameterValue,
    pub(super) tunnel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum InvocationParameterValue {
    Atomic(AtomicValue),
    AtomicSequence(Vec<AtomicValue>),
    SourceNodes(Vec<NodeId>),
    TemporaryTree(TemporaryTree),
}

impl From<AtomicValue> for InvocationParameterValue {
    fn from(value: AtomicValue) -> Self {
        Self::Atomic(value)
    }
}

pub(super) fn evaluate_template_arguments(
    arguments: &[TemplateArgument],
    variables: &RuntimeVariables,
    inputs: &SequenceInputs<'_>,
    execution: super::SequenceContext<'_>,
    control: &mut InvocationControl,
) -> Result<BTreeMap<String, InvocationParameter>, ExecutionFailure> {
    arguments
        .iter()
        .map(|argument| {
            let value =
                evaluate_template_argument(argument, variables, inputs, execution, control)?;
            Ok((
                argument.name.clone(),
                InvocationParameter {
                    value,
                    tunnel: false,
                },
            ))
        })
        .collect()
}

fn evaluate_template_argument(
    argument: &TemplateArgument,
    variables: &RuntimeVariables,
    inputs: &SequenceInputs<'_>,
    execution: super::SequenceContext<'_>,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    let context = execution.node;
    let focus_position = execution.focus_position;
    let focus_size = execution.focus_size;
    if let Some(value) = evaluate_context_argument(argument, inputs, execution, control)? {
        return Ok(value);
    }
    Ok(match &argument.value {
        TemplateArgumentValue::EmptySequence => {
            InvocationParameterValue::AtomicSequence(Vec::new())
        }
        TemplateArgumentValue::ConstructedString(nodes) => {
            evaluate_constructed_string_argument(nodes, inputs, control)?
        }
        TemplateArgumentValue::Text(value) => {
            InvocationParameterValue::Atomic(AtomicValue::string(value.clone()))
        }
        TemplateArgumentValue::Integer(value) => InvocationParameterValue::Atomic(
            AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, value.to_string()),
        ),
        TemplateArgumentValue::Boolean(value) => InvocationParameterValue::Atomic(
            AtomicValue::from_validated_lexical(BuiltinAtomicType::Boolean, value.to_string()),
        ),
        TemplateArgumentValue::ContextPosition
        | TemplateArgumentValue::ContextSize
        | TemplateArgumentValue::ContextNodeName
        | TemplateArgumentValue::ContextNodeStringLength(_)
        | TemplateArgumentValue::NamespaceCount(_)
        | TemplateArgumentValue::Xslt10CountPathUnion(_) => unreachable!(),
        TemplateArgumentValue::CurrentSourceNode => {
            let (_, context) = required_source_context(inputs, context)?;
            InvocationParameterValue::SourceNodes(vec![context])
        }
        TemplateArgumentValue::Variable(name) => {
            evaluate_variable_template_argument(inputs, variables, name, &argument.location)?
        }
        TemplateArgumentValue::Xslt10VariableString(name) => {
            InvocationParameterValue::Atomic(AtomicValue::string(xslt10_variable_string_value(
                inputs, name, variables, control,
            )?))
        }
        TemplateArgumentValue::Xslt10Concat(expression) => {
            InvocationParameterValue::Atomic(AtomicValue::string(xslt10_concat_value(
                inputs, context, expression, variables, control,
            )?))
        }
        TemplateArgumentValue::SourceVariablePath { variable, path } => {
            evaluate_source_variable_path_argument(inputs, variable, path, variables, control)?
        }
        TemplateArgumentValue::BinaryNumeric(expression) => {
            evaluate_numeric_template_argument(inputs, execution, expression, variables, control)?
        }
        TemplateArgumentValue::SourcePath(path) => {
            evaluate_source_path_argument(inputs, context, path, control)?
        }
        TemplateArgumentValue::Xslt10SumPath(path) => {
            let value = evaluate_xslt10_sum_path(inputs, context, path, control)?;
            InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                BuiltinAtomicType::Double,
                value,
            ))
        }
        TemplateArgumentValue::Xslt10Content(argument_plan) => evaluate_xslt10_content_argument(
            argument_plan,
            variables,
            inputs,
            context,
            focus_position,
            focus_size,
            control,
        )?,
        TemplateArgumentValue::ForEachPathStringContent(path) => {
            evaluate_for_each_path_string_content(inputs, context, path, control)?
        }
        TemplateArgumentValue::Xslt10ConstructedContent(nodes) => {
            evaluate_constructed_template_argument(nodes, inputs, control)?
        }
        TemplateArgumentValue::Xslt10SequenceConstructor(instructions) => {
            let nodes =
                super::execute_sequence(inputs, instructions, execution, variables, control)?;
            InvocationParameterValue::TemporaryTree(materialize_result_nodes(
                &nodes,
                inputs.request_id,
                control,
            )?)
        }
        TemplateArgumentValue::SourcePathStringComparison { left, right, equal } => {
            let value = evaluate_source_path_string_comparison(
                inputs, context, left, right, *equal, control,
            )?;
            InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                BuiltinAtomicType::Boolean,
                value.to_string(),
            ))
        }
    })
}

fn evaluate_context_argument(
    argument: &TemplateArgument,
    inputs: &SequenceInputs<'_>,
    execution: super::SequenceContext<'_>,
    control: &mut InvocationControl,
) -> Result<Option<InvocationParameterValue>, ExecutionFailure> {
    let value = match &argument.value {
        TemplateArgumentValue::Xslt10CountPathUnion(alternatives) => {
            let (source, context) = required_source_context(inputs, execution.node)?;
            let nodes =
                evaluate_location_path_union_controlled(source, context, alternatives, control)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
            integer_parameter(nodes.len())
        }
        TemplateArgumentValue::NamespaceCount(path) => integer_parameter(
            super::namespace_focus::count(inputs, path, execution, control)?,
        ),
        TemplateArgumentValue::ContextPosition => integer_parameter(execution.focus_position),
        TemplateArgumentValue::ContextSize => integer_parameter(execution.focus_size),
        TemplateArgumentValue::ContextNodeName => {
            evaluate_context_name_argument(inputs, execution.node, &argument.location, control)?
        }
        TemplateArgumentValue::ContextNodeStringLength(location) => {
            integer_parameter(super::value_evaluator::context_node_string_length(
                inputs,
                execution.node,
                location,
                control,
            )?)
        }
        _ => return Ok(None),
    };
    Ok(Some(value))
}

fn evaluate_constructed_template_argument(
    nodes: &[ConstructedNode],
    inputs: &SequenceInputs<'_>,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    Ok(InvocationParameterValue::TemporaryTree(
        materialize_temporary_nodes(nodes, inputs.source, inputs.request_id, control)?,
    ))
}

fn evaluate_source_path_argument(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    path: &crate::xpath::path_experiment::LocationPath,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let nodes = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(InvocationParameterValue::SourceNodes(nodes))
}

fn evaluate_constructed_string_argument(
    nodes: &[ConstructedNode],
    inputs: &SequenceInputs<'_>,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    let tree = materialize_temporary_nodes(nodes, inputs.source, inputs.request_id, control)?;
    Ok(InvocationParameterValue::Atomic(AtomicValue::string(
        temporary_tree_string_value(&tree, inputs.request_id, control)?,
    )))
}

fn evaluate_variable_template_argument(
    inputs: &SequenceInputs<'_>,
    variables: &RuntimeVariables,
    name: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    if let Some(value) = variables.atomics.get(name) {
        return Ok(InvocationParameterValue::Atomic(value.clone()));
    }
    if let Some(values) = variables.atomic_sequences.get(name) {
        return Ok(InvocationParameterValue::AtomicSequence(values.clone()));
    }
    if let Some(nodes) = variables.source_nodes_in_context(inputs, name)? {
        return Ok(InvocationParameterValue::SourceNodes(nodes.clone()));
    }
    if let Some(tree) = variables.temporary_tree(inputs.globals, name) {
        return Ok(InvocationParameterValue::TemporaryTree(tree.clone()));
    }
    Err(failure_at(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(inputs.request_id),
        location.clone(),
        format!("unbound template argument variable: ${name}"),
    ))
}

fn evaluate_for_each_path_string_content(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    path: &crate::xpath::path_experiment::LocationPath,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut value = String::new();
    for node in selected {
        control
            .charge(WorkDomain::XsltInstruction, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        value.push_str(
            &source
                .string_value_controlled(node, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?,
        );
    }
    Ok(InvocationParameterValue::TemporaryTree(
        materialize_parentless_temporary_node(
            TemporaryNodeKind::Text(value),
            inputs.request_id,
            control,
        )?,
    ))
}

fn integer_parameter(value: usize) -> InvocationParameterValue {
    InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
        BuiltinAtomicType::Integer,
        value.to_string(),
    ))
}

fn evaluate_source_variable_path_argument(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    path: &crate::xpath::path_experiment::LocationPath,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "a variable-rooted template argument requires a principal source",
        )
    })?;
    let nodes = super::bound_variable_path::evaluate_eligible_path(
        inputs, source, variable, path, variables, control,
    )?;
    Ok(InvocationParameterValue::SourceNodes(nodes))
}

fn evaluate_context_name_argument(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let (source, node) = super::required_source_context(inputs, context).map_err(|_| {
        failure_at(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            location.clone(),
            "fn:name requires a source context node in a template argument",
        )
    })?;
    let value = source.name(node).map_or_else(String::new, |name| {
        source.prefix(node).map_or_else(
            || name.local.clone(),
            |prefix| format!("{prefix}:{}", name.local),
        )
    });
    Ok(InvocationParameterValue::Atomic(AtomicValue::string(value)))
}

fn evaluate_numeric_template_argument(
    inputs: &SequenceInputs<'_>,
    execution: super::SequenceContext<'_>,
    expression: &crate::xpath::binary_numeric_experiment::BinaryNumericExpression,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    let value = evaluate_binary_numeric_value(
        inputs,
        execution.node,
        execution.sequence_focus(),
        expression,
        variables,
        control,
    )?;
    Ok(InvocationParameterValue::Atomic(
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Double, value),
    ))
}

fn evaluate_xslt10_content_argument(
    argument_plan: &Xslt10ContentArgument,
    variables: &RuntimeVariables,
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    focus_position: usize,
    focus_size: usize,
    control: &mut InvocationControl,
) -> Result<InvocationParameterValue, ExecutionFailure> {
    let mut content_variables = variables.clone();
    for binding in &argument_plan.bindings {
        let tree = materialize_parentless_temporary_node(
            TemporaryNodeKind::Text(binding.value.clone()),
            inputs.request_id,
            control,
        )?;
        content_variables.bind_temporary_tree(binding.name.clone(), tree);
    }
    let value = super::value_evaluator::evaluate_as_temporary_text(
        inputs,
        &argument_plan.value,
        context,
        focus_position,
        focus_size,
        &content_variables,
        control,
    )?;
    Ok(InvocationParameterValue::TemporaryTree(
        materialize_parentless_temporary_node(
            TemporaryNodeKind::Text(value),
            inputs.request_id,
            control,
        )?,
    ))
}

pub(super) fn evaluate_source_path_string_comparison(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    left: &crate::xpath::path_experiment::LocationPath,
    right: &crate::xpath::path_experiment::LocationPath,
    equal: bool,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let left = evaluate_location_path_controlled(source, context, left, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let right = evaluate_location_path_controlled(source, context, right, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    for left in left {
        let left = source
            .string_value_controlled(left, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        for right in &right {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let right = source
                .string_value_controlled(*right, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if (left == right) == equal {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

impl RuntimeVariables {
    pub(super) fn from_atomics(
        atomics: &Arc<BTreeMap<String, AtomicValue>>,
        complete_clone: bool,
    ) -> Self {
        Self {
            retained_sources: None,
            #[cfg(test)]
            retained_nodes_reference: Arc::new(BTreeMap::new()),
            atomics: if complete_clone {
                Arc::new(atomics.as_ref().clone())
            } else {
                Arc::clone(atomics)
            },
            atomic_sequences: Arc::new(BTreeMap::new()),
            source_nodes: Arc::new(BTreeMap::new()),
            detached_source_node_strings: Arc::new(BTreeMap::new()),
            temporary_trees: Arc::new(BTreeMap::new()),
            literal_documents: Arc::new(BTreeMap::new()),
            local_bindings: Arc::new(HashSet::new()),
        }
    }

    fn clear_value_kinds(&mut self, name: &str) {
        if let Some(values) = &mut self.retained_sources
            && values.contains_key(name)
        {
            Arc::make_mut(values).remove(name);
        }
        #[cfg(test)]
        if self.retained_nodes_reference.contains_key(name) {
            Arc::make_mut(&mut self.retained_nodes_reference).remove(name);
        }
        if self.atomics.contains_key(name) {
            Arc::make_mut(&mut self.atomics).remove(name);
        }
        if self.atomic_sequences.contains_key(name) {
            Arc::make_mut(&mut self.atomic_sequences).remove(name);
        }
        if self.source_nodes.contains_key(name) {
            Arc::make_mut(&mut self.source_nodes).remove(name);
        }
        if self.detached_source_node_strings.contains_key(name) {
            Arc::make_mut(&mut self.detached_source_node_strings).remove(name);
        }
        if self.temporary_trees.contains_key(name) {
            Arc::make_mut(&mut self.temporary_trees).remove(name);
        }
        if self.literal_documents.contains_key(name) {
            Arc::make_mut(&mut self.literal_documents).remove(name);
        }
        if !self.local_bindings.contains(name) {
            Arc::make_mut(&mut self.local_bindings).insert(name.to_owned());
        }
    }

    pub(super) fn bind_atomic(&mut self, name: String, value: AtomicValue) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.atomics).insert(name, value);
    }

    pub(super) fn bind_atomic_sequence(&mut self, name: String, values: Vec<AtomicValue>) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.atomic_sequences).insert(name, values);
    }

    pub(super) fn bind_source_nodes(&mut self, name: String, nodes: Vec<NodeId>) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.source_nodes).insert(name, nodes);
    }

    fn bind_detached_source_node_strings(&mut self, name: String, values: Vec<String>) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.detached_source_node_strings).insert(name, values);
    }

    pub(super) fn bind_temporary_tree(&mut self, name: String, tree: TemporaryTree) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.temporary_trees).insert(name, tree);
    }

    pub(super) fn bind_literal_document(
        &mut self,
        name: String,
        document: super::dynamic_document::DynamicDocument,
    ) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.literal_documents).insert(name, document);
    }

    pub(super) fn literal_document<'a>(
        &'a self,
        globals: &'a RuntimeGlobals,
        name: &str,
    ) -> Option<&'a super::dynamic_document::DynamicDocument> {
        self.literal_documents.get(name).or_else(|| {
            (!self.local_bindings.contains(name))
                .then(|| globals.literal_documents.get(name))
                .flatten()
        })
    }

    pub(super) fn bind_alias(
        &mut self,
        name: String,
        source: &str,
        globals: &RuntimeGlobals,
    ) -> bool {
        if let Some((owner, nodes)) = self.retained_source(globals, source) {
            let value = RetainedSourceSelection {
                owner: owner.clone(),
                nodes: nodes.clone(),
            };
            self.bind_retained_source(name, value);
            return true;
        }
        #[cfg(test)]
        if let Some(nodes) = self.retained_nodes_reference(globals, source).cloned() {
            self.bind_retained_nodes_reference(name, nodes);
            return true;
        }
        if let Some(value) = self.atomics.get(source).cloned() {
            self.bind_atomic(name, value);
            return true;
        }
        if let Some(values) = self.atomic_sequences.get(source).cloned() {
            self.bind_atomic_sequence(name, values);
            return true;
        }
        if let Some(nodes) = self.source_nodes.get(source).cloned() {
            self.bind_source_nodes(name, nodes);
            return true;
        }
        if let Some(values) = self.detached_source_node_strings.get(source).cloned() {
            self.bind_detached_source_node_strings(name, values);
            return true;
        }
        if let Some(tree) = self.temporary_trees.get(source).cloned() {
            self.bind_temporary_tree(name, tree);
            return true;
        }
        if let Some(document) = self.literal_documents.get(source).cloned() {
            self.bind_literal_document(name, document);
            return true;
        }
        if !self.allows_global_fallback(source) {
            return false;
        }
        if let Some(value) = globals.atomics.get(source).cloned() {
            self.bind_atomic(name, value);
            return true;
        }
        if globals.empty_sequences.contains(source) {
            self.bind_atomic_sequence(name, Vec::new());
            return true;
        }
        if let Some(nodes) = globals.nodes.get(source).cloned() {
            self.bind_source_nodes(name, nodes);
            return true;
        }
        if let Some(tree) = globals.temporary_trees.get(source).cloned() {
            self.bind_temporary_tree(name, tree);
            return true;
        }
        if let Some(document) = globals.literal_documents.get(source).cloned() {
            self.bind_literal_document(name, document);
            return true;
        }
        false
    }

    pub(super) fn source_nodes<'a>(
        &'a self,
        globals: &'a RuntimeGlobals,
        name: &str,
    ) -> Option<&'a Vec<NodeId>> {
        if let Some(value) = self
            .retained_sources
            .as_ref()
            .and_then(|values| values.get(name))
        {
            return Some(&value.nodes);
        }
        self.source_nodes.get(name).or_else(|| {
            (!self.local_bindings.contains(name))
                .then(|| globals.nodes.get(name))
                .flatten()
        })
    }

    pub(super) fn has_source_node_values(&self) -> bool {
        self.source_nodes.values().any(|nodes| !nodes.is_empty())
    }

    pub(super) fn detached_source_node_strings(&self, name: &str) -> Option<&Vec<String>> {
        self.detached_source_node_strings.get(name)
    }

    pub(super) fn detach_source_nodes_for_document_context(
        &self,
        source: &Document,
        request_id: &str,
        control: &mut InvocationControl,
    ) -> Result<Self, ExecutionFailure> {
        let mut detached = self.clone();
        for (name, nodes) in self.source_nodes.iter() {
            let values = nodes
                .iter()
                .map(|node| {
                    source
                        .string_value_controlled(*node, control)
                        .map_err(|failure| control_failure(failure, request_id))
                })
                .collect::<Result<Vec<_>, _>>()?;
            detached.bind_detached_source_node_strings(name.clone(), values);
        }
        Ok(detached)
    }

    pub(super) fn temporary_tree<'a>(
        &'a self,
        globals: &'a RuntimeGlobals,
        name: &str,
    ) -> Option<&'a TemporaryTree> {
        self.temporary_trees.get(name).or_else(|| {
            (!self.local_bindings.contains(name))
                .then(|| globals.temporary_trees.get(name))
                .flatten()
        })
    }

    pub(super) fn allows_global_fallback(&self, name: &str) -> bool {
        !self.local_bindings.contains(name)
    }

    #[cfg(test)]
    pub(super) fn clone_population(&self) -> (usize, usize, usize, usize) {
        (
            self.atomic_sequences.len(),
            self.source_nodes.len()
                + self
                    .retained_sources
                    .as_ref()
                    .map_or(0, |values| values.len()),
            self.temporary_trees.len(),
            self.local_bindings.len(),
        )
    }

    #[cfg(test)]
    pub(super) fn clone_complete_for_sequence(&self) -> Self {
        Self {
            retained_nodes_reference: Arc::new(self.retained_nodes_reference.as_ref().clone()),
            retained_sources: self
                .retained_sources
                .as_ref()
                .map(|values| Arc::new(values.as_ref().clone())),
            atomics: Arc::new(self.atomics.as_ref().clone()),
            atomic_sequences: Arc::new(self.atomic_sequences.as_ref().clone()),
            source_nodes: Arc::new(self.source_nodes.as_ref().clone()),
            detached_source_node_strings: Arc::new(
                self.detached_source_node_strings.as_ref().clone(),
            ),
            temporary_trees: Arc::new(self.temporary_trees.as_ref().clone()),
            literal_documents: Arc::new(self.literal_documents.as_ref().clone()),
            local_bindings: Arc::new(self.local_bindings.as_ref().clone()),
        }
    }
}

fn source_path_string_value(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "an XSLT 1.0 source-dependent temporary tree requires a principal source",
        )
    })?;
    let selected = evaluate_location_path_controlled(source, source.document_node(), path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    selected.first().map_or_else(
        || Ok(String::new()),
        |node| {
            source
                .string_value_controlled(*node, control)
                .map_err(|failure| control_failure(failure, request_id))
        },
    )
}

pub(super) fn materialize_xslt10_for_each_text_tree(
    source: &Document,
    context: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut value = String::new();
    for node in selected {
        let part = source
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, request_id))?;
        value.push_str(&part);
    }
    materialize_parentless_temporary_node(TemporaryNodeKind::Text(value), request_id, control)
}

pub(super) fn source_node_identity(node: NodeId) -> String {
    format!("fastxsltPrincipalN{}", node.index())
}

/// Format an ordinary node using its document origin, never its URI or address.
/// Dynamic root spelling agrees with the existing document-root consumer.
pub(super) fn source_node_identity_in_context(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    node: NodeId,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    for dynamic in inputs.dynamic_documents.borrow().values() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        if source.same_origin(&dynamic.document) {
            return Ok(if node == source.document_node() {
                format!("d{}", dynamic.identity)
            } else {
                format!("fastxsltDocument{}N{}", dynamic.identity, node.index())
            });
        }
    }
    Ok(source_node_identity(node))
}

pub(super) fn ensure_principal_identity_context(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if context.is_none() && inputs.source.is_some() {
        return Err(failure_at(
            "FXRT0007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            location.clone(),
            "qualified generated-ID strings require ordinary source focus",
        ));
    }
    if let Some(source) = inputs.source {
        for dynamic in inputs.dynamic_documents.borrow().values() {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|error| control_failure(error, inputs.request_id))?;
            if source.same_origin(&dynamic.document) {
                return Err(failure_at(
                    "FXRT0007",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    location.clone(),
                    "qualified generated-ID strings on secondary documents remain unsupported",
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn source_node_identity_for_path(
    source: &Document,
    context: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
    Ok(
        source_node_for_identity_path(source, context, path, request_id, control)?
            .map(source_node_identity),
    )
}

pub(super) fn source_node_for_identity_path(
    source: &Document,
    context: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<NodeId>, ExecutionFailure> {
    let nodes = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    if nodes.len() > 1 {
        return Err(failure(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(request_id),
            "generate-id() requires a zero-or-one node argument",
        ));
    }
    Ok(nodes.first().copied())
}

pub(super) fn source_node_identities_equal(
    source: &Document,
    context: NodeId,
    left: &crate::xpath::path_experiment::LocationPath,
    right: &crate::xpath::path_experiment::LocationPath,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let left = source_node_identity_for_path(source, context, left, request_id, control)?;
    let right = source_node_identity_for_path(source, context, right, request_id, control)?;
    Ok(left == right)
}

pub(super) fn temporary_tree_string_value(
    tree: &TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let mut value = String::new();
    let mut pending = tree.roots.iter().rev().copied().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        control
            .charge(WorkDomain::XdmStringValueNode, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        match &tree.nodes[node].kind {
            TemporaryNodeKind::Text(text) => value.push_str(text),
            TemporaryNodeKind::Element { .. } => {
                pending.extend(tree.nodes[node].children.iter().rev().copied());
            }
            TemporaryNodeKind::Attribute { .. }
            | TemporaryNodeKind::Comment(_)
            | TemporaryNodeKind::ProcessingInstruction { .. } => {}
        }
    }
    Ok(value)
}

pub(super) fn temporary_node_string_value(
    tree: &TemporaryTree,
    node: usize,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let mut value = String::new();
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        control
            .charge(WorkDomain::XdmStringValueNode, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        match &tree.nodes[node].kind {
            TemporaryNodeKind::Attribute { value: lexical, .. }
            | TemporaryNodeKind::Text(lexical)
            | TemporaryNodeKind::Comment(lexical)
            | TemporaryNodeKind::ProcessingInstruction { value: lexical, .. } => {
                value.push_str(lexical);
            }
            TemporaryNodeKind::Element { .. } => {
                pending.extend(tree.nodes[node].children.iter().rev().copied());
            }
        }
    }
    Ok(value)
}

pub(super) fn temporary_document_identity(
    tree: &TemporaryTree,
    descendant_local: Option<&str>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<u64>, ExecutionFailure> {
    let Some(descendant_local) = descendant_local else {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        return Ok(Some(tree.identity));
    };
    let mut selected = None;
    for (index, node) in tree.nodes.iter().enumerate() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if matches!(
            &node.kind,
            TemporaryNodeKind::Element { name, .. }
                if name.namespace.is_none() && name.local == descendant_local
        ) && selected.replace(index).is_some()
        {
            return Err(failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(request_id),
                "root() requires a zero-or-one temporary-node argument",
            ));
        }
    }
    let Some(mut node) = selected else {
        return Ok(None);
    };
    loop {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        let Some(parent) = tree.nodes[node].parent else {
            break;
        };
        node = parent;
    }
    Ok(Some(tree.identity))
}

pub(super) fn materialize_parentless_temporary_node(
    kind: TemporaryNodeKind,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let identity = allocate_temporary_tree_identity(control, request_id)?;
    Ok(TemporaryTree {
        identity,
        roots: vec![0],
        nodes: vec![TemporaryNode {
            kind,
            parent: None,
            children: Vec::new(),
        }],
        xslt10_disable_output_escaping_text: HashSet::new(),
    })
}

pub(super) fn materialize_temporary_tree(
    elements: &[ConstructedElement],
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    let mut tree = TemporaryTree {
        identity: allocate_temporary_tree_identity(control, request_id)?,
        roots: Vec::new(),
        nodes: Vec::new(),
        xslt10_disable_output_escaping_text: HashSet::new(),
    };
    for element in elements {
        let root =
            materialize_temporary_element(element, None, None, &mut tree, request_id, control)?;
        tree.roots.push(root);
    }
    Ok(tree)
}

pub(super) fn materialize_temporary_nodes(
    nodes: &[ConstructedNode],
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    let mut tree = TemporaryTree {
        identity: allocate_temporary_tree_identity(control, request_id)?,
        roots: Vec::new(),
        nodes: Vec::new(),
        xslt10_disable_output_escaping_text: HashSet::new(),
    };
    for node in nodes {
        let root = materialize_temporary_node(node, source, None, &mut tree, request_id, control)?;
        tree.roots.push(root);
    }
    Ok(tree)
}

pub(super) fn materialize_result_nodes(
    nodes: &[ResultNode],
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    let mut tree = TemporaryTree {
        identity: allocate_temporary_tree_identity(control, request_id)?,
        roots: Vec::new(),
        nodes: Vec::new(),
        xslt10_disable_output_escaping_text: HashSet::new(),
    };
    for node in nodes {
        if matches!(node, ResultNode::Xslt10RecoverableAttribute(_)) {
            continue;
        }
        let root = materialize_result_node(node, None, &mut tree, request_id, control)?;
        tree.roots.push(root);
    }
    Ok(tree)
}

fn materialize_result_node(
    result: &ResultNode,
    parent: Option<usize>,
    tree: &mut TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let node = tree.nodes.len();
    if matches!(result, ResultNode::Xslt10DisableOutputEscapingText(_)) {
        tree.xslt10_disable_output_escaping_text.insert(node);
    }
    let kind = match result {
        ResultNode::Element {
            name, namespaces, ..
        } => TemporaryNodeKind::Element {
            name: name.clone(),
            namespaces: namespaces.to_vec(),
            attributes: Vec::new(),
        },
        ResultNode::Text(value) | ResultNode::Xslt10DisableOutputEscapingText(value) => {
            TemporaryNodeKind::Text(value.clone())
        }
        ResultNode::Comment(value) => TemporaryNodeKind::Comment(value.clone()),
        ResultNode::ProcessingInstruction { target, value } => {
            TemporaryNodeKind::ProcessingInstruction {
                target: target.clone(),
                value: value.clone(),
            }
        }
        ResultNode::PendingNamespace { location, .. } => {
            return Err(super::failure_at(
                "XTDE0420",
                FailureCategory::Invalid,
                Some(request_id),
                location.clone(),
                "a namespace cannot be a top-level temporary-tree node",
            ));
        }
        ResultNode::PendingAttribute(_) => {
            return Err(failure(
                "XTDE0410",
                FailureCategory::Invalid,
                Some(request_id),
                "an attribute cannot be a top-level temporary-tree node",
            ));
        }
        ResultNode::Xslt10RecoverableAttribute(_) => {
            unreachable!("recoverable top-level attributes are filtered before materialization")
        }
    };
    tree.nodes.push(TemporaryNode {
        kind,
        parent,
        children: Vec::new(),
    });
    if let ResultNode::Element {
        attributes,
        children,
        ..
    } = result
    {
        let mut attribute_nodes = Vec::with_capacity(attributes.len());
        for attribute in attributes {
            control
                .charge(WorkDomain::XdmNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let attribute_node = tree.nodes.len();
            tree.nodes.push(TemporaryNode {
                kind: TemporaryNodeKind::Attribute {
                    name: attribute.name.clone(),
                    value: attribute.value.clone(),
                },
                parent: Some(node),
                children: Vec::new(),
            });
            attribute_nodes.push(attribute_node);
        }
        let TemporaryNodeKind::Element {
            attributes: retained,
            ..
        } = &mut tree.nodes[node].kind
        else {
            unreachable!("result element materializes as a temporary element")
        };
        *retained = attribute_nodes;
        for child in children {
            if matches!(child, ResultNode::Xslt10RecoverableAttribute(_)) {
                continue;
            }
            let child = materialize_result_node(child, Some(node), tree, request_id, control)?;
            tree.nodes[node].children.push(child);
        }
    }
    Ok(node)
}

fn allocate_temporary_tree_identity(
    control: &mut InvocationControl,
    request_id: &str,
) -> Result<u64, ExecutionFailure> {
    control.allocate_temporary_tree_identity().ok_or_else(|| {
        failure(
            "FXRT0010",
            FailureCategory::Limit,
            Some(request_id),
            "the invocation exhausted temporary-tree identity space",
        )
    })
}

fn materialize_temporary_element(
    element: &ConstructedElement,
    source: Option<&Document>,
    parent: Option<usize>,
    tree: &mut TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let node = tree.nodes.len();
    tree.nodes.push(TemporaryNode {
        kind: TemporaryNodeKind::Element {
            name: element.name.clone(),
            namespaces: element.namespaces.clone(),
            attributes: Vec::new(),
        },
        parent,
        children: Vec::new(),
    });
    let mut attributes = Vec::with_capacity(element.attributes.len());
    for attribute in &element.attributes {
        control
            .charge(WorkDomain::XdmNode, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        let attribute_node = tree.nodes.len();
        tree.nodes.push(TemporaryNode {
            kind: TemporaryNodeKind::Attribute {
                name: attribute.name.clone(),
                value: attribute.value.clone(),
            },
            parent: Some(node),
            children: Vec::new(),
        });
        attributes.push(attribute_node);
    }
    let TemporaryNodeKind::Element {
        attributes: node_attributes,
        ..
    } = &mut tree.nodes[node].kind
    else {
        unreachable!("the new temporary node is an element")
    };
    *node_attributes = attributes;
    for child in &element.children {
        let child =
            materialize_temporary_node(child, source, Some(node), tree, request_id, control)?;
        tree.nodes[node].children.push(child);
    }
    Ok(node)
}

fn materialize_temporary_node(
    constructed: &ConstructedNode,
    source: Option<&Document>,
    parent: Option<usize>,
    tree: &mut TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    match constructed {
        ConstructedNode::Element(element) => {
            materialize_temporary_element(element, source, parent, tree, request_id, control)
        }
        ConstructedNode::Text(value) => {
            control
                .charge(WorkDomain::XdmNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let node = tree.nodes.len();
            tree.nodes.push(TemporaryNode {
                kind: TemporaryNodeKind::Text(value.clone()),
                parent,
                children: Vec::new(),
            });
            Ok(node)
        }
        ConstructedNode::Comment(value) => {
            control
                .charge(WorkDomain::XdmNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let node = tree.nodes.len();
            tree.nodes.push(TemporaryNode {
                kind: TemporaryNodeKind::Comment(value.clone()),
                parent,
                children: Vec::new(),
            });
            Ok(node)
        }
        ConstructedNode::Xslt10SourcePathString(path) => {
            let value = source_path_string_value(path, source, request_id, control)?;
            control
                .charge(WorkDomain::XdmNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let node = tree.nodes.len();
            tree.nodes.push(TemporaryNode {
                kind: TemporaryNodeKind::Text(value),
                parent,
                children: Vec::new(),
            });
            Ok(node)
        }
        ConstructedNode::Xslt10ForEachText { select, value } => {
            let source = source.ok_or_else(|| {
                failure(
                    "FXRT1004",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    "an XSLT 1.0 source-dependent temporary tree requires a principal source",
                )
            })?;
            let selected =
                evaluate_location_path_controlled(source, source.document_node(), select, control)
                    .map_err(|failure| control_failure(failure, request_id))?;
            let repeated = value.repeat(selected.len());
            control
                .charge(WorkDomain::XdmNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let node = tree.nodes.len();
            tree.nodes.push(TemporaryNode {
                kind: TemporaryNodeKind::Text(repeated),
                parent,
                children: Vec::new(),
            });
            Ok(node)
        }
    }
}

pub(super) fn bind_template_parameters(
    template: &Template,
    supplied: &BTreeMap<String, InvocationParameter>,
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<RuntimeVariables, ExecutionFailure> {
    let mut frame = RuntimeVariables::from_atomics(
        &inputs.globals.atomics,
        inputs.complete_atomic_frame_clones,
    );
    for parameter in &template.parameters {
        if parameter.required
            && supplied
                .get(&parameter.name)
                .is_none_or(|supplied| supplied.tunnel != parameter.tunnel)
        {
            return Err(failure(
                "XTDE0700",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!(
                    "required template parameter was not supplied: ${}",
                    parameter.name
                ),
            ));
        }
        let supplied = supplied
            .get(&parameter.name)
            .filter(|supplied| supplied.tunnel == parameter.tunnel);
        match supplied.map(|supplied| &supplied.value) {
            Some(InvocationParameterValue::Atomic(value)) => {
                frame.bind_atomic(parameter.name.clone(), value.clone());
            }
            Some(InvocationParameterValue::AtomicSequence(values)) => {
                frame.bind_atomic_sequence(parameter.name.clone(), values.clone());
            }
            Some(InvocationParameterValue::SourceNodes(nodes)) => {
                frame.bind_source_nodes(parameter.name.clone(), nodes.clone());
            }
            Some(InvocationParameterValue::TemporaryTree(tree)) => {
                frame.bind_temporary_tree(parameter.name.clone(), tree.clone());
            }
            None => {
                bind_template_parameter_default(&mut frame, parameter, inputs, context, control)?;
            }
        }
    }
    Ok(frame)
}

fn bind_template_parameter_default(
    frame: &mut RuntimeVariables,
    parameter: &crate::xslt::golden_semantics_experiment::TemplateParameter,
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    match &parameter.default {
        TemplateParameterDefault::Text(value) => {
            frame.bind_atomic(parameter.name.clone(), AtomicValue::string(value.clone()));
        }
        TemplateParameterDefault::Integer(value) => {
            frame.bind_atomic(
                parameter.name.clone(),
                AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, value.to_string()),
            );
        }
        TemplateParameterDefault::SourcePath(path) => {
            let (source, context) = required_source_context(inputs, context)?;
            let nodes = evaluate_location_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            frame.bind_source_nodes(parameter.name.clone(), nodes);
        }
        TemplateParameterDefault::SourceVariablePath { variable, path } => {
            let source = inputs.source.ok_or_else(|| {
                failure(
                    "FXRT1004",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    "a variable-rooted template parameter default requires a principal source",
                )
            })?;
            let nodes = super::bound_variable_path::evaluate_eligible_path(
                inputs, source, variable, path, frame, control,
            )?;
            frame.bind_source_nodes(parameter.name.clone(), nodes);
        }
        TemplateParameterDefault::Variable(variable) => {
            copy_parameter_default_variable(frame, parameter, variable, inputs)?;
        }
        TemplateParameterDefault::ContextLocalName => {
            let (source, context) = required_source_context(inputs, context)?;
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            let value = source
                .name(context)
                .map_or_else(String::new, |name| name.local.clone());
            frame.bind_atomic(parameter.name.clone(), AtomicValue::string(value));
        }
        TemplateParameterDefault::Xslt10BinaryNumeric(expression) => {
            let value =
                evaluate_binary_numeric_value(inputs, context, None, expression, frame, control)?;
            frame.bind_atomic(
                parameter.name.clone(),
                AtomicValue::from_validated_lexical(BuiltinAtomicType::Double, value),
            );
        }
        TemplateParameterDefault::Xslt10SequenceConstructor(instructions) => {
            let nodes = super::execute_sequence(
                inputs,
                instructions,
                super::SequenceContext::new(context, None),
                frame,
                control,
            )?;
            frame.bind_temporary_tree(
                parameter.name.clone(),
                materialize_result_nodes(&nodes, inputs.request_id, control)?,
            );
        }
        TemplateParameterDefault::Xslt10TextChoice {
            branches,
            otherwise,
        } => {
            let value = evaluate_xslt10_template_parameter_text_choice(
                inputs, branches, otherwise, context, frame, control,
            )?;
            let tree = materialize_parentless_temporary_node(
                TemporaryNodeKind::Text(value),
                inputs.request_id,
                control,
            )?;
            frame.bind_temporary_tree(parameter.name.clone(), tree);
        }
    }
    Ok(())
}

fn copy_parameter_default_variable(
    frame: &mut RuntimeVariables,
    parameter: &crate::xslt::golden_semantics_experiment::TemplateParameter,
    variable: &str,
    inputs: &SequenceInputs<'_>,
) -> Result<(), ExecutionFailure> {
    if let Some(value) = frame.atomics.get(variable).cloned() {
        frame.bind_atomic(parameter.name.clone(), value);
    } else if let Some(values) = frame.atomic_sequences.get(variable).cloned() {
        frame.bind_atomic_sequence(parameter.name.clone(), values);
    } else if let Some((owner, nodes)) = frame.retained_source(inputs.globals, variable) {
        let value = RetainedSourceSelection {
            owner: owner.clone(),
            nodes: nodes.clone(),
        };
        frame.bind_retained_source(parameter.name.clone(), value);
    } else if let Some(nodes) = frame.source_nodes_in_context(inputs, variable)?.cloned() {
        frame.bind_source_nodes(parameter.name.clone(), nodes);
    } else if let Some(tree) = frame.temporary_tree(inputs.globals, variable).cloned() {
        frame.bind_temporary_tree(parameter.name.clone(), tree);
    } else {
        return Err(failure(
            "FXRT0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            format!("unbound template parameter default variable: ${variable}"),
        ));
    }
    Ok(())
}

pub(super) fn required_source_context<'a>(
    inputs: &SequenceInputs<'a>,
    context: Option<NodeId>,
) -> Result<(&'a Document, NodeId), ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "the instruction requires a principal source and context item",
        )
    })?;
    let context = context.ok_or_else(|| {
        failure(
            "XPDY0002",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            "the instruction requires a principal source and context item",
        )
    })?;
    Ok((source, context))
}

#[cfg(test)]
mod frame_sharing_tests {
    use std::sync::Arc;

    use super::{AtomicValue, RuntimeVariables, TemporaryTree};

    #[test]
    fn non_atomic_sequence_frames_detach_only_the_mutated_value_kind() {
        let mut parent = RuntimeVariables::default();
        parent.bind_atomic_sequence(
            "sequence".to_owned(),
            vec![AtomicValue::string("parent".to_owned())],
        );
        parent.bind_source_nodes("nodes".to_owned(), Vec::new());
        parent.bind_temporary_tree(
            "tree".to_owned(),
            TemporaryTree {
                identity: 1,
                roots: Vec::new(),
                nodes: Vec::new(),
                xslt10_disable_output_escaping_text: std::collections::HashSet::new(),
            },
        );

        let mut child = parent.clone();
        assert!(Arc::ptr_eq(&parent.atomics, &child.atomics));
        assert!(Arc::ptr_eq(
            &parent.atomic_sequences,
            &child.atomic_sequences
        ));
        assert!(Arc::ptr_eq(&parent.source_nodes, &child.source_nodes));
        assert!(Arc::ptr_eq(&parent.temporary_trees, &child.temporary_trees));
        assert!(Arc::ptr_eq(&parent.local_bindings, &child.local_bindings));

        child.bind_atomic_sequence(
            "child-sequence".to_owned(),
            vec![AtomicValue::string("child".to_owned())],
        );
        assert!(!Arc::ptr_eq(
            &parent.atomic_sequences,
            &child.atomic_sequences
        ));
        assert!(Arc::ptr_eq(&parent.source_nodes, &child.source_nodes));
        assert!(Arc::ptr_eq(&parent.temporary_trees, &child.temporary_trees));
        assert!(!Arc::ptr_eq(&parent.local_bindings, &child.local_bindings));
        assert!(!parent.atomic_sequences.contains_key("child-sequence"));

        child.bind_atomic(
            "sequence".to_owned(),
            AtomicValue::string("shadow".to_owned()),
        );
        assert!(parent.atomic_sequences.contains_key("sequence"));
        assert!(!child.atomic_sequences.contains_key("sequence"));
        assert!(child.atomics.contains_key("sequence"));
        assert!(!parent.atomics.contains_key("sequence"));
    }

    #[test]
    fn complete_sequence_frame_oracle_owns_every_map() {
        let mut parent = RuntimeVariables::default();
        parent.bind_source_nodes("nodes".to_owned(), Vec::new());
        let complete = parent.clone_complete_for_sequence();

        assert!(!Arc::ptr_eq(&parent.atomics, &complete.atomics));
        assert!(!Arc::ptr_eq(
            &parent.atomic_sequences,
            &complete.atomic_sequences
        ));
        assert!(!Arc::ptr_eq(&parent.source_nodes, &complete.source_nodes));
        assert!(!Arc::ptr_eq(
            &parent.temporary_trees,
            &complete.temporary_trees
        ));
        assert!(!Arc::ptr_eq(
            &parent.local_bindings,
            &complete.local_bindings
        ));
    }
}

#[cfg(all(test, feature = "allocation-observation"))]
mod frame_clone_measurement_tests {
    use std::{hint::black_box, sync::Arc, time::Instant};

    use crate::execution_control_experiment::InvocationControl;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document_controlled};

    use super::{
        AtomicValue, Document, RuntimeVariables, TemporaryNode, TemporaryNodeKind, TemporaryTree,
    };

    const VECTOR_ITEMS: usize = 8;

    fn populated_frame(bindings: usize) -> RuntimeVariables {
        let mut parse_control = InvocationControl::unbounded();
        let parsed = parse_document_controlled(
            "urn:fastxslt:frame-clone-measurement",
            b"<root/>",
            ParseLimits {
                max_events: 16,
                max_depth: 4,
            },
            &mut parse_control,
        )
        .expect("parse frame measurement document");
        let document = Document::from_parsed(parsed).expect("build frame measurement document");
        let node = document.document_node();
        let mut frame = RuntimeVariables::default();
        for index in 0..bindings {
            let atomic_name = format!("a{index}");
            Arc::make_mut(&mut frame.atomic_sequences).insert(
                atomic_name.clone(),
                (0..VECTOR_ITEMS)
                    .map(|item| AtomicValue::string(format!("value-{index}-{item}")))
                    .collect(),
            );
            Arc::make_mut(&mut frame.local_bindings).insert(atomic_name);
            let node_name = format!("n{index}");
            Arc::make_mut(&mut frame.source_nodes)
                .insert(node_name.clone(), vec![node; VECTOR_ITEMS]);
            Arc::make_mut(&mut frame.local_bindings).insert(node_name);
            let tree_name = format!("t{index}");
            Arc::make_mut(&mut frame.temporary_trees).insert(
                tree_name.clone(),
                TemporaryTree {
                    identity: u64::try_from(index).expect("measurement index fits u64"),
                    roots: vec![0],
                    nodes: (0..VECTOR_ITEMS)
                        .map(|item| TemporaryNode {
                            kind: TemporaryNodeKind::Text(format!("temporary-{index}-{item}")),
                            parent: None,
                            children: Vec::new(),
                        })
                        .collect(),
                    xslt10_disable_output_escaping_text: std::collections::HashSet::new(),
                },
            );
            Arc::make_mut(&mut frame.local_bindings).insert(tree_name);
        }
        frame
    }

    fn median_clone_us<T: Clone>(value: &T, iterations: usize) -> f64 {
        let mut samples = Vec::with_capacity(5);
        let divisor = f64::from(u32::try_from(iterations).expect("iterations fit u32"));
        for _ in 0..5 {
            let started = Instant::now();
            for _ in 0..iterations {
                drop(black_box(value.clone()));
            }
            samples.push(started.elapsed().as_secs_f64() * 1_000_000.0 / divisor);
        }
        samples.sort_by(f64::total_cmp);
        samples[2]
    }

    #[test]
    #[ignore = "manual release-mode non-atomic runtime-frame clone attribution"]
    fn measure_non_atomic_runtime_frame_clone_fields() {
        for bindings in [0_usize, 16, 64, 256] {
            let frame = populated_frame(bindings);
            let iterations = if bindings <= 16 { 2_000 } else { 200 };
            let atomics = allocation_counter::measure(|| drop(black_box(frame.atomics.clone())));
            let atomic_sequences = allocation_counter::measure(|| {
                drop(black_box(frame.atomic_sequences.as_ref().clone()));
            });
            let source_nodes = allocation_counter::measure(|| {
                drop(black_box(frame.source_nodes.as_ref().clone()));
            });
            let temporary_trees = allocation_counter::measure(|| {
                drop(black_box(frame.temporary_trees.as_ref().clone()));
            });
            let local_bindings = allocation_counter::measure(|| {
                drop(black_box(frame.local_bindings.as_ref().clone()));
            });
            let complete = allocation_counter::measure(|| {
                drop(black_box(frame.clone_complete_for_sequence()));
            });

            println!(
                "bindings_per_kind={bindings} vector_items={VECTOR_ITEMS} atomic_sequences_median_us={:.3} source_nodes_median_us={:.3} temporary_trees_median_us={:.3} local_bindings_median_us={:.3} complete_median_us={:.3} atomics={atomics:?} atomic_sequences={atomic_sequences:?} source_nodes={source_nodes:?} temporary_trees={temporary_trees:?} local_bindings={local_bindings:?} complete={complete:?}",
                median_clone_us(frame.atomic_sequences.as_ref(), iterations),
                median_clone_us(frame.source_nodes.as_ref(), iterations),
                median_clone_us(frame.temporary_trees.as_ref(), iterations),
                median_clone_us(frame.local_bindings.as_ref(), iterations),
                median_clone_us(&CompleteFrameClone(&frame), iterations),
            );
        }
    }

    struct CompleteFrameClone<'a>(&'a RuntimeVariables);

    impl Clone for CompleteFrameClone<'_> {
        fn clone(&self) -> Self {
            drop(self.0.clone_complete_for_sequence());
            Self(self.0)
        }
    }
}
