//! R0 P05: capability refusals must survive every tested expression owner.
use super::*;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn sheet(version: &str, top: &str, body: &str) -> Document {
    let xml = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{top}<xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
    );
    Document::from_parsed(
        parse_document(
            "urn:r0-introspection",
            xml.as_bytes(),
            ParseLimits {
                max_events: 200,
                max_depth: 20,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn selected_introspection_refusals_do_not_turn_into_availability_answers() {
    // Each route first proves it is an admitted owner with a valid control.
    // Refusing an unsupported introspection call is not a false/empty answer,
    // nor a forward-compatible runtime failure manufactured from a broad catch.
    for (owner, top, body, control) in [
        ("value", "", r#"<xsl:value-of select="EXPR"/>"#, "'ok'"),
        (
            "condition",
            "",
            r#"<xsl:if test="EXPR"><xsl:text>ok</xsl:text></xsl:if>"#,
            "true()",
        ),
        (
            "local",
            "",
            r#"<xsl:variable name="v" select="EXPR"/><xsl:value-of select="$v"/>"#,
            "'ok'",
        ),
        (
            "global",
            r#"<xsl:variable name="v" select="EXPR"/>"#,
            r#"<xsl:value-of select="$v"/>"#,
            "'ok'",
        ),
        (
            "parameter",
            "",
            r#"<xsl:param name="v" select="EXPR"/><xsl:value-of select="$v"/>"#,
            "'ok'",
        ),
        (
            "argument",
            r#"<xsl:template name="t"><xsl:param name="v"/><xsl:value-of select="$v"/></xsl:template>"#,
            r#"<xsl:call-template name="t"><xsl:with-param name="v" select="EXPR"/></xsl:call-template>"#,
            "'ok'",
        ),
    ] {
        for version in ["1.0", "2.0"] {
            let document = sheet(
                version,
                &top.replace("EXPR", control),
                &body.replace("EXPR", control),
            );
            compile(
                &document,
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            )
            .unwrap_or_else(|error| panic!("positive control {owner}/{version}: {error:?}"));
            for expression in [
                "function-available('document')",
                "element-available('xsl:sequence')",
                "system-property('xsl:vendor')",
            ] {
                let document = sheet(
                    version,
                    &top.replace("EXPR", expression),
                    &body.replace("EXPR", expression),
                );
                let Err(StrictPreprocessingFailure::Compile(error)) = compile(
                    &document,
                    Edition::Legacy,
                    &mut InvocationControl::unbounded(),
                ) else {
                    panic!("introspection escaped guard: {owner}/{version}/{expression}");
                };
                assert_eq!(
                    error.category,
                    CompileCategory::Unsupported,
                    "{owner}/{version}/{expression}"
                );
                assert_eq!(error.location.resource, "urn:r0-introspection");
            }
        }
    }
}
