//! Conditional consumers keep namespace value and sequence focus independent.

use super::{
    CancellationToken, InvocationControl, WhitespaceRepresentation, WorkDomain, WorkLimits,
    compile_stylesheet, document, run, stylesheet,
};

#[test]
fn namespace_conditionals_use_actual_uri_not_owner_text_and_preserve_mixed_positions() {
    let source = document(
        "urn:namespace:conditional",
        r#"<r xmlns:p="http://é😀" a="attr">OWNER-not-a-URI<child>needle</child></r>"#,
    );
    let body = r#"<xsl:for-each select="r/namespace::* | r/@*"><xsl:choose>
      <xsl:when test="contains(.,'http')"><xsl:if test="position() &lt;= 2 and last() = 3"><xsl:value-of select="name()"/><xsl:text>:URI|</xsl:text></xsl:if></xsl:when>
      <xsl:when test="contains(.,'attr')"><xsl:if test="position() > 2"><xsl:text>ATTR|</xsl:text></xsl:if></xsl:when>
      <xsl:otherwise><xsl:text>BAD|</xsl:text></xsl:otherwise>
    </xsl:choose></xsl:for-each><xsl:for-each select="r/child"><xsl:if test="not(contains(.,'absent')) and contains(.,'needle')"><xsl:text>TREE|</xsl:text></xsl:if></xsl:for-each>"#;
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            assert_eq!(
                run(
                    &program,
                    &source,
                    representation,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "p:URI|xml:URI|ATTR|TREE|"
            );
        }
    }
}

#[test]
fn namespace_conditionals_unicode_empty_needle_and_short_circuit_are_controlled() {
    let source = document("urn:namespace:conditional", r#"<r xmlns:p="urn:é😀"/>"#);
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, r#"<xsl:for-each select="r/namespace::p"><xsl:if test="contains(.,'é😀')"><xsl:text>unicode|</xsl:text></xsl:if><xsl:if test="contains(.,'')"><xsl:text>empty|</xsl:text></xsl:if></xsl:for-each>"#)).unwrap();
        assert_eq!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "unicode|empty|"
        );
        for test in [
            "true() or contains(.,'never')",
            "false() and contains(.,'never')",
            "not(false() and contains(.,'never'))",
        ] {
            let program = compile_stylesheet(&stylesheet(version, &format!(r#"<xsl:for-each select="r/namespace::p"><xsl:if test="{test}"><xsl:text>hit</xsl:text></xsl:if></xsl:for-each>"#))).unwrap();
            let mut cancelled = InvocationControl::unbounded()
                .cancelling_on_charge(WorkDomain::XdmStringValueNode, 0);
            let output = run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut cancelled,
            )
            .unwrap();
            assert_eq!(
                output,
                if test.starts_with("false()") {
                    ""
                } else {
                    "hit"
                }
            );
        }
    }
}

#[test]
fn namespace_conditional_charge_failures_discard_output_and_allow_reuse() {
    let source = document("urn:namespace:conditional", r#"<r xmlns:p="urn:p"/>"#);
    let program = compile_stylesheet(&stylesheet("1.0", r#"<xsl:for-each select="r/namespace::p"><xsl:if test="contains(.,'urn')"><xsl:text>hit</xsl:text></xsl:if></xsl:for-each>"#)).unwrap();
    let representation = WhitespaceRepresentation::VisibilityView;
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmStringValueNode, 0);
    let error = run(&program, &source, representation, &mut cancelled).unwrap_err();
    assert_eq!(error.code, "FXCT0001");
    assert_eq!(error.work_domain, Some(WorkDomain::XdmStringValueNode));
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        run(&program, &source, representation, &mut measured).unwrap(),
        "hit"
    );
    for domain in [
        WorkDomain::XPathOperation,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XdmStringValueNode,
    ] {
        let used = measured.consumed(domain);
        for allowance in [used - 1, used] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XPathOperation => limits.xpath_operations = allowance,
                WorkDomain::XPathNodeVisit => limits.xpath_node_visits = allowance,
                WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = allowance,
                _ => unreachable!(),
            }
            let result = run(
                &program,
                &source,
                representation,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance == used {
                assert_eq!(result.unwrap(), "hit");
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.code, "FXCT0002");
                assert_eq!(error.work_domain, Some(domain));
            }
        }
    }
    assert_eq!(
        run(
            &program,
            &source,
            representation,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "hit"
    );
}
