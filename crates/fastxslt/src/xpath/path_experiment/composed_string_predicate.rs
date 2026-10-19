//! Bounded scalar string composition in path predicates. Navigation stays in
//! the shared path evaluator; this module does not admit variables or resources.

use super::{
    ControlFailure, Document, InvocationControl, LocationPath, NodeId, SourceLocation, WorkDomain,
    evaluate_location_path_controlled, parse_location_path, split_top_level_predicate_operator,
};
use crate::xpath::static_string_experiment::split_arguments;

#[cfg(test)]
#[path = "composed_string_predicate_tests.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Comparison {
    Strings {
        left: StringValue,
        right: StringValue,
        equal: bool,
    },
    Length {
        value: StringValue,
        length: usize,
        equal: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum StringValue {
    Literal(String),
    Path(Box<LocationPath>),
    Concat(Vec<Self>),
    Before(Box<Self>, Box<Self>),
}

impl Comparison {
    pub(super) fn capacity(&self) -> usize {
        match self {
            Self::Strings { left, right, .. } => left.capacity() + right.capacity(),
            Self::Length { value, .. } => value.capacity(),
        }
    }

    pub(super) fn evaluate(
        &self,
        document: &Document,
        node: NodeId,
        control: &mut InvocationControl,
    ) -> Result<bool, ControlFailure> {
        control.charge(WorkDomain::XPathOperation, 1)?;
        match self {
            Self::Strings { left, right, equal } => {
                let left = left.evaluate(document, node, control)?;
                let right = right.evaluate(document, node, control)?;
                control.charge(WorkDomain::XPathOperation, left.len().min(right.len()))?;
                Ok((left == right) == *equal)
            }
            Self::Length {
                value,
                length,
                equal,
            } => {
                let value = value.evaluate(document, node, control)?;
                control.charge(WorkDomain::XPathOperation, value.len())?;
                Ok((value.chars().count() == *length) == *equal)
            }
        }
    }
}

impl StringValue {
    fn capacity(&self) -> usize {
        match self {
            Self::Literal(value) => value.capacity(),
            Self::Path(path) => size_of::<LocationPath>() + path.known_owned_capacity_bytes(),
            Self::Concat(parts) => {
                parts.capacity() * size_of::<Self>()
                    + parts.iter().map(Self::capacity).sum::<usize>()
            }
            Self::Before(left, right) => 2 * size_of::<Self>() + left.capacity() + right.capacity(),
        }
    }

    fn evaluate(
        &self,
        document: &Document,
        node: NodeId,
        control: &mut InvocationControl,
    ) -> Result<String, ControlFailure> {
        control.charge(WorkDomain::XPathOperation, 1)?;
        match self {
            Self::Literal(value) => {
                control.charge(WorkDomain::ResultTextByte, value.len())?;
                Ok(value.clone())
            }
            Self::Path(path) => {
                let nodes = evaluate_location_path_controlled(document, node, path, control)?;
                let mut value = String::new();
                if let Some(node) = nodes.first() {
                    use crate::xdm::owned_tree_experiment::StringValueVisitFailure;
                    document
                        .visit_string_value_controlled(*node, control, &mut |part, control| {
                            control.charge(WorkDomain::ResultTextByte, part.len())?;
                            value.push_str(part);
                            Ok(())
                        })
                        .map_err(|failure| match failure {
                            StringValueVisitFailure::Control(failure)
                            | StringValueVisitFailure::Sink(failure) => failure,
                        })?;
                }
                Ok(value)
            }
            Self::Concat(parts) => {
                let mut result = String::new();
                for part in parts {
                    let value = part.evaluate(document, node, control)?;
                    control.charge(WorkDomain::ResultTextByte, value.len())?;
                    result.push_str(&value);
                }
                Ok(result)
            }
            Self::Before(left, right) => {
                let left = left.evaluate(document, node, control)?;
                let right = right.evaluate(document, node, control)?;
                control.charge(
                    WorkDomain::XPathOperation,
                    left.len().saturating_add(right.len()),
                )?;
                let value = left.find(&right).map_or("", |end| &left[..end]);
                control.charge(WorkDomain::ResultTextByte, value.len())?;
                Ok(value.to_owned())
            }
        }
    }
}

pub(super) fn parse(expression: &str, location: &SourceLocation) -> Option<Comparison> {
    for (operator, equal) in [("!=", false), ("=", true)] {
        let Some((left, right)) = split_top_level_predicate_operator(expression, operator) else {
            continue;
        };
        for (function, constant) in [(left, right), (right, left)] {
            if let Some(argument) = call(function.trim(), "string-length") {
                let constant = constant.trim();
                let length = constant_length(constant)?;
                return Some(Comparison::Length {
                    value: parse_value(argument, location, 0)?,
                    length,
                    equal,
                });
            }
        }
        // Bare path comparisons have existential node-set semantics, not scalar
        // equality. Only explicit scalar expressions enter this branch.
        if !explicit_scalar(left.trim()) || !explicit_scalar(right.trim()) {
            return None;
        }
        return Some(Comparison::Strings {
            left: parse_value(left, location, 0)?,
            right: parse_value(right, location, 0)?,
            equal,
        });
    }
    None
}

fn explicit_scalar(value: &str) -> bool {
    value.starts_with(['\'', '"'])
        || ["string", "concat", "substring-before"]
            .iter()
            .any(|name| call(value, name).is_some())
}

fn constant_length(value: &str) -> Option<usize> {
    // Bound the input passed to the recursive shared constant folder too.
    if value.len() > 128 {
        return None;
    }
    if value.bytes().all(|byte| byte.is_ascii_digit())
        && let Ok(value) = value.parse::<u32>()
    {
        return usize::try_from(value).ok();
    }
    // Keep exact folding within an IEEE-754-exact integer envelope: at most
    // twelve single-digit operands, using only +, -, * and grouping. This is
    // an admission restriction, not a substitute for general numeric semantics.
    let mut digits = 0;
    let mut previous_digit = false;
    for byte in value.bytes() {
        if byte.is_ascii_digit() {
            if previous_digit {
                return None;
            }
            digits += 1;
            if digits > 12 {
                return None;
            }
            previous_digit = true;
        } else {
            if !matches!(
                byte,
                b'+' | b'-' | b'*' | b'(' | b')' | b' ' | b'\t' | b'\r' | b'\n'
            ) {
                return None;
            }
            previous_digit = false;
        }
    }
    crate::xpath::constant_numeric_experiment::fold_exact_integral_arithmetic(value)?
        .parse()
        .ok()
}

fn call<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    value
        .strip_prefix(name)?
        .trim_start()
        .strip_prefix('(')?
        .strip_suffix(')')
}

fn parse_value(value: &str, location: &SourceLocation, depth: usize) -> Option<StringValue> {
    // Private admission bound, not a host policy or semantic nesting limit.
    if depth >= 16 {
        return None;
    }
    let value = value.trim();
    for quote in ['\'', '"'] {
        if let Some(literal) = value
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
        {
            if !literal.contains(quote) {
                return Some(StringValue::Literal(literal.to_owned()));
            }
        }
    }
    if let Some(argument) = call(value, "string") {
        return parse_value(argument, location, depth + 1);
    }
    if let Some(arguments) = call(value, "concat") {
        let arguments = split_arguments(arguments, 16)?;
        if arguments.len() < 2 {
            return None;
        }
        return arguments
            .into_iter()
            .map(|argument| parse_value(argument, location, depth + 1))
            .collect::<Option<Vec<_>>>()
            .map(StringValue::Concat);
    }
    if let Some(arguments) = call(value, "substring-before") {
        let arguments = split_arguments(arguments, 2)?;
        let [left, right] = arguments.as_slice() else {
            return None;
        };
        return Some(StringValue::Before(
            Box::new(parse_value(left, location, depth + 1)?),
            Box::new(parse_value(right, location, depth + 1)?),
        ));
    }
    // No nested predicate compilation, function dispatch or QName resolution.
    // All admitted navigation is compiled once by the ordinary path parser.
    if value.is_empty()
        || value.chars().any(|c| {
            !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '*' | ':' | '@'))
        })
    {
        return None;
    }
    parse_location_path(value, location.clone())
        .ok()
        .map(|path| StringValue::Path(Box::new(path)))
}
