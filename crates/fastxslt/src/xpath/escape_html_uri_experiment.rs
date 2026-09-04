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

pub(crate) fn escape(literal: &str) -> String {
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
pub(crate) fn evaluate_encode_for_uri(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<EscapeHtmlUriValue, EscapeHtmlUriFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(EscapeHtmlUriFailure::Control)?;
    let expression = expression.trim();
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        return Ok(EscapeHtmlUriValue::Boolean(
            evaluate_encode_string(left, control)? == evaluate_encode_string(right, control)?,
        ));
    }
    evaluate_encode_string(expression, control).map(EscapeHtmlUriValue::String)
}

#[cfg(test)]
pub(crate) fn evaluate_iri_to_uri(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<EscapeHtmlUriValue, EscapeHtmlUriFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(EscapeHtmlUriFailure::Control)?;
    let expression = expression.trim();
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        return Ok(EscapeHtmlUriValue::Boolean(
            evaluate_iri_string(left, control)? == evaluate_iri_string(right, control)?,
        ));
    }
    evaluate_iri_string(expression, control).map(EscapeHtmlUriValue::String)
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
fn evaluate_encode_string(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<String, EscapeHtmlUriFailure> {
    let expression = strip_outer_parentheses(expression.trim());
    if let Some(value) = parse_quoted(expression) {
        return Ok(value.to_owned());
    }
    if let Some(arguments) = function_argument(expression, &["concat", "fn:concat"]) {
        let (left, right) =
            split_top_level(arguments, ",").ok_or(EscapeHtmlUriFailure::InvalidArity)?;
        if split_top_level(right, ",").is_some() {
            return Err(EscapeHtmlUriFailure::InvalidArity);
        }
        return Ok(format!(
            "{}{}",
            evaluate_encode_string(left, control)?,
            evaluate_encode_string(right, control)?
        ));
    }
    let argument = function_argument(expression, &["encode-for-uri", "fn:encode-for-uri"])
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
    } else if argument.trim().parse::<i128>().is_ok() {
        return Err(EscapeHtmlUriFailure::InvalidArgumentType);
    } else {
        return Err(EscapeHtmlUriFailure::Unsupported);
    };
    Ok(encode_for_uri(&value))
}

#[cfg(test)]
fn encode_for_uri(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(*byte));
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
    }
    encoded
}

#[cfg(test)]
fn evaluate_iri_string(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<String, EscapeHtmlUriFailure> {
    let expression = strip_outer_parentheses(expression.trim());
    if let Some(value) = parse_xpath_quoted(expression) {
        return Ok(value);
    }
    let argument = function_argument(expression, &["iri-to-uri", "fn:iri-to-uri"])
        .ok_or(EscapeHtmlUriFailure::Unsupported)?;
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        return Err(EscapeHtmlUriFailure::InvalidArity);
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(EscapeHtmlUriFailure::Control)?;
    let argument = argument.trim();
    let value = if argument == "()" {
        String::new()
    } else if let Some(value) = parse_xpath_quoted(argument) {
        value
    } else if let Some(value) = parse_string_constructor(argument) {
        value
    } else if let Some(value) = parse_codepoint_range(argument) {
        value
    } else if argument.parse::<i128>().is_ok() || is_multi_item_sequence(argument) {
        return Err(EscapeHtmlUriFailure::InvalidArgumentType);
    } else {
        return Err(EscapeHtmlUriFailure::Unsupported);
    };
    Ok(iri_to_uri(&value))
}

#[cfg(test)]
fn parse_string_constructor(expression: &str) -> Option<String> {
    let argument = function_argument(expression, &["xs:anyURI", "xs:untypedAtomic"])?;
    parse_xpath_quoted(argument.trim())
}

#[cfg(test)]
fn parse_codepoint_range(expression: &str) -> Option<String> {
    let argument = function_argument(
        expression,
        &["codepoints-to-string", "fn:codepoints-to-string"],
    )?;
    let (start, end) = split_top_level(argument, " to ")?;
    let start = start.parse::<u32>().ok()?;
    let end = end.parse::<u32>().ok()?;
    let mut value = String::new();
    for codepoint in start..=end {
        value.push(char::from_u32(codepoint)?);
    }
    Some(value)
}

#[cfg(test)]
fn is_multi_item_sequence(expression: &str) -> bool {
    expression
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .is_some_and(|value| split_top_level(value, ",").is_some())
}

#[cfg(test)]
fn parse_xpath_quoted(expression: &str) -> Option<String> {
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

#[cfg(test)]
fn iri_to_uri(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        if (b'!'..=b'~').contains(byte)
            && !matches!(
                byte,
                b'"' | b'<' | b'>' | b'\\' | b'^' | b'`' | b'{' | b'|' | b'}'
            )
        {
            encoded.push(char::from(*byte));
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4));
            encoded.push(hex_digit(byte & 0x0f));
        }
    }
    encoded
}

#[cfg(test)]
fn strip_outer_parentheses(mut expression: &str) -> &str {
    while expression.starts_with('(')
        && expression.ends_with(')')
        && balanced(&expression[1..expression.len() - 1])
    {
        expression = expression[1..expression.len() - 1].trim();
    }
    expression
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
            .map(str::trim_start)
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
    use super::{
        EscapeHtmlUriFailure, EscapeHtmlUriValue, evaluate, evaluate_encode_for_uri,
        evaluate_iri_to_uri, fold_literal,
    };
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

    #[test]
    fn evaluates_encode_for_uri_unreserved_and_composed_values() {
        for (source, expected) in [
            (
                "encode-for-uri(\"100% organic\")",
                EscapeHtmlUriValue::String("100%25%20organic".to_owned()),
            ),
            (
                "(fn:encode-for-uri(\"examples~example\"))",
                EscapeHtmlUriValue::String("examples~example".to_owned()),
            ),
            (
                "concat(\"http://www.example.com/\", encode-for-uri(\"bébé\")) eq \"http://www.example.com/b%C3%A9b%C3%A9\"",
                EscapeHtmlUriValue::Boolean(true),
            ),
        ] {
            assert_eq!(
                evaluate_encode_for_uri(source, &mut InvocationControl::unbounded()),
                Ok(expected),
                "{source}"
            );
        }
    }

    #[test]
    fn evaluates_iri_to_uri_preserved_and_encoded_values() {
        for (source, expected) in [
            (
                "iri-to-uri(\"http://example/a%20b#c\")",
                EscapeHtmlUriValue::String("http://example/a%20b#c".to_owned()),
            ),
            (
                "iri-to-uri(\"<> \"\"{}|\\^`\")",
                EscapeHtmlUriValue::String("%3C%3E%20%22%7B%7D%7C%5C%5E%60".to_owned()),
            ),
            (
                "iri-to-uri(xs:anyURI(\"a string\"))",
                EscapeHtmlUriValue::String("a%20string".to_owned()),
            ),
        ] {
            assert_eq!(
                evaluate_iri_to_uri(source, &mut InvocationControl::unbounded()),
                Ok(expected),
                "{source}"
            );
        }
    }
}
