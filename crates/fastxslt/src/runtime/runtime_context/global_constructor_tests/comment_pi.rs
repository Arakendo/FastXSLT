//! Comment and static-target PI bodies use ordinary execution and resolution.
use super::attribute_focus::{run, style};
use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;

#[test]
fn shared_constructor_comment_pi_controls_precede_publication() {
    let stylesheet = style(
        r#"<xsl:comment><xsl:value-of select="r/b"/></xsl:comment><xsl:processing-instruction name="target"><xsl:value-of select="r/b"/></xsl:processing-instruction>"#,
        "",
        true,
    );
    let compiled = compile_stylesheet(&stylesheet).unwrap();
    let source = document("<r><b>é🙂payload</b></r>");
    let domains = [
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::XdmNode,
        WorkDomain::XsltInstruction,
    ];
    let counts = with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        owner
            .with_completed("outer", &mut control, |_, _| Ok(()))
            .unwrap();
        domains.map(|domain| control.consumed(domain))
    });
    for (domain, exact) in domains.into_iter().zip(counts) {
        assert!(exact > 0);
        for (limit, cancel) in [(exact, false), (exact - 1, false), (exact, true)] {
            with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::ResultNode => limits.result_nodes = limit,
                    WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
                    WorkDomain::XdmNode => limits.xdm_nodes = limit,
                    WorkDomain::XsltInstruction => limits.xslt_instructions = limit,
                    _ => unreachable!(),
                }
                let mut control = InvocationControl::new(CancellationToken::new(), limits);
                if cancel {
                    control = control.cancelling_on_charge(domain, 1);
                }
                let result = owner.with_completed("outer", &mut control, |_, _| Ok(()));
                let succeeds = limit == exact && !cancel;
                assert_eq!(result.is_ok(), succeeds, "{domain:?}");
                if let Err(error) = result {
                    assert_eq!(error.code, if cancel { "FXCT0001" } else { "FXCT0002" });
                }
                assert_eq!(owner.is_complete("outer"), succeeds);
                assert_eq!(owner.depth.get(), 0);
            });
        }
    }
}

#[test]
fn shared_constructor_comment_pi_matches_ordinary_and_owns_results() {
    let body = r#"<out><xsl:comment>static</xsl:comment><xsl:processing-instruction name="fixed">static</xsl:processing-instruction><xsl:for-each select="r/b"><xsl:comment><xsl:value-of select="."/></xsl:comment><xsl:processing-instruction name="target"><xsl:value-of select="."/></xsl:processing-instruction></xsl:for-each></out>"#;
    let source = "<r><b>é🙂--tail-</b><b>a?&gt;b</b></r>";
    let ordinary = run(&style(body, "", false), source);
    assert_eq!(run(&style(body, "", true), source), ordinary);
    assert!(ordinary.contains("<!--é🙂- -tail- -->"), "{ordinary}");
    assert!(ordinary.contains("<?target a? >b?>"), "{ordinary}");
}

#[test]
fn shared_constructor_comment_pi_resolves_forward_dependency_once() {
    let stylesheet = style(
        r#"<xsl:comment><xsl:value-of select="$inner"/></xsl:comment><xsl:processing-instruction name="target"><xsl:value-of select="$inner"/></xsl:processing-instruction>"#,
        r#"<xsl:variable name="inner"><xsl:call-template name="dependency"/></xsl:variable><xsl:template name="dependency"><xsl:message>once</xsl:message><xsl:value-of select="r/b"/></xsl:template>"#,
        true,
    );
    let compiled = compile_stylesheet(&stylesheet).unwrap();
    let source = document("<r><b>owned</b></r>");
    with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "");
        assert_eq!(value(&owner, "outer", &mut control), "");
        assert_eq!(control.messages(), &["once"]);
        assert!(owner.is_complete("inner"));
        assert_eq!(owner.depth.get(), 0);
    });
    assert!(run(&stylesheet, "<r><b>owned</b></r>").contains("<!--owned--><?target owned?>"));
}

#[test]
fn shared_constructor_comment_pi_cycles_do_not_publish_or_restart() {
    for body in [
        r#"<xsl:comment><xsl:value-of select="$outer"/></xsl:comment>"#,
        r#"<xsl:processing-instruction name="target"><xsl:value-of select="$outer"/></xsl:processing-instruction>"#,
    ] {
        let stylesheet = style(body, "", true);
        let compiled = compile_stylesheet(&stylesheet).unwrap();
        let source = document("<r/>");
        with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            for _ in 0..2 {
                let error = owner
                    .with_completed("outer", &mut control, |_, _| Ok(()))
                    .unwrap_err();
                assert_eq!(error.code, "XTDE0640");
                assert!(!owner.is_complete("outer"));
                assert_eq!(owner.depth.get(), 0);
            }
        });
    }
}

#[test]
fn shared_constructor_comment_pi_keeps_unproved_body_and_target_guarded() {
    for body in [
        r#"<xsl:processing-instruction name="{r/@target}">body</xsl:processing-instruction>"#,
        r#"<xsl:comment><xsl:copy-of select="$outer"/></xsl:comment>"#,
    ] {
        let stylesheet = style(body, "", true);
        let compiled = compile_stylesheet(&stylesheet).unwrap();
        let source = document("<r target='target'/>");
        with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let error = owner
                .with_completed("outer", &mut control, |_, _| Ok(()))
                .unwrap_err();
            assert_eq!(error.code, "FXRT1004");
            assert!(!owner.is_complete("outer"));
            assert_eq!(owner.depth.get(), 0);
        });
    }
}
