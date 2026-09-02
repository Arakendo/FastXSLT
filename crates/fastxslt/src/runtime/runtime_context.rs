//! Invocation-local globals, variable frames, and temporary-tree preparation.

use std::{cell::RefCell, collections::BTreeMap, sync::Arc};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ConstructedElement, ConstructedNode, GlobalBinding, GlobalBindingDefault, StylesheetProgram,
    Template, TemplateArgument, TemplateArgumentValue, TemplateParameterDefault,
};

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
}

#[derive(Debug, Default)]
pub(super) struct RuntimeGlobals {
    pub(super) atomics: Arc<BTreeMap<String, AtomicValue>>,
    pub(super) nodes: BTreeMap<String, Vec<NodeId>>,
    pub(super) temporary_trees: BTreeMap<String, TemporaryTree>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct TemporaryTree {
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
    pub(super) temporary_trees: BTreeMap<String, TemporaryTree>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InvocationParameter {
    pub(super) value: AtomicValue,
    pub(super) tunnel: bool,
}

pub(super) fn evaluate_template_arguments(
    arguments: &[TemplateArgument],
    variables: &RuntimeVariables,
    request_id: &str,
) -> Result<BTreeMap<String, InvocationParameter>, ExecutionFailure> {
    arguments
        .iter()
        .map(|argument| {
            let value = match &argument.value {
                TemplateArgumentValue::Text(value) => AtomicValue::string(value.clone()),
                TemplateArgumentValue::Integer(value) => AtomicValue::from_validated_lexical(
                    BuiltinAtomicType::Integer,
                    value.to_string(),
                ),
                TemplateArgumentValue::Variable(name) => {
                    variables.atomics.get(name).cloned().ok_or_else(|| {
                        failure_at(
                            "FXRT0002",
                            FailureCategory::Invalid,
                            Some(request_id),
                            argument.location.clone(),
                            format!("unbound template argument variable: ${name}"),
                        )
                    })?
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
            temporary_trees: BTreeMap::new(),
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
                Arc::make_mut(&mut globals.atomics)
                    .insert(binding.name.clone(), parameter.value.clone());
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
        GlobalBindingDefault::Text(value) => {
            Arc::make_mut(&mut globals.atomics)
                .insert(binding.name.clone(), AtomicValue::untyped(value.clone()));
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

fn materialize_parentless_temporary_node(
    kind: TemporaryNodeKind,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(TemporaryTree {
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
    let mut tree = TemporaryTree::default();
    for element in elements {
        let root = materialize_temporary_element(element, None, &mut tree, request_id, control)?;
        tree.roots.push(root);
    }
    Ok(tree)
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
) -> RuntimeVariables {
    let mut frame = RuntimeVariables::from_atomics(base, complete_clone);
    for parameter in &template.parameters {
        let value = supplied
            .get(&parameter.name)
            .filter(|supplied| supplied.tunnel == parameter.tunnel)
            .map_or_else(
                || match &parameter.default {
                    TemplateParameterDefault::Text(value) => AtomicValue::string(value.clone()),
                    TemplateParameterDefault::Integer(value) => {
                        AtomicValue::from_validated_lexical(
                            BuiltinAtomicType::Integer,
                            value.to_string(),
                        )
                    }
                },
                |supplied| supplied.value.clone(),
            );
        Arc::make_mut(&mut frame.atomics).insert(parameter.name.clone(), value);
    }
    frame
}

pub(super) fn required_source_context<'a>(
    inputs: &SequenceInputs<'a>,
    context: Option<NodeId>,
) -> Result<(&'a Document, NodeId), ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "the instruction requires a principal source and context item",
        )
    })?;
    let context = context.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "the instruction requires a principal source and context item",
        )
    })?;
    Ok((source, context))
}
