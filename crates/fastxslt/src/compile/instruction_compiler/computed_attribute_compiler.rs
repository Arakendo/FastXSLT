//! Compiles the bounded leading `xsl:attribute` construction slice.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{ComputedAttribute, LiteralAttributeValue};

use super::value_expression_compiler::compile_xslt10_concat;
use super::{
    CompileFailure, compile_text_value, ensure_no_meaningful_children, ensure_only_attributes,
    invalid, is_ascii_ncname, is_xslt_element, meaningful_children, namespace_for_prefix,
    optional_attribute, parse_xslt10_normalize_space_path, required_attribute,
    split_top_level_union, unsupported, uses_xslt10_compatibility, xpath_string_literal,
};

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

pub(super) fn compile_computed_attributes(
    document: &Document,
    parent: NodeId,
) -> Result<(Vec<ComputedAttribute>, Vec<NodeId>), CompileFailure> {
    let mut attributes = Vec::new();
    let mut attribute_nodes = Vec::new();
    let mut body_started = false;
    for child in meaningful_children(document, parent) {
        if !is_xslt_element(document, child, "attribute") {
            body_started = true;
            continue;
        }
        if body_started {
            return Err(invalid(
                "XTDE0410",
                "xsl:attribute must precede result child construction",
                document.location(child),
            ));
        }
        attributes.push(compile_computed_attribute(document, child)?);
        attribute_nodes.push(child);
    }
    Ok((attributes, attribute_nodes))
}

pub(super) fn compile_computed_attribute(
    document: &Document,
    element: NodeId,
) -> Result<ComputedAttribute, CompileFailure> {
    ensure_only_attributes(document, element, &["name", "namespace"], "xsl:attribute")?;
    let name = required_attribute(document, element, None, "name")?;
    let namespace = optional_attribute(document, element, None, "namespace");
    if namespace.is_some_and(|value| value.contains(['{', '}'])) {
        return Err(unsupported(
            "FXST1061",
            "the private xsl:attribute namespace slice requires a static URI",
            document.location(element),
        ));
    }
    if name.contains(['{', '}']) {
        return Err(unsupported(
            "FXST1062",
            "the private computed-attribute name slice requires a static QName",
            document.location(element),
        ));
    }
    let name = compile_static_attribute_name(document, element, name, namespace)?;
    if name.local == "xmlns" || name.namespace.as_deref() == Some(XMLNS_NAMESPACE) {
        return Err(invalid(
            "XTDE0855",
            "xsl:attribute cannot construct a name in the reserved xmlns namespace",
            document.location(element),
        ));
    }
    let children = meaningful_children(document, element);
    let value = if children
        .iter()
        .all(|child| document.kind(*child) == crate::xdm::owned_tree_experiment::NodeKind::Text)
    {
        LiteralAttributeValue::Text(
            children
                .iter()
                .filter_map(|child| document.value(*child))
                .collect(),
        )
    } else if let [value_of] = children.as_slice()
        && is_xslt_element(document, *value_of, "value-of")
    {
        ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
        ensure_no_meaningful_children(document, *value_of, "xsl:value-of")?;
        let select = required_attribute(document, *value_of, None, "select")?;
        compile_computed_attribute_value(document, *value_of, select)?
    } else if let [text] = children.as_slice()
        && is_xslt_element(document, *text, "text")
    {
        LiteralAttributeValue::Text(compile_text_value(document, *text)?)
    } else if let [number] = children.as_slice()
        && is_xslt_element(document, *number, "number")
    {
        LiteralAttributeValue::Number(Box::new(super::number_compiler::compile(
            document, *number,
        )?))
    } else if let [for_each] = children.as_slice()
        && is_xslt_element(document, *for_each, "for-each")
        && uses_xslt10_compatibility(document, *for_each)
    {
        compile_xslt10_for_each_string_value(document, *for_each)?
    } else {
        return Err(unsupported(
            "FXST1033",
            "the private computed-attribute value requires literal text or one admitted text, value-of, or number instruction",
            document.location(element),
        ));
    };
    Ok(ComputedAttribute {
        name,
        value,
        location: document.location(element).clone(),
    })
}

fn compile_xslt10_for_each_string_value(
    document: &Document,
    for_each: NodeId,
) -> Result<LiteralAttributeValue, CompileFailure> {
    ensure_only_attributes(document, for_each, &["select"], "xsl:for-each")?;
    let select = required_attribute(document, for_each, None, "select")?;
    let children = meaningful_children(document, for_each);
    let [value_of] = children.as_slice() else {
        return Err(unsupported(
            "FXST1033",
            "the private XSLT 1.0 computed-attribute for-each slice requires exactly one xsl:value-of child",
            document.location(for_each),
        ));
    };
    if !is_xslt_element(document, *value_of, "value-of") {
        return Err(unsupported(
            "FXST1033",
            "the private XSLT 1.0 computed-attribute for-each slice requires exactly one xsl:value-of child",
            document.location(for_each),
        ));
    }
    ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, *value_of, "xsl:value-of")?;
    if required_attribute(document, *value_of, None, "select")?.trim() != "." {
        return Err(unsupported(
            "FXST1033",
            "the private XSLT 1.0 computed-attribute for-each slice requires xsl:value-of select=\".\"",
            document.location(*value_of),
        ));
    }
    let path =
        parse_location_path(select.trim(), document.location(for_each).clone()).map_err(|_| {
            unsupported(
                "FXXP1012",
                format!("unsupported computed-attribute for-each selection: {select}"),
                document.location(for_each),
            )
        })?;
    Ok(LiteralAttributeValue::Xslt10ForEachPathStringValue(path))
}

fn compile_static_attribute_name(
    document: &Document,
    element: NodeId,
    lexical: &str,
    namespace_override: Option<&str>,
) -> Result<ExpandedName, CompileFailure> {
    if is_ascii_ncname(lexical) {
        return Ok(ExpandedName {
            namespace: namespace_override
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
            local: lexical.to_owned(),
        });
    }
    let Some((prefix, local)) = lexical.split_once(':') else {
        return Err(invalid(
            "XTDE0850",
            format!("xsl:attribute name is not a QName: {lexical}"),
            document.location(element),
        ));
    };
    if !is_ascii_ncname(prefix) || !is_ascii_ncname(local) || local.contains(':') {
        return Err(invalid(
            "XTDE0850",
            format!("xsl:attribute name is not a QName: {lexical}"),
            document.location(element),
        ));
    }
    if prefix == "xmlns" {
        return Err(invalid(
            "XTDE0855",
            "xsl:attribute cannot construct an xmlns-prefixed name",
            document.location(element),
        ));
    }
    let namespace = match namespace_override {
        Some("") => {
            return Err(invalid(
                "XTDE0860",
                "a prefixed xsl:attribute name cannot use an empty namespace",
                document.location(element),
            ));
        }
        Some(namespace) => namespace,
        None => namespace_for_prefix(document, element, prefix).ok_or_else(|| {
            invalid(
                "XTDE0860",
                format!("xsl:attribute name uses an unbound prefix: {prefix}"),
                document.location(element),
            )
        })?,
    };
    if (prefix == "xml") != (namespace == XML_NAMESPACE) {
        return Err(invalid(
            "XTDE0860",
            "the xml prefix and namespace must be used together",
            document.location(element),
        ));
    }
    Ok(ExpandedName {
        namespace: Some(namespace.to_owned()),
        local: local.to_owned(),
    })
}

fn compile_computed_attribute_value(
    document: &Document,
    value_of: NodeId,
    select: &str,
) -> Result<LiteralAttributeValue, CompileFailure> {
    let value = if let Some(variable) = select
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        LiteralAttributeValue::Variable(variable.to_owned())
    } else if uses_xslt10_compatibility(document, value_of)
        && let Some(variable) = select
            .strip_prefix("count($")
            .and_then(|value| value.strip_suffix(')'))
            .filter(|name| is_ascii_ncname(name))
    {
        LiteralAttributeValue::CountSourceNodeVariable(variable.to_owned())
    } else if uses_xslt10_compatibility(document, value_of)
        && let Some(path) = select
            .strip_prefix("count(")
            .and_then(|value| value.strip_suffix(')'))
        && let Some(alternatives) = split_top_level_union(path)
        && let Some(alternatives) = alternatives
            .into_iter()
            .map(|alternative| {
                parse_location_path(alternative.trim(), document.location(value_of).clone()).ok()
            })
            .collect::<Option<Vec<_>>>()
    {
        LiteralAttributeValue::CountSourcePathUnion(alternatives)
    } else if uses_xslt10_compatibility(document, value_of)
        && let Some(path) = select
            .strip_prefix("count(")
            .and_then(|value| value.strip_suffix(')'))
        && let Ok(path) = parse_location_path(path.trim(), document.location(value_of).clone())
    {
        LiteralAttributeValue::CountSourcePath(path)
    } else if uses_xslt10_compatibility(document, value_of)
        && select.trim() == "string-length(normalize-space(.))"
    {
        LiteralAttributeValue::ContextNormalizedStringLength
    } else if select.trim() == "." {
        LiteralAttributeValue::ContextStringValue
    } else if select.trim() == "position()" {
        LiteralAttributeValue::ContextPosition
    } else if select.trim() == "last()" {
        LiteralAttributeValue::ContextSize
    } else if uses_xslt10_compatibility(document, value_of)
        && let Some(path) =
            parse_xslt10_normalize_space_path(select, document.location(value_of).clone())
    {
        LiteralAttributeValue::Xslt10TextAndNormalizedPath {
            prefix: String::new(),
            path,
            suffix: String::new(),
        }
    } else if let Some(attribute) = select
        .strip_prefix('@')
        .filter(|name| is_ascii_ncname(name))
    {
        LiteralAttributeValue::SourceAttribute(ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        })
    } else if uses_xslt10_compatibility(document, value_of)
        && let Some(expression) =
            compile_xslt10_concat(document, value_of, select, document.location(value_of))?
    {
        LiteralAttributeValue::Xslt10Concat(Box::new(expression))
    } else {
        let value = xpath_string_literal(select)
            .map(str::to_owned)
            .or_else(|| crate::xpath::static_string_experiment::fold_substring_literals(select))
            .or_else(|| crate::xpath::escape_html_uri_experiment::fold_literal(select));
        let Some(value) = value else {
            return Err(unsupported(
                "FXXP1012",
                format!("unsupported computed-attribute value expression: {select}"),
                document.location(value_of),
            ));
        };
        LiteralAttributeValue::Text(value)
    };
    Ok(value)
}

#[cfg(test)]
mod tests {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn compile(bytes: &[u8]) -> crate::compile::golden_stylesheet_experiment::CompileFailure {
        let parsed = parse_document(
            "urn:fastxslt:computed-attribute:test",
            bytes,
            ParseLimits {
                max_events: 64,
                max_depth: 16,
            },
        )
        .expect("parse computed-attribute stylesheet");
        let document = Document::from_parsed(parsed).expect("build computed-attribute stylesheet");
        compile_stylesheet(&document).expect_err("stylesheet must be rejected")
    }

    #[test]
    fn rejects_computed_attributes_after_result_children() {
        let failure = compile(
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc"><out><child/><xsl:attribute name="late"><xsl:value-of select="$value"/></xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
        );
        assert_eq!(failure.code, "XTDE0410");
        assert!(failure.detail.contains("precede result child"));
    }

    #[test]
    fn rejects_duplicate_literal_and_computed_result_attributes() {
        let failure = compile(
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc"><out magic="literal"><xsl:attribute name="magic"><xsl:value-of select="$value"/></xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
        );
        assert_eq!(failure.code, "XTDE0410");
        assert!(failure.detail.contains("duplicate result attribute"));
    }
}
