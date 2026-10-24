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
            "urn:pi-source",
            b"<r>value</r>",
            super::super::super::XML_LIMITS,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn selected_pi_content_policy_and_message_conversion() {
    for version in ["1.0", "2.0", "3.0"] {
        let compiled = program(version, r#"<xsl:processing-instruction name="target">A<!-- remove -->   <xsl:value-of select="r"/><xsl:text>?&gt;</xsl:text></xsl:processing-instruction>"#).unwrap();
        assert_eq!(
            execute(
                &compiled,
                &source(),
                "pi",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "<?target Avalue? >?>"
        );
    }
    let compiled = program("2.0", r#"<xsl:processing-instruction name="target"><xsl:sequence><xsl:fallback><xsl:value-of select="system-property('xsl:version')"/></xsl:fallback></xsl:sequence></xsl:processing-instruction>"#).unwrap();
    assert_eq!(
        execute(
            &compiled,
            &source(),
            "policy",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<?target 1?>"
    );
    let compiled = program("1.0", r#"<xsl:message>before<xsl:processing-instruction name="target"><xsl:value-of select="r"/></xsl:processing-instruction>after</xsl:message><out/>"#).unwrap();
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        execute(&compiled, &source(), "message", &mut control).unwrap(),
        "<out></out>"
    );
    assert_eq!(control.messages(), ["beforeafter"]);
}

#[test]
fn selected_pi_error_timing_and_guarded_targets() {
    for test in ["false()", "true()"] {
        let compiled = program("2.0", &format!(r#"<xsl:processing-instruction name="target"><xsl:if test="{test}"><xsl:value-of select="system-property()"/></xsl:if></xsl:processing-instruction>"#)).unwrap();
        let result = execute(
            &compiled,
            &source(),
            "timing",
            &mut InvocationControl::unbounded(),
        );
        if test == "false()" {
            assert!(result.is_ok());
        } else {
            let error = result.unwrap_err();
            assert!(error.contains("XPST0017") && error.contains(PRINCIPAL));
        }
    }
    let error = program("2.0", r#"<xsl:processing-instruction name="target"><xsl:if test="false()"><xsl:value-of select="$unbound"/></xsl:if></xsl:processing-instruction>"#).unwrap_err();
    assert_eq!(error.category, FailureCategory::Invalid);
    for version in ["1.0", "2.0"] {
        for attributes in [
            "name='xml'",
            "name='XmL'",
            "name='p:name'",
            "name=''",
            "name='{r}'",
            "name='target' select='r'",
            "name='target' xsl:version='1.0'",
        ] {
            let error = program(
                version,
                &format!(
                    "<xsl:processing-instruction {attributes}>value</xsl:processing-instruction>"
                ),
            )
            .unwrap_err();
            assert_eq!(error.category, FailureCategory::Unsupported);
            assert_eq!(error.location.unwrap().resource, PRINCIPAL);
        }
    }
}

#[test]
fn selected_pi_controls_and_post_failure_reuse() {
    let compiled = program("1.0", r#"<xsl:processing-instruction name="target"><xsl:value-of select="r"/></xsl:processing-instruction>"#).unwrap();
    let source = source();
    let mut measured = InvocationControl::unbounded();
    let expected = execute(&compiled, &source, "measure", &mut measured).unwrap();
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
        execute(&compiled, &source, "cancel", &mut cancelled)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        execute(
            &compiled,
            &source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}
