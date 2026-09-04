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

#[derive(Debug, Clone)]
pub(super) struct TemporaryTree {
    pub(super) identity: u64,
    pub(super) roots: Vec<usize>,
    pub(super) nodes: Vec<TemporaryNode>,
}

#[derive(Debug, Clone)]
pub(super) struct TemporaryNode {
    pub(super) kind: TemporaryNodeKind,
    pub(super) parent: Option<usize>,
    pub(super) children: Vec<usize>,
}

#[derive(Debug, Clone)]
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
    pub(super) atomic_sequences: BTreeMap<String, Vec<AtomicValue>>,
    pub(super) source_nodes: BTreeMap<String, Vec<NodeId>>,
    pub(super) temporary_trees: BTreeMap<String, TemporaryTree>,
    local_bindings: HashSet<String>,
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
            atomic_sequences: BTreeMap::new(),
            source_nodes: BTreeMap::new(),
            temporary_trees: BTreeMap::new(),
            local_bindings: HashSet::new(),
        }
    }

    fn clear_value_kinds(&mut self, name: &str) {
        Arc::make_mut(&mut self.atomics).remove(name);
        self.atomic_sequences.remove(name);
        self.source_nodes.remove(name);
        self.temporary_trees.remove(name);
        self.local_bindings.insert(name.to_owned());
    }

    pub(super) fn bind_atomic(&mut self, name: String, value: AtomicValue) {
        self.clear_value_kinds(&name);
        Arc::make_mut(&mut self.atomics).insert(name, value);
    }

    pub(super) fn bind_atomic_sequence(&mut self, name: String, values: Vec<AtomicValue>) {
        self.clear_value_kinds(&name);
        self.atomic_sequences.insert(name, values);
    }

    pub(super) fn bind_source_nodes(&mut self, name: String, nodes: Vec<NodeId>) {
        self.clear_value_kinds(&name);
        self.source_nodes.insert(name, nodes);
    }

    pub(super) fn bind_temporary_tree(&mut self, name: String, tree: TemporaryTree) {
        self.clear_value_kinds(&name);
        self.temporary_trees.insert(name, tree);
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
                        InvocationParameterValue::SourceNodes(_) => {
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
            if let Some(value) = globals.atomics.get(name).cloned() {
                Arc::make_mut(&mut globals.atomics).insert(binding.name.clone(), value);
            } else if let Some(nodes) = globals.nodes.get(name).cloned() {
                globals.nodes.insert(binding.name.clone(), nodes);
            } else {
                return Err(failure(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(request_id),
                    format!("unbound global dependency: ${name}"),
                ));
            }
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

fn materialize_parentless_temporary_node(
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
