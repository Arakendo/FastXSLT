//! Private source-free `fn:string-length` seam for executable QT3 evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, SourceLocation};

use super::path_experiment::{PathFailure, evaluate_location_path_controlled, parse_location_path};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StringLengthValue {
    Empty,
    Boolean(bool),
    Integer(usize),
    String(String),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum StringLengthFailure {
    Control(ControlFailure),
    InvalidArity,
    InvalidArgumentType,
    MissingContext,
    Path(PathFailure),
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
    if let Some((start, end, expected_length, explicit_context_argument)) =
        parse_range_length_filter(expression)
    {
        if start > end {
            return Ok(StringLengthValue::Empty);
        }
        let item_count = end
            .checked_sub(start)
            .and_then(|distance| distance.checked_add(1))
            .ok_or(StringLengthFailure::Unsupported)?;
        control
            .charge(WorkDomain::XPathOperation, item_count)
            .map_err(StringLengthFailure::Control)?;
        if explicit_context_argument {
            return Err(StringLengthFailure::InvalidArgumentType);
        }
        let selected = (start..=end)
            .filter(|value| value.to_string().chars().count() == expected_length)
            .collect::<Vec<_>>();
        return match selected.as_slice() {
            [] => Ok(StringLengthValue::Empty),
            [value] => Ok(StringLengthValue::Integer(*value)),
            _ => Err(StringLengthFailure::Unsupported),
        };
    }
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
    if let Some(result) = evaluate_named_function(expression, control) {
        return result;
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

fn evaluate_named_function(
    expression: &str,
    control: &mut InvocationControl,
) -> Option<Result<StringLengthValue, StringLengthFailure>> {
    if let Some(argument) = function_argument(expression, &["string-length", "fn:string-length"]) {
        if argument.trim().is_empty() {
            return Some(Err(StringLengthFailure::MissingContext));
        }
        if split_top_level(argument, ",").is_some() {
            return Some(Err(StringLengthFailure::InvalidArity));
        }
        return Some(evaluate(argument, control).and_then(|value| {
            optional_string_value(value)
                .map(|string| StringLengthValue::Integer(string.chars().count()))
        }));
    }
    if let Some(argument) = function_argument(expression, &["string", "fn:string", "xs:string"]) {
        if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
            return Some(Err(StringLengthFailure::InvalidArity));
        }
        return Some(
            evaluate(argument, control)
                .map(string_value)
                .map(StringLengthValue::String),
        );
    }
    if let Some(arguments) = function_argument(expression, &["concat", "fn:concat"]) {
        let Some((left, right)) = split_top_level(arguments, ",") else {
            return Some(Err(StringLengthFailure::InvalidArity));
        };
        if split_top_level(right, ",").is_some() {
            return Some(Err(StringLengthFailure::InvalidArity));
        }
        return Some(evaluate(left, control).and_then(|left| {
            evaluate(right, control).map(|right| {
                let mut result = string_value(left);
                result.push_str(&string_value(right));
                StringLengthValue::String(result)
            })
        }));
    }
    if let Some(argument) = function_argument(expression, &["boolean", "fn:boolean"]) {
        return Some(evaluate_unary_boolean(argument, control, false));
    }
    function_argument(expression, &["not", "fn:not"])
        .map(|argument| evaluate_unary_boolean(argument, control, true))
}

fn evaluate_unary_boolean(
    argument: &str,
    control: &mut InvocationControl,
    negate: bool,
) -> Result<StringLengthValue, StringLengthFailure> {
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        return Err(StringLengthFailure::InvalidArity);
    }
    evaluate(argument, control)
        .map(|value| StringLengthValue::Boolean(effective_boolean_value(&value) ^ negate))
}

pub(crate) fn evaluate_document_path(
    expression: &str,
    document: &Document,
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<StringLengthValue, StringLengthFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(StringLengthFailure::Control)?;
    let argument = function_argument(expression.trim(), &["string-length", "fn:string-length"])
        .ok_or(StringLengthFailure::Unsupported)?;
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        return Err(StringLengthFailure::InvalidArity);
    }
    let path = parse_location_path(argument.trim(), location.clone())
        .map_err(StringLengthFailure::Path)?;
    let nodes =
        evaluate_location_path_controlled(document, document.document_node(), &path, control)
            .map_err(StringLengthFailure::Control)?;
    match nodes.as_slice() {
        [] => Ok(StringLengthValue::Integer(0)),
        [node] => Ok(StringLengthValue::Integer(
            document.string_value(*node).chars().count(),
        )),
        [_, _, ..] => Err(StringLengthFailure::InvalidArgumentType),
    }
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

fn parse_range_length_filter(expression: &str) -> Option<(usize, usize, usize, bool)> {
    let (range, predicate) = expression.strip_prefix('(')?.split_once(")[")?;
    let predicate = predicate.strip_suffix(']')?.trim();
    let (start, end) = range.split_once(" to ")?;
    let (call, expected) = predicate.split_once(" = ")?;
    let explicit_context_argument = match call.trim() {
        "string-length()" | "fn:string-length()" => false,
        "string-length(.)" | "fn:string-length(.)" => true,
        _ => return None,
    };
    Some((
        start.trim().parse().ok()?,
        end.trim().parse().ok()?,
        expected.trim().parse().ok()?,
        explicit_context_argument,
    ))
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
    use super::{StringLengthFailure, StringLengthValue, evaluate, evaluate_document_path};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

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
            (
                "(1 to 100)[string-length() = 3]",
                StringLengthValue::Integer(100),
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
        assert_eq!(
            evaluate(
                "(1 to 100)[string-length(.) = 3]",
                &mut InvocationControl::unbounded()
            ),
            Err(StringLengthFailure::InvalidArgumentType)
        );
    }

    #[test]
    fn document_path_rejects_more_than_one_supplied_item() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<works><employee name='one'/><employee name='two'/></works>",
            ParseLimits {
                max_events: 16,
                max_depth: 8,
            },
        )
        .expect("parse source");
        let document = Document::from_parsed(parsed).expect("build XDM");
        let source = "string-length(.//employee/@name)";
        assert_eq!(
            evaluate_document_path(
                source,
                &document,
                &SourceLocation {
                    resource: "memory:expression".to_owned(),
                    span: 0..source.len(),
                },
                &mut InvocationControl::unbounded(),
            ),
            Err(StringLengthFailure::InvalidArgumentType)
        );
    }
}
