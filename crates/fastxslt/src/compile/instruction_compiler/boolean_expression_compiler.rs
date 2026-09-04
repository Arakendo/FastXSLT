//! Private compilation of instruction-local `XPath` boolean expressions.

use crate::compile::golden_stylesheet_experiment::CompileFailure;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xpath::constant_numeric_experiment::{self, ConstantNumericFailure};
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{
    BooleanExpression, DocumentRootReference, EqualityTest, StringComparison,
};

use super::{
    conditional_expression_compiler, invalid, is_ascii_ncname, map_path_failure,
    parse_generated_document_root, parse_generated_temporary_root, unsupported,
    xpath_string_literal,
};

pub(super) fn compile(
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
) -> Result<BooleanExpression, CompileFailure> {
    if expression.trim() == "()" {
        return Ok(BooleanExpression::Constant(false));
    }
    let parsed = strip_enclosing_parentheses(expression.trim());
    if let Some(conditional) =
        conditional_expression_compiler::parse_integer_conditional(parsed, location)?
    {
        return Ok(BooleanExpression::ConditionalInteger(Box::new(conditional)));
    }
    if let Some(variable) = parsed
        .strip_prefix("boolean(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(|value| value.trim().strip_prefix('$'))
        && is_ascii_ncname(variable)
    {
        return Ok(BooleanExpression::VariableEffectiveBooleanValue(
            variable.to_owned(),
        ));
    }
    if let Some((left, right)) = split_top_level_or(parsed) {
        return Ok(BooleanExpression::Or {
            left: Box::new(compile(left, location, comparison)?),
            right: Box::new(compile(right, location, comparison)?),
        });
    }
    if let Some(inner) = parsed
        .strip_prefix("not(")
        .and_then(|inner| inner.strip_suffix(')'))
    {
        return Ok(BooleanExpression::Not(Box::new(compile(
            inner, location, comparison,
        )?)));
    }
    if let Some((left, right)) = parse_document_root_identity_test(parsed) {
        return Ok(BooleanExpression::DocumentRootIdentityEqual {
            left: DocumentRootReference {
                base: location.resource.clone(),
                reference: left.0.to_owned(),
                descendant_local: left.1.map(str::to_owned),
            },
            right: DocumentRootReference {
                base: location.resource.clone(),
                reference: right.0.to_owned(),
                descendant_local: right.1.map(str::to_owned),
            },
        });
    }
    if let Some((variable, descendant_local)) = parse_temporary_root_identity_test(parsed) {
        return Ok(BooleanExpression::TemporaryRootIdentityEqual {
            variable: variable.to_owned(),
            descendant_local: descendant_local.to_owned(),
        });
    }
    if let Some((path, variable)) = parse_generated_root_identity_test(parsed) {
        return Ok(BooleanExpression::RootIdentityEqualsVariable {
            path: parse_location_path(path, location.clone()).map_err(map_path_failure)?,
            variable: variable.to_owned(),
        });
    }
    parse_scalar(parsed, expression, location, comparison)
}

fn split_top_level_or(expression: &str) -> Option<(&str, &str)> {
    let bytes = expression.as_bytes();
    let mut quote = None;
    let mut parentheses = 0_usize;
    let mut brackets = 0_usize;
    let mut index = 0_usize;
    while index + 1 < bytes.len() {
        let byte = bytes[index];
        if let Some(expected) = quote {
            if byte == expected {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => parentheses += 1,
            b')' => parentheses = parentheses.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'o' if bytes[index + 1] == b'r' && parentheses == 0 && brackets == 0 => {
                let left_boundary = index == 0 || !is_xpath_name_byte(bytes[index - 1]);
                let right_boundary = index + 2 == bytes.len()
                    || !is_xpath_name_byte(bytes.get(index + 2).copied().unwrap_or_default());
                if left_boundary && right_boundary {
                    let left = expression[..index].trim();
                    let right = expression[index + 2..].trim();
                    if !left.is_empty() && !right.is_empty() {
                        return Some((left, right));
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn is_xpath_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':')
}

fn parse_scalar(
    parsed: &str,
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
) -> Result<BooleanExpression, CompileFailure> {
    if parsed == "()" {
        return Ok(BooleanExpression::Constant(false));
    }
    if let Some(value) = parse_constant_string_boolean(parsed) {
        return Ok(BooleanExpression::Constant(value));
    }
    if let Some(length) = parse_context_string_length_equality(parsed) {
        return Ok(BooleanExpression::ContextStringLengthEquals(length));
    }
    if let Some(expression) = parse_path_boolean_expression(parsed, location, comparison)? {
        return Ok(expression);
    }
    if let Some((left, right)) = parse_variable_string_equality(parsed) {
        return Ok(BooleanExpression::VariableStringEquals {
            left: left.to_owned(),
            right: right.to_owned(),
            comparison,
        });
    }
    if parsed.starts_with('/') || parsed.starts_with('@') {
        return parse_location_path(parsed, location.clone())
            .map(BooleanExpression::NodeExists)
            .map_err(map_path_failure);
    }
    if is_ascii_ncname(parsed) {
        return parse_location_path(parsed, location.clone())
            .map(BooleanExpression::NodeExists)
            .map_err(map_path_failure);
    }
    if let Some(literal) = parse_context_string_equality(parsed) {
        return Ok(BooleanExpression::ContextStringEquals(literal.to_owned()));
    }
    if let Some((variable, integer)) = parsed.split_once('=') {
        let variable = variable.trim().strip_prefix('$').unwrap_or_default();
        if is_ascii_ncname(variable) {
            if integer.trim() == "()" {
                return Ok(BooleanExpression::VariableEqualsEmptySequence(
                    variable.to_owned(),
                ));
            }
            let integer = integer
                .trim()
                .parse::<i64>()
                .map_err(|_| unsupported_boolean_expression(expression, location))?;
            return Ok(BooleanExpression::VariableEqualsInteger(EqualityTest {
                variable: variable.to_owned(),
                integer,
            }));
        }
    }
    if let Some(variable) = parsed.strip_prefix('$')
        && is_ascii_ncname(variable)
    {
        return Ok(BooleanExpression::VariableEffectiveBooleanValue(
            variable.to_owned(),
        ));
    }
    if !parsed.contains('=') && !parsed.contains('>') {
        let ordering =
            constant_numeric_experiment::compare(parsed, "0").map_err(|failure| match failure {
                ConstantNumericFailure::Invalid => invalid(
                    "FXXP0004",
                    format!("invalid conditional expression: {expression}"),
                    location,
                ),
                ConstantNumericFailure::Unsupported => {
                    unsupported_boolean_expression(expression, location)
                }
            })?;
        return Ok(BooleanExpression::Constant(!ordering.is_eq()));
    }
    let (left, right, greater_than) = if let Some((left, right)) = parsed.split_once('>') {
        (left, right, true)
    } else if let Some((left, right)) = parsed.split_once('=') {
        (left, right, false)
    } else {
        return Err(unsupported_boolean_expression(expression, location));
    };
    let ordering =
        constant_numeric_experiment::compare(left.trim(), right.trim()).map_err(|failure| {
            match failure {
                ConstantNumericFailure::Invalid => invalid(
                    "FXXP0004",
                    format!("invalid conditional expression: {expression}"),
                    location,
                ),
                ConstantNumericFailure::Unsupported => {
                    unsupported_boolean_expression(expression, location)
                }
            }
        })?;
    Ok(BooleanExpression::Constant(if greater_than {
        ordering.is_gt()
    } else {
        ordering.is_eq()
    }))
}

fn parse_context_string_equality(expression: &str) -> Option<&str> {
    let (left, right) = expression.split_once('=')?;
    if left.trim() == "." {
        xpath_string_literal(right.trim())
    } else if right.trim() == "." {
        xpath_string_literal(left.trim())
    } else {
        None
    }
}

fn parse_context_string_length_equality(expression: &str) -> Option<usize> {
    let (left, right) = expression.split_once('=')?;
    (left.trim() == "string-length(.)")
        .then(|| right.trim().parse().ok())
        .flatten()
}

fn parse_path_boolean_expression(
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
) -> Result<Option<BooleanExpression>, CompileFailure> {
    if let Some((path, value)) = parse_path_context_string_predicate(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::NodeStringEquals {
                    path,
                    value: value.to_owned(),
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, local)) = parse_unqualified_name_equality(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::UnqualifiedNodeNameEquals {
                    path,
                    local: local.to_owned(),
                    comparison,
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, value)) = parse_path_string_equality(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::NodeStringEquals {
                    path,
                    value: value.to_owned(),
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, value)) = parse_path_integer_less_than(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| Some(BooleanExpression::NodeIntegerLessThan { path, value }))
            .map_err(map_path_failure);
    }
    Ok(None)
}

fn parse_constant_string_boolean(expression: &str) -> Option<bool> {
    if matches!(expression, "true()" | "false()") {
        return Some(expression == "true()");
    }
    if let Some(value) = xpath_string_literal(expression) {
        return Some(!value.is_empty());
    }
    let (left, right) = expression.split_once('=')?;
    Some(xpath_string_literal(left.trim())? == xpath_string_literal(right.trim())?)
}

fn parse_variable_string_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    Some((
        variable_string_operand(left)?,
        variable_string_operand(right)?,
    ))
}

fn variable_string_operand(expression: &str) -> Option<&str> {
    let expression = expression.trim();
    let expression = expression
        .strip_prefix("string(")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(expression)
        .trim();
    let variable = expression.strip_prefix('$')?;
    is_ascii_ncname(variable).then_some(variable)
}

fn parse_path_context_string_predicate(expression: &str) -> Option<(&str, &str)> {
    let (path, predicate) = expression.split_once('[')?;
    if path.trim().is_empty() || path.contains(']') {
        return None;
    }
    let predicate = predicate.strip_suffix(']')?;
    if predicate.contains(['[', ']']) {
        return None;
    }
    let (left, right) = predicate.split_once('=')?;
    (left.trim() == ".")
        .then(|| xpath_string_literal(right.trim()))
        .flatten()
        .map(|value| (path.trim(), value))
}

fn parse_path_integer_less_than(expression: &str) -> Option<(&str, i64)> {
    let (path, value) = expression.split_once('<')?;
    let path = path.trim();
    if path.is_empty() || path.contains(['=', '>', '<']) {
        return None;
    }
    Some((path, value.trim().parse().ok()?))
}

fn parse_unqualified_name_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let path = left.trim().strip_prefix("name(")?.strip_suffix(')')?.trim();
    let local = xpath_string_literal(right.trim())?;
    (!path.is_empty() && is_ascii_ncname(local)).then_some((path, local))
}

fn parse_path_string_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let left = left.trim();
    let right = right.trim();
    if let Some(value) = xpath_string_literal(right)
        && left != "."
        && !left.starts_with('$')
    {
        return Some((left, value));
    }
    let value = xpath_string_literal(left)?;
    (right != "." && !right.starts_with('$')).then_some((right, value))
}

fn parse_temporary_root_identity_test(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let (left_variable, left_descendant) = parse_generated_temporary_root(left.trim())?;
    let (right_variable, right_descendant) = parse_generated_temporary_root(right.trim())?;
    if left_variable != right_variable || left_descendant.is_some() {
        return None;
    }
    Some((left_variable, right_descendant?))
}

type ParsedDocumentRootReference<'a> = (&'a str, Option<&'a str>);

fn parse_document_root_identity_test(
    expression: &str,
) -> Option<(
    ParsedDocumentRootReference<'_>,
    ParsedDocumentRootReference<'_>,
)> {
    let (left, right) = expression.split_once('=')?;
    Some((
        parse_generated_document_root(left.trim())?,
        parse_generated_document_root(right.trim())?,
    ))
}

fn parse_generated_root_identity_test(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let variable = right.trim().strip_prefix('$')?;
    if !is_ascii_ncname(variable) {
        return None;
    }
    let path = left
        .trim()
        .strip_prefix("generate-id(root(")?
        .strip_suffix("))")?
        .trim();
    Some((path, variable))
}

fn strip_enclosing_parentheses(mut expression: &str) -> &str {
    loop {
        let bytes = expression.as_bytes();
        if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
            return expression;
        }
        let mut depth = 0_usize;
        let mut encloses_all = true;
        for (index, byte) in bytes.iter().copied().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && index + 1 != bytes.len() {
                        encloses_all = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !encloses_all || depth != 0 {
            return expression;
        }
        expression = expression[1..expression.len() - 1].trim();
    }
}

fn unsupported_boolean_expression(expression: &str, location: &SourceLocation) -> CompileFailure {
    unsupported(
        "FXXP1002",
        format!("unsupported conditional expression: {expression}"),
        location,
    )
}
