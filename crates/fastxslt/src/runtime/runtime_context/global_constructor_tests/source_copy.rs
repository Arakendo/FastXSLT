//! Principal-source copies preserve the effective view and result ownership.
use super::attribute_focus::{run, style};
use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;

#[test]
fn shared_constructor_source_copy_matches_ordinary_mixed_nodes_and_namespaces() {
    let source = "<r xmlns:p='urn:p' a='v'>text<p:b p:a='é🙂'><!--c--><?pi value?><p:c/></p:b><b>tail</b></r>";
    for select in ["/", ".", "*", "r/node()", "r/p:b", "r/@a", "r/missing"] {
        let body = format!(r#"<out><xsl:copy-of select="{select}"/></out>"#);
        let ordinary = run(&style(&body, "", false), source);
        let actual = run(&style(&body, "", true), source);
        assert_eq!(actual, ordinary, "{select}");
        if select == "r/p:b" {
            assert!(actual.contains("é🙂"));
            assert!(actual.contains("<!--c--><?pi value?>"));
            assert!(actual.contains("xmlns:p=\"urn:p\""));
        }
    }
}

#[test]
fn shared_constructor_source_copy_uses_iteration_focus_and_effective_whitespace() {
    let source = "<r> <b> A <c/> </b> <b xml:space='preserve'> <c/> </b> </r>";
    for strip in ["", r#"<xsl:strip-space elements="*"/>"#] {
        for select in [".", "*", "node()", "@*"] {
            let body = format!(
                r#"<out><xsl:for-each select="r/b"><e><xsl:copy-of select="{select}"/></e></xsl:for-each></out>"#
            );
            assert_eq!(
                run(&style(&body, strip, true), source),
                run(&style(&body, strip, false), source),
                "{strip} {select}"
            );
        }
    }
}

#[test]
fn shared_constructor_source_copy_nested_global_has_fresh_root() {
    let stylesheet = style(
        r#"<out><xsl:for-each select="r/b"><xsl:value-of select="$inner"/></xsl:for-each></out>"#,
        r#"<xsl:variable name="inner"><xsl:call-template name="copy"/></xsl:variable><xsl:template name="copy"><xsl:message>once</xsl:message><xsl:copy-of select="."/></xsl:template>"#,
        true,
    );
    let compiled = compile_stylesheet(&stylesheet).unwrap();
    let source = document("<r><b>A</b><b>B</b></r>");
    with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "ABAB");
        assert_eq!(control.messages(), &["once"]);
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_source_copy_controls_precede_publication() {
    let stylesheet = style(r#"<out><xsl:copy-of select="r/node()"/></out>"#, "", true);
    let compiled = compile_stylesheet(&stylesheet).unwrap();
    let source = document("<r><b a='value'>é🙂<!--comment--><?pi value?></b><b>second</b></r>");
    let domains = [
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::XdmNode,
        WorkDomain::XPathNodeVisit,
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
                    WorkDomain::XPathNodeVisit => limits.xpath_node_visits = limit,
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
