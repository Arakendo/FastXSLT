//! Typed boolean composition for bounded predicates in location paths.

use super::{
    ControlFailure, Document, InvocationControl, NodeId, NodeKind, PositionPredicate, WorkDomain,
    descendant_nodes, following_siblings, has_named_attribute, is_ncname,
    parse_attribute_value_predicate, parse_position_predicate, position_predicate_matches,
    split_top_level_predicate_operator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PathBooleanPredicate {
    Present(String),
    Equals {
        name: String,
        value: String,
    },
    NotEquals {
        name: String,
        value: String,
    },
    ContextStringEquals(String),
    ContextNameComparison {
        value: String,
        equal: bool,
    },
    ContextNameStartsWith(String),
    ContextNameLengthEquals(usize),
    ChildElementCountEquals {
        name: String,
        count: usize,
    },
    RelativeElementCountComparison {
        steps: Vec<Option<String>>,
        value: usize,
        operator: NumberComparison,
    },
    AncestorElementCountComparison {
        value: usize,
        operator: NumberComparison,
    },
    ContextPosition(PositionPredicate),
    ChildElementIntegerEquals {
        name: Option<String>,
        value: i32,
    },
    AttributeStringLengthEquals {
        name: String,
        length: usize,
    },
    AttributeStringLengthGreaterThan {
        name: String,
        length: usize,
    },
    DescendantElementComparison {
        value: String,
        equal: bool,
    },
    FollowingSiblingElementNumberComparison {
        value: i32,
        operator: NumberComparison,
    },
    FollowingSiblingDescendantStringEquals,
    PositionalChildStringEquals {
        name: String,
        position: usize,
        value: String,
    },
    NestedPositionalChildStringEquals(NestedPositionComparison),
    ChildPathStringEquals {
        left: Vec<RelativeChildStep>,
        right: Vec<RelativeChildStep>,
    },
    ChildPathStringComparison {
        path: Vec<RelativeChildStep>,
        value: String,
        equal: bool,
    },
    ChildAttributeStringComparison {
        children: Vec<RelativeChildStep>,
        attribute: String,
        value: String,
        equal: bool,
    },
    ParentAttributeStringComparison {
        attribute: String,
        value: String,
        equal: bool,
    },
    NestedChildPathExists {
        outer: Vec<RelativeChildStep>,
        inner: Vec<RelativeChildStep>,
    },
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RelativeChildStep {
    Element(String),
    Text,
}

impl PathBooleanPredicate {
    pub(super) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Present(name)
            | Self::ChildElementCountEquals { name, .. }
            | Self::AttributeStringLengthEquals { name, .. }
            | Self::AttributeStringLengthGreaterThan { name, .. }
            | Self::ChildElementIntegerEquals {
                name: Some(name), ..
            } => name.capacity(),
            Self::Equals { name, value }
            | Self::NotEquals { name, value }
            | Self::PositionalChildStringEquals { name, value, .. } => {
                name.capacity() + value.capacity()
            }
            Self::NestedPositionalChildStringEquals(comparison) => {
                comparison.outer_name.capacity()
                    + comparison.inner_name.capacity()
                    + comparison.value.capacity()
            }
            Self::ChildPathStringEquals { left, right } => {
                child_path_capacity(left) + child_path_capacity(right)
            }
            Self::ChildPathStringComparison { path, value, .. } => {
                child_path_capacity(path) + value.capacity()
            }
            Self::ChildAttributeStringComparison {
                children,
                attribute,
                value,
                ..
            } => child_path_capacity(children) + attribute.capacity() + value.capacity(),
            Self::ParentAttributeStringComparison {
                attribute, value, ..
            } => attribute.capacity() + value.capacity(),
            Self::NestedChildPathExists { outer, inner } => {
                child_path_capacity(outer) + child_path_capacity(inner)
            }
            Self::ContextStringEquals(value)
            | Self::ContextNameComparison { value, .. }
            | Self::ContextNameStartsWith(value)
            | Self::DescendantElementComparison { value, .. } => value.capacity(),
            Self::ContextNameLengthEquals(_)
            | Self::ChildElementIntegerEquals { name: None, .. }
            | Self::AncestorElementCountComparison { .. }
            | Self::ContextPosition(_)
            | Self::FollowingSiblingElementNumberComparison { .. }
            | Self::FollowingSiblingDescendantStringEquals => 0,
            Self::RelativeElementCountComparison { steps, .. } => {
                std::mem::size_of_val(steps.as_slice())
                    + steps
                        .iter()
                        .filter_map(Option::as_ref)
                        .map(String::capacity)
                        .sum::<usize>()
            }
            Self::Not(operand) => operand.known_owned_capacity_bytes(),
            Self::And(left, right) | Self::Or(left, right) => {
                left.known_owned_capacity_bytes() + right.known_owned_capacity_bytes()
            }
        }
    }
}

pub(super) fn parse(predicate: &str) -> Option<PathBooleanPredicate> {
    let predicate = strip_outer_parentheses(predicate.trim());
    if let Some(composed) = parse_boolean_composition(predicate) {
        return Some(composed);
    }
    if let Some(position) = parse_position_predicate(predicate) {
        return Some(PathBooleanPredicate::ContextPosition(position));
    }
    if let Some((name, value)) = parse_attribute_value_predicate(predicate) {
        return Some(PathBooleanPredicate::Equals {
            name: name.to_owned(),
            value,
        });
    }
    if let Some(comparison) = parse_relative_path_comparison(predicate) {
        return Some(comparison);
    }
    if let Some((name, value)) = parse_attribute_inequality(predicate) {
        return Some(PathBooleanPredicate::NotEquals {
            name: name.to_owned(),
            value,
        });
    }
    if let Some(value) = parse_context_string_equality(predicate) {
        return Some(PathBooleanPredicate::ContextStringEquals(value));
    }
    if let Some((value, equal)) = parse_context_name_comparison(predicate) {
        return Some(PathBooleanPredicate::ContextNameComparison { value, equal });
    }
    if let Some(prefix) = parse_context_name_starts_with(predicate) {
        return Some(PathBooleanPredicate::ContextNameStartsWith(prefix));
    }
    if let Some(length) = parse_context_name_length_equality(predicate) {
        return Some(PathBooleanPredicate::ContextNameLengthEquals(length));
    }
    if let Some((name, count)) = parse_child_element_count_equality(predicate) {
        return Some(PathBooleanPredicate::ChildElementCountEquals { name, count });
    }
    if let Some((steps, value, operator)) = parse_relative_element_count_comparison(predicate) {
        return Some(PathBooleanPredicate::RelativeElementCountComparison {
            steps,
            value,
            operator,
        });
    }
    if let Some((value, operator)) = parse_ancestor_element_count_comparison(predicate) {
        return Some(PathBooleanPredicate::AncestorElementCountComparison { value, operator });
    }
    if let Some((name, value)) = parse_child_element_integer_equality(predicate) {
        return Some(PathBooleanPredicate::ChildElementIntegerEquals { name, value });
    }
    if let Some((name, length)) = parse_attribute_string_length_equality(predicate) {
        return Some(PathBooleanPredicate::AttributeStringLengthEquals { name, length });
    }
    if let Some((name, length)) = parse_attribute_string_length_greater_than(predicate) {
        return Some(PathBooleanPredicate::AttributeStringLengthGreaterThan { name, length });
    }
    if let Some((value, equal)) = parse_descendant_element_comparison(predicate) {
        return Some(PathBooleanPredicate::DescendantElementComparison { value, equal });
    }
    if let Some((value, operator)) = parse_following_sibling_number_comparison(predicate) {
        return Some(
            PathBooleanPredicate::FollowingSiblingElementNumberComparison { value, operator },
        );
    }
    if parse_sibling_descendant_equality(predicate) {
        return Some(PathBooleanPredicate::FollowingSiblingDescendantStringEquals);
    }
    if let Some(comparison) = parse_nested_positional_child_string_equality(predicate) {
        return Some(PathBooleanPredicate::NestedPositionalChildStringEquals(
            comparison,
        ));
    }
    if let Some((outer, inner)) = parse_nested_child_path_existence(predicate) {
        return Some(PathBooleanPredicate::NestedChildPathExists { outer, inner });
    }
    if let Some((name, position, value)) = parse_positional_child_string_equality(predicate) {
        return Some(PathBooleanPredicate::PositionalChildStringEquals {
            name,
            position,
            value,
        });
    }
    predicate
        .strip_prefix('@')
        .filter(|name| is_ncname(name))
        .map(|name| PathBooleanPredicate::Present(name.to_owned()))
}

fn parse_boolean_composition(predicate: &str) -> Option<PathBooleanPredicate> {
    if let Some((left, right)) = split_top_level_predicate_operator(predicate, " or ") {
        return Some(PathBooleanPredicate::Or(
            Box::new(parse(left)?),
            Box::new(parse(right)?),
        ));
    }
    if let Some((left, right)) = split_top_level_predicate_operator(predicate, " and ") {
        return Some(PathBooleanPredicate::And(
            Box::new(parse(left)?),
            Box::new(parse(right)?),
        ));
    }
    let operand = predicate
        .strip_prefix("not(")
        .and_then(|value| value.strip_suffix(')'))?;
    Some(PathBooleanPredicate::Not(Box::new(parse(operand)?)))
}

pub(super) fn recognizes_child_path_string_comparison(predicate: &str) -> bool {
    let predicate = strip_outer_parentheses(predicate.trim());
    parse_child_path_string_comparison(predicate).is_some()
        || parse_child_attribute_string_comparison(predicate).is_some()
}

pub(super) fn recognizes_child_element_integer_equality(predicate: &str) -> bool {
    parse_child_element_integer_equality(strip_outer_parentheses(predicate.trim())).is_some()
}

pub(super) fn recognizes_parent_attribute_string_comparison(predicate: &str) -> bool {
    parse_parent_attribute_string_comparison(strip_outer_parentheses(predicate.trim())).is_some()
}

fn parse_relative_path_comparison(predicate: &str) -> Option<PathBooleanPredicate> {
    if let Some((left, right)) = parse_child_path_string_equality(predicate) {
        return Some(PathBooleanPredicate::ChildPathStringEquals { left, right });
    }
    if let Some((path, value, equal)) = parse_child_path_string_comparison(predicate) {
        return Some(PathBooleanPredicate::ChildPathStringComparison { path, value, equal });
    }
    if let Some((children, attribute, value, equal)) =
        parse_child_attribute_string_comparison(predicate)
    {
        return Some(PathBooleanPredicate::ChildAttributeStringComparison {
            children,
            attribute,
            value,
            equal,
        });
    }
    parse_parent_attribute_string_comparison(predicate).map(|(attribute, value, equal)| {
        PathBooleanPredicate::ParentAttributeStringComparison {
            attribute,
            value,
            equal,
        }
    })
}

pub(super) fn evaluate(
    document: &Document,
    node: NodeId,
    predicate: &PathBooleanPredicate,
    context_position: usize,
    context_size: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    match predicate {
        PathBooleanPredicate::Not(operand) => evaluate_not(
            document,
            node,
            operand,
            context_position,
            context_size,
            control,
        ),
        PathBooleanPredicate::And(left, right) => evaluate_and(
            document,
            node,
            left,
            right,
            context_position,
            context_size,
            control,
        ),
        PathBooleanPredicate::Or(left, right) => evaluate_or(
            document,
            node,
            left,
            right,
            context_position,
            context_size,
            control,
        ),
        _ => evaluate_atomic(
            document,
            node,
            predicate,
            context_position,
            context_size,
            control,
        ),
    }
}

fn evaluate_atomic(
    document: &Document,
    node: NodeId,
    predicate: &PathBooleanPredicate,
    context_position: usize,
    context_size: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    match predicate {
        PathBooleanPredicate::Present(name) => {
            has_named_attribute(document, node, name, None, control)
        }
        PathBooleanPredicate::Equals { name, value } => {
            has_named_attribute(document, node, name, Some(value), control)
        }
        PathBooleanPredicate::NotEquals { name, value } => {
            attribute_not_equal(document, node, name, value, control)
        }
        PathBooleanPredicate::ContextStringEquals(value) => {
            context_string_equals(document, node, value, control)
        }
        PathBooleanPredicate::ContextNameComparison { value, equal } => {
            control.charge(WorkDomain::XPathOperation, 1)?;
            Ok((context_lexical_name(document, node) == *value) == *equal)
        }
        PathBooleanPredicate::ContextNameStartsWith(prefix) => {
            control.charge(WorkDomain::XPathOperation, 1)?;
            Ok(context_lexical_name(document, node).starts_with(prefix))
        }
        PathBooleanPredicate::ContextNameLengthEquals(length) => {
            control.charge(WorkDomain::XPathOperation, 1)?;
            Ok(context_lexical_name(document, node).chars().count() == *length)
        }
        PathBooleanPredicate::ChildElementCountEquals { name, count } => {
            child_element_count_equals(document, node, name, *count, control)
        }
        PathBooleanPredicate::RelativeElementCountComparison {
            steps,
            value,
            operator,
        } => relative_element_count_compare(document, node, steps, *value, *operator, control),
        PathBooleanPredicate::AncestorElementCountComparison { value, operator } => {
            ancestor_element_count_compare(document, node, *value, *operator, control)
        }
        PathBooleanPredicate::ContextPosition(predicate) => Ok(position_predicate_matches(
            Some(predicate),
            context_position,
            context_size,
        )),
        PathBooleanPredicate::ChildElementIntegerEquals { name, value } => {
            child_integer_equals(document, node, name.as_deref(), *value, control)
        }
        PathBooleanPredicate::AttributeStringLengthEquals { name, length } => {
            attribute_string_length_compare(document, node, name, *length, false, control)
        }
        PathBooleanPredicate::AttributeStringLengthGreaterThan { name, length } => {
            attribute_string_length_compare(document, node, name, *length, true, control)
        }
        PathBooleanPredicate::DescendantElementComparison { value, equal } => {
            descendant_element_comparison(document, node, value, *equal, control)
        }
        PathBooleanPredicate::FollowingSiblingElementNumberComparison { value, operator } => {
            following_sibling_element_number_compare(document, node, *value, *operator, control)
        }
        PathBooleanPredicate::FollowingSiblingDescendantStringEquals => {
            sibling_descendant_string_equals(document, node, control)
        }
        PathBooleanPredicate::PositionalChildStringEquals {
            name,
            position,
            value,
        } => positional_child_string_equals(document, node, name, *position, value, control),
        PathBooleanPredicate::NestedPositionalChildStringEquals(comparison) => {
            nested_positional_child_string_equals(document, node, comparison, control)
        }
        PathBooleanPredicate::ChildPathStringEquals { left, right } => {
            child_path_string_equals(document, node, left, right, control)
        }
        PathBooleanPredicate::ChildPathStringComparison { path, value, equal } => {
            child_path_string_comparison(document, node, path, value, *equal, control)
        }
        PathBooleanPredicate::ChildAttributeStringComparison {
            children,
            attribute,
            value,
            equal,
        } => child_attribute_string_comparison(
            document, node, children, attribute, value, *equal, control,
        ),
        PathBooleanPredicate::ParentAttributeStringComparison {
            attribute,
            value,
            equal,
        } => parent_attribute_string_comparison(document, node, attribute, value, *equal, control),
        PathBooleanPredicate::NestedChildPathExists { outer, inner } => {
            nested_child_path_exists(document, node, outer, inner, control)
        }
        PathBooleanPredicate::Not(_)
        | PathBooleanPredicate::And(_, _)
        | PathBooleanPredicate::Or(_, _) => {
            unreachable!("boolean composition is dispatched by evaluate")
        }
    }
}

fn evaluate_not(
    document: &Document,
    node: NodeId,
    operand: &PathBooleanPredicate,
    context_position: usize,
    context_size: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    Ok(!evaluate(
        document,
        node,
        operand,
        context_position,
        context_size,
        control,
    )?)
}

fn evaluate_and(
    document: &Document,
    node: NodeId,
    left: &PathBooleanPredicate,
    right: &PathBooleanPredicate,
    context_position: usize,
    context_size: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    if !evaluate(
        document,
        node,
        left,
        context_position,
        context_size,
        control,
    )? {
        return Ok(false);
    }
    evaluate(
        document,
        node,
        right,
        context_position,
        context_size,
        control,
    )
}

fn evaluate_or(
    document: &Document,
    node: NodeId,
    left: &PathBooleanPredicate,
    right: &PathBooleanPredicate,
    context_position: usize,
    context_size: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    if evaluate(
        document,
        node,
        left,
        context_position,
        context_size,
        control,
    )? {
        return Ok(true);
    }
    evaluate(
        document,
        node,
        right,
        context_position,
        context_size,
        control,
    )
}

fn following_sibling_element_number_compare(
    document: &Document,
    node: NodeId,
    expected: i32,
    operator: NumberComparison,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for sibling in following_siblings(document, node) {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(sibling) == NodeKind::Element
            && crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(
                &document.string_value(sibling),
            )
            .is_some_and(|actual| operator.evaluate(actual, f64::from(expected)))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn child_path_capacity(path: &[RelativeChildStep]) -> usize {
    std::mem::size_of_val(path)
        + path
            .iter()
            .map(|step| match step {
                RelativeChildStep::Element(name) => name.capacity(),
                RelativeChildStep::Text => 0,
            })
            .sum::<usize>()
}

fn nested_child_path_exists(
    document: &Document,
    node: NodeId,
    outer: &[RelativeChildStep],
    inner: &[RelativeChildStep],
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for outer_node in select_relative_child_path(document, node, outer, control)? {
        if !select_relative_child_path(document, outer_node, inner, control)?.is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn context_string_equals(
    document: &Document,
    node: NodeId,
    value: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    control.charge(WorkDomain::XPathOperation, 1)?;
    Ok(document.string_value(node) == value)
}

fn descendant_element_comparison(
    document: &Document,
    node: NodeId,
    value: &str,
    equal: bool,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for descendant in descendant_nodes(document, node, control)? {
        if document.kind(descendant) == NodeKind::Element
            && (document.string_value(descendant) == value) == equal
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn parse_child_path_string_equality(
    predicate: &str,
) -> Option<(Vec<RelativeChildStep>, Vec<RelativeChildStep>)> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    Some((
        parse_relative_child_path(left.trim())?,
        parse_relative_child_path(right.trim())?,
    ))
}

fn parse_child_path_string_comparison(
    predicate: &str,
) -> Option<(Vec<RelativeChildStep>, String, bool)> {
    for (operator, equal) in [("!=", false), ("=", true)] {
        let Some((left, right)) = split_top_level_predicate_operator(predicate, operator) else {
            continue;
        };
        if let Some((path, value)) = parse_child_path_string_operand(left.trim(), right.trim())
            .or_else(|| parse_child_path_string_operand(right.trim(), left.trim()))
        {
            return Some((path, value, equal));
        }
    }
    None
}

fn parse_child_path_string_operand(
    path: &str,
    literal: &str,
) -> Option<(Vec<RelativeChildStep>, String)> {
    Some((
        parse_relative_child_path(path)?,
        xpath_string_literal(literal)?.to_owned(),
    ))
}

fn parse_child_attribute_string_comparison(
    predicate: &str,
) -> Option<(Vec<RelativeChildStep>, String, String, bool)> {
    for (operator, equal) in [("!=", false), ("=", true)] {
        let Some((left, right)) = split_top_level_predicate_operator(predicate, operator) else {
            continue;
        };
        if let Some((children, attribute, value)) =
            parse_child_attribute_string_operand(left.trim(), right.trim())
                .or_else(|| parse_child_attribute_string_operand(right.trim(), left.trim()))
        {
            return Some((children, attribute, value, equal));
        }
    }
    None
}

fn parse_child_attribute_string_operand(
    path: &str,
    literal: &str,
) -> Option<(Vec<RelativeChildStep>, String, String)> {
    let (children, attribute) = path.rsplit_once("/@")?;
    Some((
        parse_relative_child_path(children)?,
        is_ncname(attribute).then(|| attribute.to_owned())?,
        xpath_string_literal(literal)?.to_owned(),
    ))
}

fn parse_parent_attribute_string_comparison(predicate: &str) -> Option<(String, String, bool)> {
    for (operator, equal) in [("!=", false), ("=", true)] {
        let Some((left, right)) = split_top_level_predicate_operator(predicate, operator) else {
            continue;
        };
        if let Some((attribute, value)) =
            parse_parent_attribute_string_operand(left.trim(), right.trim())
                .or_else(|| parse_parent_attribute_string_operand(right.trim(), left.trim()))
        {
            return Some((attribute, value, equal));
        }
    }
    None
}

fn parse_parent_attribute_string_operand(path: &str, literal: &str) -> Option<(String, String)> {
    let attribute = path.strip_prefix("../@")?;
    Some((
        is_ncname(attribute).then(|| attribute.to_owned())?,
        xpath_string_literal(literal)?.to_owned(),
    ))
}

fn parse_relative_child_path(path: &str) -> Option<Vec<RelativeChildStep>> {
    const MAX_STEPS: usize = 4;
    let steps = path.split('/').collect::<Vec<_>>();
    if steps.is_empty() || steps.len() > MAX_STEPS {
        return None;
    }
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            if *step == "text()" && index + 1 == steps.len() {
                Some(RelativeChildStep::Text)
            } else {
                is_ncname(step).then(|| RelativeChildStep::Element((*step).to_owned()))
            }
        })
        .collect()
}

fn parse_nested_child_path_existence(
    predicate: &str,
) -> Option<(Vec<RelativeChildStep>, Vec<RelativeChildStep>)> {
    let nested = predicate.strip_prefix('(')?;
    let (outer, inner) = nested.split_once(")[")?;
    let inner = inner.strip_suffix(']')?;
    Some((
        parse_relative_child_path(outer.trim())?,
        parse_relative_child_path(inner.trim())?,
    ))
}

fn child_path_string_equals(
    document: &Document,
    node: NodeId,
    left: &[RelativeChildStep],
    right: &[RelativeChildStep],
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let left = select_relative_child_path(document, node, left, control)?;
    let right = select_relative_child_path(document, node, right, control)?;
    for left in left {
        for right in right.iter().copied() {
            control.charge(WorkDomain::XPathOperation, 1)?;
            if document.string_value(left) == document.string_value(right) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn child_path_string_comparison(
    document: &Document,
    node: NodeId,
    path: &[RelativeChildStep],
    value: &str,
    equal: bool,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for selected in select_relative_child_path(document, node, path, control)? {
        control.charge(WorkDomain::XPathOperation, 1)?;
        if (document.string_value(selected) == value) == equal {
            return Ok(true);
        }
    }
    Ok(false)
}

fn child_attribute_string_comparison(
    document: &Document,
    node: NodeId,
    children: &[RelativeChildStep],
    attribute: &str,
    value: &str,
    equal: bool,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for parent in select_relative_child_path(document, node, children, control)? {
        for selected in document.attributes(parent).iter().copied() {
            control.charge(WorkDomain::XPathNodeVisit, 1)?;
            if unnamespaced_attribute_named(document, selected, attribute) {
                control.charge(WorkDomain::XPathOperation, 1)?;
                if (document.value(selected).unwrap_or_default() == value) == equal {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn select_relative_child_path(
    document: &Document,
    node: NodeId,
    path: &[RelativeChildStep],
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let mut selected = vec![node];
    for step in path {
        let mut next = Vec::new();
        for parent in selected {
            for child in document.children(parent).iter().copied() {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                let matches = match step {
                    RelativeChildStep::Element(name) => {
                        document.kind(child) == NodeKind::Element
                            && document.name(child).is_some_and(|candidate| {
                                candidate.namespace.is_none() && candidate.local == *name
                            })
                    }
                    RelativeChildStep::Text => document.kind(child) == NodeKind::Text,
                };
                if matches {
                    next.push(child);
                }
            }
        }
        selected = next;
    }
    Ok(selected)
}

fn sibling_descendant_string_equals(
    document: &Document,
    node: NodeId,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let descendants = descendant_nodes(document, node, control)?;
    for sibling in following_siblings(document, node) {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(sibling) != NodeKind::Element {
            continue;
        }
        let sibling_value = document.string_value(sibling);
        for descendant in descendants.iter().copied() {
            if document.kind(descendant) != NodeKind::Element {
                continue;
            }
            control.charge(WorkDomain::XPathOperation, 1)?;
            if sibling_value == document.string_value(descendant) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn positional_child_string_equals(
    document: &Document,
    node: NodeId,
    name: &str,
    position: usize,
    value: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut matched = 0usize;
    for child in document.children(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(child) == NodeKind::Element
            && document
                .name(child)
                .is_some_and(|candidate| candidate.namespace.is_none() && candidate.local == name)
        {
            matched += 1;
            if matched == position {
                control.charge(WorkDomain::XPathOperation, 1)?;
                return Ok(document.string_value(child) == value);
            }
        }
    }
    Ok(false)
}

fn nested_positional_child_string_equals(
    document: &Document,
    node: NodeId,
    comparison: &NestedPositionComparison,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut outer_index = 0usize;
    for child in document.children(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(child) != NodeKind::Element
            || !document.name(child).is_some_and(|candidate| {
                candidate.namespace.is_none() && candidate.local == comparison.outer_name
            })
        {
            continue;
        }
        outer_index += 1;
        if comparison
            .outer_position
            .is_some_and(|required| outer_index != required)
        {
            continue;
        }
        if positional_child_string_equals(
            document,
            child,
            &comparison.inner_name,
            comparison.inner_position,
            &comparison.value,
            control,
        )? {
            return Ok(true);
        }
        if comparison.outer_position.is_some() {
            return Ok(false);
        }
    }
    Ok(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NumberComparison {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl NumberComparison {
    fn evaluate(self, left: f64, right: f64) -> bool {
        match self {
            Self::Equal => left
                .partial_cmp(&right)
                .is_some_and(std::cmp::Ordering::is_eq),
            Self::NotEqual => left.partial_cmp(&right).is_none_or(|order| !order.is_eq()),
            Self::LessThan => left < right,
            Self::LessThanOrEqual => left <= right,
            Self::GreaterThan => left > right,
            Self::GreaterThanOrEqual => left >= right,
        }
    }

    fn reversed(self) -> Self {
        match self {
            Self::Equal => Self::Equal,
            Self::NotEqual => Self::NotEqual,
            Self::LessThan => Self::GreaterThan,
            Self::LessThanOrEqual => Self::GreaterThanOrEqual,
            Self::GreaterThan => Self::LessThan,
            Self::GreaterThanOrEqual => Self::LessThanOrEqual,
        }
    }

    fn evaluate_usize(self, left: usize, right: usize) -> bool {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::LessThan => left < right,
            Self::LessThanOrEqual => left <= right,
            Self::GreaterThan => left > right,
            Self::GreaterThanOrEqual => left >= right,
        }
    }
}

fn attribute_not_equal(
    document: &Document,
    node: NodeId,
    name: &str,
    value: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for attribute in document.attributes(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if unnamespaced_attribute_named(document, attribute, name)
            && document.value(attribute) != Some(value)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn parent_attribute_string_comparison(
    document: &Document,
    node: NodeId,
    attribute: &str,
    value: &str,
    equal: bool,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    control.charge(WorkDomain::XPathNodeVisit, 1)?;
    let Some(parent) = document.parent(node) else {
        return Ok(false);
    };
    if equal {
        has_named_attribute(document, parent, attribute, Some(value), control)
    } else {
        attribute_not_equal(document, parent, attribute, value, control)
    }
}

fn child_element_count_equals(
    document: &Document,
    node: NodeId,
    name: &str,
    expected: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut actual = 0usize;
    for child in document.children(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(child) == NodeKind::Element
            && document
                .name(child)
                .is_some_and(|candidate| candidate.namespace.is_none() && candidate.local == name)
        {
            actual += 1;
        }
    }
    Ok(actual == expected)
}

fn relative_element_count_compare(
    document: &Document,
    node: NodeId,
    steps: &[Option<String>],
    expected: usize,
    operator: NumberComparison,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut current = vec![node];
    for required_name in steps {
        let mut next = Vec::new();
        for parent in current {
            for child in document.children(parent).iter().copied() {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                if document.kind(child) == NodeKind::Element
                    && required_name.as_ref().is_none_or(|required_name| {
                        document.name(child).is_some_and(|candidate| {
                            candidate.namespace.is_none() && candidate.local == *required_name
                        })
                    })
                {
                    next.push(child);
                }
            }
        }
        current = next;
        if current.is_empty() {
            break;
        }
    }
    control.charge(WorkDomain::XPathOperation, 1)?;
    Ok(operator.evaluate_usize(current.len(), expected))
}

fn ancestor_element_count_compare(
    document: &Document,
    node: NodeId,
    expected: usize,
    operator: NumberComparison,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut actual = 0usize;
    let mut current = document.parent(node);
    while let Some(ancestor) = current {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(ancestor) == NodeKind::Element {
            actual += 1;
        }
        current = document.parent(ancestor);
    }
    control.charge(WorkDomain::XPathOperation, 1)?;
    Ok(operator.evaluate_usize(actual, expected))
}

fn child_integer_equals(
    document: &Document,
    node: NodeId,
    name: Option<&str>,
    expected: i32,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for child in document.children(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(child) != NodeKind::Element
            || name.is_some_and(|name| {
                !document.name(child).is_some_and(|candidate| {
                    candidate.namespace.is_none() && candidate.local == name
                })
            })
        {
            continue;
        }
        control.charge(WorkDomain::XPathOperation, 1)?;
        if crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(
            &document.string_value(child),
        )
        .is_some_and(|actual| NumberComparison::Equal.evaluate(actual, f64::from(expected)))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn attribute_string_length_compare(
    document: &Document,
    node: NodeId,
    name: &str,
    expected: usize,
    greater_than: bool,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for attribute in document.attributes(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if unnamespaced_attribute_named(document, attribute, name) {
            control.charge(WorkDomain::XPathOperation, 1)?;
            let actual = document
                .value(attribute)
                .map_or(0, |value| value.chars().count());
            return Ok(if greater_than {
                actual > expected
            } else {
                actual == expected
            });
        }
    }
    control.charge(WorkDomain::XPathOperation, 1)?;
    Ok(!greater_than && expected == 0)
}

fn unnamespaced_attribute_named(document: &Document, attribute: NodeId, name: &str) -> bool {
    document.kind(attribute) == NodeKind::Attribute
        && document
            .name(attribute)
            .is_some_and(|candidate| candidate.namespace.is_none() && candidate.local == name)
}

fn context_lexical_name(document: &Document, node: NodeId) -> String {
    let Some(name) = document.name(node) else {
        return String::new();
    };
    document.prefix(node).map_or_else(
        || name.local.clone(),
        |prefix| format!("{prefix}:{}", name.local),
    )
}

fn parse_context_name_starts_with(predicate: &str) -> Option<String> {
    let arguments = predicate.strip_prefix("starts-with(")?.strip_suffix(')')?;
    let (name, prefix) = split_top_level_predicate_operator(arguments, ",")?;
    matches!(name.trim(), "name()" | "name(.)")
        .then(|| xpath_string_literal(prefix.trim()).map(str::to_owned))
        .flatten()
}

fn parse_context_name_comparison(predicate: &str) -> Option<(String, bool)> {
    parse_context_name_comparison_operator(predicate, "!=", false)
        .or_else(|| parse_context_name_comparison_operator(predicate, "=", true))
}

fn parse_context_name_comparison_operator(
    predicate: &str,
    operator: &str,
    equal: bool,
) -> Option<(String, bool)> {
    let (left, right) = split_top_level_predicate_operator(predicate, operator)?;
    let left = left.trim();
    let right = right.trim();
    if matches!(left, "name()" | "name(.)") {
        xpath_string_literal(right).map(|value| (value.to_owned(), equal))
    } else if matches!(right, "name()" | "name(.)") {
        xpath_string_literal(left).map(|value| (value.to_owned(), equal))
    } else {
        None
    }
}

fn parse_context_name_length_equality(predicate: &str) -> Option<usize> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    parse_context_name_length_operand(left.trim(), right.trim())
        .or_else(|| parse_context_name_length_operand(right.trim(), left.trim()))
}

fn parse_context_name_length_operand(function: &str, integer: &str) -> Option<usize> {
    matches!(function, "string-length(name())" | "string-length(name(.))")
        .then(|| integer.parse().ok())
        .flatten()
}

fn parse_child_element_count_equality(predicate: &str) -> Option<(String, usize)> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    parse_child_element_count_operand(left.trim(), right.trim())
        .or_else(|| parse_child_element_count_operand(right.trim(), left.trim()))
}

fn parse_relative_element_count_comparison(
    predicate: &str,
) -> Option<(Vec<Option<String>>, usize, NumberComparison)> {
    const OPERATORS: [(&str, NumberComparison); 6] = [
        ("!=", NumberComparison::NotEqual),
        ("<=", NumberComparison::LessThanOrEqual),
        (">=", NumberComparison::GreaterThanOrEqual),
        ("=", NumberComparison::Equal),
        ("<", NumberComparison::LessThan),
        (">", NumberComparison::GreaterThan),
    ];

    for (token, operator) in OPERATORS {
        let Some((left, right)) = split_top_level_predicate_operator(predicate, token) else {
            continue;
        };
        if let Some(parsed) =
            parse_relative_element_count_operand(left.trim(), right.trim(), operator)
        {
            return Some(parsed);
        }
        if let Some(parsed) =
            parse_relative_element_count_operand(right.trim(), left.trim(), operator.reversed())
        {
            return Some(parsed);
        }
    }
    None
}

fn parse_ancestor_element_count_comparison(predicate: &str) -> Option<(usize, NumberComparison)> {
    const OPERATORS: [(&str, NumberComparison); 6] = [
        ("!=", NumberComparison::NotEqual),
        ("<=", NumberComparison::LessThanOrEqual),
        (">=", NumberComparison::GreaterThanOrEqual),
        ("=", NumberComparison::Equal),
        ("<", NumberComparison::LessThan),
        (">", NumberComparison::GreaterThan),
    ];

    for (token, operator) in OPERATORS {
        let Some((left, right)) = split_top_level_predicate_operator(predicate, token) else {
            continue;
        };
        if left.trim() == "count(ancestor::*)" {
            return Some((right.trim().parse().ok()?, operator));
        }
        if right.trim() == "count(ancestor::*)" {
            return Some((left.trim().parse().ok()?, operator.reversed()));
        }
    }
    None
}

fn parse_relative_element_count_operand(
    function: &str,
    integer: &str,
    operator: NumberComparison,
) -> Option<(Vec<Option<String>>, usize, NumberComparison)> {
    let path = function.strip_prefix("count(")?.strip_suffix(')')?.trim();
    let path = path.strip_prefix("./").unwrap_or(path);
    let steps = path
        .split('/')
        .map(|step| match step.trim() {
            "*" => Some(None),
            name if is_ncname(name) => Some(Some(name.to_owned())),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    if steps.is_empty() || steps.len() > 8 {
        return None;
    }
    Some((steps, integer.parse().ok()?, operator))
}

fn parse_child_element_integer_equality(predicate: &str) -> Option<(Option<String>, i32)> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    parse_child_element_integer_operand(left.trim(), right.trim())
        .or_else(|| parse_child_element_integer_operand(right.trim(), left.trim()))
}

fn parse_child_element_integer_operand(
    child_test: &str,
    integer: &str,
) -> Option<(Option<String>, i32)> {
    let name = if child_test == "*" {
        None
    } else if is_ncname(child_test) {
        Some(child_test.to_owned())
    } else {
        return None;
    };
    Some((name, integer.parse().ok()?))
}

fn parse_child_element_count_operand(function: &str, integer: &str) -> Option<(String, usize)> {
    let path = function.strip_prefix("count(")?.strip_suffix(')')?.trim();
    let name = path.strip_prefix("./")?;
    let count = integer.parse().ok()?;
    is_ncname(name).then(|| (name.to_owned(), count))
}

fn parse_attribute_string_length_equality(predicate: &str) -> Option<(String, usize)> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    parse_attribute_string_length_operand(left.trim(), right.trim())
        .or_else(|| parse_attribute_string_length_operand(right.trim(), left.trim()))
}

fn parse_attribute_string_length_greater_than(predicate: &str) -> Option<(String, usize)> {
    let (function, integer) = split_top_level_predicate_operator(predicate, ">")?;
    parse_attribute_string_length_operand(function.trim(), integer.trim())
}

fn parse_attribute_string_length_operand(function: &str, integer: &str) -> Option<(String, usize)> {
    let argument = function
        .strip_prefix("string-length(")?
        .strip_suffix(')')?
        .trim();
    let name = argument.strip_prefix('@')?;
    let length = integer.parse().ok()?;
    is_ncname(name).then(|| (name.to_owned(), length))
}

fn parse_context_string_equality(predicate: &str) -> Option<String> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    let left = left.trim();
    let right = right.trim();
    if left == "." {
        xpath_string_literal(right).map(str::to_owned)
    } else if right == "." {
        xpath_string_literal(left).map(str::to_owned)
    } else {
        None
    }
}

fn parse_attribute_inequality(predicate: &str) -> Option<(&str, String)> {
    let (name, value) = split_top_level_predicate_operator(predicate, "!=")?;
    let name = name.trim().strip_prefix('@')?;
    if !is_ncname(name) {
        return None;
    }
    xpath_string_literal(value.trim()).map(|value| (name, value.to_owned()))
}

fn parse_following_sibling_number_comparison(predicate: &str) -> Option<(i32, NumberComparison)> {
    for (token, operator) in [
        ("!=", NumberComparison::NotEqual),
        ("<=", NumberComparison::LessThanOrEqual),
        (">=", NumberComparison::GreaterThanOrEqual),
        ("=", NumberComparison::Equal),
        ("<", NumberComparison::LessThan),
        (">", NumberComparison::GreaterThan),
    ] {
        let Some((left, right)) = split_top_level_predicate_operator(predicate, token) else {
            continue;
        };
        let left = left.trim();
        let right = right.trim();
        if left == "following-sibling::*" {
            return Some((right.parse().ok()?, operator));
        }
        if right == "following-sibling::*" {
            return Some((left.parse().ok()?, operator.reversed()));
        }
    }
    None
}

fn parse_sibling_descendant_equality(predicate: &str) -> bool {
    let Some((left, right)) = split_top_level_predicate_operator(predicate, "=") else {
        return false;
    };
    matches!(
        (left.trim(), right.trim()),
        ("following-sibling::*", "descendant::*") | ("descendant::*", "following-sibling::*")
    )
}

fn parse_positional_child_string_equality(predicate: &str) -> Option<(String, usize, String)> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    parse_positional_child_string_operand(left.trim(), right.trim())
        .or_else(|| parse_positional_child_string_operand(right.trim(), left.trim()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NestedPositionComparison {
    outer_name: String,
    outer_position: Option<usize>,
    inner_name: String,
    inner_position: usize,
    value: String,
}

fn parse_nested_positional_child_string_equality(
    predicate: &str,
) -> Option<NestedPositionComparison> {
    let predicate = strip_outer_parentheses(predicate);
    let (outer_name, remainder) = predicate.split_once('[')?;
    if !is_ncname(outer_name) {
        return None;
    }
    let (outer_position, nested) = if remainder.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        let (position, nested) = remainder.split_once(']')?;
        let position = position.parse().ok()?;
        if position == 0 {
            return None;
        }
        (Some(position), nested)
    } else {
        (None, predicate.strip_prefix(outer_name)?)
    };
    let nested = nested.strip_prefix('[')?.strip_suffix(']')?;
    let (inner_name, inner_position, value) = parse_positional_child_string_equality(nested)?;
    Some(NestedPositionComparison {
        outer_name: outer_name.to_owned(),
        outer_position,
        inner_name,
        inner_position,
        value,
    })
}

fn parse_positional_child_string_operand(
    path: &str,
    literal: &str,
) -> Option<(String, usize, String)> {
    let path = strip_outer_parentheses(path);
    let (name, position) = path.split_once('[')?;
    let position = position.strip_suffix(']')?.parse().ok()?;
    if position == 0 || !is_ncname(name) {
        return None;
    }
    Some((
        name.to_owned(),
        position,
        xpath_string_literal(literal)?.to_owned(),
    ))
}

fn parse_descendant_element_comparison(predicate: &str) -> Option<(String, bool)> {
    parse_descendant_element_comparison_operator(predicate, "!=", false)
        .or_else(|| parse_descendant_element_comparison_operator(predicate, "=", true))
}

fn parse_descendant_element_comparison_operator(
    predicate: &str,
    operator: &str,
    equal: bool,
) -> Option<(String, bool)> {
    let (left, right) = split_top_level_predicate_operator(predicate, operator)?;
    let left = left.trim();
    let right = right.trim();
    if left == "descendant::*" {
        xpath_string_literal(right).map(|value| (value.to_owned(), equal))
    } else if right == "descendant::*" {
        xpath_string_literal(left).map(|value| (value.to_owned(), equal))
    } else {
        None
    }
}

fn xpath_string_literal(expression: &str) -> Option<&str> {
    for delimiter in ['\'', '"'] {
        if let Some(value) = expression
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter))
            .filter(|value| !value.contains(delimiter))
        {
            return Some(value);
        }
    }
    None
}

fn strip_outer_parentheses(mut predicate: &str) -> &str {
    loop {
        let Some(inner) = predicate
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
        else {
            return predicate;
        };
        let mut depth = 0usize;
        let mut quote = None;
        let mut encloses_all = true;
        for (index, byte) in predicate.bytes().enumerate() {
            match byte {
                b'\'' | b'"' if quote == Some(byte) => quote = None,
                b'\'' | b'"' if quote.is_none() => quote = Some(byte),
                b'(' if quote.is_none() => depth += 1,
                b')' if quote.is_none() => {
                    let Some(next) = depth.checked_sub(1) else {
                        return predicate;
                    };
                    depth = next;
                    if depth == 0 && index + 1 != predicate.len() {
                        encloses_all = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !encloses_all || depth != 0 || quote.is_some() {
            return predicate;
        }
        predicate = inner.trim();
    }
}
