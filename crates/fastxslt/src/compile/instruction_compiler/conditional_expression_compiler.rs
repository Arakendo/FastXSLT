//! Narrow compiled ownership for the admitted `XPath` conditional-expression forms.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xpath::constant_numeric_experiment;
use crate::xpath::path_experiment::{LocationPath, parse_location_path};
use crate::xslt::golden_semantics_experiment::{
    ConditionalIntegerBranch, ConditionalIntegerCondition, ConditionalIntegerExpression,
    ConditionalPathBranch, ConditionalPathExpression, IntegerComparisonOperator,
    IntegerPathComparison, ValueExpression,
};

use super::{
    CompileFailure, XML_SCHEMA_NAMESPACE, is_ascii_ncname, map_path_failure, namespace_for_prefix,
    unsupported_boolean_expression, xpath_string_literal,
};

pub(super) fn compile_value(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    if let Some(value) = parse_path_conditional(document, element, expression, location)? {
        return Ok(Some(ValueExpression::ConditionalPath(Box::new(value))));
    }
    Ok(parse_integer_conditional(expression, location)?
        .map(|value| ValueExpression::ConditionalInteger(Box::new(value))))
}

pub(super) fn parse_integer_conditional(
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ConditionalIntegerExpression>, CompileFailure> {
    let expression = expression.trim();
    if !expression.starts_with("if (") {
        return Ok(None);
    }
    let closing = matching_parenthesis(expression, 3)
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    let condition = parse_integer_condition(&expression[4..closing], location)?;
    let branches = expression[closing + 1..]
        .trim_start()
        .strip_prefix("then ")
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    let (when_true, when_false) = split_top_level_else(branches)
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    Ok(Some(ConditionalIntegerExpression {
        condition,
        when_true: parse_integer_branch(when_true, location)?,
        when_false: parse_integer_branch(when_false, location)?,
    }))
}

fn parse_integer_condition(
    expression: &str,
    location: &SourceLocation,
) -> Result<ConditionalIntegerCondition, CompileFailure> {
    let expression = expression.trim();
    if let Some(arguments) = expression
        .strip_prefix("contains(")
        .and_then(|value| value.strip_suffix(')'))
        && let Some((path, needle)) = arguments.split_once(',')
        && let Some(needle) = xpath_string_literal(needle.trim())
    {
        let path = parse_location_path(path.trim(), location.clone()).map_err(map_path_failure)?;
        return Ok(ConditionalIntegerCondition::Contains {
            path,
            needle: needle.to_owned(),
        });
    }
    if let Some((left, right)) = expression.split_once('<') {
        let ordering = constant_numeric_experiment::compare(left.trim(), right.trim())
            .map_err(|_| unsupported_boolean_expression(expression, location))?;
        return Ok(ConditionalIntegerCondition::Constant(ordering.is_lt()));
    }
    Err(unsupported_boolean_expression(expression, location))
}

fn parse_integer_branch(
    expression: &str,
    location: &SourceLocation,
) -> Result<ConditionalIntegerBranch, CompileFailure> {
    if let Some(conditional) = parse_integer_conditional(expression, location)? {
        return Ok(ConditionalIntegerBranch::Conditional(Box::new(conditional)));
    }
    expression
        .trim()
        .parse::<i64>()
        .map(ConditionalIntegerBranch::Integer)
        .map_err(|_| unsupported_boolean_expression(expression, location))
}

fn parse_path_conditional(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ConditionalPathExpression>, CompileFailure> {
    let expression = expression.trim();
    if !expression.starts_with("if (") || !expression.contains(":integer(") {
        return Ok(None);
    }
    let closing = matching_parenthesis(expression, 3)
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    let condition = parse_path_comparison(document, element, &expression[4..closing], location)?;
    let branches = expression[closing + 1..]
        .trim_start()
        .strip_prefix("then ")
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    let (when_true, when_false) = split_top_level_else(branches)
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    Ok(Some(ConditionalPathExpression {
        condition,
        when_true: parse_path_branch(document, element, when_true, location)?,
        when_false: parse_path_branch(document, element, when_false, location)?,
    }))
}

fn parse_path_comparison(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<IntegerPathComparison, CompileFailure> {
    let (left, right, operator) = split_path_comparison(expression)
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    Ok(IntegerPathComparison {
        left: parse_integer_path_cast(document, element, left, location)?,
        right: parse_integer_path_cast(document, element, right, location)?,
        operator,
    })
}

fn split_path_comparison(expression: &str) -> Option<(&str, &str, IntegerComparisonOperator)> {
    let mut depth = 0_usize;
    for (index, byte) in expression.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b'>' if depth == 0 => {
                return Some((
                    expression[..index].trim(),
                    expression[index + 1..].trim(),
                    IntegerComparisonOperator::GreaterThan,
                ));
            }
            b'=' if depth == 0 => {
                return Some((
                    expression[..index].trim(),
                    expression[index + 1..].trim(),
                    IntegerComparisonOperator::Equal,
                ));
            }
            _ => {}
        }
    }
    None
}

fn parse_integer_path_cast(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<LocationPath, CompileFailure> {
    let expression = expression.trim();
    let (prefix, argument) = expression
        .split_once(":integer(")
        .and_then(|(prefix, argument)| Some((prefix, argument.strip_suffix(')')?)))
        .ok_or_else(|| unsupported_boolean_expression(expression, location))?;
    if !is_ascii_ncname(prefix)
        || namespace_for_prefix(document, element, prefix) != Some(XML_SCHEMA_NAMESPACE)
    {
        return Err(unsupported_boolean_expression(expression, location));
    }
    parse_location_path(argument.trim(), location.clone()).map_err(map_path_failure)
}

fn parse_path_branch(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<ConditionalPathBranch, CompileFailure> {
    if let Some(conditional) = parse_path_conditional(document, element, expression, location)? {
        return Ok(ConditionalPathBranch::Conditional(Box::new(conditional)));
    }
    if let Some((numerator, denominator)) = expression.split_once(" div ") {
        return Ok(ConditionalPathBranch::Division {
            numerator: parse_integer_path_cast(document, element, numerator, location)?,
            denominator: parse_integer_path_cast(document, element, denominator, location)?,
        });
    }
    parse_location_path(expression.trim(), location.clone())
        .map(ConditionalPathBranch::Path)
        .map_err(map_path_failure)
}

fn matching_parenthesis(expression: &str, opening: usize) -> Option<usize> {
    let mut depth = 0_usize;
    let mut quote = None;
    for (index, byte) in expression.bytes().enumerate().skip(opening) {
        if let Some(expected) = quote {
            if byte == expected {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => depth += 1,
            b')' if depth == 1 => return Some(index),
            b')' => depth = depth.checked_sub(1)?,
            _ => {}
        }
    }
    None
}

fn split_top_level_else(expression: &str) -> Option<(&str, &str)> {
    let bytes = expression.as_bytes();
    let mut depth = 0_usize;
    let mut nested_conditionals = 0_usize;
    let mut quote = None;
    let mut index = 0_usize;
    while index + 6 <= bytes.len() {
        if quote.is_none()
            && depth == 0
            && expression[index..].starts_with("if (")
            && (index == 0 || bytes[index - 1].is_ascii_whitespace())
        {
            nested_conditionals += 1;
        }
        match bytes[index] {
            byte if quote == Some(byte) => quote = None,
            b'\'' | b'"' if quote.is_none() => quote = Some(bytes[index]),
            b'(' if quote.is_none() => depth += 1,
            b')' if quote.is_none() => depth = depth.saturating_sub(1),
            b' ' if quote.is_none() && depth == 0 && expression[index..].starts_with(" else ") => {
                if nested_conditionals > 0 {
                    nested_conditionals -= 1;
                    index += 6;
                    continue;
                }
                return Some((expression[..index].trim(), expression[index + 6..].trim()));
            }
            _ => {}
        }
        index += 1;
    }
    None
}
