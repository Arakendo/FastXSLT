//! Literal result-attribute compilation for the admitted private AVT slice.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{LiteralAttribute, LiteralAttributeValue};

use super::{CompileFailure, XSLT_NAMESPACE, invalid, is_ascii_ncname, unsupported};

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
        return Err(invalid(
            "FXST0031",
            format!("invalid variable-only attribute value template: {lexical}"),
            location,
        ));
    }
    if lexical.contains(['{', '}']) {
        if let Some(text) = unescape_static_braces(lexical) {
            return Ok(LiteralAttributeValue::Text(text));
        }
        if let Some((prefix, expression, suffix)) = single_dynamic_expression(lexical) {
            if let Ok(path) = parse_location_path(expression.trim(), location.clone()) {
                return Ok(LiteralAttributeValue::Xslt10TextAndPath {
                    prefix: prefix.to_owned(),
                    path,
                    suffix: suffix.to_owned(),
                });
            }
            if let Some((name, offset)) = parse_attribute_integer_offset(expression) {
                return Ok(LiteralAttributeValue::Xslt10TextAndAttributeIntegerOffset {
                    prefix: prefix.to_owned(),
                    name: ExpandedName {
                        namespace: None,
                        local: name.to_owned(),
                    },
                    offset,
                    suffix: suffix.to_owned(),
                });
            }
            if let Some((left, right)) = parse_source_attribute_concat(expression) {
                let expanded_name = |local: &str| ExpandedName {
                    namespace: None,
                    local: local.to_owned(),
                };
                return Ok(LiteralAttributeValue::Xslt10TextAndSourceAttributeConcat {
                    prefix: prefix.to_owned(),
                    left: expanded_name(left),
                    right: expanded_name(right),
                    suffix: suffix.to_owned(),
                });
            }
            if let Some((value, prefix_attribute)) =
                parse_two_source_attribute_function(expression, "starts-with(")
            {
                let expanded_name = |local: &str| ExpandedName {
                    namespace: None,
                    local: local.to_owned(),
                };
                return Ok(
                    LiteralAttributeValue::Xslt10TextAndSourceAttributeStartsWith {
                        prefix: prefix.to_owned(),
                        value: expanded_name(value),
                        prefix_attribute: expanded_name(prefix_attribute),
                        suffix: suffix.to_owned(),
                    },
                );
            }
        }
        return Err(unsupported(
            "FXST1031",
            format!("unsupported attribute value template: {lexical}"),
            location,
        ));
    }
    Ok(LiteralAttributeValue::Text(lexical.to_owned()))
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
    fn compiles_context_string_value_avt() {
        assert_eq!(
            parse_literal_attribute_value("{.}", &location())
                .expect("the context-item string-value AVT should compile"),
            LiteralAttributeValue::ContextStringValue
        );
    }
}
