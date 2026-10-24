use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

fn program(version: &str, body: &str) -> Result<StylesheetProgram, ExecutionFailure> {
    selected(&[(
        PRINCIPAL,
        &format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
        ),
    )])
}

fn source() -> Document {
    Document::from_parsed(
        parse_document(
            "urn:comment-source",
            b"<r>source</r>",
            super::super::super::XML_LIMITS,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn selected_comment_content_uses_shared_execution_and_strict_preprocessing() {
    for version in ["1.0", "2.0", "3.0"] {
        for (body, expected) in [
            (
                r#"<xsl:comment>A<!-- removed -->   <xsl:value-of select="r"/><xsl:text>--end-</xsl:text></xsl:comment>"#,
                "<!--Asource- -end- -->",
            ),
            (
                r#"<xsl:comment><xsl:text disable-output-escaping="yes">&lt;&amp;</xsl:text></xsl:comment>"#,
                "<!--<&-->",
            ),
            (
                "<xsl:comment><ignored>not text</ignored><xsl:text>kept</xsl:text></xsl:comment>",
                "<!--kept-->",
            ),
        ] {
            let compiled = program(version, body).unwrap();
            for _ in 0..2 {
                assert_eq!(
                    execute(
                        &compiled,
                        &source(),
                        "comment",
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
    }
    let compiled = program("1.0", r#"<xsl:message>before<xsl:comment><xsl:value-of select="r"/></xsl:comment>after</xsl:message><out/>"#).unwrap();
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        execute(&compiled, &source(), "message-comment", &mut control).unwrap(),
        "<out></out>"
    );
    assert_eq!(control.messages(), ["beforeafter"]);
}

#[test]
fn selected_comment_keeps_policy_error_timing_and_refusal_boundaries() {
    let compiled = program("2.0", r#"<xsl:comment><xsl:sequence><xsl:fallback><xsl:value-of select="system-property('xsl:version')"/></xsl:fallback></xsl:sequence></xsl:comment>"#).unwrap();
    assert_eq!(
        execute(
            &compiled,
            &source(),
            "fallback",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<!--1-->"
    );
    for condition in ["false()", "true()"] {
        let compiled = program("2.0", &format!(r#"<xsl:comment><xsl:if test="{condition}"><xsl:value-of select="system-property()"/></xsl:if></xsl:comment>"#)).unwrap();
        let result = execute(
            &compiled,
            &source(),
            "timing",
            &mut InvocationControl::unbounded(),
        );
        if condition == "false()" {
            assert_eq!(result.unwrap(), "<!---->");
        } else {
            let failure = result.unwrap_err();
            assert!(failure.contains("XPST0017") && failure.contains(PRINCIPAL));
        }
    }
    for version in ["1.0", "2.0"] {
        for attribute in ["select=\"'not content'\"", "name='x'", "xsl:version='1.0'"] {
            let failure = program(
                version,
                &format!("<xsl:comment {attribute}>text</xsl:comment>"),
            )
            .unwrap_err();
            assert_eq!(failure.category, FailureCategory::Unsupported);
            assert_eq!(failure.location.unwrap().resource, PRINCIPAL);
        }
    }
    let failure = program("2.0", r#"<xsl:comment><xsl:if test="false()"><xsl:value-of select="$unbound"/></xsl:if></xsl:comment>"#).unwrap_err();
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.location.unwrap().resource, PRINCIPAL);
    let failure = program(
        "1.0",
        r#"<xsl:comment><xsl:value-of select="count(//*)"/></xsl:comment>"#,
    )
    .unwrap_err();
    assert_eq!(failure.category, FailureCategory::Unsupported);
}

#[test]
fn selected_comment_controls_and_reuse_survive_failure() {
    let compiled = program(
        "1.0",
        r#"<xsl:comment><xsl:value-of select="r"/></xsl:comment>"#,
    )
    .unwrap();
    let source = source();
    let mut measured = InvocationControl::unbounded();
    let expected = execute(&compiled, &source, "measured", &mut measured).unwrap();
    let required = measured.consumed(WorkDomain::ResultNode);
    assert!(required > 0);
    for budget in [required - 1, required] {
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = budget;
        let result = execute(
            &compiled,
            &source,
            "budget",
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if budget == required {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().contains("FXCT0002"));
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XsltInstruction, 1);
    assert!(
        execute(&compiled, &source, "cancelled", &mut cancelled)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        execute(
            &compiled,
            &source,
            "recovered",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}
