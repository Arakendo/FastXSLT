//! Private compiled-template selection and source-pattern evaluation.

use std::{cell::RefCell, collections::BTreeMap, mem::size_of};

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xpath::path_experiment::{PathStep, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::{
    ChildPresenceTest, MatchNodeTest, MatchPattern, MatchStringPredicate, MatchedTemplate,
    NamedSiblingBoundary, StylesheetProgram,
};

use super::MultipleMatchPolicy;
use super::match_sequence_predicate::{
    context_number_greater_than_variable, evaluate as evaluate_sequence_predicate,
    required_attribute,
};
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
    pub(super) document_rooted_matches: &'a RefCell<DocumentRootedMatchCache>,
}

const MAX_DOCUMENT_ROOTED_MATCH_CACHE_BYTES: usize = 1_048_576;
const MAX_DOCUMENT_ROOTED_MATCH_CACHE_ENTRIES: usize = 1_024;

#[derive(Debug, Default)]
pub(super) struct DocumentRootedMatchCache {
    memberships: BTreeMap<usize, Box<[u64]>>,
    retained_membership_bytes: usize,
}

impl DocumentRootedMatchCache {
    fn lookup(&self, template_index: usize, node: NodeId) -> Option<bool> {
        self.memberships.get(&template_index).map(|membership| {
            let index = node.index();
            membership[index / u64::BITS as usize] & (1_u64 << (index % u64::BITS as usize)) != 0
        })
    }

    fn insert(
        &mut self,
        template_index: usize,
        node_count: usize,
        selected: &[NodeId],
    ) -> Option<usize> {
        let word_count = node_count.div_ceil(u64::BITS as usize);
        let retained_bytes = word_count * size_of::<u64>();
        if self.memberships.len() >= MAX_DOCUMENT_ROOTED_MATCH_CACHE_ENTRIES
            || retained_bytes
                > MAX_DOCUMENT_ROOTED_MATCH_CACHE_BYTES
                    .saturating_sub(self.retained_membership_bytes)
        {
            return None;
        }
        let mut membership = vec![0_u64; word_count].into_boxed_slice();
        for node in selected {
            let index = node.index();
            membership[index / u64::BITS as usize] |= 1_u64 << (index % u64::BITS as usize);
        }
        self.memberships.insert(template_index, membership);
        self.retained_membership_bytes += retained_bytes;
        Some(retained_bytes)
    }
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
        control
            .charge_template_candidate()
            .map_err(|failure| control_failure(failure, selection.request_id))?;
        if !accepts_mode(&template.modes, selection.mode)
            || !matches_pattern(index, &template.pattern, selection, control)?
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
        control
            .charge_template_candidate()
            .map_err(|failure| control_failure(failure, selection.request_id))?;
        let rank = (template.import_precedence, template.priority, index);
        let lower_rank = rank < current_rank;
        if !lower_rank
            || !accepts_mode(&template.modes, selection.mode)
            || !matches_pattern(index, &template.pattern, selection, control)?
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
        control
            .charge_template_candidate()
            .map_err(|failure| control_failure(failure, selection.request_id))?;
        if template.import_precedence >= current_precedence
            || !accepts_mode(&template.modes, selection.mode)
            || !matches_pattern(index, &template.pattern, selection, control)?
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
            || (matches!(candidate.as_str(), "#default" | "#unnamed") && mode.is_none())
            || mode.is_some_and(|requested| candidate == requested)
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "the exhaustive private pattern dispatch remains one semantic ownership point"
)]
fn matches_pattern(
    template_index: usize,
    pattern: &MatchPattern,
    selection: &TemplateSelectionContext<'_>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let source = selection.source;
    let node = selection.node;
    let variables = selection.variables;
    let request_id = selection.request_id;
    match pattern {
        MatchPattern::AtomicIntegerGreaterOrEqual(_) => Ok(false),
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
        MatchPattern::AnyElementWithAttribute(attribute) => {
            if source.kind(node) != NodeKind::Element {
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
        MatchPattern::AnyElementWithAttributeValue { attribute, value } => {
            if source.kind(node) != NodeKind::Element {
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
        MatchPattern::AnyElementNumberEquals(expected) => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            Ok(source.kind(node) == NodeKind::Element
                && crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(
                    &source.string_value(node),
                ) == Some(f64::from(*expected)))
        }
        MatchPattern::AnyElementWithAttributeNumberEquals { attribute, value } => {
            if source.kind(node) != NodeKind::Element {
                return Ok(false);
            }
            for candidate in source.attributes(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                if source.name(*candidate) == Some(attribute) {
                    control
                        .charge(WorkDomain::XPathOperation, 1)
                        .map_err(|failure| control_failure(failure, request_id))?;
                    if crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(
                        &source.string_value(*candidate),
                    ) == Some(f64::from(*value))
                    {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        MatchPattern::NodeStringPredicate {
            node_test,
            predicate,
        } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            Ok(source_node_test_matches(source, node, node_test)
                && match_string_predicate(&source.string_value(node), predicate))
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
        MatchPattern::ElementWithTwoAttributeValues {
            element,
            first_attribute,
            first_value,
            second_attribute,
            second_value,
        } => {
            if source.name(node) != Some(element) {
                return Ok(false);
            }
            let mut first_matches = false;
            let mut second_matches = false;
            for candidate in source.attributes(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                let name = source.name(*candidate);
                let value = source.string_value(*candidate);
                first_matches |= name == Some(first_attribute) && value == *first_value;
                second_matches |= name == Some(second_attribute) && value == *second_value;
            }
            Ok(first_matches && second_matches)
        }
        MatchPattern::ElementWithChild { element, child } => {
            if source.name(node) != Some(element) {
                return Ok(false);
            }
            for candidate in source.children(node) {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                let matches = match child {
                    ChildPresenceTest::Element(name) => source.name(*candidate) == Some(name),
                    ChildPresenceTest::Text => source.kind(*candidate) == NodeKind::Text,
                };
                if matches {
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
        MatchPattern::ElementNumberGreaterThanVariable { element, variable } => {
            if source.name(node) != Some(element) {
                return Ok(false);
            }
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            Ok(variables.get(variable).is_some_and(|variable| {
                context_number_greater_than_variable(&source.string_value(node), variable)
            }))
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
        MatchPattern::ElementAtNamedSiblingWithAttributeValue {
            element,
            position,
            attribute,
            value,
            attribute_filters_position,
        } => matches_named_sibling_attribute_value(
            source,
            node,
            element,
            *position,
            attribute,
            value,
            *attribute_filters_position,
            request_id,
            control,
        ),
        MatchPattern::ElementWithSequentialPredicates {
            element,
            predicates,
        } => matches_sequential_predicates(source, node, element, predicates, request_id, control),
        MatchPattern::Path(path) => match_path_pattern(
            source,
            node,
            template_index,
            path,
            selection.document_rooted_matches,
            request_id,
            control,
        ),
        MatchPattern::QualifiedElementPathAlternatives(alternatives) => {
            matches_qualified_path_alternatives(selection, alternatives, control)
        }
        MatchPattern::UnionAlternatives(alternatives) => {
            for alternative in alternatives {
                if matches_pattern(template_index, alternative, selection, control)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MatchPattern::Attribute(name) => {
            Ok(source.kind(node) == NodeKind::Attribute && source.name(node) == Some(name))
        }
        MatchPattern::AttributeNamespace(namespace) => Ok(source.kind(node) == NodeKind::Attribute
            && source
                .name(node)
                .is_some_and(|name| name.namespace.as_deref() == Some(namespace.as_str()))),
        MatchPattern::AttributeNameEquals(expected) => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            Ok(source.kind(node) == NodeKind::Attribute
                && source.prefix(node).is_none()
                && source
                    .name(node)
                    .is_some_and(|name| name.namespace.is_none() && name.local == *expected))
        }
        MatchPattern::AnyAttribute => Ok(source.kind(node) == NodeKind::Attribute),
        MatchPattern::Comment => Ok(source.kind(node) == NodeKind::Comment),
        MatchPattern::Text => Ok(source.kind(node) == NodeKind::Text),
        MatchPattern::ProcessingInstruction => {
            Ok(source.kind(node) == NodeKind::ProcessingInstruction)
        }
        MatchPattern::ProcessingInstructionNamed(required) => Ok(source.kind(node)
            == NodeKind::ProcessingInstruction
            && source
                .name(node)
                .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())),
        MatchPattern::AnyNode => Ok(matches_any_node(source.kind(node))),
    }
}

fn matches_sequential_predicates(
    source: &Document,
    node: NodeId,
    element: &crate::xml::quick_xml_experiment::ExpandedName,
    predicates: &[crate::xslt::golden_semantics_experiment::MatchSequencePredicate],
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if source.name(node) != Some(element) {
        return Ok(false);
    }
    let Some(parent) = source.parent(node) else {
        return Ok(false);
    };
    let mut candidates = Vec::new();
    for sibling in source.children(parent) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.name(*sibling) == Some(element) {
            candidates.push(*sibling);
        }
    }
    for predicate in predicates {
        let size = candidates.len();
        let mut filtered = Vec::with_capacity(size);
        for (offset, candidate) in candidates.into_iter().enumerate() {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let attribute_value = if let Some(required) = required_attribute(predicate) {
                let mut value = None;
                for attribute in source.attributes(candidate) {
                    control
                        .charge(WorkDomain::XPathNodeVisit, 1)
                        .map_err(|failure| control_failure(failure, request_id))?;
                    if source.name(*attribute) == Some(required) {
                        value = Some(source.string_value(*attribute));
                        break;
                    }
                }
                value
            } else {
                None
            };
            if evaluate_sequence_predicate(predicate, offset + 1, size, attribute_value.as_deref())
            {
                filtered.push(candidate);
            }
        }
        candidates = filtered;
    }
    Ok(candidates.contains(&node))
}

fn match_string_predicate(value: &str, predicate: &MatchStringPredicate) -> bool {
    match predicate {
        MatchStringPredicate::Equals(expected) => value == expected,
        MatchStringPredicate::NotEquals(expected) => value != expected,
        MatchStringPredicate::EqualsEither(left, right) => value == left || value == right,
        MatchStringPredicate::Contains(expected) => value.contains(expected),
    }
}

fn source_node_test_matches(source: &Document, node: NodeId, node_test: &MatchNodeTest) -> bool {
    match node_test {
        MatchNodeTest::Element(name) => {
            source.kind(node) == NodeKind::Element && source.name(node) == Some(name)
        }
        MatchNodeTest::Text => source.kind(node) == NodeKind::Text,
        MatchNodeTest::Comment => source.kind(node) == NodeKind::Comment,
        MatchNodeTest::ProcessingInstruction(target) => {
            source.kind(node) == NodeKind::ProcessingInstruction
                && target.as_ref().is_none_or(|target| {
                    source
                        .name(node)
                        .is_some_and(|name| name.namespace.is_none() && name.local == *target)
                })
        }
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
    let mut named_position = 0usize;
    let mut candidate_position = None;
    for sibling in source.children(parent).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.name(sibling) == Some(element) {
            named_position += 1;
        }
        if sibling == node {
            reached_candidate = true;
            candidate_position = Some(named_position);
        } else if reached_candidate && source.name(sibling) == Some(element) {
            later_match = true;
        }
    }
    Ok(match boundary {
        NamedSiblingBoundary::Exact(expected) => candidate_position == Some(expected),
        NamedSiblingBoundary::Before(expected) => candidate_position.is_some_and(|p| p < expected),
        NamedSiblingBoundary::BeforeLast => later_match,
        NamedSiblingBoundary::Last => !later_match,
    })
}

#[allow(clippy::too_many_arguments)]
fn matches_named_sibling_attribute_value(
    source: &Document,
    node: NodeId,
    element: &crate::xml::quick_xml_experiment::ExpandedName,
    position: usize,
    attribute: &crate::xml::quick_xml_experiment::ExpandedName,
    value: &str,
    attribute_filters_position: bool,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if source.name(node) != Some(element)
        || !source_has_attribute_value(source, node, attribute, value, request_id, control)?
    {
        return Ok(false);
    }
    let Some(parent) = source.parent(node) else {
        return Ok(false);
    };
    let mut filtered_position = 0usize;
    for sibling in source.children(parent).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.name(sibling) != Some(element) {
            continue;
        }
        if !attribute_filters_position
            || source_has_attribute_value(source, sibling, attribute, value, request_id, control)?
        {
            filtered_position += 1;
        }
        if sibling == node {
            return Ok(filtered_position == position);
        }
    }
    Ok(false)
}

fn source_has_attribute_value(
    source: &Document,
    node: NodeId,
    required_name: &crate::xml::quick_xml_experiment::ExpandedName,
    required_value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    for attribute in source.attributes(node) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        if source.name(*attribute) == Some(required_name)
            && source.string_value(*attribute) == required_value
        {
            return Ok(true);
        }
    }
    Ok(false)
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
    template_index: usize,
    path: &crate::xpath::path_experiment::LocationPath,
    document_rooted_matches: &RefCell<DocumentRootedMatchCache>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if path.starts_at_document_node() {
        if control.document_rooted_match_cache_enabled()
            && let Some(matches) = document_rooted_matches
                .borrow()
                .lookup(template_index, node)
        {
            control.observe_document_rooted_match_cache_hit();
            return Ok(matches);
        }
        control.observe_document_rooted_match_evaluation();
        let selected =
            evaluate_location_path_controlled(source, source.document_node(), path, control)
                .map_err(|failure| control_failure(failure, request_id))?;
        let matches = selected.contains(&node);
        if control.document_rooted_match_cache_enabled()
            && let Some(retained_bytes) = document_rooted_matches.borrow_mut().insert(
                template_index,
                source.node_count(),
                &selected,
            )
        {
            control.observe_document_rooted_match_cache_build(retained_bytes);
        }
        return Ok(matches);
    }
    let descendant_pair = match path.steps.as_slice() {
        [
            PathStep::ChildNamed(ancestor),
            PathStep::DescendantNamed(candidate),
        ]
        | [
            PathStep::ChildNamed(ancestor),
            PathStep::DescendantOrSelfAnyNode,
            PathStep::ChildNamed(candidate),
        ] => Some((ancestor, candidate)),
        _ => None,
    };
    if let Some((ancestor, candidate)) = descendant_pair {
        if source.kind(node) != NodeKind::Element
            || source
                .name(node)
                .is_none_or(|name| name.namespace.is_some() || name.local != *candidate)
        {
            return Ok(false);
        }
        let mut current = source.parent(node);
        while let Some(parent) = current {
            control
                .charge(WorkDomain::XPathNodeVisit, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            if source.kind(parent) == NodeKind::Element
                && source
                    .name(parent)
                    .is_some_and(|name| name.namespace.is_none() && name.local == *ancestor)
            {
                return Ok(true);
            }
            current = source.parent(parent);
        }
        return Ok(false);
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

#[cfg(test)]
mod cache_tests {
    use super::{
        DocumentRootedMatchCache, MAX_DOCUMENT_ROOTED_MATCH_CACHE_BYTES,
        MAX_DOCUMENT_ROOTED_MATCH_CACHE_ENTRIES,
    };

    #[test]
    fn cache_refuses_membership_beyond_its_byte_ceiling_without_mutation() {
        let mut cache = DocumentRootedMatchCache::default();
        let oversized_nodes =
            (MAX_DOCUMENT_ROOTED_MATCH_CACHE_BYTES / size_of::<u64>() + 1) * u64::BITS as usize;

        assert_eq!(cache.insert(0, oversized_nodes, &[]), None);
        assert!(cache.memberships.is_empty());
        assert_eq!(cache.retained_membership_bytes, 0);
    }

    #[test]
    fn cache_refuses_entries_beyond_its_count_ceiling() {
        let mut cache = DocumentRootedMatchCache::default();
        for index in 0..MAX_DOCUMENT_ROOTED_MATCH_CACHE_ENTRIES {
            assert_eq!(cache.insert(index, 0, &[]), Some(0));
        }

        assert_eq!(
            cache.insert(MAX_DOCUMENT_ROOTED_MATCH_CACHE_ENTRIES, 0, &[]),
            None
        );
        assert_eq!(
            cache.memberships.len(),
            MAX_DOCUMENT_ROOTED_MATCH_CACHE_ENTRIES
        );
    }
}
