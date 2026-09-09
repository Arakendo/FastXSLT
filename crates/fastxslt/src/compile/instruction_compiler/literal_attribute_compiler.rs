//! Literal result-attribute compilation for the admitted private AVT slice.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{LiteralAttribute, LiteralAttributeValue};

use super::{
    CompileFailure, XSLT_NAMESPACE, invalid, is_ascii_ncname, parse_xslt10_normalize_space_path,
    unsupported,
};

pub(crate) fn compile_literal_result_attributes(
    document: &Document,
    element: NodeId,
) -> Result<Vec<LiteralAttribute>, CompileFailure> {
    let mut attributes = Vec::new();
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
            continue;
        }
        let lexical = document.string_value(*attribute);
        let value = parse_literal_attribute_value(&lexical, document.location(*attribute))?;
        attributes.push(LiteralAttribute {
            name: name.clone(),
            value,
            location: document.location(*attribute).clone(),
        });
    }
    Ok(attributes)
}

fn parse_literal_attribute_value(
    lexical: &str,
    location: &SourceLocation,
) -> Result<LiteralAttributeValue, CompileFailure> {
    if lexical == "{position()}" {
        return Ok(LiteralAttributeValue::ContextPosition);
    }
    if lexical == "{last()}" {
        return Ok(LiteralAttributeValue::ContextSize);
    }
    if lexical == "{local-name()}" {
        return Ok(LiteralAttributeValue::ContextLocalName);
    }
    if lexical == "{.}" {
        return Ok(LiteralAttributeValue::ContextStringValue);
    }
    if let Some(name) = lexical
        .strip_prefix("{@")
        .and_then(|value| value.strip_suffix('}'))
        .filter(|name| is_ascii_ncname(name))
    {
        return Ok(LiteralAttributeValue::SourceAttribute(ExpandedName {
            namespace: None,
            local: name.to_owned(),
        }));
    }
    if let Some(variable) = lexical
        .strip_prefix("{$")
        .and_then(|value| value.strip_suffix('}'))
    {
        if is_ascii_ncname(variable) {
            return Ok(LiteralAttributeValue::Variable(variable.to_owned()));
        }
        if !variable.contains(['{', '}']) {
            return Err(invalid(
                "FXST0031",
                format!("invalid variable-only attribute value template: {lexical}"),
                location,
            ));
        }
    }
    if lexical.contains(['{', '}']) {
        if let Some(text) = unescape_static_braces(lexical) {
            return Ok(LiteralAttributeValue::Text(text));
        }
        if let Some(value) = parse_variable_and_path(lexical, location) {
            return Ok(value);
        }
        if let Some(value) = parse_single_dynamic_attribute_value(lexical, location) {
            return Ok(value);
        }
        return Err(unsupported(
            "FXST1031",
            format!("unsupported attribute value template: {lexical}"),
            location,
        ));
    }
    Ok(LiteralAttributeValue::Text(lexical.to_owned()))
}

fn parse_variable_and_path(
    lexical: &str,
    location: &SourceLocation,
) -> Option<LiteralAttributeValue> {
    let remainder = lexical.strip_prefix("{$")?;
    let (variable, remainder) = remainder.split_once('}')?;
    if !is_ascii_ncname(variable) {
        return None;
    }
    let path_open = remainder.rfind('{')?;
    let separator = &remainder[..path_open];
    let path = remainder[path_open + 1..].strip_suffix('}')?;
    if separator.contains(['{', '}']) || path.contains(['{', '}']) {
        return None;
    }
    let path = parse_location_path(path.trim(), location.clone()).ok()?;
    Some(LiteralAttributeValue::Xslt10VariableAndPath {
        variable: variable.to_owned(),
        separator: separator.to_owned(),
        path,
    })
}

fn parse_single_dynamic_attribute_value(
    lexical: &str,
    location: &SourceLocation,
) -> Option<LiteralAttributeValue> {
    let (prefix, expression, suffix) = single_dynamic_expression(lexical)?;
    if let Some(path) = parse_xslt10_normalize_space_path(expression, location.clone()) {
        return Some(LiteralAttributeValue::Xslt10TextAndNormalizedPath {
            prefix: prefix.to_owned(),
            path,
            suffix: suffix.to_owned(),
        });
    }
    if let Ok(path) = parse_location_path(expression.trim(), location.clone()) {
        return Some(LiteralAttributeValue::Xslt10TextAndPath {
            prefix: prefix.to_owned(),
            path,
            suffix: suffix.to_owned(),
        });
    }
    if let Some((name, offset)) = parse_attribute_integer_offset(expression) {
        return Some(LiteralAttributeValue::Xslt10TextAndAttributeIntegerOffset {
            prefix: prefix.to_owned(),
            name: expanded_unqualified_name(name),
            offset,
            suffix: suffix.to_owned(),
        });
    }
    if let Some((left, right)) = parse_source_attribute_concat(expression) {
        return Some(LiteralAttributeValue::Xslt10TextAndSourceAttributeConcat {
            prefix: prefix.to_owned(),
            left: expanded_unqualified_name(left),
            right: expanded_unqualified_name(right),
            suffix: suffix.to_owned(),
        });
    }
    if let Some((value, prefix_attribute)) =
        parse_two_source_attribute_function(expression, "starts-with(")
    {
        return Some(
            LiteralAttributeValue::Xslt10TextAndSourceAttributeStartsWith {
                prefix: prefix.to_owned(),
                value: expanded_unqualified_name(value),
                prefix_attribute: expanded_unqualified_name(prefix_attribute),
                suffix: suffix.to_owned(),
            },
        );
    }
    parse_literal_variable_concat(expression).map(|(literal, variable)| {
        LiteralAttributeValue::Xslt10TextAndLiteralVariableConcat {
            prefix: prefix.to_owned(),
            literal: literal.to_owned(),
            variable: variable.to_owned(),
            suffix: suffix.to_owned(),
        }
    })
}

fn expanded_unqualified_name(local: &str) -> ExpandedName {
    ExpandedName {
        namespace: None,
        local: local.to_owned(),
    }
}

fn parse_attribute_integer_offset(expression: &str) -> Option<(&str, i64)> {
    let mut tokens = expression.split_whitespace();
    let attribute = tokens.next()?.strip_prefix('@')?;
    let operator = tokens.next()?;
    let operand = tokens.next()?.parse::<i64>().ok()?;
    if tokens.next().is_some() || !is_ascii_ncname(attribute) {
        return None;
    }
    let offset = match operator {
        "+" => operand,
        "-" => operand.checked_neg()?,
        _ => return None,
    };
    Some((attribute, offset))
}

fn parse_source_attribute_concat(expression: &str) -> Option<(&str, &str)> {
    parse_two_source_attribute_function(expression, "concat(")
}

fn parse_two_source_attribute_function<'a>(
    expression: &'a str,
    prefix: &str,
) -> Option<(&'a str, &'a str)> {
    let arguments = expression.trim().strip_prefix(prefix)?.strip_suffix(')')?;
    let (left, right) = arguments.split_once(',')?;
    let left = left.trim().strip_prefix('@')?;
    let right = right.trim().strip_prefix('@')?;
    (is_ascii_ncname(left) && is_ascii_ncname(right) && !right.contains(','))
        .then_some((left, right))
}

fn parse_literal_variable_concat(expression: &str) -> Option<(&str, &str)> {
    let arguments = expression
        .trim()
        .strip_prefix("concat(")?
        .strip_suffix(')')?;
    let arguments = crate::xpath::static_string_experiment::split_arguments(arguments, 2)?;
    let [literal, variable] = arguments.as_slice() else {
        return None;
    };
    let literal = xpath_string_literal(literal)?;
    let variable = variable.trim().strip_prefix('$')?;
    is_ascii_ncname(variable).then_some((literal, variable))
}

fn xpath_string_literal(expression: &str) -> Option<&str> {
    let expression = expression.trim();
    let quote = expression.as_bytes().first().copied()?;
    if !matches!(quote, b'\'' | b'"')
        || expression.as_bytes().last().copied() != Some(quote)
        || expression.len() < 2
    {
        return None;
    }
    Some(&expression[1..expression.len() - 1])
}

fn single_dynamic_expression(lexical: &str) -> Option<(&str, &str, &str)> {
    let open = lexical.find('{')?;
    let close = lexical[open + 1..].find('}')? + open + 1;
    let prefix = &lexical[..open];
    let expression = &lexical[open + 1..close];
    let suffix = &lexical[close + 1..];
    (!expression.trim().is_empty()
        && !prefix.contains(['{', '}'])
        && !expression.contains(['{', '}'])
        && !suffix.contains(['{', '}']))
    .then_some((prefix, expression, suffix))
}

fn unescape_static_braces(lexical: &str) -> Option<String> {
    let mut characters = lexical.chars().peekable();
    let mut result = String::with_capacity(lexical.len());
    while let Some(character) = characters.next() {
        if matches!(character, '{' | '}') {
            characters.next_if_eq(&character)?;
        }
        result.push(character);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use crate::xdm::owned_tree_experiment::SourceLocation;
    use crate::xslt::golden_semantics_experiment::LiteralAttributeValue;

    use super::parse_literal_attribute_value;

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "memory:stylesheet.xsl".to_owned(),
            span: 10..20,
        }
    }

    #[test]
    fn unescapes_only_paired_static_avt_braces() {
        assert_eq!(
            parse_literal_attribute_value("{{font:helvetica}}", &location())
                .expect("paired braces should be static text"),
            LiteralAttributeValue::Text("{font:helvetica}".to_owned())
        );
        assert!(parse_literal_attribute_value("{{broken}", &location()).is_err());
    }

    #[test]
    fn compiles_one_source_path_inside_literal_text() {
        let compiled = parse_literal_attribute_value(
            "/cgi-bin/app?p_parm1={.//doc2/doc3/a/@level}",
            &location(),
        )
        .expect("one mixed source-path AVT should compile");
        let LiteralAttributeValue::Xslt10TextAndPath { prefix, suffix, .. } = compiled else {
            panic!("expected the bounded mixed AVT representation");
        };
        assert_eq!(prefix, "/cgi-bin/app?p_parm1=");
        assert_eq!(suffix, "");
    }

    #[test]
    fn compiles_one_source_attribute_integer_offset_inside_literal_text() {
        let compiled = parse_literal_attribute_value("before{@indice - 1}after", &location())
            .expect("one bounded source-attribute offset AVT should compile");
        assert_eq!(
            compiled,
            LiteralAttributeValue::Xslt10TextAndAttributeIntegerOffset {
                prefix: "before".to_owned(),
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "indice".to_owned(),
                },
                offset: -1,
                suffix: "after".to_owned(),
            }
        );
    }

    #[test]
    fn compiles_two_source_attribute_concat_inside_literal_text() {
        let compiled = parse_literal_attribute_value("Before{concat(@a,@b)}After", &location())
            .expect("the bounded source-attribute concat AVT should compile");
        let LiteralAttributeValue::Xslt10TextAndSourceAttributeConcat {
            prefix,
            left,
            right,
            suffix,
        } = compiled
        else {
            panic!("expected the bounded source-attribute concat representation");
        };
        assert_eq!(prefix, "Before");
        assert_eq!(left.local, "a");
        assert_eq!(right.local, "b");
        assert_eq!(suffix, "After");
    }

    #[test]
    fn compiles_source_attribute_starts_with_inside_literal_text() {
        let compiled =
            parse_literal_attribute_value("Before{starts-with(@a,@b)}After", &location())
                .expect("the bounded source-attribute starts-with AVT should compile");
        let LiteralAttributeValue::Xslt10TextAndSourceAttributeStartsWith {
            prefix,
            value,
            prefix_attribute,
            suffix,
        } = compiled
        else {
            panic!("expected the bounded source-attribute starts-with representation");
        };
        assert_eq!(prefix, "Before");
        assert_eq!(value.local, "a");
        assert_eq!(prefix_attribute.local, "b");
        assert_eq!(suffix, "After");
    }

    #[test]
    fn compiles_literal_and_variable_concat_inside_literal_text() {
        let compiled =
            parse_literal_attribute_value("{concat('border: solid ',$color)}", &location())
                .expect("the bounded literal-variable concat AVT should compile");
        let LiteralAttributeValue::Xslt10TextAndLiteralVariableConcat {
            prefix,
            literal,
            variable,
            suffix,
        } = compiled
        else {
            panic!("expected the bounded literal-variable concat representation");
        };
        assert!(prefix.is_empty());
        assert_eq!(literal, "border: solid ");
        assert_eq!(variable, "color");
        assert!(suffix.is_empty());
    }

    #[test]
    fn compiles_variable_and_path_avt_composition() {
        let compiled = parse_literal_attribute_value("{$image-dir}/{href}", &location())
            .expect("the bounded variable/path AVT should compile");
        let LiteralAttributeValue::Xslt10VariableAndPath {
            variable,
            separator,
            path,
        } = compiled
        else {
            panic!("expected the bounded variable/path representation");
        };
        assert_eq!(variable, "image-dir");
        assert_eq!(separator, "/");
        assert_eq!(path.steps.len(), 1);
    }

    #[test]
    fn compiles_context_string_value_avt() {
        assert_eq!(
            parse_literal_attribute_value("{.}", &location())
                .expect("the context-item string-value AVT should compile"),
            LiteralAttributeValue::ContextStringValue
        );
    }
}
