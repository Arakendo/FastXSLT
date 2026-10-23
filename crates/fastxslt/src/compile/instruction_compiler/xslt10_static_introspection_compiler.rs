//! Compile-time evaluation of XSLT 1.0 implementation introspection functions.

use super::{
    Document, NodeId, XSLT_NAMESPACE, is_ascii_ncname, namespace_for_prefix, xpath_string_literal,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum StaticIntrospectionValue {
    String(String),
    Boolean(bool),
}

pub(super) fn fold(
    document: &Document,
    element: NodeId,
    expression: &str,
) -> Option<StaticIntrospectionValue> {
    let expression = expression.trim();
    let (function, argument) = parse_call(expression)?;
    let lexical_name = xpath_string_literal(argument.trim())?;
    match function {
        "system-property" => fold_system_property(document, element, lexical_name),
        "function-available" => Some(StaticIntrospectionValue::Boolean(function_available(
            document,
            element,
            lexical_name,
        )?)),
        "element-available" => Some(StaticIntrospectionValue::Boolean(element_available(
            document,
            element,
            lexical_name,
        )?)),
        _ => None,
    }
}

// XPath 1.0 converts a numeric argument to string before QName validation.
// Finite numbers and negative infinity cannot spell a QName. Positive infinity
// spells `Infinity`, which is a valid name and must not be classified as an error.
pub(super) fn numeric_literal_qname_error(expression: &str) -> Option<(&str, &'static str)> {
    let (function, argument) = parse_call(expression.trim())?;
    if split_arguments(argument).is_some() || xpath_string_literal(argument.trim()).is_some() {
        return None;
    }
    let number = crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(argument)?;
    if number == f64::INFINITY {
        return None;
    }
    let code = match function {
        "system-property" => "XTDE1390",
        "function-available" => "XTDE1400",
        "element-available" => "XTDE1440",
        _ => return None,
    };
    Some((function, code))
}

pub(super) fn system_property_arity_error(expression: &str) -> bool {
    parse_call(expression.trim()).is_some_and(|(function, argument)| {
        function == "system-property"
            && (argument.trim().is_empty()
                || crate::xpath::static_string_experiment::split_arguments(argument, 64)
                    .is_some_and(|arguments| arguments.len() != 1))
    })
}

pub(super) fn fold_effective_boolean(
    document: &Document,
    element: NodeId,
    expression: &str,
) -> Option<bool> {
    let expression = expression.trim();
    if let Some(value) = fold(document, element, expression) {
        return Some(match value {
            StaticIntrospectionValue::String(value) => !value.is_empty(),
            StaticIntrospectionValue::Boolean(value) => value,
        });
    }
    if let Some(arguments) = expression
        .strip_prefix("contains(")
        .and_then(|tail| tail.strip_suffix(')'))
        && let Some((value, sought)) = split_arguments(arguments)
        && let StaticIntrospectionValue::String(value) = fold(document, element, value.trim())?
    {
        return Some(value.contains(xpath_string_literal(sought.trim())?));
    }
    for operator in [">=", "<=", "!=", ">", "<", "="] {
        let Some((left, right)) = expression.split_once(operator) else {
            continue;
        };
        let StaticIntrospectionValue::String(left) = fold(document, element, left.trim())? else {
            return None;
        };
        let ordering =
            crate::xpath::constant_numeric_experiment::compare(&left, right.trim()).ok()?;
        return Some(match operator {
            ">=" => ordering.is_gt() || ordering.is_eq(),
            "<=" => ordering.is_lt() || ordering.is_eq(),
            "!=" => !ordering.is_eq(),
            ">" => ordering.is_gt(),
            "<" => ordering.is_lt(),
            "=" => ordering.is_eq(),
            _ => unreachable!("operators are enumerated above"),
        });
    }
    None
}

fn split_arguments(arguments: &str) -> Option<(&str, &str)> {
    let mut quote = None;
    let mut depth = 0_usize;
    for (index, character) in arguments.char_indices() {
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some((&arguments[..index], &arguments[index + 1..])),
            _ => {}
        }
    }
    None
}

fn parse_call(expression: &str) -> Option<(&str, &str)> {
    ["system-property", "function-available", "element-available"]
        .into_iter()
        .find_map(|name| {
            expression
                .strip_prefix(name)
                .and_then(|tail| tail.strip_prefix('('))
                .and_then(|tail| tail.strip_suffix(')'))
                .map(|argument| (name, argument))
        })
}

fn fold_system_property(
    document: &Document,
    element: NodeId,
    lexical_name: &str,
) -> Option<StaticIntrospectionValue> {
    let (namespace, local) = expanded_name(document, element, lexical_name, false)?;
    let value = match (namespace.as_deref(), local) {
        (Some(XSLT_NAMESPACE), "version") => {
            if super::uses_xslt10_compatibility(document, element) {
                "1"
            } else {
                "3.0"
            }
        }
        (Some(XSLT_NAMESPACE), "vendor") => "FastXSLT",
        (Some(XSLT_NAMESPACE), "vendor-url") => "https://github.com/Arakendo/FastXSLT",
        _ => "",
    };
    Some(StaticIntrospectionValue::String(value.to_owned()))
}

fn function_available(document: &Document, element: NodeId, lexical_name: &str) -> Option<bool> {
    let (namespace, local) = expanded_name(document, element, lexical_name, false)?;
    if namespace.is_some() {
        return Some(false);
    }
    Some(matches!(
        local,
        "boolean"
            | "ceiling"
            | "concat"
            | "contains"
            | "count"
            | "current"
            | "document"
            | "element-available"
            | "false"
            | "floor"
            | "format-number"
            | "function-available"
            | "generate-id"
            | "id"
            | "key"
            | "lang"
            | "last"
            | "local-name"
            | "name"
            | "namespace-uri"
            | "normalize-space"
            | "not"
            | "number"
            | "position"
            | "round"
            | "starts-with"
            | "string"
            | "string-length"
            | "substring"
            | "substring-after"
            | "substring-before"
            | "sum"
            | "system-property"
            | "translate"
            | "true"
    ))
}

fn element_available(document: &Document, element: NodeId, lexical_name: &str) -> Option<bool> {
    let (namespace, local) = expanded_name(document, element, lexical_name, true)?;
    if namespace.as_deref() != Some(XSLT_NAMESPACE) {
        return Some(false);
    }
    Some(matches!(
        local,
        "apply-imports"
            | "apply-templates"
            | "attribute"
            | "call-template"
            | "choose"
            | "comment"
            | "copy"
            | "copy-of"
            | "element"
            | "fallback"
            | "for-each"
            | "if"
            | "message"
            | "number"
            | "processing-instruction"
            | "text"
            | "value-of"
            | "variable"
    ))
}

fn expanded_name<'a>(
    document: &'a Document,
    element: NodeId,
    lexical_name: &'a str,
    use_default_namespace: bool,
) -> Option<(Option<String>, &'a str)> {
    let lexical_name = lexical_name.trim();
    let Some((prefix, local)) = lexical_name.split_once(':') else {
        if !is_ascii_ncname(lexical_name) {
            return None;
        }
        let namespace = use_default_namespace
            .then(|| namespace_for_prefix(document, element, "").map(str::to_owned))
            .flatten();
        return Some((namespace, lexical_name));
    };
    if !is_ascii_ncname(prefix) || !is_ascii_ncname(local) || local.contains(':') {
        return None;
    }
    let namespace = namespace_for_prefix(document, element, prefix)?.to_owned();
    Some((Some(namespace), local))
}

// Reuse lexical/QName mechanics without declaration-derived processor policy.
// None means outside the caller's admitted capability, not unavailable.
pub(super) fn fold_with_policy<T>(
    document: &Document,
    element: NodeId,
    expression: &str,
    policy: impl FnOnce(&str, Option<&str>, &str) -> Option<T>,
) -> Option<T> {
    let (function, argument) = parse_call(expression.trim())?;
    let lexical = xpath_string_literal(argument.trim())?;
    let (namespace, local) =
        expanded_name(document, element, lexical, function == "element-available")?;
    policy(function, namespace.as_deref(), local)
}

#[cfg(test)]
mod tests {
    use super::{StaticIntrospectionValue, fold, numeric_literal_qname_error};
    use crate::compile::golden_stylesheet_experiment::document_element;
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn advertised_message_and_fallback_capabilities_execute_through_shared_runtime() {
        use crate::execution_control_experiment::InvocationControl;
        use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test;
        let parse = |identity, bytes: &[u8]| {
            Document::from_parsed(
                parse_document(
                    identity,
                    bytes,
                    ParseLimits {
                        max_events: 128,
                        max_depth: 16,
                    },
                )
                .unwrap(),
            )
            .unwrap()
        };
        let source = parse("urn:capability-source", b"<r/>");
        for terminate in ["no", "yes"] {
            let xml = format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:alias="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:unimplemented" extension-element-prefixes="e" version="1.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="element-available('alias:message')"/><xsl:text>|</xsl:text><xsl:value-of select="element-available('xsl:fallback')"/><xsl:text>|</xsl:text><xsl:if test="element-available('xsl:message')"><xsl:message terminate="{terminate}">notice</xsl:message></xsl:if><xsl:if test="element-available('xsl:fallback')"><e:missing><xsl:fallback><xsl:text>fallback</xsl:text></xsl:fallback></e:missing><xsl:fallback><xsl:message terminate="yes">must not run</xsl:message></xsl:fallback></xsl:if></xsl:template></xsl:stylesheet>"#
            );
            let document = parse("urn:capability-style", xml.as_bytes());
            let root = document_element(&document).unwrap();
            for name in [
                "e:message",
                "e:fallback",
                "xsl:param",
                "xsl:sort",
                "xsl:stylesheet",
                "xsl:sequence",
            ] {
                assert_eq!(
                    fold(&document, root, &format!("element-available('{name}')")),
                    Some(StaticIntrospectionValue::Boolean(false))
                );
            }
            let program =
                crate::compile::golden_stylesheet_experiment::compile_stylesheet(&document)
                    .unwrap();
            let mut control = InvocationControl::unbounded();
            let result = execute_compiled_stylesheet_controlled_for_test(
                &program,
                &source,
                "capability",
                &mut control,
            );
            if terminate == "no" {
                assert_eq!(result.unwrap(), "true|true|fallback");
            } else {
                assert!(result.unwrap_err().contains("XTMM9000"));
            }
            assert_eq!(control.messages(), ["notice"]);
        }
    }

    #[test]
    fn system_property_wrong_arity_is_invalid_and_deferred_in_legacy_future_context() {
        use crate::compile::golden_stylesheet_experiment::{CompileCategory, compile_stylesheet};
        use crate::xslt::golden_semantics_experiment::Instruction;
        for expression in [
            "system-property()",
            "system-property('xsl:version', 'foo:bar')",
        ] {
            for version in ["1.0", "1.5", "3.0"] {
                let xml = format!(
                    r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="never"><xsl:value-of select="{expression}"/></xsl:template><xsl:template match="/"/></xsl:stylesheet>"#
                );
                let document = Document::from_parsed(
                    parse_document(
                        "urn:introspection:arity",
                        xml.as_bytes(),
                        ParseLimits {
                            max_events: 64,
                            max_depth: 8,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
                let result = compile_stylesheet(&document);
                if version == "1.5" {
                    assert!(result.unwrap().matched_templates.iter().any(|template| matches!(template.template.body.as_slice(), [Instruction::Xslt10DeferredFailure { code, .. }] if *code == "XPST0017")));
                } else {
                    let failure = result.unwrap_err();
                    assert_eq!(failure.category, CompileCategory::Invalid);
                    assert_eq!(failure.code, "XPST0017");
                    assert_eq!(failure.location.resource, "urn:introspection:arity");
                }
            }
        }
        for expression in [
            "system-property('a,b')",
            "system-property(concat('a', 'b'))",
            "function-available('f', 2)",
            "system-property('a') + other('b', 'c')",
        ] {
            assert!(!super::system_property_arity_error(expression));
        }
    }

    #[test]
    fn folds_namespace_aware_xslt10_introspection() {
        let parsed = parse_document(
            "https://example.test/style.xsl",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:value-of xmlns="http://www.w3.org/1999/XSL/Transform"/></xsl:stylesheet>"#,
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("stylesheet XML");
        let document = Document::from_parsed(parsed).expect("stylesheet XDM");
        let root = document_element(&document).expect("stylesheet element");
        let element = document.children(root)[0];

        assert_eq!(
            fold(&document, element, "system-property('xsl:version')"),
            Some(StaticIntrospectionValue::String("1".to_owned()))
        );
        assert_eq!(
            fold(&document, element, "function-available('format-number')"),
            Some(StaticIntrospectionValue::Boolean(true))
        );
        assert_eq!(
            fold(&document, element, "function-available('document')"),
            Some(StaticIntrospectionValue::Boolean(true))
        );
        assert_eq!(
            fold(&document, element, "function-available('missing')"),
            Some(StaticIntrospectionValue::Boolean(false))
        );
        assert_eq!(
            fold(&document, element, "element-available('xsl:value-of')"),
            Some(StaticIntrospectionValue::Boolean(true))
        );
        assert_eq!(
            fold(&document, element, "element-available('apply-templates')"),
            Some(StaticIntrospectionValue::Boolean(true))
        );
        assert_eq!(
            fold(&document, element, "element-available('xsl:stylesheet')"),
            Some(StaticIntrospectionValue::Boolean(false))
        );
        assert_eq!(
            super::fold_effective_boolean(
                &document,
                element,
                "system-property('xsl:version') >= 1"
            ),
            Some(true)
        );
        assert_eq!(
            super::fold_effective_boolean(
                &document,
                element,
                "contains(system-property('xsl:vendor-url'), 'Arakendo')"
            ),
            Some(true)
        );
        assert_eq!(
            super::fold_effective_boolean(&document, element, "element-available('xsl:value-of')"),
            Some(true)
        );
    }

    #[test]
    fn recognizes_only_numeric_literal_introspection_qname_errors() {
        assert_eq!(
            numeric_literal_qname_error("system-property(123)"),
            Some(("system-property", "XTDE1390"))
        );
        assert_eq!(
            numeric_literal_qname_error("function-available(1.5)"),
            Some(("function-available", "XTDE1400"))
        );
        assert_eq!(
            numeric_literal_qname_error("element-available(-2)"),
            Some(("element-available", "XTDE1440"))
        );
        assert_eq!(
            numeric_literal_qname_error("system-property('xsl:version')"),
            None
        );
        assert_eq!(
            numeric_literal_qname_error("system-property(concat('xsl:', 'version'))"),
            None
        );
        assert_eq!(numeric_literal_qname_error("system-property(1, 2)"), None);
        for argument in ["NaN", "Infinity", "inf", "+123", "1e3", "--2", ".", "$n"] {
            assert_eq!(
                numeric_literal_qname_error(&format!("system-property({argument})")),
                None
            );
        }
        assert_eq!(
            numeric_literal_qname_error(&format!("system-property({})", "9".repeat(400))),
            None
        );
        assert_eq!(
            numeric_literal_qname_error(&format!("system-property(-{})", "9".repeat(400))),
            Some(("system-property", "XTDE1390"))
        );
    }
}
