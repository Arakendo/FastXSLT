//! Compiled `fn:encode-for-uri` expressions for the production XSLT path.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

use super::escape_html_uri_experiment::encode_for_uri;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EncodeForUriExpression {
    Value(EncodeForUriValueExpression),
    Equals {
        value: EncodeForUriValueExpression,
        expected: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EncodeForUriValueExpression {
    Encoded(EncodeForUriArgument),
    Prefixed {
        prefix: String,
        argument: EncodeForUriArgument,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EncodeForUriArgument {
    Empty,
    String(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EncodeForUriValue {
    Boolean(bool),
    String(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EncodeForUriParseFailure {
    InvalidArity,
    InvalidArgumentType,
    Unsupported,
}

impl EncodeForUriExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Value(value) => value.known_owned_capacity_bytes(),
            Self::Equals { value, expected } => {
                value.known_owned_capacity_bytes() + expected.capacity()
            }
        }
    }
}

impl EncodeForUriValueExpression {
    fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Encoded(argument) => argument.known_owned_capacity_bytes(),
            Self::Prefixed { prefix, argument } => {
                prefix.capacity() + argument.known_owned_capacity_bytes()
            }
        }
    }
}

impl EncodeForUriArgument {
    fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::String(value) => value.capacity(),
        }
    }
}

pub(crate) fn recognizes(expression: &str) -> bool {
    expression.contains("encode-for-uri")
}

pub(crate) fn parse(expression: &str) -> Result<EncodeForUriExpression, EncodeForUriParseFailure> {
    let expression = strip_outer_parentheses(expression.trim());
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        let value = parse_value(left)?;
        let expected = parse_quoted(right).ok_or(EncodeForUriParseFailure::Unsupported)?;
        return Ok(EncodeForUriExpression::Equals { value, expected });
    }
    parse_value(expression).map(EncodeForUriExpression::Value)
}

pub(crate) fn evaluate(
    expression: &EncodeForUriExpression,
    control: &mut InvocationControl,
) -> Result<EncodeForUriValue, ControlFailure> {
    control.charge(WorkDomain::XPathOperation, 1)?;
    match expression {
        EncodeForUriExpression::Value(value) => {
            evaluate_value(value, control).map(EncodeForUriValue::String)
        }
        EncodeForUriExpression::Equals { value, expected } => evaluate_value(value, control)
            .map(|actual| EncodeForUriValue::Boolean(actual == *expected)),
    }
}

fn parse_value(expression: &str) -> Result<EncodeForUriValueExpression, EncodeForUriParseFailure> {
    let expression = strip_outer_parentheses(expression.trim());
    if let Some(arguments) = function_argument(expression, &["concat", "fn:concat"]) {
        let (prefix, encoded) =
            split_top_level(arguments, ",").ok_or(EncodeForUriParseFailure::InvalidArity)?;
        if split_top_level(encoded, ",").is_some() {
            return Err(EncodeForUriParseFailure::InvalidArity);
        }
        let prefix = parse_quoted(prefix).ok_or(EncodeForUriParseFailure::Unsupported)?;
        return Ok(EncodeForUriValueExpression::Prefixed {
            prefix,
            argument: parse_call(encoded)?,
        });
    }
    parse_call(expression).map(EncodeForUriValueExpression::Encoded)
}

fn parse_call(expression: &str) -> Result<EncodeForUriArgument, EncodeForUriParseFailure> {
    let argument = function_argument(expression, &["encode-for-uri", "fn:encode-for-uri"])
        .ok_or(EncodeForUriParseFailure::Unsupported)?;
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        return Err(EncodeForUriParseFailure::InvalidArity);
    }
    let argument = argument.trim();
    if argument == "()" {
        return Ok(EncodeForUriArgument::Empty);
    }
    if let Some(value) = parse_quoted(argument) {
        return Ok(EncodeForUriArgument::String(value));
    }
    if argument.parse::<i128>().is_ok() {
        return Err(EncodeForUriParseFailure::InvalidArgumentType);
    }
    Err(EncodeForUriParseFailure::Unsupported)
}

fn evaluate_value(
    expression: &EncodeForUriValueExpression,
    control: &mut InvocationControl,
) -> Result<String, ControlFailure> {
    let (prefix, argument) = match expression {
        EncodeForUriValueExpression::Encoded(argument) => (None, argument),
        EncodeForUriValueExpression::Prefixed { prefix, argument } => (Some(prefix), argument),
    };
    let argument = match argument {
        EncodeForUriArgument::Empty => "",
        EncodeForUriArgument::String(value) => value,
    };
    control.charge(WorkDomain::XPathOperation, argument.chars().count().max(1))?;
    let encoded = encode_for_uri(argument);
    Ok(match prefix {
        Some(prefix) => format!("{prefix}{encoded}"),
        None => encoded,
    })
}

fn function_argument<'a>(expression: &'a str, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .map(str::trim_start)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
            .filter(|inner| balanced(inner))
    })
}

fn parse_quoted(expression: &str) -> Option<String> {
    let expression = expression.trim();
    let quote = expression.chars().next()?;
    if !matches!(quote, '"' | '\'') || !expression.ends_with(quote) {
        return None;
    }
    let inner = &expression[quote.len_utf8()..expression.len() - quote.len_utf8()];
    let doubled = format!("{quote}{quote}");
    let mut remainder = inner;
    let mut value = String::with_capacity(inner.len());
    while let Some(index) = remainder.find(quote) {
        value.push_str(&remainder[..index]);
        if !remainder[index..].starts_with(&doubled) {
            return None;
        }
        value.push(quote);
        remainder = &remainder[index + doubled.len()..];
    }
    value.push_str(remainder);
    Some(value)
}

fn strip_outer_parentheses(mut expression: &str) -> &str {
    while expression.starts_with('(')
        && expression.ends_with(')')
        && balanced(&expression[1..expression.len() - 1])
    {
        expression = expression[1..expression.len() - 1].trim();
    }
    expression
}

fn split_top_level<'a>(expression: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0usize;
    let mut quote = None;
    for (index, character) in expression.char_indices() {
        if matches!(character, '"' | '\'') {
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            _ if depth == 0 && expression[index..].starts_with(separator) => {
                let right = index + separator.len();
                return Some((expression[..index].trim(), expression[right..].trim()));
            }
            _ => {}
        }
    }
    None
}

fn balanced(expression: &str) -> bool {
    let mut depth = 0usize;
    let mut quote = None;
    for character in expression.chars() {
        if matches!(character, '"' | '\'') {
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                let Some(next) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next;
            }
            _ => {}
        }
    }
    depth == 0 && quote.is_none()
}

#[cfg(test)]
mod tests {
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};

    use super::{EncodeForUriParseFailure, EncodeForUriValue, evaluate, parse};

    #[test]
    fn parses_and_evaluates_the_admitted_typed_shapes() {
        for (source, expected) in [
            (
                "(fn:encode-for-uri(\"examples#example\"))",
                EncodeForUriValue::String("examples%23example".to_owned()),
            ),
            (
                "encode-for-uri(()) eq \"\"",
                EncodeForUriValue::Boolean(true),
            ),
            (
                "concat(\"http://www.example.com/\", encode-for-uri(\"~bébé\")) eq \"http://www.example.com/~b%C3%A9b%C3%A9\"",
                EncodeForUriValue::Boolean(true),
            ),
        ] {
            let expression = parse(source).expect("expression should compile");
            let mut control = InvocationControl::unbounded();
            assert_eq!(
                evaluate(&expression, &mut control),
                Ok(expected),
                "{source}"
            );
            assert!(control.consumed(WorkDomain::XPathOperation) > 1);
        }
    }

    #[test]
    fn rejects_static_arity_and_argument_type_errors() {
        assert_eq!(
            parse("encode-for-uri()"),
            Err(EncodeForUriParseFailure::InvalidArity)
        );
        assert_eq!(
            parse("encode-for-uri('',())"),
            Err(EncodeForUriParseFailure::InvalidArity)
        );
        assert_eq!(
            parse("encode-for-uri(12)"),
            Err(EncodeForUriParseFailure::InvalidArgumentType)
        );
    }
}
