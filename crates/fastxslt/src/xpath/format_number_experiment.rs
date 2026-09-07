//! Bounded default-decimal formatting shared by admitted XSLT 1.0 and 3.0 expressions.

use std::collections::BTreeMap;

use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xpath::binary_numeric_experiment::ExactRational;

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

impl FormatNumberExpression {
    pub(crate) fn variable_names(&self) -> impl Iterator<Item = &str> {
        [&self.number, &self.picture]
            .into_iter()
            .filter_map(|operand| match operand {
                Operand::Variable(name) => Some(name.as_str()),
                Operand::Literal(_) => None,
            })
    }
}

#[cfg(feature = "workbench")]
impl FormatNumberExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        operand_capacity(&self.number) + operand_capacity(&self.picture)
    }
}

#[cfg(feature = "workbench")]
fn operand_capacity(value: &Operand) -> usize {
    match value {
        Operand::Literal(value) | Operand::Variable(value) => value.capacity(),
    }
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
    format_default_decimal(number, picture).ok_or(FormatNumberEvaluationFailure::Unsupported)
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

fn format_default_decimal(value: &str, picture: &str) -> Option<String> {
    let number = evaluate_source_free_number(value)?;
    let (positive, negative) = split_subpictures(picture)?;
    let negative_value = number.is_sign_negative();
    let selected = if negative_value {
        negative.unwrap_or(positive)
    } else {
        positive
    };
    let parsed = parse_subpicture(selected)?;
    let implicit_minus = negative_value && negative.is_none();
    let mut output = String::new();
    if implicit_minus {
        output.push('-');
    }
    output.push_str(parsed.prefix);
    if number.is_nan() {
        output.push_str("NaN");
    } else if number.is_infinite() {
        output.push_str("Infinity");
    } else {
        let exact = evaluate_source_free_exact(value).or_else(|| {
            ExactRational::parse_decimal(&evaluate_source_free_number(value)?.to_string())
        })?;
        output.push_str(&format_finite_exact(&exact.format_decimal().ok()?, parsed)?);
    }
    output.push_str(parsed.suffix);
    Some(output)
}

#[derive(Clone, Copy)]
struct ParsedSubpicture<'a> {
    prefix: &'a str,
    suffix: &'a str,
    integer: &'a str,
    fraction: &'a str,
    scale: i128,
}

fn split_subpictures(picture: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = picture.split(';');
    let positive = parts.next()?;
    let negative = parts.next();
    (parts.next().is_none() && !positive.is_empty() && negative != Some(""))
        .then_some((positive, negative))
}

fn parse_subpicture(picture: &str) -> Option<ParsedSubpicture<'_>> {
    let first_digit = picture.find(['#', '0'])?;
    let first = if first_digit > 0 && picture.as_bytes()[first_digit - 1] == b'.' {
        first_digit - 1
    } else {
        first_digit
    };
    let last = picture.rfind(['#', '0'])?;
    let prefix = &picture[..first];
    let suffix = &picture[last + 1..];
    let numeric = &picture[first..=last];
    if prefix.contains(['#', '0', '.', ',', ';'])
        || suffix.contains(['#', '0', '.', ',', ';'])
        || numeric
            .chars()
            .any(|character| !matches!(character, '#' | '0' | '.' | ','))
        || numeric.matches('.').count() > 1
    {
        return None;
    }
    let (integer, fraction) = numeric.split_once('.').unwrap_or((numeric, ""));
    let integer_digits = integer.chars().filter(|character| *character != ',');
    if integer.starts_with(',')
        || integer.ends_with(',')
        || integer.contains(",,")
        || fraction.contains(',')
        || !placeholders_are_ordered(integer_digits, '#', '0')
        || !placeholders_are_ordered(fraction.chars(), '0', '#')
    {
        return None;
    }
    let percent = picture.matches('%').count();
    let per_mille = picture.matches('‰').count();
    if percent + per_mille > 1 {
        return None;
    }
    let scale = if percent == 1 {
        100
    } else if per_mille == 1 {
        1000
    } else {
        1
    };
    Some(ParsedSubpicture {
        prefix,
        suffix,
        integer,
        fraction,
        scale,
    })
}

fn placeholders_are_ordered(
    placeholders: impl Iterator<Item = char>,
    first: char,
    second: char,
) -> bool {
    let mut saw_second = false;
    for placeholder in placeholders {
        if placeholder == second {
            saw_second = true;
        } else if placeholder == first && saw_second {
            return false;
        }
    }
    true
}

fn evaluate_source_free_number(value: &str) -> Option<f64> {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        return evaluate_source_free_number(inner);
    }
    if let Some(quoted) = quoted(value) {
        return Some(quoted.trim().parse().unwrap_or(f64::NAN));
    }
    if let Some(inner) = value
        .strip_prefix("round(")
        .and_then(|inner| inner.strip_suffix(')'))
    {
        let number = evaluate_source_free_number(inner)?;
        return Some(if (-0.5..0.0).contains(&number) {
            -0.0
        } else {
            (number + 0.5).floor()
        });
    }
    for (operator, operation) in [
        (
            " div ",
            (|left: f64, right: f64| left / right) as fn(f64, f64) -> f64,
        ),
        (
            "*",
            (|left: f64, right: f64| left * right) as fn(f64, f64) -> f64,
        ),
    ] {
        if let Some((left, right)) = value.split_once(operator) {
            return Some(operation(
                evaluate_source_free_number(left)?,
                evaluate_source_free_number(right)?,
            ));
        }
    }
    value.parse().ok()
}

fn evaluate_source_free_exact(value: &str) -> Option<ExactRational> {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        return evaluate_source_free_exact(inner);
    }
    if let Some(quoted) = quoted(value) {
        return ExactRational::parse_decimal(quoted);
    }
    if let Some((left, right)) = value.split_once(" div ") {
        return evaluate_source_free_exact(left)?
            .checked_divide(evaluate_source_free_exact(right)?)
            .ok();
    }
    if let Some((left, right)) = value.split_once('*') {
        return evaluate_source_free_exact(left)?
            .checked_multiply(evaluate_source_free_exact(right)?)
            .ok();
    }
    ExactRational::parse_decimal(value)
}

fn format_finite_exact(value: &str, picture: ParsedSubpicture<'_>) -> Option<String> {
    let maximum_fraction = picture.fraction.len();
    let minimum_fraction = picture
        .fraction
        .bytes()
        .filter(|digit| *digit == b'0')
        .count();
    let minimum_integer = picture
        .integer
        .bytes()
        .filter(|digit| *digit == b'0')
        .count();
    let (_negative, value) = value
        .strip_prefix('-')
        .map_or((false, value), |value| (true, value));
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let magnitude = whole
        .bytes()
        .chain(fraction.bytes())
        .try_fold(0_i128, |value, digit| {
            value
                .checked_mul(10)?
                .checked_add(i128::from(digit.checked_sub(b'0')?))
        })?
        .checked_mul(picture.scale)?;
    let source_scale = fraction.len();
    let rounded = if source_scale <= maximum_fraction {
        magnitude
            .checked_mul(10_i128.checked_pow((maximum_fraction - source_scale).try_into().ok()?)?)?
    } else {
        let divisor = 10_i128.checked_pow((source_scale - maximum_fraction).try_into().ok()?)?;
        let quotient = magnitude / divisor;
        let remainder = magnitude % divisor;
        quotient.checked_add(i128::from(remainder.checked_mul(2)? >= divisor))?
    };
    let power = 10_i128.checked_pow(maximum_fraction.try_into().ok()?)?;
    let whole = rounded / power;
    let fractional = rounded % power;
    let mut fixed = if maximum_fraction == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{fractional:0maximum_fraction$}")
    };
    if maximum_fraction > minimum_fraction {
        while fixed.ends_with('0')
            && fixed.len() - fixed.find('.').unwrap_or(fixed.len()) - 1 > minimum_fraction
        {
            fixed.pop();
        }
        if fixed.ends_with('.') {
            fixed.pop();
        }
    }
    let (whole, fraction) = fixed.split_once('.').unwrap_or((&fixed, ""));
    let mut whole = whole.to_owned();
    if whole == "0" && picture.integer.is_empty() {
        whole.clear();
    }
    if whole.len() < minimum_integer {
        whole.insert_str(0, &"0".repeat(minimum_integer - whole.len()));
    }
    if let Some(separator) = picture.integer.rfind(',') {
        let group_size = picture.integer.len() - separator - 1;
        if group_size == 0 {
            return None;
        }
        let mut grouped = String::with_capacity(whole.len() + whole.len() / group_size);
        for (index, character) in whole.chars().rev().enumerate() {
            if index > 0 && index % group_size == 0 {
                grouped.push(',');
            }
            grouped.push(character);
        }
        whole = grouped.chars().rev().collect();
    }
    let formatted = if fraction.is_empty() {
        whole
    } else {
        format!("{whole}.{fraction}")
    };
    Some(formatted)
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
            "format-number(1.23, '0.0.0')",
            "format-number(1.23, '#,')",
            "format-number(0, '#.#0')",
            "format-number(0, '0#.#')",
        ] {
            let expression = parse(source, &location()).expect("expression shape should parse");
            assert_eq!(
                evaluate(&expression, &variables),
                Err(FormatNumberEvaluationFailure::Unsupported)
            );
        }
    }

    #[test]
    fn formats_static_default_decimal_pictures() {
        for (source, expected) in [
            ("format-number(.12, '.000')", ".120"),
            ("format-number(239236.588, '00000.00')", "239236.59"),
            ("format-number(0.4857, '###.###%')", "48.57%"),
            (
                "format-number(-87505.2812, '+###,###.000###;-###,###.000###')",
                "-87,505.2812",
            ),
            (
                "format-number(2392.14*(-36.58), '+###,###.000###;-###,###.000###')",
                "-87,504.4812",
            ),
            (
                "format-number(185.2812, 'PREFIX##00.000###SUFFIX')",
                "PREFIX185.2812SUFFIX",
            ),
            (
                "format-number(987654321, '###,##0,00.00')",
                "9,87,65,43,21.00",
            ),
            ("format-number('foo', '#############')", "NaN"),
        ] {
            let expression = parse(source, &location()).expect("parse static format-number");
            assert_eq!(
                evaluate(&expression, &BTreeMap::new()),
                Ok(expected.to_owned())
            );
        }
    }
}
