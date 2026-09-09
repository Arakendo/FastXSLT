//! Typed boolean composition for attribute predicates in location paths.

use super::{
    ControlFailure, Document, InvocationControl, NodeId, has_named_attribute, is_ascii_ncname,
    parse_attribute_value_predicate, split_top_level_predicate_operator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AttributeBooleanPredicate {
    Present(String),
    Equals { name: String, value: String },
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl AttributeBooleanPredicate {
    pub(super) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Present(name) => name.capacity(),
            Self::Equals { name, value } => name.capacity() + value.capacity(),
            Self::And(left, right) | Self::Or(left, right) => {
                left.known_owned_capacity_bytes() + right.known_owned_capacity_bytes()
            }
        }
    }
}

pub(super) fn parse(predicate: &str) -> Option<AttributeBooleanPredicate> {
    let predicate = strip_outer_parentheses(predicate.trim());
    if let Some((left, right)) = split_top_level_predicate_operator(predicate, " or ") {
        return Some(AttributeBooleanPredicate::Or(
            Box::new(parse(left)?),
            Box::new(parse(right)?),
        ));
    }
    if let Some((left, right)) = split_top_level_predicate_operator(predicate, " and ") {
        return Some(AttributeBooleanPredicate::And(
            Box::new(parse(left)?),
            Box::new(parse(right)?),
        ));
    }
    if let Some((name, value)) = parse_attribute_value_predicate(predicate) {
        return Some(AttributeBooleanPredicate::Equals {
            name: name.to_owned(),
            value,
        });
    }
    predicate
        .strip_prefix('@')
        .filter(|name| is_ascii_ncname(name))
        .map(|name| AttributeBooleanPredicate::Present(name.to_owned()))
}

pub(super) fn evaluate(
    document: &Document,
    node: NodeId,
    predicate: &AttributeBooleanPredicate,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    match predicate {
        AttributeBooleanPredicate::Present(name) => {
            has_named_attribute(document, node, name, None, control)
        }
        AttributeBooleanPredicate::Equals { name, value } => {
            has_named_attribute(document, node, name, Some(value), control)
        }
        AttributeBooleanPredicate::And(left, right) => {
            if !evaluate(document, node, left, control)? {
                return Ok(false);
            }
            evaluate(document, node, right, control)
        }
        AttributeBooleanPredicate::Or(left, right) => {
            if evaluate(document, node, left, control)? {
                return Ok(true);
            }
            evaluate(document, node, right, control)
        }
    }
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
