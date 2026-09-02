//! Compiles the bounded leading `xsl:attribute` construction slice.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{ComputedAttribute, LiteralAttributeValue};

use super::{
    CompileFailure, ensure_no_meaningful_children, ensure_only_attributes, invalid,
    is_ascii_ncname, is_xslt_element, meaningful_children, required_attribute, unsupported,
};

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

fn compile_computed_attribute(
    document: &Document,
    element: NodeId,
) -> Result<ComputedAttribute, CompileFailure> {
    ensure_only_attributes(document, element, &["name"], "xsl:attribute")?;
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(unsupported(
            "FXST1033",
            format!("the private computed-attribute slice requires an unprefixed NCName: {name}"),
            document.location(element),
        ));
    }
    let children = meaningful_children(document, element);
    let [value_of] = children.as_slice() else {
        return Err(unsupported(
            "FXST1033",
            "the private computed-attribute slice requires one xsl:value-of child",
            document.location(element),
        ));
    };
    if !is_xslt_element(document, *value_of, "value-of") {
        return Err(unsupported(
            "FXST1033",
            "the private computed-attribute value must use xsl:value-of",
            document.location(*value_of),
        ));
    }
    ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, *value_of, "xsl:value-of")?;
    let select = required_attribute(document, *value_of, None, "select")?;
    let value = if let Some(variable) = select
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        LiteralAttributeValue::Variable(variable.to_owned())
    } else {
        let Some(escaped) = crate::xpath::escape_html_uri_experiment::fold_literal(select) else {
            return Err(unsupported(
                "FXXP1012",
                format!("unsupported computed-attribute value expression: {select}"),
                document.location(*value_of),
            ));
        };
        LiteralAttributeValue::Text(escaped)
    };
    Ok(ComputedAttribute {
        name: ExpandedName {
            namespace: None,
            local: name.to_owned(),
        },
        value,
        location: document.location(element).clone(),
    })
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
