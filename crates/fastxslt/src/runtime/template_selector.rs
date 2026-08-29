//! Private compiled-template selection and source-pattern evaluation.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{MatchPattern, MatchedTemplate, StylesheetProgram};

use super::runtime_failure::{ExecutionFailure, control_failure};

pub(super) fn select_template_with_index<'a>(
    program: &'a StylesheetProgram,
    source: &Document,
    node: NodeId,
    mode: Option<&str>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let mut selected_template = None;
    let mut selected_priority = None;
    for (index, template) in program.matched_templates.iter().enumerate() {
        if !accepts_mode(&template.modes, mode)
            || !matches_pattern(&template.pattern, source, node, request_id, control)?
        {
            continue;
        }
        if selected_priority.is_none_or(|priority| template.priority >= priority) {
            selected_template = Some((index, template));
            selected_priority = Some(template.priority);
        }
    }
    Ok(selected_template)
}

pub(super) fn select_next_template<'a>(
    program: &'a StylesheetProgram,
    source: &Document,
    node: NodeId,
    mode: Option<&str>,
    current_index: usize,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let current_priority = program.matched_templates[current_index].priority;
    let mut selected_template = None;
    let mut selected_priority = None;
    for (index, template) in program.matched_templates.iter().enumerate() {
        let lower_rank = template.priority < current_priority
            || (template.priority == current_priority && index < current_index);
        if !lower_rank
            || !accepts_mode(&template.modes, mode)
            || !matches_pattern(&template.pattern, source, node, request_id, control)?
        {
            continue;
        }
        if selected_priority.is_none_or(|priority| template.priority >= priority) {
            selected_template = Some((index, template));
            selected_priority = Some(template.priority);
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
    source: &Document,
    node: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
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
        MatchPattern::Path(path) => match_path_pattern(source, node, path, request_id, control),
        MatchPattern::Attribute(name) => {
            Ok(source.kind(node) == NodeKind::Attribute && source.name(node) == Some(name))
        }
        MatchPattern::Comment => Ok(source.kind(node) == NodeKind::Comment),
        MatchPattern::Text => Ok(source.kind(node) == NodeKind::Text),
        MatchPattern::ProcessingInstruction => {
            Ok(source.kind(node) == NodeKind::ProcessingInstruction)
        }
        MatchPattern::AnyNode => Ok(matches!(
            source.kind(node),
            NodeKind::Element
                | NodeKind::Text
                | NodeKind::Comment
                | NodeKind::ProcessingInstruction
        )),
    }
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
