//! Private compiled-template selection and source-pattern evaluation.

use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    MatchPattern, MatchedTemplate, NamedSiblingBoundary, StylesheetProgram,
};

use super::MultipleMatchPolicy;
use super::runtime_failure::{
    ExecutionFailure, FailureCategory, control_failure, failure, failure_at,
};
use super::variable_filtered_path::{attribute_equals_atomic, matches as matches_variable_path};

pub(super) struct TemplateSelectionContext<'a> {
    pub(super) source: &'a Document,
    pub(super) node: NodeId,
    pub(super) mode: Option<&'a str>,
    pub(super) variables: &'a BTreeMap<String, AtomicValue>,
    pub(super) request_id: &'a str,
}

pub(super) fn select_template_with_index<'a>(
    program: &'a StylesheetProgram,
    selection: &TemplateSelectionContext<'_>,
    multiple_match_policy: MultipleMatchPolicy,
    control: &mut InvocationControl,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let mut selected_template = None;
    let mut selected_semantic_rank = None;
    let mut top_rank_is_ambiguous = false;
    for (index, template) in program.matched_templates.iter().enumerate() {
        if !accepts_mode(&template.modes, selection.mode)
            || !matches_pattern(&template.pattern, selection, control)?
        {
            continue;
        }
        let semantic_rank = (template.import_precedence, template.priority);
        if selected_semantic_rank.is_none_or(|selected| semantic_rank > selected) {
            selected_template = Some((index, template));
            selected_semantic_rank = Some(semantic_rank);
            top_rank_is_ambiguous = false;
        } else if selected_semantic_rank == Some(semantic_rank) {
            selected_template = Some((index, template));
            top_rank_is_ambiguous = true;
        }
    }
    if multiple_match_policy == MultipleMatchPolicy::Error && top_rank_is_ambiguous {
        let (_, selected) = selected_template.expect("an ambiguous top rank has a template");
        return Err(failure_at(
            "XTDE0540",
            FailureCategory::Invalid,
            Some(selection.request_id),
            selected.template.location.clone(),
            "more than one template rule matches at the highest import precedence and priority",
        ));
    }
    Ok(selected_template)
}

pub(super) fn select_next_template<'a>(
    program: &'a StylesheetProgram,
    selection: &TemplateSelectionContext<'_>,
    current_index: usize,
    multiple_match_policy: MultipleMatchPolicy,
    control: &mut InvocationControl,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let current = &program.matched_templates[current_index];
    let current_rank = (current.import_precedence, current.priority, current_index);
    let mut selected_template = None;
    let mut selected_semantic_rank = None;
    let mut top_rank_is_ambiguous = false;
    for (index, template) in program.matched_templates.iter().enumerate() {
        let rank = (template.import_precedence, template.priority, index);
        let lower_rank = rank < current_rank;
        if !lower_rank
            || !accepts_mode(&template.modes, selection.mode)
            || !matches_pattern(&template.pattern, selection, control)?
        {
            continue;
        }
        let semantic_rank = (template.import_precedence, template.priority);
        if selected_semantic_rank.is_none_or(|selected| semantic_rank > selected) {
            selected_template = Some((index, template));
            selected_semantic_rank = Some(semantic_rank);
            top_rank_is_ambiguous = false;
        } else if selected_semantic_rank == Some(semantic_rank) {
            selected_template = Some((index, template));
            top_rank_is_ambiguous = true;
        }
    }
    if multiple_match_policy == MultipleMatchPolicy::Error && top_rank_is_ambiguous {
        let (_, selected) = selected_template.expect("an ambiguous top rank has a template");
        return Err(failure_at(
            "XTDE0540",
            FailureCategory::Invalid,
            Some(selection.request_id),
            selected.template.location.clone(),
            "more than one next-match template rule matches at the highest eligible import precedence and priority",
        ));
    }
    Ok(selected_template)
}

pub(super) fn select_imported_template<'a>(
    program: &'a StylesheetProgram,
    selection: &TemplateSelectionContext<'_>,
    current_index: usize,
    control: &mut InvocationControl,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let current_precedence = program.matched_templates[current_index].import_precedence;
    let mut selected_template = None;
    let mut selected_rank = None;
    for (index, template) in program.matched_templates.iter().enumerate() {
        if template.import_precedence >= current_precedence
            || !accepts_mode(&template.modes, selection.mode)
            || !matches_pattern(&template.pattern, selection, control)?
        {
            continue;
        }
        let rank = (template.import_precedence, template.priority, index);
        if selected_rank.is_none_or(|selected| rank >= selected) {
            selected_template = Some((index, template));
            selected_rank = Some(rank);
        }
    }
    Ok(selected_template)
}

pub(super) fn accepts_mode(modes: &[String], mode: Option<&str>) -> bool {
    if modes.is_empty() {
        return mode.is_none();
    }
    modes.iter().any(|candidate| {
        candidate == "#all"
            || (candidate == "#default" && mode.is_none())
            || mode.is_some_and(|requested| candidate == requested)
    })
}

fn matches_pattern(
    pattern: &MatchPattern,
    selection: &TemplateSelectionContext<'_>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let source = selection.source;
    let node = selection.node;
    let variables = selection.variables;
    let request_id = selection.request_id;
    match pattern {
        MatchPattern::Document => Ok(source.kind(node) == NodeKind::Document),
        MatchPattern::DocumentElement(required) => {
            matches_document_element(source, node, required.as_ref(), request_id, control)
        }
        MatchPattern::Element(name) => Ok(source.name(node) == Some(name)),
        MatchPattern::ElementLocal(local) => Ok(source
            .name(node)
            .is_some_and(|name| name.local == local.as_str())),
        MatchPattern::ElementNamespace(namespace) => Ok(source
            .name(node)
            .is_some_and(|name| name.namespace.as_deref() == Some(namespace.as_str()))),
        MatchPattern::DescendantAnyElement | MatchPattern::AnyElement => {
            Ok(source.kind(node) == NodeKind::Element)
        }
        MatchPattern::ElementWithAttribute { element, attribute } => {
            if source.name(node) != Some(element) {
                return Ok(false);
            }
            for candidate in source.attributes(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                if source.name(*candidate) == Some(attribute) {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MatchPattern::ElementWithAttributeValue {
            element,
            attribute,
            value,
        } => {
            if source.name(node) != Some(element) {
                return Ok(false);
            }
            for candidate in source.attributes(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                if source.name(*candidate) == Some(attribute)
                    && source.string_value(*candidate) == value.as_str()
                {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MatchPattern::AnyElementWithAttributeVariable {
            attribute,
            variable,
        } => {
            if source.kind(node) != NodeKind::Element {
                return Ok(false);
            }
            let Some(value) = variables.get(variable) else {
                return Ok(false);
            };
            for candidate in source.attributes(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                if source.name(*candidate) == Some(attribute)
                    && attribute_equals_atomic(&source.string_value(*candidate), value)
                {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MatchPattern::VariableFilteredElementPath(path) => {
            matches_variable_path(source, node, path, variables, request_id, control)
        }
        MatchPattern::ElementWithSameNamedChild
        | MatchPattern::ElementWithSameNamedParent
        | MatchPattern::ElementWithSameNamedParentAtPosition(_) => {
            matches_name_relation(pattern, source, node, request_id, control)
        }
        MatchPattern::ElementAtNamedSiblingBoundary { element, boundary } => {
            matches_named_sibling_boundary(source, node, element, *boundary, request_id, control)
        }
        MatchPattern::Path(path) => match_path_pattern(source, node, path, request_id, control),
        MatchPattern::QualifiedElementPathAlternatives(alternatives) => {
            matches_qualified_path_alternatives(selection, alternatives, control)
        }
        MatchPattern::Attribute(name) => {
            Ok(source.kind(node) == NodeKind::Attribute && source.name(node) == Some(name))
        }
        MatchPattern::Comment => Ok(source.kind(node) == NodeKind::Comment),
        MatchPattern::Text => Ok(source.kind(node) == NodeKind::Text),
        MatchPattern::ProcessingInstruction => {
            Ok(source.kind(node) == NodeKind::ProcessingInstruction)
        }
        MatchPattern::AnyNode => Ok(matches_any_node(source.kind(node))),
    }
}

fn matches_qualified_path_alternatives(
    selection: &TemplateSelectionContext<'_>,
    alternatives: &[Vec<crate::xml::quick_xml_experiment::ExpandedName>],
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let TemplateSelectionContext {
        source,
        node,
        request_id,
        ..
    } = selection;
    for path in alternatives {
        let mut current = Some(*node);
        let mut matches = true;
        for expected in path.iter().rev() {
            let Some(candidate) = current else {
                matches = false;
                break;
            };
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            if source.name(candidate) != Some(expected) {
                matches = false;
                break;
            }
            current = source.parent(candidate);
        }
        if matches {
            return Ok(true);
        }
    }
    Ok(false)
}

fn matches_any_node(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Element | NodeKind::Text | NodeKind::Comment | NodeKind::ProcessingInstruction
    )
}

fn matches_named_sibling_boundary(
    source: &Document,
    node: NodeId,
    element: &crate::xml::quick_xml_experiment::ExpandedName,
    boundary: NamedSiblingBoundary,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if source.name(node) != Some(element) {
        return Ok(false);
    }
    let Some(parent) = source.parent(node) else {
        return Ok(false);
    };
    let mut reached_candidate = false;
    let mut later_match = false;
    for sibling in source.children(parent).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if sibling == node {
            reached_candidate = true;
        } else if reached_candidate && source.name(sibling) == Some(element) {
            later_match = true;
        }
    }
    Ok(match boundary {
        NamedSiblingBoundary::BeforeLast => later_match,
        NamedSiblingBoundary::Last => !later_match,
    })
}

fn matches_name_relation(
    pattern: &MatchPattern,
    source: &Document,
    node: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    match pattern {
        MatchPattern::ElementWithSameNamedChild => {
            matches_same_named_child(source, node, request_id, control)
        }
        MatchPattern::ElementWithSameNamedParent => {
            matches_same_named_parent(source, node, request_id, control)
        }
        MatchPattern::ElementWithSameNamedParentAtPosition(position) => {
            matches_same_named_parent_at_position(source, node, *position, request_id, control)
        }
        _ => unreachable!("matches_name_relation receives a name relation"),
    }
}

fn matches_same_named_child(
    source: &Document,
    node: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if source.kind(node) != NodeKind::Element {
        return Ok(false);
    }
    let parent_name = source
        .name(node)
        .expect("element pattern candidate has a name");
    if parent_name.namespace.is_some() {
        return Err(unsupported_name_comparison(request_id));
    }
    for child in source.children(node) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.kind(*child) != NodeKind::Element {
            continue;
        }
        let child_name = source.name(*child).expect("element child has a name");
        if child_name.namespace.is_some() {
            return Err(unsupported_name_comparison(request_id));
        }
        if child_name.local == parent_name.local {
            return Ok(true);
        }
    }
    Ok(false)
}

fn matches_same_named_parent(
    source: &Document,
    node: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if source.kind(node) != NodeKind::Element {
        return Ok(false);
    }
    let Some(parent) = source.parent(node) else {
        return Ok(false);
    };
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    if source.kind(parent) != NodeKind::Element {
        return Ok(false);
    }
    let node_name = source.name(node).expect("element candidate has a name");
    let parent_name = source.name(parent).expect("element parent has a name");
    if node_name.namespace.is_some() || parent_name.namespace.is_some() {
        return Err(unsupported_name_comparison(request_id));
    }
    Ok(node_name.local == parent_name.local)
}

fn matches_same_named_parent_at_position(
    source: &Document,
    node: NodeId,
    required_position: usize,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if !matches_same_named_parent(source, node, request_id, control)? {
        return Ok(false);
    }
    let parent = source
        .parent(node)
        .expect("same-named candidate has a parent");
    let Some(grandparent) = source.parent(parent) else {
        return Ok(false);
    };
    let candidate_name = source.name(node).expect("element candidate has a name");
    let mut filtered_position = 0;
    for sibling in source.children(grandparent) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.kind(*sibling) != NodeKind::Element {
            continue;
        }
        let sibling_name = source.name(*sibling).expect("element sibling has a name");
        if sibling_name.namespace.is_some() {
            return Err(unsupported_name_comparison(request_id));
        }
        if sibling_name.local == candidate_name.local {
            filtered_position += 1;
        }
        if *sibling == parent {
            return Ok(filtered_position == required_position);
        }
    }
    Ok(false)
}

fn unsupported_name_comparison(request_id: &str) -> ExecutionFailure {
    failure(
        "FXRT1013",
        FailureCategory::Unsupported,
        Some(request_id),
        "name() pattern comparison requires unnamespaced elements in the private slice",
    )
}

fn matches_document_element(
    source: &Document,
    node: NodeId,
    required: Option<&crate::xml::quick_xml_experiment::ExpandedName>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if source.kind(node) != NodeKind::Document {
        return Ok(false);
    }
    for child in source.children(node) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.kind(*child) == NodeKind::Element {
            return Ok(required.is_none_or(|name| source.name(*child) == Some(name)));
        }
    }
    Ok(false)
}

fn match_path_pattern(
    source: &Document,
    node: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if path.starts_at_document_node() {
        return evaluate_location_path_controlled(source, source.document_node(), path, control)
            .map(|selected| selected.contains(&node))
            .map_err(|failure| control_failure(failure, request_id));
    }
    let mut first_step = node;
    for _ in 1..path.steps.len() {
        let Some(parent) = source.parent(first_step) else {
            return Ok(false);
        };
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        first_step = parent;
    }
    let Some(context) = source.parent(first_step) else {
        return Ok(false);
    };
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    evaluate_location_path_controlled(source, context, path, control)
        .map(|selected| selected.contains(&node))
        .map_err(|failure| control_failure(failure, request_id))
}
