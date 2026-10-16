//! Shared typed relational operands during recursive global initialization.
use super::*;

#[test]
fn shared_constructor_pair_relations_use_effective_whitespace_and_release_owners() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
    for strip in [false, true] {
        let stylesheet = document(&format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            {}<xsl:output method="text"/>
            <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
            <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="10"/>
            <xsl:template name="compare"><xsl:if test="$left > $right">yes</xsl:if></xsl:template>
            <xsl:template match="/"><xsl:value-of select="$outer"/></xsl:template>
          </xsl:stylesheet>"#,
            if strip {
                r#"<xsl:strip-space elements="*"/>"#
            } else {
                ""
            }
        ));
        let compiled = compile_stylesheet(&stylesheet).unwrap();
        let source = document("<r><b><e>1</e> <e>2</e></b></r>");
        let mut control = InvocationControl::unbounded();
        let result =
            execute_program(&compiled, &source, "effective-relation", &mut control).unwrap();
        let output = compiled.output.clone();
        drop(compiled);
        drop(source);
        drop(stylesheet);
        assert_eq!(
            serialize_xml(&result, &output, "effective-relation", 1024, &mut control).unwrap(),
            if strip { "yes" } else { "" }
        );
    }
}

#[test]
fn shared_constructor_pair_relations_isolate_concurrent_sources() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="2"/>
      <xsl:template name="compare"><xsl:if test="$left > $right">yes</xsl:if></xsl:template>"#,
    );
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
            assert_eq!(handle.join().unwrap(), if index > 2 { "yes" } else { "" });
        }
    });
}

#[test]
fn shared_constructor_pair_relations_match_ordinary_execution() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    for (left, right, operator, expected) in [
        ("r/b", "r/c", "&lt;", "yes"),
        ("r/b", "r/c", ">", "yes"),
        ("r/b", "r/c", "&lt;=", "yes"),
        ("r/b", "r/c", ">=", "yes"),
        ("r/missing", "r/c", "&lt;", ""),
        ("r/nan", "r/c", "&lt;", ""),
        ("r/b", "true()", ">", ""),
        ("r/b", "false()", ">", "yes"),
        ("r/missing", "true()", "&lt;", "yes"),
        ("true()", "r/missing", ">", "yes"),
        ("true()", "r/b", ">=", "yes"),
        ("false()", "1", "&lt;", "yes"),
        ("'3'", "2", ">", "yes"),
        ("'invalid'", "2", ">", ""),
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
                <xsl:variable name="left" select="{left}"/><xsl:variable name="right" select="{right}"/>
                <xsl:template match="/">{entry}</xsl:template>
                <xsl:template name="compare"><xsl:if test="$left {operator} $right">yes</xsl:if></xsl:template>
              </xsl:stylesheet>"#
            ));
            let compiled = compile_stylesheet(&stylesheet).unwrap();
            assert_eq!(
                execute_compiled_stylesheet_for_test(
                    &compiled,
                    &document("<r><b>1</b><b>3</b><c>2</c><nan>invalid</nan></r>"),
                    "pair-relations"
                )
                .unwrap(),
                expected,
                "global={global}: {left} {operator} {right}"
            );
        }
    }
}

#[test]
fn shared_constructor_pair_relations_keep_left_owned_during_recursive_right() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/>
      <xsl:variable name="right"><xsl:call-template name="build"/></xsl:variable>
      <xsl:variable name="third" select="'2'"/>
      <xsl:template name="build"><xsl:message>once</xsl:message><xsl:value-of select="$third"/></xsl:template>
      <xsl:template name="compare"><xsl:if test="$left > $right">yes</xsl:if></xsl:template>"#,
    );
    let source = document("<r><b>1</b><b>3</b></r>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "yes");
        assert_eq!(value(&owner, "outer", &mut control), "yes");
        assert_eq!(control.messages(), &["once"]);
        assert!(owner.is_complete("third"));
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_pair_relations_preserve_typed_parameter_values() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:param name="left"><xsl:call-template name="fail"/></xsl:param>
      <xsl:variable name="right" select="1"/>
      <xsl:template name="fail"><xsl:message terminate="yes">default</xsl:message></xsl:template>
      <xsl:template name="compare"><xsl:if test="$left > $right">yes</xsl:if></xsl:template>"#,
    );
    let source = document("<r/>");
    for (kind, lexical, expected) in [
        (BuiltinAtomicType::Double, "INF", Some("yes")),
        (BuiltinAtomicType::Float, "-INF", Some("")),
        (BuiltinAtomicType::Double, "NaN", Some("")),
        (BuiltinAtomicType::Decimal, "1.5", Some("yes")),
        (BuiltinAtomicType::Boolean, "true", Some("")),
        (BuiltinAtomicType::String, "INF", Some("")),
        (BuiltinAtomicType::Date, "2026-10-15", None),
    ] {
        let supplied = BTreeMap::from([(
            "left".into(),
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
                let error = result.unwrap_err();
                assert_eq!(error.code, "FXRT1015");
                assert_eq!(error.category, FailureCategory::Unsupported);
                assert!(!owner.is_complete("outer"));
            }
            assert!(control.messages().is_empty());
        });
    }
}

#[test]
fn shared_constructor_pair_relations_preserve_shadowing_and_actual_cycles() {
    for (local, expected) in [
        ("", None),
        (r#"<xsl:variable name="outer" select="3"/>"#, Some("yes")),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:variable name="right" select="2"/>
          <xsl:template name="compare">{local}<xsl:if test="$outer > $right">yes</xsl:if></xsl:template>"#
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
fn shared_constructor_pair_relations_bound_buffers_traversal_and_pair_work() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="r/c"/>
      <xsl:template name="compare"><xsl:if test="$left > $right">yes</xsl:if></xsl:template>"#,
    );
    let source = document("<r><b>1</b><b>3</b><c>2</c></r>");
    let prepare = |owner: &GlobalInitialization<'_>| {
        value(owner, "left", &mut InvocationControl::unbounded());
        value(owner, "right", &mut InvocationControl::unbounded());
    };
    let domains = [
        WorkDomain::XdmNode,
        WorkDomain::XdmStringValueNode,
        WorkDomain::XPathOperation,
    ];
    let counts = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        prepare(&owner);
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
                prepare(&owner);
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::XdmNode => limits.xdm_nodes = limit,
                    WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
                    WorkDomain::XPathOperation => limits.xpath_operations = limit,
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
                assert!(owner.is_complete("left") && owner.is_complete("right"));
                assert_eq!(owner.depth.get(), 0);
            });
        }
    }
}
