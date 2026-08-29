//! Invocation-local globals, variable frames, and temporary-tree preparation.

use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ConstructedElement, GlobalBindingDefault, StylesheetProgram, Template, TemplateArgument,
    TemplateArgumentValue,
};

use super::{ExecutionFailure, FailureCategory, control_failure, failure, failure_at};

pub(super) struct SequenceInputs<'a> {
    pub(super) program: &'a StylesheetProgram,
    pub(super) source: Option<&'a Document>,
    pub(super) request_id: &'a str,
    pub(super) globals: &'a RuntimeGlobals,
}

#[derive(Debug, Default)]
pub(super) struct RuntimeGlobals {
    pub(super) atomics: BTreeMap<String, AtomicValue>,
    pub(super) nodes: BTreeMap<String, Vec<NodeId>>,
    pub(super) temporary_trees: BTreeMap<String, TemporaryTree>,
}

#[derive(Debug, Default)]
pub(super) struct TemporaryTree {
    pub(super) roots: Vec<usize>,
    pub(super) nodes: Vec<TemporaryNode>,
}

#[derive(Debug)]
pub(super) struct TemporaryNode {
    pub(super) name: ExpandedName,
    pub(super) namespaces: Vec<NamespaceBinding>,
    pub(super) children: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct RuntimeVariables {
    pub(super) atomics: BTreeMap<String, AtomicValue>,
    pub(super) atomic_sequences: BTreeMap<String, Vec<AtomicValue>>,
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
    pub(super) fn from_atomics(atomics: &BTreeMap<String, AtomicValue>) -> Self {
        Self {
            atomics: atomics.clone(),
            atomic_sequences: BTreeMap::new(),
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
                globals
                    .atomics
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
        match &binding.default {
            GlobalBindingDefault::Text(value) => {
                globals
                    .atomics
                    .insert(binding.name.clone(), AtomicValue::untyped(value.clone()));
            }
            GlobalBindingDefault::Integer(value) => {
                globals.atomics.insert(
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
                let nodes = evaluate_location_path_controlled(
                    source,
                    source.document_node(),
                    path,
                    control,
                )
                .map_err(|failure| control_failure(failure, request_id))?;
                globals.nodes.insert(binding.name.clone(), nodes);
            }
            GlobalBindingDefault::Variable(name) => {
                if let Some(value) = globals.atomics.get(name).cloned() {
                    globals.atomics.insert(binding.name.clone(), value);
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
        }
    }
    Ok(globals)
}

fn materialize_temporary_tree(
    elements: &[ConstructedElement],
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    let mut tree = TemporaryTree::default();
    for element in elements {
        let root = materialize_temporary_element(element, &mut tree, request_id, control)?;
        tree.roots.push(root);
    }
    Ok(tree)
}

fn materialize_temporary_element(
    element: &ConstructedElement,
    tree: &mut TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let node = tree.nodes.len();
    tree.nodes.push(TemporaryNode {
        name: element.name.clone(),
        namespaces: element.namespaces.clone(),
        children: Vec::new(),
    });
    for child in &element.children {
        let child = materialize_temporary_element(child, tree, request_id, control)?;
        tree.nodes[node].children.push(child);
    }
    Ok(node)
}

pub(super) fn bind_template_parameters(
    template: &Template,
    supplied: &BTreeMap<String, InvocationParameter>,
    base: &BTreeMap<String, AtomicValue>,
) -> RuntimeVariables {
    let mut frame = RuntimeVariables::from_atomics(base);
    for parameter in &template.parameters {
        let value = supplied
            .get(&parameter.name)
            .filter(|supplied| supplied.tunnel == parameter.tunnel)
            .map_or_else(
                || AtomicValue::string(""),
                |supplied| supplied.value.clone(),
            );
        frame.atomics.insert(parameter.name.clone(), value);
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
