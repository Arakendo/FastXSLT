//! Bounded source-free case-conversion semantics used by QT3 evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CaseValue {
    Boolean(bool),
    Empty,
    Integer(i128),
    Integers(Vec<u32>),
    String(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaseFailure {
    Control(ControlFailure),
    InvalidArity,
    InvalidArgumentType,
    MissingContext,
    Unsupported,
}

pub(crate) fn evaluate(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<CaseValue, CaseFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(CaseFailure::Control)?;
    evaluate_value(expression.trim(), control)
}

fn evaluate_value(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<CaseValue, CaseFailure> {
    let expression = strip_outer_parentheses(expression.trim());
    if let Some(rest) = expression.strip_prefix("if(false()) then ") {
        let (_, otherwise) = split_top_level(rest, " else ").ok_or(CaseFailure::Unsupported)?;
        return evaluate_value(otherwise, control);
    }
    if let Some((left, right)) = split_top_level(expression, " and ") {
        return Ok(CaseValue::Boolean(
            effective_boolean(&evaluate_value(left, control)?)
                && effective_boolean(&evaluate_value(right, control)?),
        ));
    }
    if let Some((left, right)) = split_top_level(expression, " or ") {
        return Ok(CaseValue::Boolean(
            effective_boolean(&evaluate_value(left, control)?)
                || effective_boolean(&evaluate_value(right, control)?),
        ));
    }
    if let Some((left, right)) = split_top_level(expression, " eq ") {
        return Ok(CaseValue::Boolean(
            evaluate_value(left, control)? == evaluate_value(right, control)?,
        ));
    }
    if let Some(value) = parse_quoted(expression) {
        return Ok(CaseValue::String(value));
    }
    if let Ok(value) = expression.parse::<i128>() {
        return Ok(CaseValue::Integer(value));
    }

    let (name, argument) = parse_function(expression).ok_or(CaseFailure::Unsupported)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(CaseFailure::Control)?;
    evaluate_function(name, argument, control)
}

fn evaluate_function(
    name: &str,
    argument: &str,
    control: &mut InvocationControl,
) -> Result<CaseValue, CaseFailure> {
    match name {
        "lower-case" | "fn:lower-case" => {
            let value = optional_string_argument(argument, control)?;
            Ok(CaseValue::String(
                value.chars().flat_map(char::to_lowercase).collect(),
            ))
        }
        "upper-case" | "fn:upper-case" => {
            let value = optional_string_argument(argument, control)?;
            Ok(CaseValue::String(
                value.chars().flat_map(char::to_uppercase).collect(),
            ))
        }
        "string" | "fn:string" | "xs:string" => {
            require_one_argument(argument)?;
            Ok(CaseValue::String(
                match evaluate_value(argument, control)? {
                    CaseValue::String(value) => value,
                    CaseValue::Integer(value) => value.to_string(),
                    _ => return Err(CaseFailure::Unsupported),
                },
            ))
        }
        "xs:integer" => {
            require_one_argument(argument)?;
            let lexical =
                parse_quoted(argument.trim()).unwrap_or_else(|| argument.trim().to_owned());
            let value = lexical
                .parse::<i128>()
                .map_err(|_| CaseFailure::Unsupported)?;
            Ok(CaseValue::Integer(value))
        }
        "count" | "fn:count" => {
            require_one_argument(argument)?;
            let count = match evaluate_value(argument, control)? {
                CaseValue::Integers(values) => values.len(),
                _ => 1,
            };
            Ok(CaseValue::Integer(count as i128))
        }
        "boolean" | "fn:boolean" | "xs:boolean" => {
            require_one_argument(argument)?;
            Ok(CaseValue::Boolean(effective_boolean(&evaluate_value(
                argument, control,
            )?)))
        }
        "not" | "fn:not" => {
            require_one_argument(argument)?;
            Ok(CaseValue::Boolean(!effective_boolean(&evaluate_value(
                argument, control,
            )?)))
        }
        "concat" | "fn:concat" => {
            let (left, right) = exactly_two_arguments(argument)?;
            let left = require_string(evaluate_value(left, control)?)?;
            let right = require_string(evaluate_value(right, control)?)?;
            Ok(CaseValue::String(format!("{left}{right}")))
        }
        "codepoints-to-string" | "fn:codepoints-to-string" => {
            let values = parse_codepoint_sequence(argument)?;
            let mut string = String::new();
            for value in values {
                string.push(char::from_u32(value).ok_or(CaseFailure::Unsupported)?);
            }
            Ok(CaseValue::String(string))
        }
        "string-to-codepoints" | "fn:string-to-codepoints" => {
            require_one_argument(argument)?;
            let value = require_string(evaluate_value(argument, control)?)?;
            Ok(CaseValue::Integers(value.chars().map(u32::from).collect()))
        }
        "codepoint-equal" | "fn:codepoint-equal" => {
            let (left, right) = exactly_two_arguments(argument)?;
            let left = optional_comparison_string(left, control)?;
            let right = optional_comparison_string(right, control)?;
            match (left, right) {
                (Some(left), Some(right)) => Ok(CaseValue::Boolean(left == right)),
                _ => Ok(CaseValue::Empty),
            }
        }
        "normalize-space" | "fn:normalize-space" => {
            if argument.trim().is_empty() {
                return Err(CaseFailure::MissingContext);
            }
            let value = optional_string_argument(argument, control)?;
            Ok(CaseValue::String(normalize_xml_space(&value)))
        }
        "true" | "fn:true" if argument.trim().is_empty() => Ok(CaseValue::Boolean(true)),
        "false" | "fn:false" if argument.trim().is_empty() => Ok(CaseValue::Boolean(false)),
        _ => Err(CaseFailure::Unsupported),
    }
}

fn normalize_xml_space(value: &str) -> String {
    value
        .split(['\u{9}', '\u{a}', '\u{d}', '\u{20}'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn optional_comparison_string(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<Option<String>, CaseFailure> {
    if expression.trim() == "()" {
        return Ok(None);
    }
    match evaluate_value(expression, control)? {
        CaseValue::String(value) => Ok(Some(value)),
        _ => Err(CaseFailure::InvalidArgumentType),
    }
}

fn optional_string_argument(
    argument: &str,
    control: &mut InvocationControl,
) -> Result<String, CaseFailure> {
    require_one_argument(argument)?;
    if argument.trim() == "()" {
        return Ok(String::new());
    }
    require_string(evaluate_value(argument, control)?)
}

fn require_string(value: CaseValue) -> Result<String, CaseFailure> {
    match value {
        CaseValue::String(value) => Ok(value),
        _ => Err(CaseFailure::Unsupported),
    }
}

fn effective_boolean(value: &CaseValue) -> bool {
    match value {
        CaseValue::Boolean(value) => *value,
        CaseValue::Empty => false,
        CaseValue::Integer(value) => *value != 0,
        CaseValue::Integers(values) => !values.is_empty(),
        CaseValue::String(value) => !value.is_empty(),
    }
}

fn require_one_argument(argument: &str) -> Result<(), CaseFailure> {
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        Err(CaseFailure::InvalidArity)
    } else {
        Ok(())
    }
}

fn exactly_two_arguments(argument: &str) -> Result<(&str, &str), CaseFailure> {
    let (left, right) = split_top_level(argument, ",").ok_or(CaseFailure::InvalidArity)?;
    if split_top_level(right, ",").is_some() {
        Err(CaseFailure::InvalidArity)
    } else {
        Ok((left, right))
    }
}

fn parse_codepoint_sequence(argument: &str) -> Result<Vec<u32>, CaseFailure> {
    if let Some((start, end)) = split_top_level(argument, " to ") {
        let start = start.parse::<u32>().map_err(|_| CaseFailure::Unsupported)?;
        let end = end.parse::<u32>().map_err(|_| CaseFailure::Unsupported)?;
        return Ok((start..=end).collect());
    }
    Ok(vec![
        argument
            .trim()
            .parse::<u32>()
            .map_err(|_| CaseFailure::Unsupported)?,
    ])
}

fn parse_function(expression: &str) -> Option<(&str, &str)> {
    let open = expression.find('(')?;
    let name = expression[..open].trim();
    if name.is_empty() || !expression.ends_with(')') {
        return None;
    }
    let argument = &expression[open + 1..expression.len() - 1];
    balanced(argument).then_some((name, argument))
}

fn parse_quoted(expression: &str) -> Option<String> {
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
    use super::{CaseValue, evaluate};
    use crate::execution_control_experiment::InvocationControl;

    #[test]
    fn evaluates_composed_case_conversion_and_expanding_mapping() {
        assert_eq!(
            evaluate(
                "fn:concat(fn:lower-case(\"AB\"), fn:lower-case(\"Cd\"))",
                &mut InvocationControl::unbounded(),
            ),
            Ok(CaseValue::String("abcd".to_owned()))
        );
        assert_eq!(
            evaluate(
                "fn:string-to-codepoints(fn:lower-case(fn:codepoints-to-string(304)))",
                &mut InvocationControl::unbounded(),
            ),
            Ok(CaseValue::Integers(vec![105, 775]))
        );
    }
}
