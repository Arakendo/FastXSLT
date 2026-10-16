//! Existing global initializer execution and publication, private to the runtime.
//! Initializers return an owned value before publishing to the completed maps.

#[path = "global_resolution.rs"]
mod global_resolution;
pub(in crate::runtime) use global_resolution::GlobalInitialization;

#[cfg(test)]
#[path = "global_initialization_tests.rs"]
mod tests;

use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use super::super::{ExecutionFailure, FailureCategory, control_failure, failure};
use super::{
    GlobalDocumentContext, InvocationParameter, InvocationParameterValue, RuntimeGlobals,
    TemporaryNode, TemporaryNodeKind, TemporaryTree, allocate_temporary_tree_identity,
    materialize_parentless_temporary_node, materialize_temporary_nodes,
    materialize_xslt10_for_each_text_tree, source_node_identity, source_path_string_value,
    temporary_tree_string_value,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    GlobalBinding, GlobalBindingDefault, StylesheetProgram, Xslt10ConcatExpression,
    Xslt10ConcatPart,
};

/// A transient initializer result, not a second completed-value cache.
/// Source IDs remain tied to the invocation's effective principal owner.
#[derive(Debug)]
enum InitializedGlobal {
    Empty,
    Atomic(AtomicValue),
    SourceNodes(Vec<NodeId>),
    TemporaryTree(TemporaryTree),
    LiteralDocument(super::super::dynamic_document::DynamicDocument),
}

impl InitializedGlobal {
    fn publish(self, globals: &mut RuntimeGlobals, name: &str) {
        match self {
            Self::Empty => {
                globals.empty_sequences.insert(name.to_owned());
            }
            Self::Atomic(value) => {
                Arc::make_mut(&mut globals.atomics).insert(name.to_owned(), value);
            }
            Self::SourceNodes(nodes) => {
                globals.nodes.insert(name.to_owned(), nodes);
            }
            Self::TemporaryTree(tree) => {
                globals.temporary_trees.insert(name.to_owned(), tree);
            }
            Self::LiteralDocument(document) => {
                globals.literal_documents.insert(name.to_owned(), document);
            }
        }
    }
}

/// Resolve before borrowing completed values. The callback cannot return a
/// borrowed map value or recursively resolve another binding while it runs.
/// This is private initializer plumbing, not a document/provider or host API.
trait GlobalInitializerReads {
    fn with_completed<T>(
        &self,
        name: &str,
        control: &mut InvocationControl,
        read: impl FnOnce(&RuntimeGlobals, &mut InvocationControl) -> Result<T, ExecutionFailure>,
    ) -> Result<T, ExecutionFailure>;
}

impl GlobalInitializerReads for RuntimeGlobals {
    fn with_completed<T>(
        &self,
        _name: &str,
        control: &mut InvocationControl,
        read: impl FnOnce(&RuntimeGlobals, &mut InvocationControl) -> Result<T, ExecutionFailure>,
    ) -> Result<T, ExecutionFailure> {
        read(self, control)
    }
}

pub(in super::super) fn materialize_global_defaults(
    program: &StylesheetProgram,
    source: Option<&Document>,
    parameters: &BTreeMap<String, InvocationParameter>,
    request_id: &str,
    documents: &GlobalDocumentContext<'_>,
    control: &mut InvocationControl,
) -> Result<RuntimeGlobals, ExecutionFailure> {
    if program.global_bindings.is_empty() {
        return Ok(RuntimeGlobals::default());
    }
    let owner = global_resolution::GlobalInitialization::new(
        program, source, parameters, request_id, documents,
    );
    let mut globals = owner.initialize_all(control)?;
    if !globals.nodes.is_empty() {
        let source = source.ok_or_else(|| {
            failure(
                "FXRT1004",
                FailureCategory::Unsupported,
                Some(request_id),
                "source-node globals require an effective principal owner",
            )
        })?;
        globals.principal_owner = Some(
            source
                .retain_effective_owner(control)
                .map_err(|failure| control_failure(failure, request_id))?,
        );
    }
    Ok(globals)
}

fn materialize_global_default(
    globals: &impl GlobalInitializerReads,
    binding: &GlobalBinding,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    match &binding.default {
        GlobalBindingDefault::SharedConstructor { .. } => {
            unreachable!("shared constructors execute through the initialization owner")
        }
        GlobalBindingDefault::EmptySequence => Ok(InitializedGlobal::Empty),
        GlobalBindingDefault::Text(value) => Ok(InitializedGlobal::Atomic(AtomicValue::untyped(
            value.clone(),
        ))),
        GlobalBindingDefault::Atomic(value) => Ok(InitializedGlobal::Atomic(value.clone())),
        GlobalBindingDefault::Integer(value) => Ok(InitializedGlobal::Atomic(
            AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, value.to_string()),
        )),
        GlobalBindingDefault::DoubleDivision { .. } => {
            materialize_double_division(binding, source, request_id, control)
        }
        GlobalBindingDefault::CountLocationPath(path) => {
            materialize_global_count(path, source, request_id, control)
        }
        GlobalBindingDefault::LocationPath(path) => {
            materialize_global_location_path(path, source, request_id, control)
        }
        GlobalBindingDefault::Xslt10NumberLocationPath(path) => {
            materialize_global_number(path, source, request_id, control)
        }
        GlobalBindingDefault::SourceNodeIdentity(path) => {
            materialize_source_node_identity(path, source, request_id, control)
        }
        GlobalBindingDefault::Xslt10LiteralDocument(_) => unreachable!(),
        GlobalBindingDefault::Variable(name) => {
            materialize_global_alias(globals, name, request_id, control)
        }
        GlobalBindingDefault::SourceVariablePath { variable, path } => {
            materialize_global_source_variable_path(
                globals, variable, path, source, request_id, control,
            )
        }
        GlobalBindingDefault::Xslt10Concat(expression) => {
            materialize_global_xslt10_concat(globals, expression, source, request_id, control)
        }
        GlobalBindingDefault::TemporaryTree(nodes) => {
            materialize_temporary_nodes(nodes, source, request_id, control)
                .map(InitializedGlobal::TemporaryTree)
        }
        GlobalBindingDefault::TemporaryText(value) => materialize_parentless_temporary_node(
            TemporaryNodeKind::Text(value.clone()),
            request_id,
            control,
        )
        .map(InitializedGlobal::TemporaryTree),
        GlobalBindingDefault::Xslt10TemporarySourceString(_)
        | GlobalBindingDefault::Xslt10TemporarySourceCopy(_) => {
            materialize_global_xslt10_source_tree(binding, source, request_id, control)
        }
        GlobalBindingDefault::Xslt10TemporaryTextParts(parts) => {
            materialize_xslt10_temporary_text_parts(globals, parts, source, request_id, control)
                .map(InitializedGlobal::TemporaryTree)
        }
        GlobalBindingDefault::Xslt10ConditionalText {
            variable,
            expected,
            when_true,
            when_false,
        } => {
            let actual = global_string_value(globals, variable, source, request_id, control)?;
            let value = if actual == *expected {
                when_true
            } else {
                when_false
            };
            materialize_parentless_temporary_node(
                TemporaryNodeKind::Text(value.clone()),
                request_id,
                control,
            )
            .map(InitializedGlobal::TemporaryTree)
        }
        GlobalBindingDefault::Xslt10ForEachText(path) => {
            materialize_global_xslt10_for_each_text(path, source, request_id, control)
        }
        GlobalBindingDefault::TemporaryAttribute { .. }
        | GlobalBindingDefault::TemporaryComment(_)
        | GlobalBindingDefault::TemporaryProcessingInstruction { .. } => {
            materialize_parentless_global(binding, request_id, control)
        }
    }
}

fn materialize_global_literal_document(
    reference: &crate::xslt::golden_semantics_experiment::DocumentRootReference,
    request_id: &str,
    documents: &GlobalDocumentContext<'_>,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let document = super::super::dynamic_document::prepare_document_from_snapshot(
        documents.resource_snapshot,
        documents.denied_resources,
        documents.dynamic_documents,
        request_id,
        reference,
        control,
    )?;
    Ok(InitializedGlobal::LiteralDocument(document))
}

fn materialize_global_location_path(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "a source-dependent global binding requires a principal source",
        )
    })?;
    let nodes = evaluate_location_path_controlled(source, source.document_node(), path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(InitializedGlobal::SourceNodes(nodes))
}

fn materialize_global_source_variable_path(
    globals: &impl GlobalInitializerReads,
    variable: &str,
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "a variable-rooted global binding requires a principal source",
        )
    })?;
    globals.with_completed(variable, control, |globals, control| {
        let roots = globals.nodes.get(variable).ok_or_else(|| {
            failure(
                "XPTY0004",
                FailureCategory::Invalid,
                Some(request_id),
                format!("variable-rooted global binding requires source nodes: ${variable}"),
            )
        })?;
        let mut selected = Vec::new();
        for root in roots {
            selected.extend(
                evaluate_location_path_controlled(source, *root, path, control)
                    .map_err(|failure| control_failure(failure, request_id))?,
            );
        }
        selected.sort_unstable_by_key(|node| source.document_order(*node));
        selected.dedup();
        Ok(InitializedGlobal::SourceNodes(selected))
    })
}

fn materialize_global_xslt10_concat(
    globals: &impl GlobalInitializerReads,
    expression: &Xslt10ConcatExpression,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut value = String::new();
    for part in &expression.parts {
        match part {
            Xslt10ConcatPart::Literal(literal) => value.push_str(literal),
            Xslt10ConcatPart::Variable(name) => {
                value.push_str(&global_string_value(
                    globals, name, source, request_id, control,
                )?);
            }
            Xslt10ConcatPart::VariablePosition { .. }
            | Xslt10ConcatPart::Path(_)
            | Xslt10ConcatPart::GeneratedNodeIdentity(_)
            | Xslt10ConcatPart::QualifiedGeneratedIdentity(_)
            | Xslt10ConcatPart::PathSubstring(_)
            | Xslt10ConcatPart::ComposedString(_)
            | Xslt10ConcatPart::SumPath(_) => {
                unreachable!("global concat compilation admits literal and variable parts only")
            }
        }
    }
    Ok(InitializedGlobal::Atomic(
        AtomicValue::from_validated_lexical(BuiltinAtomicType::String, value),
    ))
}

fn materialize_global_number(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "an XSLT 1.0 numeric global requires a principal source",
        )
    })?;
    let nodes = evaluate_location_path_controlled(source, source.document_node(), path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    let lexical = if let Some(node) = nodes.first().copied() {
        source
            .string_value_controlled(node, control)
            .map_err(|failure| control_failure(failure, request_id))?
    } else {
        String::new()
    };
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let lexical = crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&lexical)
        .map_or_else(|| "NaN".to_owned(), xslt10_double_lexical);
    Ok(InitializedGlobal::Atomic(
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Double, lexical),
    ))
}

fn xslt10_double_lexical(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value == f64::INFINITY {
        "Infinity".to_owned()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".to_owned()
    } else if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

fn materialize_parentless_global(
    binding: &GlobalBinding,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let kind = match &binding.default {
        GlobalBindingDefault::TemporaryAttribute { name, value } => TemporaryNodeKind::Attribute {
            name: name.clone(),
            value: value.clone(),
        },
        GlobalBindingDefault::TemporaryComment(value) => TemporaryNodeKind::Comment(value.clone()),
        GlobalBindingDefault::TemporaryProcessingInstruction { target, value } => {
            TemporaryNodeKind::ProcessingInstruction {
                target: target.clone(),
                value: value.clone(),
            }
        }
        _ => unreachable!("parentless global dispatch receives one parentless node"),
    };
    let tree = materialize_parentless_temporary_node(kind, request_id, control)?;
    Ok(InitializedGlobal::TemporaryTree(tree))
}

fn materialize_global_xslt10_source_tree(
    binding: &GlobalBinding,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let tree = match &binding.default {
        GlobalBindingDefault::Xslt10TemporarySourceString(path) => {
            materialize_xslt10_temporary_source_string(path, source, request_id, control)?
        }
        GlobalBindingDefault::Xslt10TemporarySourceCopy(path) => {
            materialize_xslt10_temporary_source_copy(path, source, request_id, control)?
        }
        _ => unreachable!("source-tree dispatch receives only source-tree defaults"),
    };
    Ok(InitializedGlobal::TemporaryTree(tree))
}

fn materialize_xslt10_temporary_text_parts(
    globals: &impl GlobalInitializerReads,
    parts: &[crate::xslt::golden_semantics_experiment::Xslt10TemporaryTextPart],
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    use crate::xslt::golden_semantics_experiment::Xslt10TemporaryTextPart;

    let mut value = String::new();
    for part in parts {
        match part {
            Xslt10TemporaryTextPart::Text(text) => value.push_str(text),
            Xslt10TemporaryTextPart::Message(message) => {
                control
                    .charge(WorkDomain::XsltInstruction, 2)
                    .and_then(|()| control.charge(WorkDomain::ResultNode, 1))
                    .and_then(|()| control.charge(WorkDomain::ResultTextByte, message.len()))
                    .map_err(|failure| control_failure(failure, request_id))?;
                control.record_message(message.clone());
            }
            Xslt10TemporaryTextPart::Variable(name) => {
                value.push_str(&global_string_value(
                    globals, name, source, request_id, control,
                )?);
            }
            Xslt10TemporaryTextPart::SourcePath(path) => {
                value.push_str(&source_path_string_value(
                    path, source, request_id, control,
                )?);
            }
        }
    }
    materialize_parentless_temporary_node(TemporaryNodeKind::Text(value), request_id, control)
}

fn global_string_value(
    globals: &impl GlobalInitializerReads,
    name: &str,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    globals.with_completed(name, control, |globals, control| {
        if let Some(value) = globals.atomics.get(name) {
            return Ok(value.lexical().to_owned());
        }
        if globals.empty_sequences.contains(name) {
            return Ok(String::new());
        }
        if let Some(nodes) = globals.nodes.get(name) {
            let source = source.ok_or_else(|| {
                failure(
                    "FXRT1004",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    "a source-node global value requires a principal source",
                )
            })?;
            return nodes.first().map_or_else(
                || Ok(String::new()),
                |node| {
                    source
                        .string_value_controlled(*node, control)
                        .map_err(|failure| control_failure(failure, request_id))
                },
            );
        }
        if let Some(tree) = globals.temporary_trees.get(name) {
            return temporary_tree_string_value(tree, request_id, control);
        }
        Err(failure(
            "FXRT0002",
            FailureCategory::Invalid,
            Some(request_id),
            format!("unbound global dependency: ${name}"),
        ))
    })
}

fn materialize_global_count(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
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
    Ok(InitializedGlobal::Atomic(
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, nodes.len().to_string()),
    ))
}

fn materialize_global_alias(
    globals: &impl GlobalInitializerReads,
    dependency: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    globals.with_completed(dependency, control, |globals, _control| {
        if let Some(value) = globals.atomics.get(dependency).cloned() {
            return Ok(InitializedGlobal::Atomic(value));
        }
        if let Some(nodes) = globals.nodes.get(dependency).cloned() {
            return Ok(InitializedGlobal::SourceNodes(nodes));
        }
        Err(failure(
            "FXRT0002",
            FailureCategory::Invalid,
            Some(request_id),
            format!("unbound global dependency: ${dependency}"),
        ))
    })
}

fn materialize_xslt10_temporary_source_string(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<TemporaryTree, ExecutionFailure> {
    let value = source_path_string_value(path, source, request_id, control)?;
    materialize_parentless_temporary_node(TemporaryNodeKind::Text(value), request_id, control)
}

fn materialize_xslt10_temporary_source_copy(
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
            "an XSLT 1.0 source-copy temporary tree requires a principal source",
        )
    })?;
    let selected = evaluate_location_path_controlled(source, source.document_node(), path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut tree = TemporaryTree {
        identity: allocate_temporary_tree_identity(control, request_id)?,
        roots: Vec::new(),
        nodes: Vec::new(),
        xslt10_disable_output_escaping_text: HashSet::new(),
    };
    for selected in selected {
        append_source_copy_roots(source, selected, &mut tree, request_id, control)?;
    }
    Ok(tree)
}

fn append_source_copy_roots(
    source: &Document,
    node: NodeId,
    tree: &mut TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if source.kind(node) == crate::xdm::owned_tree_experiment::NodeKind::Document {
        for child in source.children(node) {
            let copied = copy_source_to_temporary(source, *child, None, tree, request_id, control)?;
            tree.roots.push(copied);
        }
        return Ok(());
    }
    let copied = copy_source_to_temporary(source, node, None, tree, request_id, control)?;
    tree.roots.push(copied);
    Ok(())
}

fn copy_source_to_temporary(
    source: &Document,
    node: NodeId,
    parent: Option<usize>,
    tree: &mut TemporaryTree,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    use crate::xdm::owned_tree_experiment::NodeKind;

    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let copied = tree.nodes.len();
    let kind = match source.kind(node) {
        NodeKind::Element => TemporaryNodeKind::Element {
            name: source
                .name(node)
                .expect("source element has a name")
                .clone(),
            namespaces: source.in_scope_namespaces(node),
            attributes: Vec::new(),
        },
        NodeKind::Attribute => TemporaryNodeKind::Attribute {
            name: source
                .name(node)
                .expect("source attribute has a name")
                .clone(),
            value: source.value(node).unwrap_or_default().to_owned(),
        },
        NodeKind::Text => {
            TemporaryNodeKind::Text(source.value(node).unwrap_or_default().to_owned())
        }
        NodeKind::Comment => {
            TemporaryNodeKind::Comment(source.value(node).unwrap_or_default().to_owned())
        }
        NodeKind::ProcessingInstruction => TemporaryNodeKind::ProcessingInstruction {
            target: source
                .name(node)
                .expect("source processing instruction has a target")
                .local
                .clone(),
            value: source.value(node).unwrap_or_default().to_owned(),
        },
        NodeKind::Document => unreachable!("document copies are expanded at the root boundary"),
    };
    tree.nodes.push(TemporaryNode {
        kind,
        parent,
        children: Vec::new(),
    });
    if source.kind(node) == NodeKind::Element {
        let mut attributes = Vec::new();
        for attribute in source.attributes(node) {
            attributes.push(copy_source_to_temporary(
                source,
                *attribute,
                Some(copied),
                tree,
                request_id,
                control,
            )?);
        }
        let TemporaryNodeKind::Element {
            attributes: retained,
            ..
        } = &mut tree.nodes[copied].kind
        else {
            unreachable!("source element copy retains an element kind")
        };
        *retained = attributes;
        for child in source.children(node) {
            let child =
                copy_source_to_temporary(source, *child, Some(copied), tree, request_id, control)?;
            tree.nodes[copied].children.push(child);
        }
    }
    Ok(copied)
}

fn materialize_global_xslt10_for_each_text(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let source = source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "an XSLT 1.0 for-each temporary tree requires a principal source",
        )
    })?;
    let tree = materialize_xslt10_for_each_text_tree(
        source,
        source.document_node(),
        path,
        request_id,
        control,
    )?;
    Ok(InitializedGlobal::TemporaryTree(tree))
}

fn materialize_double_division(
    binding: &GlobalBinding,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
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
        return Ok(InitializedGlobal::Empty);
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
    Ok(InitializedGlobal::Atomic(
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Double, lexical),
    ))
}

fn materialize_source_node_identity(
    path: &crate::xpath::path_experiment::LocationPath,
    source: Option<&Document>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
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
    Ok(InitializedGlobal::Atomic(AtomicValue::string(
        source_node_identity(*node),
    )))
}
