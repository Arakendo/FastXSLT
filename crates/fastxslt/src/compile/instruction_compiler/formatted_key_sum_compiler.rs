//! Bounded aggregate/conversion composition; delegates key and format semantics.
use super::{
    CompileFailure, Document, NodeId, SourceLocation, ValueExpression, compile_format_number,
    compile_xslt10_literal_key_lookup, xpath_string_literal,
};
use crate::xslt::golden_semantics_experiment::Xslt10FormattedKeySum;

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    let Some(format_arguments) = arguments(expression, "format-number(", 3) else {
        return Ok(None);
    };
    if !(2..=3).contains(&format_arguments.len()) {
        return Ok(None);
    }
    let mut number = format_arguments[0].trim();
    let translation = if let Some(parts) = arguments_of_translate(number) {
        let Some(search) = constant_string(parts[1]) else {
            return Ok(None);
        };
        let Some(replacement) = constant_string(parts[2]) else {
            return Ok(None);
        };
        number = parts[0].trim();
        Some((search, replacement))
    } else {
        None
    };
    let mut string_coercion = translation.is_some();
    if let Some(inner) = arguments(number, "string(", 1) {
        string_coercion = true;
        number = inner[0];
    }
    let Some(sum_arguments) = arguments(number, "sum(", 1) else {
        return Ok(None);
    };
    let lookup = sum_arguments[0];
    if !lookup.starts_with("key(") {
        return Ok(None);
    }
    let lookup = compile_xslt10_literal_key_lookup(document, element, lookup, location)?;
    let mut format = compile_format_number(document, element, expression, location, true)?;
    format.use_supplied_number();
    Ok(Some(ValueExpression::Xslt10FormattedKeySum(Box::new(
        Xslt10FormattedKeySum {
            lookup,
            string_coercion,
            translation,
            format,
        },
    ))))
}

fn arguments<'a>(expression: &'a str, prefix: &str, max: usize) -> Option<Vec<&'a str>> {
    crate::xpath::static_string_experiment::split_arguments(
        expression.trim().strip_prefix(prefix)?.strip_suffix(')')?,
        max,
    )
}

fn arguments_of_translate(expression: &str) -> Option<Vec<&str>> {
    arguments(expression, "translate(", 3).filter(|parts| parts.len() == 3)
}

fn constant_string(expression: &str) -> Option<String> {
    if let Some(value) = xpath_string_literal(expression.trim()) {
        return Some(value.to_owned());
    }
    // XPath numeric arguments are converted to strings, not treated as literals.
    let value = crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(expression)?;
    if !value.is_finite() {
        return None;
    }
    Some(if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    })
}
