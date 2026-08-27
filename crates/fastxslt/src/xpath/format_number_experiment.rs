//! Narrow exact-decimal formatting for XSLT30 data-manipulation 009 through 019.

use std::collections::BTreeMap;

use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormatNumberExpression {
    number: Operand,
    picture: Operand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operand {
    Literal(String),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormatNumberFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FormatNumberEvaluationFailure {
    UnboundVariable(String),
    Unsupported,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<FormatNumberExpression, FormatNumberFailure> {
    let arguments = expression
        .trim()
        .strip_prefix("format-number(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(expression, location))?;
    let (number, picture) =
        split_top_level_comma(arguments).ok_or_else(|| unsupported(expression, location))?;
    let number = parse_number(number.trim()).ok_or_else(|| unsupported(expression, location))?;
    let picture = parse_picture(picture.trim()).ok_or_else(|| unsupported(expression, location))?;
    Ok(FormatNumberExpression { number, picture })
}

pub(crate) fn evaluate(
    expression: &FormatNumberExpression,
    variables: &BTreeMap<String, AtomicValue>,
) -> Result<String, FormatNumberEvaluationFailure> {
    let number = resolve(&expression.number, variables)?;
    let picture = resolve(&expression.picture, variables)?;
    format_exact_decimal(number, picture).ok_or(FormatNumberEvaluationFailure::Unsupported)
}

fn resolve<'a>(
    operand: &'a Operand,
    variables: &'a BTreeMap<String, AtomicValue>,
) -> Result<&'a str, FormatNumberEvaluationFailure> {
    match operand {
        Operand::Literal(value) => Ok(value),
        Operand::Variable(name) => variables
            .get(name)
            .map(AtomicValue::lexical)
            .ok_or_else(|| FormatNumberEvaluationFailure::UnboundVariable(name.clone())),
    }
}

fn parse_number(expression: &str) -> Option<Operand> {
    if let Some(variable) = variable(expression) {
        return Some(Operand::Variable(variable.to_owned()));
    }
    if let Some(inner) = expression
        .strip_prefix("number(")
        .and_then(|value| value.strip_suffix(')'))
    {
        quoted(inner.trim()).map(|value| Operand::Literal(value.to_owned()))
    } else {
        Some(Operand::Literal(expression.to_owned()))
    }
}

fn parse_picture(expression: &str) -> Option<Operand> {
    if let Some(variable) = variable(expression) {
        return Some(Operand::Variable(variable.to_owned()));
    }
    if let Some(value) = quoted(expression) {
        return Some(Operand::Literal(value.to_owned()));
    }
    let arguments = expression
        .strip_prefix("substring-after(")?
        .strip_suffix(')')?;
    let (value, delimiter) = split_top_level_comma(arguments)?;
    let value = quoted(value.trim())?;
    let delimiter = quoted(delimiter.trim())?;
    let offset = value.find(delimiter)? + delimiter.len();
    Some(Operand::Literal(value[offset..].to_owned()))
}

fn variable(value: &str) -> Option<&str> {
    let name = value.strip_prefix('$')?;
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| {
            (first.is_ascii_alphabetic() || first == '_')
                && characters.all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
                })
        })
        .then_some(name)
}

fn format_exact_decimal(value: &str, picture: &str) -> Option<String> {
    if picture != "#,###.00" {
        return None;
    }
    let (whole, fraction) = value.split_once('.')?;
    if whole.is_empty()
        || fraction.is_empty()
        || fraction.len() > 2
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let reversed: Vec<_> = whole.chars().rev().collect();
    let mut grouped = String::with_capacity(whole.len() + whole.len() / 3);
    for (index, character) in reversed.iter().copied().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    let grouped: String = grouped.chars().rev().collect();
    Some(format!("{grouped}.{fraction:0<2}"))
}

fn quoted(value: &str) -> Option<&str> {
    value.strip_prefix('\'')?.strip_suffix('\'')
}

fn split_top_level_comma(value: &str) -> Option<(&str, &str)> {
    let mut depth = 0_usize;
    let mut quote = false;
    for (offset, character) in value.char_indices() {
        match character {
            '\'' => quote = !quote,
            '(' if !quote => depth += 1,
            ')' if !quote => depth = depth.checked_sub(1)?,
            ',' if !quote && depth == 0 => return Some((&value[..offset], &value[offset + 1..])),
            _ => {}
        }
    }
    None
}

fn unsupported(expression: &str, location: &SourceLocation) -> FormatNumberFailure {
    FormatNumberFailure {
        detail: format!(
            "the private formatting slice supports exact nonnegative decimals or variables, number() over a string literal, substring-after() over string literals, and picture '#,###.00': {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::xdm::atomic_value_experiment::AtomicValue;
    use crate::xdm::owned_tree_experiment::SourceLocation;

    use super::{FormatNumberEvaluationFailure, evaluate, parse};

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "memory:format-number".to_owned(),
            span: 0..1,
        }
    }

    #[test]
    fn composes_admitted_constant_number_and_picture_functions() {
        for source in [
            "format-number(1234.78,substring-after('this#,###.00','this'))",
            "format-number(number('1234.78'),'#,###.00')",
            "format-number(number('1234.78'),substring-after('this#,###.00','this'))",
        ] {
            let expression = parse(source, &location()).expect("admitted formatting expression");
            assert_eq!(
                evaluate(&expression, &BTreeMap::new()),
                Ok("1,234.78".to_owned())
            );
        }
    }

    #[test]
    fn resolves_invocation_local_variable_operands() {
        let expression = parse("format-number($value,$picture)", &location())
            .expect("variable operands should parse");
        let mut variables = BTreeMap::new();
        variables.insert("value".to_owned(), AtomicValue::string("1234.78"));
        variables.insert("picture".to_owned(), AtomicValue::string("#,###.00"));
        assert_eq!(evaluate(&expression, &variables), Ok("1,234.78".to_owned()));
        variables.remove("picture");
        assert_eq!(
            evaluate(&expression, &variables),
            Err(FormatNumberEvaluationFailure::UnboundVariable(
                "picture".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_unadmitted_formatting() {
        let variables = BTreeMap::new();
        for source in [
            "format-number(1.234, '#,###.00')",
            "format-number(1.23, '0.00')",
        ] {
            let expression = parse(source, &location()).expect("expression shape should parse");
            assert_eq!(
                evaluate(&expression, &variables),
                Err(FormatNumberEvaluationFailure::Unsupported)
            );
        }
    }
}
