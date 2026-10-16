//! Two-operand ownership, typed equality and charged temporary buffers.
use super::*;

#[test]
fn shared_constructor_pair_inequality_condition_stays_explicitly_unsupported() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    let stylesheet = document(
        r#"
      <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
        <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
        <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="r/c"/>
        <xsl:template match="/"/>
        <xsl:template name="compare"><xsl:if test="$left != $right">wrong-if-negated</xsl:if></xsl:template>
      </xsl:stylesheet>"#,
    );
    let error = compile_stylesheet(&stylesheet).unwrap_err();
    assert_eq!(error.code, "FXXP1002");
    assert_eq!(
        error.category,
        crate::compile::golden_stylesheet_experiment::CompileCategory::Unsupported
    );
}

#[test]
fn shared_constructor_pair_equality_isolates_concurrent_sources() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="r/c"/>
      <xsl:template name="compare"><xsl:if test="$left = $right">selected</xsl:if></xsl:template>"#,
    );
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|index| {
                let program = &program;
                scope.spawn(move || {
                    let source = document(&format!("<r><b>{index}</b><c>3</c></r>"));
                    with_owner(program, Some(&source), &BTreeMap::new(), |owner| {
                        value(&owner, "outer", &mut InvocationControl::unbounded())
                    })
                })
            })
            .collect::<Vec<_>>();
        for (index, handle) in handles.into_iter().enumerate() {
            assert_eq!(
                handle.join().unwrap(),
                if index == 3 { "selected" } else { "" }
            );
        }
    });
}

#[test]
fn shared_constructor_pair_equality_keeps_local_shadow_and_explicit_first_node_rules() {
    for (local, test, expected) in [
        ("", "$left = $right", "selected"),
        ("", "string($left) = string($right)", ""),
        ("", "not(string($left) = string($right))", "selected"),
        (
            r#"<xsl:variable name="left" select="'2'"/>"#,
            "$left = $right",
            "selected",
        ),
        (
            r#"<xsl:variable name="left" select="'other'"/>"#,
            "$left = $right",
            "",
        ),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="r/c"/>
          <xsl:template name="compare">{local}<xsl:if test="{test}">selected</xsl:if></xsl:template>"#
        ));
        let source = document("<r><b>1</b><b>2</b><c>2</c></r>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            assert_eq!(
                value(&owner, "outer", &mut InvocationControl::unbounded()),
                expected,
                "{local}: {test}"
            );
        });
    }
}

#[test]
fn shared_constructor_pair_equality_preserves_supplied_atomic_types() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:param name="left"><xsl:call-template name="fail"/></xsl:param>
      <xsl:param name="right"><xsl:call-template name="fail"/></xsl:param>
      <xsl:template name="fail"><xsl:message terminate="yes">default</xsl:message></xsl:template>
      <xsl:template name="compare"><xsl:if test="$left = $right">E</xsl:if><xsl:if test="not($left = $right)">N</xsl:if></xsl:template>"#,
    );
    let source = document("<r/>");
    for (left_kind, left, right_kind, right, expected) in [
        (
            BuiltinAtomicType::Boolean,
            "false",
            BuiltinAtomicType::String,
            "false",
            Some("N"),
        ),
        (
            BuiltinAtomicType::Double,
            "INF",
            BuiltinAtomicType::Float,
            "INF",
            Some("E"),
        ),
        (
            BuiltinAtomicType::Double,
            "NaN",
            BuiltinAtomicType::Double,
            "NaN",
            Some("N"),
        ),
        (
            BuiltinAtomicType::Decimal,
            "2.5",
            BuiltinAtomicType::String,
            "2.50",
            Some("E"),
        ),
        (
            BuiltinAtomicType::Date,
            "2026-10-15",
            BuiltinAtomicType::String,
            "2026-10-15",
            None,
        ),
    ] {
        let supplied = BTreeMap::from([
            (
                "left".into(),
                InvocationParameter {
                    value: InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                        left_kind, left,
                    )),
                    tunnel: false,
                },
            ),
            (
                "right".into(),
                InvocationParameter {
                    value: InvocationParameterValue::Atomic(AtomicValue::from_validated_lexical(
                        right_kind, right,
                    )),
                    tunnel: false,
                },
            ),
        ]);
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
fn shared_constructor_pair_equality_matches_ordinary_typed_semantics() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    for (left, right, expected) in [
        (r#"select="r/b""#, r#"select="r/c""#, "E"),
        (r#"select="r/missing""#, r#"select="r/c""#, "N"),
        (r#"select="r/b""#, r#"select="2""#, "E"),
        (r#"select="2""#, r#"select="r/b""#, "E"),
        (r#"select="r/b""#, r#"select="true()""#, "E"),
        (r#"select="r/missing""#, r#"select="false()""#, "E"),
        (r#"select="false()""#, r#"select="'false'""#, "N"),
        (r#"select="2""#, r#"select="'02'""#, "E"),
        (r#"select="'2'""#, r#"select="'02'""#, "N"),
        (r#"select="2""#, r#"select="'invalid'""#, "N"),
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
                <xsl:variable name="left" {left}/><xsl:variable name="right" {right}/>
                <xsl:template match="/">{entry}</xsl:template>
                <xsl:template name="compare">
                  <xsl:if test="$left = $right">E</xsl:if>
                  <xsl:if test="not($left = $right)">N</xsl:if>
                </xsl:template>
              </xsl:stylesheet>"#
            ));
            let compiled = compile_stylesheet(&stylesheet).unwrap();
            assert_eq!(
                execute_compiled_stylesheet_for_test(
                    &compiled,
                    &document("<r><b>1</b><b>2</b><c>2</c></r>"),
                    "pair-parity"
                )
                .unwrap(),
                expected,
                "global={global}: {left} / {right}"
            );
        }
    }
}

#[test]
fn shared_constructor_pair_equality_retains_left_while_right_initializes() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/>
      <xsl:variable name="right"><xsl:call-template name="build"/></xsl:variable>
      <xsl:variable name="third" select="'2'"/>
      <xsl:template name="build"><xsl:message>once</xsl:message><xsl:value-of select="$third"/></xsl:template>
      <xsl:template name="compare">
        <xsl:if test="$left = $right">E</xsl:if>
        <xsl:if test="not($left = $right)">N</xsl:if>
      </xsl:template>"#,
    );
    let source = document("<r><b>1</b><b>2</b></r>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "E");
        assert_eq!(value(&owner, "outer", &mut control), "E");
        assert_eq!(control.messages(), &["once"]);
        assert!(owner.is_complete("third"));
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_pair_equality_preserves_fragment_and_explicit_string_kinds() {
    for (test, expected) in [
        ("$left = $right", "selected"),
        ("not($left = $right)", ""),
        ("string($left) = $right", ""),
        ("not(string($left) = $right)", "selected"),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
          <xsl:variable name="left"><xsl:if test="false()">text</xsl:if></xsl:variable>
          <xsl:variable name="right" select="true()"/>
          <xsl:template name="compare"><xsl:if test="{test}">selected</xsl:if></xsl:template>"#
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            assert_eq!(
                value(&owner, "outer", &mut InvocationControl::unbounded()),
                expected,
                "{test}"
            );
        });
    }
}

#[test]
fn shared_constructor_pair_equality_cycles_do_not_publish_or_replay_messages() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/>
      <xsl:variable name="right"><xsl:call-template name="cycle"/></xsl:variable>
      <xsl:template name="compare"><xsl:if test="$left = $right">wrong</xsl:if></xsl:template>
      <xsl:template name="cycle"><xsl:message>once</xsl:message><xsl:value-of select="$outer"/></xsl:template>"#,
    );
    let source = document("<r><b>2</b></r>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
        assert_eq!(error.code, "XTDE0640");
        assert_eq!(
            owner.read_legacy_string("outer", &mut control).unwrap_err(),
            error
        );
        assert!(owner.is_complete("left"));
        assert!(!owner.is_complete("right"));
        assert!(!owner.is_complete("outer"));
        assert_eq!(control.messages(), &["once"]);
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_pair_equality_buffers_and_work_are_bounded() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="compare"/></xsl:variable>
      <xsl:variable name="left" select="r/b"/><xsl:variable name="right" select="r/c"/>
      <xsl:template name="compare"><xsl:if test="$left = $right">selected</xsl:if></xsl:template>"#,
    );
    let source = document("<r><b>1</b><b>2</b><c>2</c></r>");
    let prepare = |owner: &GlobalInitialization<'_>| {
        value(owner, "left", &mut InvocationControl::unbounded());
        value(owner, "right", &mut InvocationControl::unbounded());
    };
    let domains = [
        WorkDomain::XdmNode,
        WorkDomain::XdmStringValueNode,
        WorkDomain::ResultTextByte,
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
                    WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
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
