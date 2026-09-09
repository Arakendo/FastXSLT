//! Invocation-local globals, variable frames, and temporary-tree preparation.

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
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ConstructedElement, ConstructedNode, GlobalBinding, GlobalBindingDefault, StylesheetProgram,
    Template, TemplateArgument, TemplateArgumentValue, TemplateParameterDefault,
};

use super::dynamic_document::DynamicDocument;
use super::template_selector::DocumentRootedMatchCache;
use super::value_evaluator::evaluate_xslt10_sum_path;
use super::{
    ExecutionFailure, FailureCategory, MultipleMatchPolicy, control_failure, failure, failure_at,
};

pub(super) struct SequenceInputs<'a> {
    pub(super) program: &'a StylesheetProgram,
    pub(super) source: Option<&'a Document>,
    pub(super) request_id: &'a str,
    pub(super) globals: &'a RuntimeGlobals,
    pub(super) multiple_match_policy: MultipleMatchPolicy,
    pub(super) document_rooted_matches: RefCell<DocumentRootedMatchCache>,
    pub(super) complete_atomic_frame_clones: bool,
    pub(super) resource_snapshot: Option<&'a ResourceSnapshot>,
    pub(super) denied_resources: Option<&'a HashSet<String>>,
    pub(super) dynamic_documents: RefCell<BTreeMap<String, DynamicDocument>>,
}

#[derive(Debug, Default)]
pub(super) struct RuntimeGlobals {
    pub(super) atomics: Arc<BTreeMap<String, AtomicValue>>,
    pub(super) empty_sequences: HashSet<String>,
    pub(super) nodes: BTreeMap<String, Vec<NodeId>>,
    pub(super) temporary_trees: BTreeMap<String, TemporaryTree>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TemporaryTree {
    pub(super) identity: u64,
    pub(super) roots: Vec<usize>,
    pub(super) nodes: Vec<TemporaryNode>,
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
    pub(super) atomics: Arc<BTreeMap<String, AtomicValue>>,
    pub(super) atomic_sequences: Arc<BTreeMap<String, Vec<AtomicValue>>>,
    pub(super) source_nodes: Arc<BTreeMap<String, Vec<NodeId>>>,
    pub(super) temporary_trees: Arc<BTreeMap<String, TemporaryTree>>,
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
    context: Option<NodeId>,
    focus_position: usize,
    focus_size: usize,
    control: &mut InvocationControl,
) -> Result<BTreeMap<String, InvocationParameter>, ExecutionFailure> {
    arguments
        .iter()
        .map(|argument| {
            let value = match &argument.value {
                TemplateArgumentValue::Text(value) => {
                    InvocationParameterValue::Atomic(AtomicValue::string(value.clone()))
                }
                TemplateArgumentValue::Integer(value) => {
                    InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                        BuiltinAtomicType::Integer,
                        value.to_string(),
                    ))
                }
                TemplateArgumentValue::Boolean(value) => {
                    InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                        BuiltinAtomicType::Boolean,
                        value.to_string(),
                    ))
                }
                TemplateArgumentValue::ContextPosition => {
                    InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                        BuiltinAtomicType::Integer,
                        focus_position.to_string(),
                    ))
                }
                TemplateArgumentValue::ContextSize => {
                    InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                        BuiltinAtomicType::Integer,
                        focus_size.to_string(),
                    ))
                }
                TemplateArgumentValue::CurrentSourceNode => {
                    let (_, context) = required_source_context(inputs, context)?;
                    InvocationParameterValue::SourceNodes(vec![context])
                }
                TemplateArgumentValue::Variable(name) => {
                    if let Some(value) = variables.atomics.get(name) {
                        InvocationParameterValue::Atomic(value.clone())
                    } else if let Some(nodes) = variables.source_nodes(inputs.globals, name) {
                        InvocationParameterValue::SourceNodes(nodes.clone())
                    } else {
                        return Err(failure_at(
                            "FXRT0002",
                            FailureCategory::Invalid,
                            Some(inputs.request_id),
                            argument.location.clone(),
                            format!("unbound template argument variable: ${name}"),
                        ));
                    }
                }
                TemplateArgumentValue::SourcePath(path) => {
                    let (source, context) = required_source_context(inputs, context)?;
                    let nodes = evaluate_location_path_controlled(source, context, path, control)
                        .map_err(|failure| control_failure(failure, inputs.request_id))?;
                    InvocationParameterValue::SourceNodes(nodes)
                }
                TemplateArgumentValue::Xslt10SumPath(path) => {
                    let value = evaluate_xslt10_sum_path(inputs, context, path, control)?;
                    InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                        BuiltinAtomicType::Double,
                        value,
                    ))
                }
                TemplateArgumentValue::Xslt10Value(expression) => {
                    let value = super::value_evaluator::evaluate_as_temporary_text(
                        inputs,
                        expression,
                        context,
                        focus_position,
                        focus_size,
                        variables,
                        control,
                    )?;
                    InvocationParameterValue::TemporaryTree(materialize_parentless_temporary_node(
                        TemporaryNodeKind::Text(value),
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
            };
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
            atomics: if complete_clone {
                Arc::new(atomics.as_ref().clone())
            } else {
                Arc::clone(atomics)
            },
            atomic_sequences: Arc::new(BTreeMap::new()),
            source_nodes: Arc::new(BTreeMap::new()),
            temporary_trees: Arc::new(BTreeMap::new()),
            local_bindings: Arc::new(HashSet::new()),
        }
    }

    fn clear_value_kinds(&mut self, name: &str) {
        if self.atomics.contains_key(name) {
            Arc::make_mut(&mut self.atomics).remove(name);
        }
        if self.atomic_sequences.contains_key(name) {
            Arc::make_mut(&mut self.atomic_sequences).remove(name);
        }
        if self.source_nodes.contains_key(name) {
            Arc::make_mut(&mut self.source_nodes).remove(name);
        }
        if self.temporary_trees.contains_key(name) {
            Arc::make_mut(&mut self.temporary_trees).remove(name);
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

    pub(super) fn bind_temporary_tree(&mut self, name: String, tree: TemporaryTree) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.temporary_trees).insert(name, tree);
    }

    pub(super) fn source_nodes<'a>(
        &'a self,
        globals: &'a RuntimeGlobals,
        name: &str,
    ) -> Option<&'a Vec<NodeId>> {
        self.source_nodes.get(name).or_else(|| {
            (!self.local_bindings.contains(name))
                .then(|| globals.nodes.get(name))
                .flatten()
        })
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
            self.source_nodes.len(),
            self.temporary_trees.len(),
            self.local_bindings.len(),
        )
    }

    #[cfg(test)]
    pub(super) fn clone_complete_for_sequence(&self) -> Self {
        Self {
            atomics: Arc::new(self.atomics.as_ref().clone()),
            atomic_sequences: Arc::new(self.atomic_sequences.as_ref().clone()),
            source_nodes: Arc::new(self.source_nodes.as_ref().clone()),
            temporary_trees: Arc::new(self.temporary_trees.as_ref().clone()),
            local_bindings: Arc::new(self.local_bindings.as_ref().clone()),
        }
    }
}

pub(super) fn materialize_global_defaults(
    program: &StylesheetProgram,
    source: Option<&Document>,
    parameters: &BTreeMap<String, InvocationParameter>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<RuntimeGlobals, ExecutionFailure> {
    let mut globals = RuntimeGlobals::default();
    for binding in &program.global_bindings {
        if binding.kind == crate::xslt::golden_semantics_experiment::GlobalBindingKind::Parameter {
            if let Some(parameter) = parameters
                .get(&binding.name)
                .filter(|parameter| !parameter.tunnel)
            {
                Arc::make_mut(&mut globals.atomics).insert(
                    binding.name.clone(),
                    match &parameter.value {
                        InvocationParameterValue::Atomic(value) => value.clone(),
                        InvocationParameterValue::SourceNodes(_)
                        | InvocationParameterValue::TemporaryTree(_) => {
                            return Err(failure(
                                "XTTE0590",
                                FailureCategory::Invalid,
                                Some(request_id),
                                format!(
                                    "global parameter requires an atomic host value: ${}",
                                    binding.name
                                ),
                            ));
                        }
                    },
                );
                continue;
            }
            if binding.required {
                return Err(failure(
                    "XTDE0050",
                    FailureCategory::Invalid,
                    Some(request_id),
                    format!(
                        "required global parameter was not supplied: ${}",
                        binding.name
                    ),
                ));
            }
        }
        materialize_global_default(&mut globals, binding, source, request_id, control)?;
    }
    Ok(globals)
}

fn materialize_global_default(
    globals: &mut RuntimeGlobals,
    binding: &GlobalBinding,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    match &binding.default {
        GlobalBindingDefault::EmptySequence => {
            globals.empty_sequences.insert(binding.name.clone());
        }
        GlobalBindingDefault::Text(value) => {
            Arc::make_mut(&mut globals.atomics)
                .insert(binding.name.clone(), AtomicValue::untyped(value.clone()));
        }
        GlobalBindingDefault::Atomic(value) => {
            Arc::make_mut(&mut globals.atomics).insert(binding.name.clone(), value.clone());
        }
        GlobalBindingDefault::Integer(value) => {
            Arc::make_mut(&mut globals.atomics).insert(
                binding.name.clone(),
                AtomicValue::from_validated_lexical(
                    crate::xdm::atomic_value_experiment::BuiltinAtomicType::Integer,
                    value.to_string(),
                ),
            );
        }
        GlobalBindingDefault::DoubleDivision { .. } => {
            materialize_double_division(globals, binding, source, request_id, control)?;
        }
        GlobalBindingDefault::CountLocationPath(path) => {
            materialize_global_count(globals, binding, path, source, request_id, control)?;
        }
        GlobalBindingDefault::LocationPath(path) => {
            let source = source.ok_or_else(|| {
                failure(
                    "FXRT1004",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    "a source-dependent global binding requires a principal source",
                )
            })?;
            let nodes =
                evaluate_location_path_controlled(source, source.document_node(), path, control)
                    .map_err(|failure| control_failure(failure, request_id))?;
            globals.nodes.insert(binding.name.clone(), nodes);
        }
        GlobalBindingDefault::SourceNodeIdentity(path) => {
            materialize_source_node_identity(globals, binding, path, source, request_id, control)?;
        }
        GlobalBindingDefault::Variable(name) => {
            materialize_global_alias(globals, binding, name, request_id)?;
        }
        GlobalBindingDefault::TemporaryTree(elements) => {
            let tree = materialize_temporary_tree(elements, request_id, control)?;
            globals.temporary_trees.insert(binding.name.clone(), tree);
        }
        GlobalBindingDefault::TemporaryText(value) => {
            let tree = materialize_parentless_temporary_node(
                TemporaryNodeKind::Text(value.clone()),
                request_id,
                control,
            )?;
            globals.temporary_trees.insert(binding.name.clone(), tree);
        }
        GlobalBindingDefault::Xslt10TemporarySourceString(path) => {
            let tree =
                materialize_xslt10_temporary_source_string(path, source, request_id, control)?;
            globals.temporary_trees.insert(binding.name.clone(), tree);
        }
        GlobalBindingDefault::TemporaryAttribute { name, value } => {
            let tree = materialize_parentless_temporary_node(
                TemporaryNodeKind::Attribute {
                    name: name.clone(),
                    value: value.clone(),
                },
                request_id,
                control,
            )?;
            globals.temporary_trees.insert(binding.name.clone(), tree);
        }
        GlobalBindingDefault::TemporaryComment(value) => {
            let tree = materialize_parentless_temporary_node(
                TemporaryNodeKind::Comment(value.clone()),
                request_id,
                control,
            )?;
            globals.temporary_trees.insert(binding.name.clone(), tree);
        }
        GlobalBindingDefault::TemporaryProcessingInstruction { target, value } => {
            let tree = materialize_parentless_temporary_node(
                TemporaryNodeKind::ProcessingInstruction {
                    target: target.clone(),
                    value: value.clone(),
                },
                request_id,
                control,
            )?;
            globals.temporary_trees.insert(binding.name.clone(), tree);
        }
    }
    Ok(())
}

fn materialize_global_count(
    globals: &mut RuntimeGlobals,
    binding: &GlobalBinding,
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "a source-dependent global count requires a principal source",
        )
    })?;
    let nodes = evaluate_location_path_controlled(source, source.document_node(), path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    Arc::make_mut(&mut globals.atomics).insert(
        binding.name.clone(),
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, nodes.len().to_string()),
    );
    Ok(())
}

fn materialize_global_alias(
    globals: &mut RuntimeGlobals,
    binding: &GlobalBinding,
    dependency: &str,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if let Some(value) = globals.atomics.get(dependency).cloned() {
        Arc::make_mut(&mut globals.atomics).insert(binding.name.clone(), value);
        return Ok(());
    }
    if let Some(nodes) = globals.nodes.get(dependency).cloned() {
        globals.nodes.insert(binding.name.clone(), nodes);
        return Ok(());
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(request_id),
        format!("unbound global dependency: ${dependency}"),
    ))
}

fn materialize_xslt10_temporary_source_string(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
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
    let value = selected.first().map_or_else(
        || Ok(String::new()),
        |node| {
            source
                .string_value_controlled(*node, control)
                .map_err(|failure| control_failure(failure, request_id))
        },
    )?;
    materialize_parentless_temporary_node(TemporaryNodeKind::Text(value), request_id, control)
}

fn materialize_double_division(
    globals: &mut RuntimeGlobals,
    binding: &GlobalBinding,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let GlobalBindingDefault::DoubleDivision {
        numerator,
        denominator,
    } = &binding.default
    else {
        unreachable!("double-division materialization requires its compiled representation");
    };
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "a source-dependent numeric global requires a principal source",
        )
    })?;
    let numerator =
        evaluate_location_path_controlled(source, source.document_node(), numerator, control)
            .map_err(|failure| control_failure(failure, request_id))?;
    let denominator =
        evaluate_location_path_controlled(source, source.document_node(), denominator, control)
            .map_err(|failure| control_failure(failure, request_id))?;
    if numerator.is_empty() || denominator.is_empty() {
        globals.empty_sequences.insert(binding.name.clone());
        return Ok(());
    }
    let ([numerator], [denominator]) = (numerator.as_slice(), denominator.as_slice()) else {
        return Err(failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "the private numeric-global slice requires singleton path operands",
        ));
    };
    let numerator = source
        .string_value_controlled(*numerator, control)
        .map_err(|failure| control_failure(failure, request_id))?
        .trim()
        .parse::<f64>()
        .map_err(|_| {
            failure(
                "FORG0001",
                FailureCategory::Invalid,
                Some(request_id),
                "the numeric global numerator cannot be converted to xs:double",
            )
        })?;
    let denominator = source
        .string_value_controlled(*denominator, control)
        .map_err(|failure| control_failure(failure, request_id))?
        .trim()
        .parse::<f64>()
        .map_err(|_| {
            failure(
                "FORG0001",
                FailureCategory::Invalid,
                Some(request_id),
                "the numeric global denominator cannot be converted to xs:double",
            )
        })?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let quotient = numerator / denominator;
    let lexical = if quotient.is_nan() {
        "NaN".to_owned()
    } else {
        quotient.to_string()
    };
    Arc::make_mut(&mut globals.atomics).insert(
        binding.name.clone(),
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Double, lexical),
    );
    Ok(())
}

fn materialize_source_node_identity(
    globals: &mut RuntimeGlobals,
    binding: &GlobalBinding,
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "a source identity global requires a principal source",
        )
    })?;
    let nodes = evaluate_location_path_controlled(source, source.document_node(), path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    let [node] = nodes.as_slice() else {
        return Err(failure(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(request_id),
            "generate-id() requires exactly one source node in this private slice",
        ));
    };
    Arc::make_mut(&mut globals.atomics).insert(
        binding.name.clone(),
        AtomicValue::string(source_node_identity(*node)),
    );
    Ok(())
}

pub(super) fn source_node_identity(node: NodeId) -> String {
    format!("fastxslt-principal-n{}", node.index())
}

pub(super) fn source_node_identity_for_path(
    source: &Document,
    context: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
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
    Ok(nodes.first().copied().map(source_node_identity))
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
    };
    for element in elements {
        let root = materialize_temporary_element(element, None, &mut tree, request_id, control)?;
        tree.roots.push(root);
    }
    Ok(tree)
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
        let child = materialize_temporary_node(child, Some(node), tree, request_id, control)?;
        tree.nodes[node].children.push(child);
    }
    Ok(node)
}

fn materialize_temporary_node(
    constructed: &ConstructedNode,
    parent: Option<usize>,
    tree: &mut TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    match constructed {
        ConstructedNode::Element(element) => {
            materialize_temporary_element(element, parent, tree, request_id, control)
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
    }
}

pub(super) fn bind_template_parameters(
    template: &Template,
    supplied: &BTreeMap<String, InvocationParameter>,
    base: &Arc<BTreeMap<String, AtomicValue>>,
    complete_clone: bool,
    request_id: &str,
) -> Result<RuntimeVariables, ExecutionFailure> {
    let mut frame = RuntimeVariables::from_atomics(base, complete_clone);
    for parameter in &template.parameters {
        if parameter.required
            && supplied
                .get(&parameter.name)
                .is_none_or(|supplied| supplied.tunnel != parameter.tunnel)
        {
            return Err(failure(
                "XTDE0700",
                FailureCategory::Invalid,
                Some(request_id),
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
            Some(InvocationParameterValue::SourceNodes(nodes)) => {
                frame.bind_source_nodes(parameter.name.clone(), nodes.clone());
            }
            Some(InvocationParameterValue::TemporaryTree(tree)) => {
                frame.bind_temporary_tree(parameter.name.clone(), tree.clone());
            }
            None => {
                let value = match &parameter.default {
                    TemplateParameterDefault::Text(value) => AtomicValue::string(value.clone()),
                    TemplateParameterDefault::Integer(value) => {
                        AtomicValue::from_validated_lexical(
                            BuiltinAtomicType::Integer,
                            value.to_string(),
                        )
                    }
                };
                frame.bind_atomic(parameter.name.clone(), value);
            }
        }
    }
    Ok(frame)
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
