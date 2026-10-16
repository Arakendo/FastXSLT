//! Ordinary attribute focus remains separate from recursive global entry focus.
use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};

pub(super) fn style(body: &str, declarations: &str, global: bool) -> Document {
    let outer = if global {
        r#"<xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>"#
    } else {
        ""
    };
    let entry = if global {
        r#"<xsl:copy-of select="$outer"/>"#
    } else {
        r#"<xsl:call-template name="build"/>"#
    };
    document(&format!(
        r#"
      <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:p" version="1.0">
        <xsl:output omit-xml-declaration="yes"/>{outer}{declarations}
        <xsl:template name="build">{body}</xsl:template>
        <xsl:template match="/">{entry}</xsl:template>
      </xsl:stylesheet>"#
    ))
}

pub(super) fn run(style: &Document, source: &str) -> String {
    let compiled = compile_stylesheet(style).unwrap();
    let source = document(source);
    let mut control = InvocationControl::unbounded();
    let result = execute_program(&compiled, &source, "attribute-focus", &mut control).unwrap();
    let output = compiled.output.clone();
    drop(compiled);
    drop(source);
    serialize_xml(&result, &output, "attribute-focus", 16384, &mut control).unwrap()
}

#[test]
fn shared_constructor_attribute_focus_matches_ordinary_iteration() {
    let body = r#"<xsl:for-each select="r/*"><e pos="{position()}" size="{last()}" name="{name()}" local="{local-name()}" text="{.}" source="{@a}" missing="{@missing}"/></xsl:for-each>"#;
    let source = "<r xmlns:p='urn:p'><p:b a='é🙂'>one</p:b><c>two</c></r>";
    let ordinary = run(&style(body, "", false), source);
    let constructed = run(&style(body, "", true), source);
    assert_eq!(constructed, ordinary);
    for expected in [
        "pos=\"1\"",
        "pos=\"2\"",
        "size=\"2\"",
        "name=\"p:b\"",
        "local=\"b\"",
        "text=\"one\"",
        "source=\"é🙂\"",
        "missing=\"\"",
    ] {
        assert!(constructed.contains(expected), "{constructed}");
    }
}

#[test]
fn shared_constructor_attribute_focus_recursive_global_starts_at_root() {
    let declarations = r#"
      <xsl:variable name="inner"><xsl:call-template name="root"/></xsl:variable>
      <xsl:template name="root"><xsl:message>once</xsl:message><root pos="{position()}" size="{last()}" name="{name()}" local="{local-name()}" text="{.}" source="{@a}"/></xsl:template>"#;
    let style = style(
        r#"<xsl:for-each select="r/b"><e dependency="{$inner}"/></xsl:for-each>"#,
        declarations,
        true,
    );
    let compiled = compile_stylesheet(&style).unwrap();
    let source = document("<r a='not-a-root-attribute'><b>A</b><b>B</b></r>");
    with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        owner
            .with_completed("outer", &mut control, |_, _| Ok(()))
            .unwrap();
        owner
            .with_completed("inner", &mut control, |globals, _| {
                use crate::runtime::golden_runtime_experiment::runtime_context::TemporaryNodeKind;
                let values = globals.temporary_trees["inner"]
                    .nodes
                    .iter()
                    .filter_map(|node| {
                        if let TemporaryNodeKind::Attribute { name, value } = &node.kind {
                            Some((name.local.as_str(), value.as_str()))
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeMap<_, _>>();
                assert_eq!(
                    values,
                    BTreeMap::from([
                        ("pos", "1"),
                        ("size", "1"),
                        ("name", ""),
                        ("local", ""),
                        ("text", "AB"),
                        ("source", "")
                    ])
                );
                Ok(())
            })
            .unwrap();
        assert_eq!(control.messages(), &["once"]);
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_attribute_focus_preserves_effective_whitespace() {
    let body = r#"<xsl:for-each select="r/node()"><e pos="{position()}" size="{last()}" text="{.}"/></xsl:for-each>"#;
    for strip in [false, true] {
        let declarations = if strip {
            r#"<xsl:strip-space elements="*"/>"#
        } else {
            ""
        };
        let source = "<r> <b>A</b> <b>B</b> </r>";
        let ordinary = run(&style(body, declarations, false), source);
        let constructed = run(&style(body, declarations, true), source);
        assert_eq!(constructed, ordinary);
        assert!(constructed.contains(if strip { "size=\"2\"" } else { "size=\"5\"" }));
    }
}

#[test]
fn shared_constructor_attribute_focus_isolates_concurrent_sources() {
    let stylesheet = style(
        r#"<xsl:for-each select="r/b"><e pos="{position()}" size="{last()}" source="{@a}"/></xsl:for-each>"#,
        "",
        true,
    );
    let compiled = compile_stylesheet(&stylesheet).unwrap();
    std::thread::scope(|scope| {
        let handles = (1..=8)
            .map(|count| {
                let compiled = &compiled;
                scope.spawn(move || {
                    let source = document(&format!(
                        "<r>{}</r>",
                        format!("<b a='{count}'/>").repeat(count)
                    ));
                    let mut control = InvocationControl::unbounded();
                    let result =
                        execute_program(compiled, &source, "concurrent-focus", &mut control)
                            .unwrap();
                    let xml = serialize_xml(
                        &result,
                        &compiled.output,
                        "concurrent-focus",
                        16384,
                        &mut control,
                    )
                    .unwrap();
                    assert_eq!(xml.matches(&format!("size=\"{count}\"")).count(), count);
                    assert_eq!(xml.matches(&format!("source=\"{count}\"")).count(), count);
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn shared_constructor_attribute_focus_charges_before_publication() {
    let stylesheet = style(
        r#"<xsl:for-each select="r/b"><e name="{name()}" text="{.}" source="{@a}"/></xsl:for-each>"#,
        "",
        true,
    );
    let compiled = compile_stylesheet(&stylesheet).unwrap();
    let source = document("<r><b a='one'>é🙂</b><b a='two'>second</b></r>");
    let domains = [
        WorkDomain::ResultNode,
        WorkDomain::XdmNode,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XdmStringValueNode,
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
        assert!(exact > 0, "{domain:?}");
        for (limit, cancel) in [(exact, false), (exact - 1, false), (exact, true)] {
            with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::ResultNode => limits.result_nodes = limit,
                    WorkDomain::XdmNode => limits.xdm_nodes = limit,
                    WorkDomain::XPathNodeVisit => limits.xpath_node_visits = limit,
                    WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
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
