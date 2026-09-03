//! Private source-free `fn:string-length` seam for executable QT3 evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StringLengthValue {
    Empty,
    Boolean(bool),
    Integer(usize),
    String(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StringLengthFailure {
    Control(ControlFailure),
    InvalidArity,
    MissingContext,
    Unsupported,
}

pub(crate) fn evaluate(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<StringLengthValue, StringLengthFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(StringLengthFailure::Control)?;
    evaluate_inner(expression.trim(), control)
}

fn evaluate_inner(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<StringLengthValue, StringLengthFailure> {
    if let Some((condition, when_true, when_false)) = parse_if(expression) {
        return if effective_boolean_value(&evaluate(condition, control)?) {
            evaluate(when_true, control)
        } else {
            evaluate(when_false, control)
        };
    }
    if let Some((left, right)) = split_top_level(expression, " and ") {
        let left = effective_boolean_value(&evaluate(left, control)?);
        if !left {
            return Ok(StringLengthValue::Boolean(false));
        }
        return Ok(StringLengthValue::Boolean(effective_boolean_value(
            &evaluate(right, control)?,
        )));
    }
    if let Some(operand) = expression.strip_suffix(" instance of xs:integer") {
        return evaluate(operand, control).map(|value| {
            StringLengthValue::Boolean(matches!(value, StringLengthValue::Integer(_)))
        });
    }
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        return Ok(StringLengthValue::Boolean(
            evaluate(left, control)? == evaluate(right, control)?,
        ));
    }
    if let Some((left, right)) = split_top_level(expression, " + ") {
        let left = integer_value(&evaluate(left, control)?)?;
        let right = integer_value(&evaluate(right, control)?)?;
        return left
            .checked_add(right)
            .map(StringLengthValue::Integer)
            .ok_or(StringLengthFailure::Unsupported);
    }
    if let Some(argument) = function_argument(expression, &["string-length", "fn:string-length"]) {
        if argument.trim().is_empty() {
            return Err(StringLengthFailure::MissingContext);
        }
        if split_top_level(argument, ",").is_some() {
            return Err(StringLengthFailure::InvalidArity);
        }
        let value = evaluate(argument, control)?;
        let string = optional_string_value(value)?;
        return Ok(StringLengthValue::Integer(string.chars().count()));
    }
    if let Some(argument) = function_argument(expression, &["string", "fn:string", "xs:string"]) {
        if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
            return Err(StringLengthFailure::InvalidArity);
        }
        return Ok(StringLengthValue::String(string_value(evaluate(
            argument, control,
        )?)));
    }
    if let Some(arguments) = function_argument(expression, &["concat", "fn:concat"]) {
        let (left, right) =
            split_top_level(arguments, ",").ok_or(StringLengthFailure::InvalidArity)?;
        if split_top_level(right, ",").is_some() {
            return Err(StringLengthFailure::InvalidArity);
        }
        let mut result = string_value(evaluate(left, control)?);
        result.push_str(&string_value(evaluate(right, control)?));
        return Ok(StringLengthValue::String(result));
    }
    if let Some(argument) = function_argument(expression, &["boolean", "fn:boolean"]) {
        if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
            return Err(StringLengthFailure::InvalidArity);
        }
        return Ok(StringLengthValue::Boolean(effective_boolean_value(
            &evaluate(argument, control)?,
        )));
    }
    if let Some(argument) = function_argument(expression, &["not", "fn:not"]) {
        if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
            return Err(StringLengthFailure::InvalidArity);
        }
        return Ok(StringLengthValue::Boolean(!effective_boolean_value(
            &evaluate(argument, control)?,
        )));
    }
    if expression == "()" {
        return Ok(StringLengthValue::Empty);
    }
    if matches!(expression, "true()" | "fn:true()") {
        return Ok(StringLengthValue::Boolean(true));
    }
    if matches!(expression, "false()" | "fn:false()") {
        return Ok(StringLengthValue::Boolean(false));
    }
    if let Some(value) = parse_quoted_string(expression) {
        return Ok(StringLengthValue::String(value));
    }
    expression
        .parse::<usize>()
        .map(StringLengthValue::Integer)
        .map_err(|_| StringLengthFailure::Unsupported)
}

fn optional_string_value(value: StringLengthValue) -> Result<String, StringLengthFailure> {
    match value {
        StringLengthValue::Empty => Ok(String::new()),
        StringLengthValue::String(value) => Ok(value),
        StringLengthValue::Boolean(_) | StringLengthValue::Integer(_) => {
            Err(StringLengthFailure::Unsupported)
        }
    }
}

fn string_value(value: StringLengthValue) -> String {
    match value {
        StringLengthValue::Empty => String::new(),
        StringLengthValue::Boolean(value) => if value { "true" } else { "false" }.to_owned(),
        StringLengthValue::Integer(value) => value.to_string(),
        StringLengthValue::String(value) => value,
    }
}

fn integer_value(value: &StringLengthValue) -> Result<usize, StringLengthFailure> {
    match value {
        StringLengthValue::Integer(value) => Ok(*value),
        _ => Err(StringLengthFailure::Unsupported),
    }
}

fn effective_boolean_value(value: &StringLengthValue) -> bool {
    match value {
        StringLengthValue::Empty => false,
        StringLengthValue::Boolean(value) => *value,
        StringLengthValue::Integer(value) => *value != 0,
        StringLengthValue::String(value) => !value.is_empty(),
    }
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

fn parse_quoted_string(expression: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(value) = expression
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            .filter(|value| !value.contains(quote))
        {
            return Some(value.to_owned());
        }
    }
    None
}

fn parse_if(expression: &str) -> Option<(&str, &str, &str)> {
    let tail = expression.strip_prefix("if(")?;
    let mut depth = 1usize;
    let mut quote = None;
    for (offset, character) in tail.char_indices() {
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
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    let condition = &tail[..offset];
                    let remainder = tail[offset + 1..].strip_prefix(" then ")?;
                    let (when_true, when_false) = split_top_level(remainder, " else ")?;
                    return Some((condition, when_true, when_false));
                }
            }
            _ => {}
        }
    }
    None
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
            ')' => {
                let next = depth.checked_sub(1)?;
                depth = next;
            }
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
    use super::{StringLengthFailure, StringLengthValue, evaluate};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};

    #[test]
    fn evaluates_typed_string_length_composition_and_lazy_conditionals() {
        for (source, expected) in [
            ("string-length(\"abc\")", StringLengthValue::Integer(3)),
            ("string-length(())", StringLengthValue::Integer(0)),
            (
                "string-length(\"abc\") + string-length(\"de\")",
                StringLengthValue::Integer(5),
            ),
            (
                "concat(string-length(\"abc\"), string-length(\"de\"))",
                StringLengthValue::String("32".to_owned()),
            ),
            (
                "if(false()) then string-length() else true()",
                StringLengthValue::Boolean(true),
            ),
        ] {
            let mut control = InvocationControl::unbounded();
            assert_eq!(evaluate(source, &mut control), Ok(expected), "{source}");
            assert!(control.consumed(WorkDomain::XPathOperation) > 0);
        }
    }

    #[test]
    fn distinguishes_missing_context_and_invalid_arity() {
        assert_eq!(
            evaluate("string-length()", &mut InvocationControl::unbounded()),
            Err(StringLengthFailure::MissingContext)
        );
        assert_eq!(
            evaluate(
                "string-length(\"one\", \"two\")",
                &mut InvocationControl::unbounded()
            ),
            Err(StringLengthFailure::InvalidArity)
        );
    }
}
