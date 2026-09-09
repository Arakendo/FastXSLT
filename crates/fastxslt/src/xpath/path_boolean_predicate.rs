//! Typed boolean composition for bounded predicates in location paths.

use super::{
    ControlFailure, Document, InvocationControl, NodeId, NodeKind, WorkDomain, descendant_nodes,
    following_siblings, has_named_attribute, is_ascii_ncname, parse_attribute_value_predicate,
    split_top_level_predicate_operator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PathBooleanPredicate {
    Present(String),
    Equals { name: String, value: String },
    DescendantElementComparison { value: String, equal: bool },
    FollowingSiblingElementNumberEquals(i32),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl PathBooleanPredicate {
    pub(super) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Present(name) => name.capacity(),
            Self::Equals { name, value } => name.capacity() + value.capacity(),
            Self::DescendantElementComparison { value, .. } => value.capacity(),
            Self::FollowingSiblingElementNumberEquals(_) => 0,
            Self::Not(operand) => operand.known_owned_capacity_bytes(),
            Self::And(left, right) | Self::Or(left, right) => {
                left.known_owned_capacity_bytes() + right.known_owned_capacity_bytes()
            }
        }
    }
}

pub(super) fn parse(predicate: &str) -> Option<PathBooleanPredicate> {
    let predicate = strip_outer_parentheses(predicate.trim());
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
    if let Some(operand) = predicate
        .strip_prefix("not(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return Some(PathBooleanPredicate::Not(Box::new(parse(operand)?)));
    }
    if let Some((name, value)) = parse_attribute_value_predicate(predicate) {
        return Some(PathBooleanPredicate::Equals {
            name: name.to_owned(),
            value,
        });
    }
    if let Some((value, equal)) = parse_descendant_element_comparison(predicate) {
        return Some(PathBooleanPredicate::DescendantElementComparison { value, equal });
    }
    if let Some(value) = parse_following_sibling_number_equality(predicate) {
        return Some(PathBooleanPredicate::FollowingSiblingElementNumberEquals(
            value,
        ));
    }
    predicate
        .strip_prefix('@')
        .filter(|name| is_ascii_ncname(name))
        .map(|name| PathBooleanPredicate::Present(name.to_owned()))
}

pub(super) fn evaluate(
    document: &Document,
    node: NodeId,
    predicate: &PathBooleanPredicate,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    match predicate {
        PathBooleanPredicate::Present(name) => {
            has_named_attribute(document, node, name, None, control)
        }
        PathBooleanPredicate::Equals { name, value } => {
            has_named_attribute(document, node, name, Some(value), control)
        }
        PathBooleanPredicate::DescendantElementComparison { value, equal } => {
            for descendant in descendant_nodes(document, node, control)? {
                if document.kind(descendant) == NodeKind::Element
                    && (document.string_value(descendant) == *value) == *equal
                {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        PathBooleanPredicate::FollowingSiblingElementNumberEquals(value) => {
            for sibling in following_siblings(document, node) {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                if document.kind(sibling) == NodeKind::Element
                    && crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(
                        &document.string_value(sibling),
                    ) == Some(f64::from(*value))
                {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        PathBooleanPredicate::Not(operand) => Ok(!evaluate(document, node, operand, control)?),
        PathBooleanPredicate::And(left, right) => {
            if !evaluate(document, node, left, control)? {
                return Ok(false);
            }
            evaluate(document, node, right, control)
        }
        PathBooleanPredicate::Or(left, right) => {
            if evaluate(document, node, left, control)? {
                return Ok(true);
            }
            evaluate(document, node, right, control)
        }
    }
}

fn parse_following_sibling_number_equality(predicate: &str) -> Option<i32> {
    let (left, right) = split_top_level_predicate_operator(predicate, "=")?;
    let left = left.trim();
    let right = right.trim();
    if left == "following-sibling::*" {
        right.parse().ok()
    } else if right == "following-sibling::*" {
        left.parse().ok()
    } else {
        None
    }
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
