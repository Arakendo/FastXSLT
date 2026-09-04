//! Compiled `fn:iri-to-uri` expressions for the production XSLT path.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

use super::escape_html_uri_experiment::iri_to_uri;

const MAX_LITERAL_CODEPOINT_RANGE: u32 = 4_096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IriToUriExpression {
    Value(IriToUriArgument),
    Equals {
        argument: IriToUriArgument,
        expected: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IriToUriArgument {
    Empty,
    String(String),
    CodepointRange { start: u32, end: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IriToUriValue {
    Boolean(bool),
    String(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IriToUriParseFailure {
    InvalidArity,
    InvalidArgumentType,
    Unsupported,
}

impl IriToUriExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Value(argument) => argument.known_owned_capacity_bytes(),
            Self::Equals { argument, expected } => {
                argument.known_owned_capacity_bytes() + expected.capacity()
            }
        }
    }
}

impl IriToUriArgument {
    fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::String(value) => value.capacity(),
            Self::Empty | Self::CodepointRange { .. } => 0,
        }
    }
}

pub(crate) fn recognizes(expression: &str) -> bool {
    expression.contains("iri-to-uri")
}

pub(crate) fn parse(expression: &str) -> Result<IriToUriExpression, IriToUriParseFailure> {
    let expression = expression.trim();
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        let argument = parse_call(left)?;
        let expected = parse_quoted(right).ok_or(IriToUriParseFailure::Unsupported)?;
        return Ok(IriToUriExpression::Equals { argument, expected });
    }
    parse_call(expression).map(IriToUriExpression::Value)
}

pub(crate) fn evaluate(
    expression: &IriToUriExpression,
    control: &mut InvocationControl,
) -> Result<IriToUriValue, ControlFailure> {
    control.charge(WorkDomain::XPathOperation, 1)?;
    match expression {
        IriToUriExpression::Value(argument) => {
            evaluate_argument(argument, control).map(IriToUriValue::String)
        }
        IriToUriExpression::Equals { argument, expected } => evaluate_argument(argument, control)
            .map(|actual| IriToUriValue::Boolean(actual == *expected)),
    }
}

fn parse_call(expression: &str) -> Result<IriToUriArgument, IriToUriParseFailure> {
    let argument = function_argument(expression.trim(), &["iri-to-uri", "fn:iri-to-uri"])
        .ok_or(IriToUriParseFailure::Unsupported)?;
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        return Err(IriToUriParseFailure::InvalidArity);
    }
    let argument = argument.trim();
    if argument == "()" {
        return Ok(IriToUriArgument::Empty);
    }
    if let Some(value) = parse_quoted(argument) {
        return Ok(IriToUriArgument::String(value));
    }
    if let Some(value) = parse_string_constructor(argument) {
        return Ok(IriToUriArgument::String(value));
    }
    if let Some((start, end)) = parse_codepoint_range(argument) {
        return Ok(IriToUriArgument::CodepointRange { start, end });
    }
    if argument.parse::<i128>().is_ok() || is_multi_item_sequence(argument) {
        return Err(IriToUriParseFailure::InvalidArgumentType);
    }
    Err(IriToUriParseFailure::Unsupported)
}

fn evaluate_argument(
    argument: &IriToUriArgument,
    control: &mut InvocationControl,
) -> Result<String, ControlFailure> {
    match argument {
        IriToUriArgument::Empty => {
            control.charge(WorkDomain::XPathOperation, 1)?;
            Ok(String::new())
        }
        IriToUriArgument::String(value) => {
            control.charge(WorkDomain::XPathOperation, value.chars().count().max(1))?;
            Ok(iri_to_uri(value))
        }
        IriToUriArgument::CodepointRange { start, end } => {
            let count = usize::try_from(end - start + 1).unwrap_or(usize::MAX);
            control.charge(WorkDomain::XPathOperation, count)?;
            let mut value = String::new();
            for codepoint in *start..=*end {
                value.push(char::from_u32(codepoint).expect("compiled codepoint is valid"));
            }
            Ok(iri_to_uri(&value))
        }
    }
}

fn parse_string_constructor(expression: &str) -> Option<String> {
    let argument = function_argument(expression, &["xs:anyURI", "xs:untypedAtomic"])?;
    parse_quoted(argument)
}

fn parse_codepoint_range(expression: &str) -> Option<(u32, u32)> {
    let argument = function_argument(
        expression,
        &["codepoints-to-string", "fn:codepoints-to-string"],
    )?;
    let (start, end) = split_top_level(argument, " to ")?;
    let start = start.parse::<u32>().ok()?;
    let end = end.parse::<u32>().ok()?;
    let count = end.checked_sub(start)?.checked_add(1)?;
    (count <= MAX_LITERAL_CODEPOINT_RANGE
        && (start..=end).all(|value| char::from_u32(value).is_some()))
    .then_some((start, end))
}

fn is_multi_item_sequence(expression: &str) -> bool {
    expression
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .is_some_and(|value| split_top_level(value, ",").is_some())
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

    use super::{IriToUriParseFailure, IriToUriValue, evaluate, parse};

    #[test]
    fn parses_and_evaluates_the_admitted_typed_shapes() {
        for (source, expected) in [
            (
                "fn:iri-to-uri(\"example example\")",
                IriToUriValue::String("example%20example".to_owned()),
            ),
            (
                "iri-to-uri(xs:anyURI(\"a string\"))",
                IriToUriValue::String("a%20string".to_owned()),
            ),
            ("iri-to-uri(()) eq \"\"", IriToUriValue::Boolean(true)),
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
            parse("iri-to-uri()"),
            Err(IriToUriParseFailure::InvalidArity)
        );
        assert_eq!(
            parse("iri-to-uri(('a', 'b'))"),
            Err(IriToUriParseFailure::InvalidArgumentType)
        );
        assert_eq!(
            parse("iri-to-uri(1)"),
            Err(IriToUriParseFailure::InvalidArgumentType)
        );
        assert_eq!(
            parse("iri-to-uri(codepoints-to-string(55295 to 55297))"),
            Err(IriToUriParseFailure::Unsupported)
        );
    }
}
