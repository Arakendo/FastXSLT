//! Constant exact-decimal formatting for XSLT30 data-manipulation 009 through 011.

use crate::xdm::owned_tree_experiment::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstantFormatNumberExpression {
    formatted: String,
}

impl ConstantFormatNumberExpression {
    pub(crate) fn formatted(&self) -> &str {
        &self.formatted
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstantFormatNumberFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<ConstantFormatNumberExpression, ConstantFormatNumberFailure> {
    let arguments = expression
        .trim()
        .strip_prefix("format-number(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(expression, location))?;
    let (number, picture) =
        split_top_level_comma(arguments).ok_or_else(|| unsupported(expression, location))?;
    let number = parse_number(number.trim()).ok_or_else(|| unsupported(expression, location))?;
    let picture = parse_picture(picture.trim()).ok_or_else(|| unsupported(expression, location))?;
    let formatted =
        format_exact_decimal(number, &picture).ok_or_else(|| unsupported(expression, location))?;
    Ok(ConstantFormatNumberExpression { formatted })
}

fn parse_number(expression: &str) -> Option<&str> {
    if let Some(inner) = expression
        .strip_prefix("number(")
        .and_then(|value| value.strip_suffix(')'))
    {
        quoted(inner.trim())
    } else {
        Some(expression)
    }
}

fn parse_picture(expression: &str) -> Option<String> {
    if let Some(value) = quoted(expression) {
        return Some(value.to_owned());
    }
    let arguments = expression
        .strip_prefix("substring-after(")?
        .strip_suffix(')')?;
    let (value, delimiter) = split_top_level_comma(arguments)?;
    let value = quoted(value.trim())?;
    let delimiter = quoted(delimiter.trim())?;
    let offset = value.find(delimiter)? + delimiter.len();
    Some(value[offset..].to_owned())
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

fn unsupported(expression: &str, location: &SourceLocation) -> ConstantFormatNumberFailure {
    ConstantFormatNumberFailure {
        detail: format!(
            "the private constant formatting slice supports exact nonnegative decimals, number() over a string literal, substring-after() over string literals, and picture '#,###.00': {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::xdm::owned_tree_experiment::SourceLocation;

    use super::parse;

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "memory:constant-format-number".to_owned(),
            span: 0..1,
        }
    }

    #[test]
    fn composes_admitted_constant_number_and_picture_functions() {
        for expression in [
            "format-number(1234.78,substring-after('this#,###.00','this'))",
            "format-number(number('1234.78'),'#,###.00')",
            "format-number(number('1234.78'),substring-after('this#,###.00','this'))",
        ] {
            assert_eq!(
                parse(expression, &location())
                    .expect("admitted constant formatting expression")
                    .formatted(),
                "1,234.78"
            );
        }
    }

    #[test]
    fn rejects_dynamic_or_unadmitted_formatting() {
        assert!(parse("format-number($value, '#,###.00')", &location()).is_err());
        assert!(parse("format-number(1.234, '#,###.00')", &location()).is_err());
        assert!(parse("format-number(1.23, '0.00')", &location()).is_err());
    }
}
