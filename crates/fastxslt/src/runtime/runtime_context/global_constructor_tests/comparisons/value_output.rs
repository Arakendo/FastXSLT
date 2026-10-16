//! Boolean/comparison output uses the same demand readers as conditions.
use super::*;

#[test]
fn shared_constructor_comparison_output_specialization_declines_expressions() {
    for named in [false, true] {
        let body = r#"<xsl:value-of select="$left = $right"/>"#;
        let call = if named {
            r#"<xsl:call-template name="compare"/>"#
        } else {
            body
        };
        let program = program(&format!(
            r#"
          <xsl:variable name="outer">{call}</xsl:variable>
          <xsl:variable name="left" select="'x'"/><xsl:variable name="right" select="'x'"/>
          <xsl:template name="compare">{body}</xsl:template>"#
        ));
        assert!(matches!(
            program
                .global_bindings
                .iter()
                .find(|b| b.name == "outer")
                .unwrap()
                .default,
            GlobalBindingDefault::SharedConstructor { .. }
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            assert_eq!(
                value(&owner, "outer", &mut InvocationControl::unbounded()),
                "true"
            );
        });
    }
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="read"/></xsl:variable>
      <xsl:variable name="left" select="'x'"/>
      <xsl:template name="read"><xsl:value-of select="$left"/></xsl:template>"#,
    );
    assert!(matches!(
        program
            .global_bindings
            .iter()
            .find(|b| b.name == "outer")
            .unwrap()
            .default,
        GlobalBindingDefault::Xslt10TemporaryTextParts(_)
    ));
}

#[test]
fn shared_constructor_comparison_output_fallback_does_not_accept_invalid_references() {
    use crate::compile::golden_stylesheet_experiment::{CompileCategory, compile_stylesheet};
    for expression in ["$bad:", "$", "$missing:name"] {
        let stylesheet = document(&format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            <xsl:variable name="outer"><xsl:call-template name="read"/></xsl:variable>
            <xsl:template name="read"><xsl:value-of select="{expression}"/></xsl:template>
            <xsl:template match="/"/>
          </xsl:stylesheet>"#
        ));
        let error = compile_stylesheet(&stylesheet).unwrap_err();
        assert_eq!(
            error.category,
            CompileCategory::Invalid,
            "{expression}: {error:?}"
        );
        assert_eq!(error.location.resource, "urn:initializer-handoff");
    }
}

#[test]
fn shared_constructor_comparison_output_preserves_overrides_and_invocation_isolation() {
    use crate::xdm::atomic_value_experiment::AtomicValue;
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/>
      <xsl:param name="right"><xsl:call-template name="fail"/></xsl:param>
      <xsl:template name="fail"><xsl:message terminate="yes">default</xsl:message></xsl:template>
      <xsl:template name="compare"><xsl:value-of select="$left = $right"/></xsl:template>"#,
    );
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|index| {
                let program = &program;
                scope.spawn(move || {
                    let source = document(&format!("<r><b>{index}</b></r>"));
                    let supplied = BTreeMap::from([(
                        "right".into(),
                        InvocationParameter {
                            value: InvocationParameterValue::Atomic(AtomicValue::string("3")),
                            tunnel: false,
                        },
                    )]);
                    with_owner(program, Some(&source), &supplied, |owner| {
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
fn shared_constructor_comparison_output_retains_unproved_scalar_guards() {
    for expression in ["$left = 'false'", "$left = 1"] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:variable name="left" select="false()"/>
          <xsl:template name="compare"><xsl:value-of select="{expression}"/></xsl:template>"#
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let error = owner
                .read_legacy_string("outer", &mut InvocationControl::unbounded())
                .unwrap_err();
            assert_eq!(error.category, FailureCategory::Unsupported);
            assert_eq!(error.code, "FXRT1004");
            assert!(!owner.is_complete("outer"));
        });
    }
}

#[test]
fn shared_constructor_comparison_output_matches_ordinary_execution() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    for (expression, expected) in [
        ("boolean($left)", "true"),
        ("$left = true()", "true"),
        ("$left != false()", "true"),
        ("$left = '2'", "true"),
        ("$left != '2'", "true"),
        ("not($left = '2')", "false"),
        ("$left = 2", "true"),
        ("$left != 2", "true"),
        ("not($left = 2)", "false"),
        ("$left = $right", "true"),
        ("$left != $right", "true"),
    ] {
        for global in [false, true] {
            let outer = if global {
                r#"<xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>"#
            } else {
                ""
            };
            let entry = if global {
                r#"<xsl:value-of select="$outer"/>"#
            } else {
                r#"<xsl:call-template name="compare"/>"#
            };
            let stylesheet = document(&format!(
                r#"
              <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
                <xsl:output method="text"/>{outer}
                <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="r/c"/>
                <xsl:template match="/">{entry}</xsl:template>
                <xsl:template name="compare"><xsl:value-of select="{expression}"/></xsl:template>
              </xsl:stylesheet>"#
            ));
            let compiled = compile_stylesheet(&stylesheet).unwrap();
            assert_eq!(
                execute_compiled_stylesheet_for_test(
                    &compiled,
                    &document("<r><b>1</b><b>2</b><c>2</c></r>"),
                    "comparison-output"
                )
                .unwrap(),
                expected,
                "global={global}: {expression}"
            );
        }
    }
}

#[test]
fn shared_constructor_comparison_output_preserves_empty_and_fragment_kinds() {
    for (declaration, expression, expected) in [
        (r#"select="r/missing""#, "$left = '2'", "false"),
        (r#"select="r/missing""#, "$left != '2'", "false"),
        (r#"select="r/missing""#, "$left = 2", "false"),
        (r#"select="r/missing""#, "$left != 2", "false"),
        (r#"select="r/missing""#, "boolean($left)", "false"),
        (r#"select="false()""#, "boolean($left)", "false"),
        (r#"select="'false'""#, "boolean($left)", "true"),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:variable name="left" {declaration}/>
          <xsl:template name="compare"><xsl:value-of select="{expression}"/></xsl:template>"#
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            assert_eq!(
                value(&owner, "outer", &mut InvocationControl::unbounded()),
                expected
            );
        });
    }
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left"><xsl:if test="false()">unused</xsl:if></xsl:variable>
      <xsl:template name="compare"><xsl:value-of select="boolean($left)"/></xsl:template>"#,
    );
    let source = document("<r/>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        assert_eq!(
            value(&owner, "outer", &mut InvocationControl::unbounded()),
            "true"
        );
    });
}

#[test]
fn shared_constructor_comparison_output_cycles_preserve_failure_and_shadowing() {
    for (local, expected) in [
        ("", None),
        (r#"<xsl:variable name="outer" select="'2'"/>"#, Some("true")),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:variable name="right" select="'2'"/>
          <xsl:template name="compare">{local}<xsl:value-of select="$outer = $right"/></xsl:template>"#
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let result = owner.read_legacy_string("outer", &mut control);
            if let Some(expected) = expected {
                assert_eq!(result.unwrap(), expected);
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.code, "XTDE0640");
                assert_eq!(
                    owner.read_legacy_string("outer", &mut control).unwrap_err(),
                    error
                );
                assert!(!owner.is_complete("outer"));
            }
            assert_eq!(owner.depth.get(), 0);
        });
    }
}

#[test]
fn shared_constructor_comparison_output_charges_before_publication() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="r/c"/>
      <xsl:template name="compare"><xsl:value-of select="$left != $right"/></xsl:template>"#,
    );
    let source = document("<r><b>1</b><b>2</b><c>2</c></r>");
    let domains = [
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::XdmNode,
        WorkDomain::XdmStringValueNode,
        WorkDomain::XPathOperation,
    ];
    let counts = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        owner
            .with_completed("outer", &mut control, |_, _| Ok(()))
            .unwrap();
        domains.map(|domain| control.consumed(domain))
    });
    for (domain, exact) in domains.into_iter().zip(counts) {
        assert!(exact > 0);
        for (limit, cancel) in [(exact, false), (exact - 1, false), (exact, true)] {
            with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::ResultNode => limits.result_nodes = limit,
                    WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
                    WorkDomain::XdmNode => limits.xdm_nodes = limit,
                    WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
                    WorkDomain::XPathOperation => limits.xpath_operations = limit,
                    _ => unreachable!(),
                }
                let mut control = InvocationControl::new(CancellationToken::new(), limits);
                if cancel {
                    control = control.cancelling_on_charge(domain, 0);
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
