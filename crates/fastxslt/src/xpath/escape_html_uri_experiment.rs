//! Bounded constant folding for the `XPath` `escape-html-uri` function.

#[cfg(test)]
use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EscapeHtmlUriValue {
    Boolean(bool),
    String(String),
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EscapeHtmlUriFailure {
    Control(ControlFailure),
    InvalidArity,
    InvalidArgumentType,
    Unsupported,
}

pub(crate) fn fold_literal(expression: &str) -> Option<String> {
    let argument = expression
        .trim()
        .strip_prefix("escape-html-uri(")?
        .strip_suffix(')')?
        .trim();
    let literal = argument.strip_prefix('\'')?.strip_suffix('\'')?;
    if literal.contains('\'') {
        return None;
    }
    Some(escape(literal))
}

fn escape(literal: &str) -> String {
    let mut escaped = String::with_capacity(literal.len());
    for character in literal.chars() {
        if ('\u{20}'..='\u{7e}').contains(&character) {
            escaped.push(character);
        } else {
            let mut encoded = [0_u8; 4];
            for byte in character.encode_utf8(&mut encoded).as_bytes() {
                escaped.push('%');
                escaped.push(hex_digit(byte >> 4));
                escaped.push(hex_digit(byte & 0x0f));
            }
        }
    }
    escaped
}

#[cfg(test)]
pub(crate) fn evaluate(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<EscapeHtmlUriValue, EscapeHtmlUriFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(EscapeHtmlUriFailure::Control)?;
    let expression = expression.trim();
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        return Ok(EscapeHtmlUriValue::Boolean(
            evaluate_string(left, control)? == evaluate_string(right, control)?,
        ));
    }
    evaluate_string(expression, control).map(EscapeHtmlUriValue::String)
}

#[cfg(test)]
fn evaluate_string(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<String, EscapeHtmlUriFailure> {
    let expression = expression.trim();
    if let Some(value) = parse_quoted(expression) {
        return Ok(value.to_owned());
    }
    let argument = function_argument(expression, &["escape-html-uri", "fn:escape-html-uri"])
        .ok_or(EscapeHtmlUriFailure::Unsupported)?;
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        return Err(EscapeHtmlUriFailure::InvalidArity);
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(EscapeHtmlUriFailure::Control)?;
    let value = if argument.trim() == "()" {
        String::new()
    } else if let Some(value) = parse_quoted(argument.trim()) {
        value.to_owned()
    } else if let Some(value) = parse_codepoints_to_string(argument.trim()) {
        value
    } else if argument.trim().parse::<i128>().is_ok() {
        return Err(EscapeHtmlUriFailure::InvalidArgumentType);
    } else {
        return Err(EscapeHtmlUriFailure::Unsupported);
    };
    Ok(escape(&value))
}

#[cfg(test)]
fn parse_codepoints_to_string(expression: &str) -> Option<String> {
    let inner = function_argument(
        expression,
        &["codepoints-to-string", "fn:codepoints-to-string"],
    )?;
    let inner = inner.trim().strip_prefix('(')?.strip_suffix(')')?;
    let mut result = String::new();
    for lexical in inner.split(',') {
        let codepoint = lexical.trim().parse::<u32>().ok()?;
        result.push(char::from_u32(codepoint)?);
    }
    Some(result)
}

#[cfg(test)]
fn function_argument<'a>(expression: &'a str, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
            .filter(|inner| balanced(inner))
    })
}

#[cfg(test)]
fn parse_quoted(expression: &str) -> Option<&str> {
    for quote in ['"', '\''] {
        if let Some(value) = expression
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            .filter(|value| !value.contains(quote))
        {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
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

#[cfg(test)]
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

fn hex_digit(value: u8) -> char {
    char::from(if value < 10 {
        b'0' + value
    } else {
        b'A' + value - 10
    })
}

#[cfg(test)]
mod tests {
    use super::{EscapeHtmlUriFailure, EscapeHtmlUriValue, evaluate, fold_literal};
    use crate::execution_control_experiment::InvocationControl;

    #[test]
    fn escapes_non_ascii_utf8_without_unicode_normalization() {
        assert_eq!(
            fold_literal("escape-html-uri('http://example/\u{fb4f}/\u{e5}/a\u{30a}')").as_deref(),
            Some("http://example/%EF%AD%8F/%C3%A5/a%CC%8A")
        );
        assert_eq!(fold_literal("escape-html-uri($uri)"), None);
    }

    #[test]
    fn evaluates_empty_unicode_comparison_and_type_boundaries() {
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
            assert_eq!(
                evaluate(source, &mut InvocationControl::unbounded()),
                Ok(expected),
                "{source}"
            );
        }
        assert_eq!(
            evaluate("escape-html-uri(12)", &mut InvocationControl::unbounded()),
            Err(EscapeHtmlUriFailure::InvalidArgumentType)
        );
        assert_eq!(
            evaluate("escape-html-uri()", &mut InvocationControl::unbounded()),
            Err(EscapeHtmlUriFailure::InvalidArity)
        );
    }
}
