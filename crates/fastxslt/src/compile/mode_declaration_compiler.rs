//! Private validation for the admitted inert `xsl:mode` declaration slice.

use crate::xdm::owned_tree_experiment::{Document, NodeId};

use super::{
    CompileFailure, ensure_no_meaningful_children, ensure_only_attributes, invalid,
    optional_attribute, parse_template_modes, required_attribute, unsupported,
};

pub(super) fn validate_mode_declaration(
    document: &Document,
    element: NodeId,
    declared_version: &str,
) -> Result<(), CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["name", "warning-on-multiple-match"],
        "xsl:mode",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:mode")?;

    let lexical_name = required_attribute(document, element, None, "name")?;
    let names = parse_template_modes(document, element, lexical_name)?;
    if names.len() != 1 || names[0].starts_with('#') {
        return Err(unsupported(
            "FXST1037",
            "the private xsl:mode declaration slice requires one named mode",
            document.location(element),
        ));
    }

    let warning_enabled = optional_attribute(document, element, None, "warning-on-multiple-match")
        .map(|value| parse_boolean(value, declared_version, document, element))
        .transpose()?
        .unwrap_or(false);
    if warning_enabled {
        return Err(unsupported(
            "FXST1038",
            "warning-on-multiple-match requires an owned warning delivery channel",
            document.location(element),
        ));
    }
    Ok(())
}

fn parse_boolean(
    value: &str,
    declared_version: &str,
    document: &Document,
    element: NodeId,
) -> Result<bool, CompileFailure> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ if declared_version == "3.0" => match value.trim() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(invalid(
                "XTSE0020",
                "warning-on-multiple-match has an invalid XSLT 3.0 boolean value",
                document.location(element),
            )),
        },
        _ => Err(invalid(
            "XTSE0020",
            "warning-on-multiple-match must be 'yes' or 'no'",
            document.location(element),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompileCategory, compile_stylesheet};
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn compile(value: &str) -> Result<(), super::CompileFailure> {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:mode name="m" warning-on-multiple-match="{value}"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#
        );
        let parsed = parse_document(
            "memory:mode-declaration.xsl",
            xml.as_bytes(),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("parse mode declaration fixture");
        let document = Document::from_parsed(parsed).expect("build mode declaration fixture");
        compile_stylesheet(&document).map(|_| ())
    }

    #[test]
    fn admits_only_warning_disabled_lexicals_without_a_warning_channel() {
        for value in ["no", "false", " 0 "] {
            compile(value).expect("warning-disabled declaration should be inert");
        }

        for value in ["yes", "true", " 1 "] {
            let failure = compile(value).expect_err("warning delivery is not yet owned");
            assert_eq!(failure.code, "FXST1038");
            assert_eq!(failure.category, CompileCategory::Unsupported);
        }

        let failure = compile("Yes").expect_err("boolean lexicals are case-sensitive");
        assert_eq!(failure.code, "XTSE0020");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }
}
