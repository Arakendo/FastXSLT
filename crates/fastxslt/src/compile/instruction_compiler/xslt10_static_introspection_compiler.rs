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
        (Some(XSLT_NAMESPACE), "version") => "1",
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
            | "for-each"
            | "if"
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

#[cfg(test)]
mod tests {
    use super::{StaticIntrospectionValue, fold};
    use crate::compile::golden_stylesheet_experiment::document_element;
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

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
}
