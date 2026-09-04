//! Compiled `fn:escape-html-uri` expressions for the production XSLT path.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

use super::escape_html_uri_experiment::escape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EscapeHtmlUriExpression {
    Value(EscapeHtmlUriArgument),
    Equals {
        argument: EscapeHtmlUriArgument,
        expected: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EscapeHtmlUriArgument {
    Empty,
    String(String),
    Codepoints(Vec<u32>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EscapeHtmlUriValue {
    Boolean(bool),
    String(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EscapeHtmlUriParseFailure {
    InvalidArity,
    InvalidArgumentType,
    Unsupported,
}

impl EscapeHtmlUriExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Value(argument) => argument.known_owned_capacity_bytes(),
            Self::Equals { argument, expected } => {
                argument.known_owned_capacity_bytes() + expected.capacity()
            }
        }
    }
}

impl EscapeHtmlUriArgument {
    fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::String(value) => value.capacity(),
            Self::Codepoints(values) => values.capacity() * size_of::<u32>(),
        }
    }
}

pub(crate) fn recognizes(expression: &str) -> bool {
    expression.contains("escape-html-uri")
}

pub(crate) fn parse(
    expression: &str,
) -> Result<EscapeHtmlUriExpression, EscapeHtmlUriParseFailure> {
    let expression = expression.trim();
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        let argument = parse_call(left)?;
        let expected = parse_quoted(right).ok_or(EscapeHtmlUriParseFailure::Unsupported)?;
        return Ok(EscapeHtmlUriExpression::Equals { argument, expected });
    }
    parse_call(expression).map(EscapeHtmlUriExpression::Value)
}

pub(crate) fn evaluate(
    expression: &EscapeHtmlUriExpression,
    control: &mut InvocationControl,
) -> Result<EscapeHtmlUriValue, ControlFailure> {
    control.charge(WorkDomain::XPathOperation, 1)?;
    match expression {
        EscapeHtmlUriExpression::Value(argument) => {
            evaluate_argument(argument, control).map(EscapeHtmlUriValue::String)
        }
        EscapeHtmlUriExpression::Equals { argument, expected } => {
            evaluate_argument(argument, control)
                .map(|actual| EscapeHtmlUriValue::Boolean(actual == *expected))
        }
    }
}

fn evaluate_argument(
    argument: &EscapeHtmlUriArgument,
    control: &mut InvocationControl,
) -> Result<String, ControlFailure> {
    let value = match argument {
        EscapeHtmlUriArgument::Empty => String::new(),
        EscapeHtmlUriArgument::String(value) => value.clone(),
        EscapeHtmlUriArgument::Codepoints(codepoints) => codepoints
            .iter()
            .map(|codepoint| char::from_u32(*codepoint).expect("compiled codepoint is valid"))
            .collect(),
    };
    control.charge(WorkDomain::XPathOperation, value.chars().count().max(1))?;
    Ok(escape(&value))
}

fn parse_call(expression: &str) -> Result<EscapeHtmlUriArgument, EscapeHtmlUriParseFailure> {
    let argument = function_argument(expression, &["escape-html-uri", "fn:escape-html-uri"])
        .ok_or(EscapeHtmlUriParseFailure::Unsupported)?;
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        return Err(EscapeHtmlUriParseFailure::InvalidArity);
    }
    let argument = argument.trim();
    if argument == "()" {
        return Ok(EscapeHtmlUriArgument::Empty);
    }
    if let Some(value) = parse_quoted(argument) {
        return Ok(EscapeHtmlUriArgument::String(value));
    }
    if let Some(codepoints) = parse_codepoints(argument) {
        return Ok(EscapeHtmlUriArgument::Codepoints(codepoints));
    }
    if argument.parse::<i128>().is_ok() {
        return Err(EscapeHtmlUriParseFailure::InvalidArgumentType);
    }
    Err(EscapeHtmlUriParseFailure::Unsupported)
}

fn parse_codepoints(expression: &str) -> Option<Vec<u32>> {
    let inner = function_argument(
        expression,
        &["codepoints-to-string", "fn:codepoints-to-string"],
    )?;
    let inner = inner.trim().strip_prefix('(')?.strip_suffix(')')?;
    inner
        .split(',')
        .map(|lexical| {
            let codepoint = lexical.trim().parse::<u32>().ok()?;
            char::from_u32(codepoint).map(|_| codepoint)
        })
        .collect()
}

fn function_argument<'a>(expression: &'a str, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
            .filter(|inner| balanced(inner))
    })
}

fn parse_quoted(expression: &str) -> Option<String> {
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

    use super::{EscapeHtmlUriParseFailure, EscapeHtmlUriValue, evaluate, parse};

    #[test]
    fn parses_and_evaluates_the_admitted_typed_shapes() {
        for (source, expected) in [
            (
                "fn:escape-html-uri(())",
                EscapeHtmlUriValue::String(String::new()),
            ),
            (
                "escape-html-uri(codepoints-to-string((9, 65, 128)))",
                EscapeHtmlUriValue::String("%09A%C2%80".to_owned()),
            ),
            (
                "escape-html-uri(\"bébé\") eq \"b%C3%A9b%C3%A9\"",
                EscapeHtmlUriValue::Boolean(true),
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
            parse("escape-html-uri()"),
            Err(EscapeHtmlUriParseFailure::InvalidArity)
        );
        assert_eq!(
            parse("escape-html-uri('',())"),
            Err(EscapeHtmlUriParseFailure::InvalidArity)
        );
        assert_eq!(
            parse("escape-html-uri(12)"),
            Err(EscapeHtmlUriParseFailure::InvalidArgumentType)
        );
    }
}
