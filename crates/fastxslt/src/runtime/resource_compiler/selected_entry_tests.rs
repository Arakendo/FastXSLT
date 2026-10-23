//! The same explicit entry is present in non-test workbench/WASM builds.
use super::*;
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document;
use crate::xslt::golden_semantics_experiment::{
    ProcessorPolicyStamp, StylesheetPreprocessingStamp,
};

const PRINCIPAL: &str = "https://example.invalid/main.xsl";

fn selected(modules: &[(&str, &str)]) -> Result<StylesheetProgram, ExecutionFailure> {
    selected_at(modules, PRINCIPAL)
}

fn selected_at(
    modules: &[(&str, &str)],
    identity: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(8, 32768, 65536));
    for (identity, xml) in modules {
        builder.admit(*identity, xml.as_bytes().to_vec()).unwrap();
    }
    let snapshot = builder.seal();
    let mut resolver =
        SnapshotResolver::new(&snapshot, std::iter::empty(), ResolutionLimits::new(8));
    // No test-only selected setter: the entry itself selects the coherent pair.
    compile_selected_legacy_resource(
        &mut resolver,
        identity,
        StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS),
        &mut InvocationControl::unbounded(),
    )
}

#[test]
fn non_test_selected_entry_uses_legacy_semantics_and_retains_qualification() {
    let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:include href="part.xsl"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="part"/></xsl:template></xsl:stylesheet>"#;
    let part = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template name="part"><xsl:sequence select="'modern'"><xsl:fallback><xsl:text>legacy</xsl:text></xsl:fallback></xsl:sequence></xsl:template></xsl:stylesheet>"#;
    let program = selected(&[
        (PRINCIPAL, principal),
        ("https://example.invalid/part.xsl", part),
    ])
    .unwrap();
    assert_eq!(program.processor, ProcessorPolicyStamp::Xslt10Reference);
    assert_eq!(
        program.preprocessing,
        StylesheetPreprocessingStamp::StrictXslt10Reference
    );
    // The snapshot/resolver and parsed stylesheet documents have been dropped.
    let source = Document::from_parsed(
        parse_document("urn:source", b"<r/>", super::super::XML_LIMITS).unwrap(),
    )
    .unwrap();
    assert_eq!(
        execute_compiled_stylesheet_for_test(&program, &source, "selected-entry").unwrap(),
        "legacy"
    );
}

#[test]
fn non_test_selected_entry_does_not_fall_back_to_baseline_for_guarded_grammar() {
    let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><xsl:value-of select="count(//*)"/></xsl:template></xsl:stylesheet>"#;
    let failure = selected(&[(PRINCIPAL, principal)]).unwrap_err();
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.location.as_ref().unwrap().resource, PRINCIPAL);
}

#[test]
fn non_test_selected_entry_refuses_literal_result_stylesheet_as_unsupported() {
    let principal = r#"<out xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xsl:version="1.0">valid simplified stylesheet</out>"#;
    let failure = selected(&[(PRINCIPAL, principal)]).unwrap_err();
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.location.as_ref().unwrap().resource, PRINCIPAL);
}

#[test]
fn non_test_selected_entry_refuses_embedded_principals_and_dependencies() {
    let embedded = r#"<wrapper><xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:id="chosen" version="1.0"><xsl:template name="t"><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet></wrapper>"#;
    let failure = selected_at(
        &[(PRINCIPAL, embedded)],
        "https://example.invalid/main.xsl#chosen",
    )
    .unwrap_err();
    assert_eq!(failure.category, FailureCategory::Unsupported);
    // A principal URI fragment is refused as a resolver base before XML is
    // admitted; there is intentionally no fabricated stylesheet location.
    assert!(failure.location.is_none());
    let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="part.xsl#chosen"/><xsl:template match="/"><xsl:call-template name="t"/></xsl:template></xsl:stylesheet>"#;
    let failure = selected(&[
        (PRINCIPAL, principal),
        ("https://example.invalid/part.xsl", embedded),
    ])
    .unwrap_err();
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(
        failure.location.as_ref().unwrap().resource,
        "https://example.invalid/part.xsl"
    );
}

#[test]
fn non_test_selected_entry_preserves_six_owner_introspection_boundary() {
    for (top, body, control) in [
        ("", r#"<xsl:value-of select="EXPR"/>"#, "'ok'"),
        (
            "",
            r#"<xsl:if test="EXPR"><xsl:text>ok</xsl:text></xsl:if>"#,
            "true()",
        ),
        (
            "",
            r#"<xsl:variable name="v" select="EXPR"/><xsl:value-of select="$v"/>"#,
            "'ok'",
        ),
        (
            r#"<xsl:variable name="v" select="EXPR"/>"#,
            r#"<xsl:value-of select="$v"/>"#,
            "'ok'",
        ),
        (
            "",
            r#"<xsl:param name="v" select="EXPR"/><xsl:value-of select="$v"/>"#,
            "'ok'",
        ),
        (
            r#"<xsl:template name="t"><xsl:param name="v"/><xsl:value-of select="$v"/></xsl:template>"#,
            r#"<xsl:call-template name="t"><xsl:with-param name="v" select="EXPR"/></xsl:call-template>"#,
            "'ok'",
        ),
    ] {
        for version in ["1.0", "2.0"] {
            for expression in [
                control,
                "function-available('document')",
                "element-available('xsl:sequence')",
                "system-property('xsl:vendor')",
            ] {
                let sheet = format!(
                    r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{}<xsl:template match="/">{}</xsl:template></xsl:stylesheet>"#,
                    top.replace("EXPR", expression),
                    body.replace("EXPR", expression)
                );
                let result = selected(&[(PRINCIPAL, &sheet)]);
                if expression == control {
                    let program = result.unwrap();
                    assert_eq!(program.processor, ProcessorPolicyStamp::Xslt10Reference);
                } else {
                    let failure = result.unwrap_err();
                    assert_eq!(failure.category, FailureCategory::Unsupported);
                    assert_eq!(failure.location.as_ref().unwrap().resource, PRINCIPAL);
                }
            }
        }
    }
}

#[test]
fn non_test_selected_entry_keeps_processor_reporting_and_error_timing() {
    let source = Document::from_parsed(
        parse_document("urn:source", b"<r/>", super::super::XML_LIMITS).unwrap(),
    )
    .unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        let sheet = format!(
            r#"<xsl:transform xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="system-property('p:version')"/></xsl:template></xsl:transform>"#
        );
        let program = selected(&[(PRINCIPAL, &sheet)]).unwrap();
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &source, "reporting").unwrap(),
            "1"
        );
    }
    for (condition, succeeds) in [("true()", false), ("false()", true)] {
        let sheet = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/"><xsl:if test="{condition}"><xsl:value-of select="system-property()"/></xsl:if></xsl:template></xsl:stylesheet>"#
        );
        let program = selected(&[(PRINCIPAL, &sheet)]).unwrap();
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &source, "timing").is_ok(),
            succeeds
        );
    }
    let sheet = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/"><xsl:if test="false()"><xsl:value-of select="$unbound"/></xsl:if></xsl:template></xsl:stylesheet>"#;
    let failure = selected(&[(PRINCIPAL, sheet)]).unwrap_err();
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn non_test_selected_entry_preserves_budget_cancel_and_recovery() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let sheet = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:output method="text"/><xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet>"#;
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 32768, 32768));
    builder.admit(PRINCIPAL, sheet.as_bytes().to_vec()).unwrap();
    let snapshot = builder.seal();
    let compile = |control: &mut InvocationControl| {
        let mut resolver =
            SnapshotResolver::new(&snapshot, std::iter::empty(), ResolutionLimits::new(8));
        compile_selected_legacy_resource(
            &mut resolver,
            PRINCIPAL,
            StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS),
            control,
        )
    };
    let mut measured = InvocationControl::unbounded();
    let retained = compile(&mut measured).unwrap();
    let required = measured.consumed(WorkDomain::XdmNode);
    assert!(required > 0);
    for budget in 0..=required {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = budget;
        let result = compile(&mut InvocationControl::new(
            CancellationToken::new(),
            limits,
        ));
        if budget == required {
            assert!(result.is_ok());
        } else {
            let failure = result.unwrap_err();
            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(failure.category, FailureCategory::Limit);
        }
    }
    let failure =
        compile(&mut InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 1))
            .unwrap_err();
    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.category, FailureCategory::Cancelled);
    assert_eq!(failure.location.as_ref().unwrap().resource, PRINCIPAL);
    let recovered = compile(&mut InvocationControl::unbounded()).unwrap();
    assert_eq!(recovered.processor, retained.processor);
    let source = Document::from_parsed(
        parse_document("urn:source", b"<r/>", super::super::XML_LIMITS).unwrap(),
    )
    .unwrap();
    for program in [&retained, &recovered] {
        assert_eq!(
            execute_compiled_stylesheet_for_test(program, &source, "recovery").unwrap(),
            "ok"
        );
    }
}
