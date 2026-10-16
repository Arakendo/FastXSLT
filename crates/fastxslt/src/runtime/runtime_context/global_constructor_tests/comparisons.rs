//! Global-constructor comparison readers, parity and control boundaries.
use super::*;

#[path = "comparisons/pair_equality.rs"]
mod pair_equality;

#[path = "comparisons/pair_relations.rs"]
mod pair_relations;

#[path = "comparisons/value_output.rs"]
mod value_output;

fn numeric_program(declaration: &str, expression: &str) -> StylesheetProgram {
    program(&format!(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      {declaration}
      <xsl:template name="compare"><xsl:if test="{expression}">selected</xsl:if></xsl:template>"#
    ))
}

#[test]
fn shared_constructor_numeric_relations_preserve_parameter_override() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:param name="nodes"><xsl:call-template name="fail"/></xsl:param>
      <xsl:template name="compare"><xsl:if test="$nodes > 2">selected</xsl:if></xsl:template>
      <xsl:template name="fail"><xsl:message terminate="yes">default</xsl:message></xsl:template>"#,
    );
    let source = document("<r/>");
    for (kind, lexical, expected) in [
        (BuiltinAtomicType::Integer, "3", Some("selected")),
        (BuiltinAtomicType::Decimal, "2.5", Some("selected")),
        (BuiltinAtomicType::UntypedAtomic, "invalid", Some("")),
        (BuiltinAtomicType::Boolean, "true", None),
        (BuiltinAtomicType::Double, "INF", None),
    ] {
        let supplied = BTreeMap::from([(
            "nodes".into(),
            InvocationParameter {
                value: InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                    kind, lexical,
                )),
                tunnel: false,
            },
        )]);
        with_owner(&program, Some(&source), &supplied, |owner| {
            let mut control = InvocationControl::unbounded();
            let result = owner.read_legacy_string("outer", &mut control);
            if let Some(expected) = expected {
                assert_eq!(result.unwrap(), expected);
                owner.initialize_all(&mut control).unwrap();
            } else {
                assert_eq!(result.unwrap_err().code, "FXRT1004");
            }
            assert!(control.messages().is_empty());
        });
    }
}

#[test]
fn shared_constructor_numeric_relations_keep_invocations_isolated() {
    let program = numeric_program(r#"<xsl:variable name="nodes" select="r/b"/>"#, "$nodes > 2");
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|index| {
                let program = &program;
                scope.spawn(move || {
                    let source = document(&format!("<r><b>0</b><b>{index}</b></r>"));
                    with_owner(program, Some(&source), &BTreeMap::new(), |owner| {
                        value(&owner, "outer", &mut InvocationControl::unbounded())
                    })
                })
            })
            .collect::<Vec<_>>();
        for (index, handle) in handles.into_iter().enumerate() {
            assert_eq!(
                handle.join().unwrap(),
                if index > 2 { "selected" } else { "" }
            );
        }
    });
}

#[test]
fn shared_constructor_numeric_relations_scan_later_nodes_and_preserve_nan() {
    for (expression, xml, expected) in [
        ("$nodes > 2", "<r><b>1</b><b>3</b></r>", "selected"),
        ("$nodes &lt; 2", "<r><b>3</b><b>1</b></r>", "selected"),
        ("$nodes >= 2", "<r><b>1</b><b>2</b></r>", "selected"),
        ("$nodes &lt;= 2", "<r><b>3</b><b>2</b></r>", "selected"),
        ("2 &lt; $nodes", "<r><b>1</b><b>3</b></r>", "selected"),
        ("2 >= $nodes", "<r><b>3</b><b>2</b></r>", "selected"),
        ("$nodes > 2", "<r/>", ""),
        ("$nodes &lt;= 2", "<r><b>invalid</b></r>", ""),
        ("$nodes > 2", "<r><b>invalid</b><b>3</b></r>", "selected"),
    ] {
        let program = numeric_program(r#"<xsl:variable name="nodes" select="r/b"/>"#, expression);
        let source = document(xml);
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            assert_eq!(
                value(&owner, "outer", &mut InvocationControl::unbounded()),
                expected,
                "{expression}: {xml}"
            );
        });
    }
}

#[test]
fn shared_constructor_numeric_relations_match_ordinary_execution() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    for (declaration, expected) in [
        (r#"<xsl:variable name="nodes" select="r/b"/>"#, "selected"),
        (r#"<xsl:variable name="nodes" select="r/missing"/>"#, ""),
        (r#"<xsl:variable name="nodes" select="3"/>"#, "selected"),
        (r#"<xsl:variable name="nodes" select="'3'"/>"#, "selected"),
        (r#"<xsl:variable name="nodes" select="'invalid'"/>"#, ""),
        (
            r#"<xsl:variable name="nodes"><xsl:text>3</xsl:text></xsl:variable>"#,
            "selected",
        ),
        (
            r#"<xsl:variable name="nodes"><xsl:if test="false()">3</xsl:if></xsl:variable>"#,
            "",
        ),
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
                <xsl:output method="text"/>{outer}{declaration}
                <xsl:template match="/">{entry}</xsl:template>
                <xsl:template name="compare"><xsl:if test="$nodes > 2">selected</xsl:if></xsl:template>
              </xsl:stylesheet>"#
            ));
            let compiled = compile_stylesheet(&stylesheet).unwrap();
            assert_eq!(
                execute_compiled_stylesheet_for_test(
                    &compiled,
                    &document("<r><b>1</b><b>3</b></r>"),
                    "numeric-parity"
                )
                .unwrap(),
                expected,
                "global={global}: {declaration}"
            );
        }
    }
}

#[test]
fn shared_constructor_numeric_relations_preserve_shadowing_and_cycles() {
    for (local, expected) in [
        ("", None),
        (
            r#"<xsl:variable name="outer" select="3"/>"#,
            Some("selected"),
        ),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:template name="compare">{local}<xsl:if test="$outer > 2">selected</xsl:if></xsl:template>"#
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
fn shared_constructor_numeric_relations_charge_cached_source_traversal() {
    let program = numeric_program(r#"<xsl:variable name="nodes" select="r/b"/>"#, "$nodes > 2");
    let source = document("<r><b>1</b><b>3</b></r>");
    let consumed = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        assert_eq!(
            value(&owner, "nodes", &mut InvocationControl::unbounded()),
            "1"
        );
        let mut control = InvocationControl::unbounded();
        owner
            .with_completed("outer", &mut control, |_, _| Ok(()))
            .unwrap();
        assert_eq!(
            value(&owner, "outer", &mut InvocationControl::unbounded()),
            "selected"
        );
        control.consumed(WorkDomain::XdmStringValueNode)
    });
    assert!(consumed > 1);
    for limit in [consumed, consumed - 1] {
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            value(&owner, "nodes", &mut InvocationControl::unbounded());
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_string_value_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let result = owner.with_completed("outer", &mut control, |_, _| Ok(()));
            assert_eq!(result.is_ok(), limit == consumed);
            assert_eq!(owner.is_complete("outer"), limit == consumed);
            assert!(owner.is_complete("nodes"));
            assert_eq!(owner.depth.get(), 0);
        });
    }
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        value(&owner, "nodes", &mut InvocationControl::unbounded());
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmStringValueNode, 2);
        let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert_eq!(
            owner.read_legacy_string("outer", &mut control).unwrap_err(),
            error
        );
        assert!(!owner.is_complete("outer"));
        assert!(owner.is_complete("nodes"));
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_numeric_relations_keep_unproved_types_unsupported() {
    let program = numeric_program(
        r#"<xsl:variable name="nodes" select="true()"/>"#,
        "$nodes > 0",
    );
    let source = document("<r/>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let error = owner
            .read_legacy_string("outer", &mut InvocationControl::unbounded())
            .unwrap_err();
        assert_eq!(error.code, "FXRT1004");
        assert_eq!(error.category, FailureCategory::Unsupported);
        assert!(!owner.is_complete("outer"));
    });
}

#[test]
fn shared_constructor_global_string_comparison_resolves_forward() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:variable name="flag" select="'yes'"/>
      <xsl:template name="choose"><xsl:if test="$flag = 'yes'">wrong-if-skipped</xsl:if></xsl:template>"#,
    );
    let source = document("<r/>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        assert_eq!(
            value(&owner, "outer", &mut InvocationControl::unbounded()),
            "wrong-if-skipped"
        );
        assert!(owner.is_complete("flag"));
    });
}

#[test]
fn shared_constructor_node_set_string_comparisons_are_independently_existential() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="nodes" select="r/b"/>
      <xsl:template name="compare">
        <xsl:if test="$nodes = 'yes'">equal</xsl:if>
        <xsl:if test="$nodes != 'yes'">different</xsl:if>
      </xsl:template>"#,
    );
    for (xml, expected) in [
        ("<r/>", ""),
        ("<r><b>yes</b></r>", "equal"),
        ("<r><b>no</b></r>", "different"),
        ("<r><b>no</b><b>yes</b></r>", "equaldifferent"),
        ("<r><b>yes</b><b>no</b></r>", "equaldifferent"),
    ] {
        let source = document(xml);
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            assert_eq!(
                value(&owner, "outer", &mut InvocationControl::unbounded()),
                expected
            );
        });
    }
}

#[test]
fn shared_constructor_string_comparison_preserves_cycles_and_local_shadowing() {
    for (local, expected) in [
        ("", None),
        (
            "<xsl:variable name='outer' select=\"'yes'\"/>",
            Some("selected"),
        ),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:template name="compare">{local}
            <xsl:if test="$outer = 'yes'">selected</xsl:if>
          </xsl:template>"#
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
fn shared_constructor_string_comparison_matches_ordinary_execution() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    for declaration in [
        r#"<xsl:variable name="nodes" select="r/b"/>"#,
        r#"<xsl:variable name="nodes" select="r/missing"/>"#,
        r#"<xsl:variable name="nodes" select="'yes'"/>"#,
        r#"<xsl:variable name="nodes"><xsl:text>yes</xsl:text></xsl:variable>"#,
        r#"<xsl:variable name="nodes"><xsl:if test="false()">unused</xsl:if></xsl:variable>"#,
    ] {
        let mut results = Vec::new();
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
                <xsl:output method="text"/>{outer}{declaration}
                <xsl:template match="/">{entry}</xsl:template>
                <xsl:template name="compare">
                  <xsl:if test="$nodes = 'yes'">equal</xsl:if>
                  <xsl:if test="$nodes != 'yes'">different</xsl:if>
                </xsl:template>
              </xsl:stylesheet>"#
            ));
            let compiled = compile_stylesheet(&stylesheet).unwrap();
            results.push(
                execute_compiled_stylesheet_for_test(
                    &compiled,
                    &document("<r><b>no</b><b>yes</b></r>"),
                    "comparison-parity",
                )
                .unwrap(),
            );
        }
        assert_eq!(results[0], results[1], "{declaration}");
    }
}

#[test]
fn shared_constructor_string_comparison_controls_prevent_partial_publication() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="nodes" select="r/b"/>
      <xsl:template name="compare"><xsl:if test="$nodes = 'yes'">selected</xsl:if></xsl:template>"#,
    );
    let source = document("<r><b>no</b><b>yes</b></r>");
    let consumed = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "selected");
        control.consumed(WorkDomain::XPathNodeVisit)
    });
    assert!(consumed > 0);
    for limit in [consumed, consumed - 1] {
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xpath_node_visits: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let result = owner.read_legacy_string("outer", &mut control);
            assert_eq!(result.is_ok(), limit == consumed);
            assert_eq!(owner.is_complete("outer"), limit == consumed);
            assert_eq!(owner.depth.get(), 0);
        });
    }
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 1);
        let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert!(!owner.is_complete("outer"));
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_string_comparison_does_not_erase_typed_scalar_semantics() {
    for (declaration, local) in [
        (r#"<xsl:variable name="flag" select="false()"/>"#, ""),
        (r#"<xsl:variable name="flag" select="1"/>"#, ""),
        ("", r#"<xsl:variable name="flag" select="1"/>"#),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          {declaration}<xsl:template name="compare">{local}
            <xsl:if test="$flag = 'false'">wrong</xsl:if>
          </xsl:template>"#
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let error = owner
                .read_legacy_string("outer", &mut InvocationControl::unbounded())
                .unwrap_err();
            assert_eq!(error.code, "FXRT1004");
            assert_eq!(error.category, FailureCategory::Unsupported);
            assert!(!owner.is_complete("outer"));
        });
    }
}
