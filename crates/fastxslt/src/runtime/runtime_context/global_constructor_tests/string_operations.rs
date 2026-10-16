//! String consumers retain the ordinary conversion and control path.
use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;

fn stylesheet(expression: &str, declarations: &str, global: bool) -> Document {
    let outer = if global {
        r#"<xsl:variable name="outer"><xsl:call-template name="read"/></xsl:variable>"#
    } else {
        ""
    };
    let entry = if global {
        r#"<xsl:value-of select="$outer"/>"#
    } else {
        r#"<xsl:call-template name="read"/>"#
    };
    document(&format!(
        r#"
      <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
        <xsl:output method="text"/>{outer}{declarations}
        <xsl:template match="/">{entry}</xsl:template>
        <xsl:template name="read"><xsl:value-of select="{expression}"/></xsl:template>
      </xsl:stylesheet>"#
    ))
}

#[test]
fn shared_constructor_string_operations_match_ordinary_unicode_execution() {
    for (expression, expected) in [
        ("normalize-space($text)", "é🙂 x"),
        ("string-length($text)", "8"),
        ("contains($text, '🙂')", "true"),
        ("starts-with($text, '  é')", "true"),
        ("substring-before($text, '🙂')", "  é"),
        ("substring-after($text, '🙂')", "  x "),
        ("substring($text, 3, 2)", "é🙂"),
        ("contains($text, $needle)", "true"),
    ] {
        for global in [false, true] {
            let style = stylesheet(
                expression,
                r#"<xsl:variable name="text" select="r/b"/><xsl:variable name="needle" select="'🙂'"/>"#,
                global,
            );
            let compiled = compile_stylesheet(&style).unwrap();
            let actual = execute_compiled_stylesheet_for_test(
                &compiled,
                &document("<r><b>  é🙂  x </b><b>not the first node</b></r>"),
                "strings",
            )
            .unwrap();
            assert_eq!(actual, expected, "{expression}, global={global}");
        }
    }
}

#[test]
fn shared_constructor_string_operations_preserve_value_kinds() {
    for (declaration, expected) in [
        (r#"<xsl:variable name="text" select="r/missing"/>"#, "0"),
        (r#"<xsl:variable name="text" select="false()"/>"#, "5"),
        (r#"<xsl:variable name="text" select="12"/>"#, "2"),
        (
            r#"<xsl:variable name="text"><xsl:if test="false()">unused</xsl:if></xsl:variable>"#,
            "0",
        ),
        (
            r#"<xsl:variable name="text"><xsl:call-template name="fragment"/></xsl:variable><xsl:template name="fragment"><a>é</a><b>🙂</b></xsl:template>"#,
            "2",
        ),
    ] {
        for global in [false, true] {
            let style = stylesheet("string-length($text)", declaration, global);
            let compiled = compile_stylesheet(&style).unwrap();
            assert_eq!(
                execute_compiled_stylesheet_for_test(&compiled, &document("<r/>"), "kinds")
                    .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn shared_constructor_string_operations_reenter_without_borrowing_completed_values() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="read"/></xsl:variable>
      <xsl:variable name="text" select="'abc'"/>
      <xsl:variable name="needle"><xsl:call-template name="build"/></xsl:variable>
      <xsl:template name="read"><xsl:value-of select="contains($text, $needle)"/><xsl:value-of select="contains($text, $needle)"/></xsl:template>
      <xsl:template name="build"><xsl:message>once</xsl:message><xsl:value-of select="substring($text, 2, 1)"/></xsl:template>"#,
    );
    let source = document("<r/>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "truetrue");
        assert_eq!(control.messages(), &["once"]);
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_string_operations_cycles_and_shadowing() {
    for expression in [
        "normalize-space($outer)",
        "string-length($outer)",
        "substring($outer, 1, 2)",
        "contains($outer, 'x')",
    ] {
        for local in ["", r#"<xsl:variable name="outer" select="'x'"/>"#] {
            let program = program(&format!(
                r#"
              <xsl:variable name="outer"><xsl:call-template name="read"/></xsl:variable>
              <xsl:template name="read">{local}<xsl:value-of select="{expression}"/></xsl:template>"#
            ));
            let source = document("<r/>");
            with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
                let mut control = InvocationControl::unbounded();
                let result = owner.read_legacy_string("outer", &mut control);
                if local.is_empty() {
                    let error = result.unwrap_err();
                    assert_eq!(error.code, "XTDE0640");
                    assert_eq!(
                        owner.read_legacy_string("outer", &mut control).unwrap_err(),
                        error
                    );
                    assert!(!owner.is_complete("outer"));
                } else {
                    assert!(result.is_ok());
                }
                assert_eq!(owner.depth.get(), 0);
            });
        }
    }
}

#[test]
fn shared_constructor_string_operations_controls_precede_publication() {
    for expression in [
        "normalize-space($text)",
        "string-length($text)",
        "substring($text, 1, 2)",
        "contains($text, $needle)",
    ] {
        let style = stylesheet(
            expression,
            r#"<xsl:variable name="text" select="r/b"/><xsl:variable name="needle" select="'x'"/>"#,
            true,
        );
        let program = compile_stylesheet(&style).unwrap();
        let source = document("<r><b> x 🙂 </b></r>");
        let domains = [
            WorkDomain::XPathOperation,
            WorkDomain::ResultTextByte,
            WorkDomain::ResultNode,
            WorkDomain::XdmStringValueNode,
        ];
        let counts = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            owner
                .with_completed("outer", &mut control, |_, _| Ok(()))
                .unwrap();
            domains.map(|d| control.consumed(d))
        });
        for (domain, exact) in domains.into_iter().zip(counts) {
            assert!(exact > 0);
            for (limit, cancel) in [(exact, false), (exact - 1, false), (exact, true)] {
                with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
                    let mut limits = WorkLimits::unbounded();
                    match domain {
                        WorkDomain::XPathOperation => limits.xpath_operations = limit,
                        WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
                        WorkDomain::ResultNode => limits.result_nodes = limit,
                        WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
                        _ => unreachable!(),
                    }
                    let mut control = InvocationControl::new(CancellationToken::new(), limits);
                    if cancel {
                        control = control.cancelling_on_charge(domain, 0);
                    }
                    let result = owner.with_completed("outer", &mut control, |_, _| Ok(()));
                    let succeeds = limit == exact && !cancel;
                    assert_eq!(result.is_ok(), succeeds, "{expression}: {domain:?}");
                    if let Err(error) = result {
                        assert_eq!(error.code, if cancel { "FXCT0001" } else { "FXCT0002" });
                    }
                    assert_eq!(owner.is_complete("outer"), succeeds);
                    assert_eq!(owner.depth.get(), 0);
                });
            }
        }
    }
}

#[test]
fn shared_constructor_string_operations_preserve_effective_view_and_result_lifetime() {
    use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
    for strip in [false, true] {
        let style = stylesheet(
            "string-length($text)",
            &format!(
                r#"
          {}<xsl:variable name="text" select="r/b"/>"#,
                if strip {
                    r#"<xsl:strip-space elements="*"/>"#
                } else {
                    ""
                }
            ),
            true,
        );
        let compiled = compile_stylesheet(&style).unwrap();
        let source = document("<r><b><e>é</e> <e>🙂</e></b></r>");
        let mut control = InvocationControl::unbounded();
        let result = execute_program(&compiled, &source, "string-view", &mut control).unwrap();
        let output = compiled.output.clone();
        drop(compiled);
        drop(style);
        drop(source);
        assert_eq!(
            serialize_xml(&result, &output, "string-view", 1024, &mut control).unwrap(),
            if strip { "2" } else { "3" }
        );
    }
}

#[test]
fn shared_constructor_string_operations_parameter_overrides_isolate_concurrent_invocations() {
    use crate::xdm::atomic_value_experiment::AtomicValue;
    let style = stylesheet(
        "contains($text, $needle)",
        r#"
      <xsl:variable name="text" select="r/b"/>
      <xsl:param name="needle"><xsl:call-template name="fail"/></xsl:param>
      <xsl:template name="fail"><xsl:message terminate="yes">unused</xsl:message></xsl:template>"#,
        true,
    );
    let compiled = compile_stylesheet(&style).unwrap();
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|index| {
                let compiled = &compiled;
                scope.spawn(move || {
                    let source = document(&format!("<r><b>{index}</b></r>"));
                    let parameters = BTreeMap::from([(
                        "needle".into(),
                        InvocationParameter {
                            value: InvocationParameterValue::Atomic(AtomicValue::string("3")),
                            tunnel: false,
                        },
                    )]);
                    with_owner(compiled, Some(&source), &parameters, |owner| {
                        let mut control = InvocationControl::unbounded();
                        let result = value(&owner, "outer", &mut control);
                        owner.initialize_all(&mut control).unwrap();
                        assert!(control.messages().is_empty());
                        result
                    })
                })
            })
            .collect::<Vec<_>>();
        for (index, handle) in handles.into_iter().enumerate() {
            assert_eq!(
                handle.join().unwrap(),
                if index == 3 { "true" } else { "false" }
            );
        }
    });
}

#[test]
fn shared_constructor_string_operations_keep_originating_failure_without_replay() {
    let style = stylesheet(
        "contains($text, $needle)",
        r#"
      <xsl:variable name="text" select="'abc'"/>
      <xsl:variable name="needle"><xsl:call-template name="fail"/></xsl:variable>
      <xsl:template name="fail"><xsl:message>before</xsl:message><xsl:message terminate="yes">stop</xsl:message></xsl:template>"#,
        true,
    );
    let compiled = compile_stylesheet(&style).unwrap();
    let source = document("<r/>");
    with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
        assert_eq!(error.code, "XTMM9000");
        assert_eq!(
            error.location.as_ref().unwrap().resource,
            "urn:initializer-handoff"
        );
        let messages = control.messages().to_vec();
        assert_eq!(
            owner.read_legacy_string("outer", &mut control).unwrap_err(),
            error
        );
        assert_eq!(control.messages(), messages);
        assert!(owner.is_complete("text"));
        assert!(!owner.is_complete("needle") && !owner.is_complete("outer"));
        assert_eq!(owner.depth.get(), 0);
    });
}
