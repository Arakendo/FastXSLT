//! Principal-source iteration uses ordinary focus construction and frame isolation.
use super::*;

#[test]
fn shared_constructor_iteration_preserves_focus_and_recursive_global_entry() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="iterate"/></xsl:variable>
      <xsl:variable name="inner"><xsl:call-template name="root"/></xsl:variable>
      <xsl:variable name="local" select="'global'"/>
      <xsl:template name="iterate"><xsl:for-each select="r/b">
        <xsl:variable name="local" select="'caller'"/>
        <xsl:value-of select="position()"/><xsl:value-of select="last()"/>
        <xsl:value-of select="."/><xsl:value-of select="$inner"/>
      </xsl:for-each></xsl:template>
      <xsl:template name="root"><xsl:message>once</xsl:message>
        <xsl:value-of select="name(.)"/><xsl:value-of select="position()"/><xsl:value-of select="last()"/>
        <xsl:value-of select="r/@v"/><xsl:value-of select="$local"/>
      </xsl:template>"#,
    );
    let source = document("<r v='root'><b>A</b><b>B</b></r>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(
            value(&owner, "outer", &mut control),
            "12A11rootglobal22B11rootglobal"
        );
        assert_eq!(control.messages(), &["once"]);
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_iteration_matches_ordinary_empty_nested_and_ordered_paths() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    for (selection, expected) in [("r/missing", ""), ("r/b", "12A22B"), ("r/b/..", "11AB")] {
        for global in [false, true] {
            let outer = if global {
                r#"<xsl:variable name="outer"><xsl:call-template name="iterate"/></xsl:variable>"#
            } else {
                ""
            };
            let entry = if global {
                r#"<xsl:value-of select="$outer"/>"#
            } else {
                r#"<xsl:call-template name="iterate"/>"#
            };
            let style = document(&format!(
                r#"
              <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
                <xsl:output method="text"/>{outer}
                <xsl:template match="/">{entry}</xsl:template>
                <xsl:template name="iterate"><xsl:for-each select="{selection}">
                  <xsl:value-of select="position()"/><xsl:value-of select="last()"/>
                  <xsl:for-each select="."><xsl:value-of select="."/></xsl:for-each>
                </xsl:for-each></xsl:template>
              </xsl:stylesheet>"#
            ));
            let compiled = compile_stylesheet(&style).unwrap();
            assert_eq!(
                execute_compiled_stylesheet_for_test(
                    &compiled,
                    &document("<r><b>A</b><b>B</b></r>"),
                    "iteration"
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn shared_constructor_iteration_empty_selection_avoids_unexecuted_cycle() {
    for (selection, cyclic) in [("r/missing", false), ("r/b", true)] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="iterate"/></xsl:variable>
          <xsl:template name="iterate"><xsl:for-each select="{selection}"><xsl:value-of select="$outer"/></xsl:for-each></xsl:template>"#
        ));
        let source = document("<r><b/></r>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let result = owner.read_legacy_string("outer", &mut control);
            if cyclic {
                let error = result.unwrap_err();
                assert_eq!(error.code, "XTDE0640");
                assert_eq!(
                    owner.read_legacy_string("outer", &mut control).unwrap_err(),
                    error
                );
            } else {
                assert_eq!(result.unwrap(), "");
            }
            assert_eq!(owner.is_complete("outer"), !cyclic);
            assert_eq!(owner.depth.get(), 0);
        });
    }
}

#[test]
fn shared_constructor_iteration_keeps_sort_and_variable_selection_guarded() {
    for body in [
        r#"<xsl:for-each select="r/b"><xsl:sort select="."/><xsl:value-of select="."/></xsl:for-each>"#,
        r#"<xsl:for-each select="$nodes"><xsl:value-of select="."/></xsl:for-each>"#,
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="iterate"/></xsl:variable>
          <xsl:variable name="nodes" select="r/b"/>
          <xsl:template name="iterate">{body}</xsl:template>"#
        ));
        let source = document("<r><b>B</b><b>A</b></r>");
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

#[test]
fn shared_constructor_iteration_bounds_partial_result_and_cancellation() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="iterate"/></xsl:variable>
      <xsl:template name="iterate"><xsl:for-each select="r/b"><xsl:message>visited</xsl:message><xsl:value-of select="."/></xsl:for-each></xsl:template>"#,
    );
    let source = document("<r><b>A</b><b>B</b><b>C</b></r>");
    let domains = [
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XsltInstruction,
    ];
    let counts = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        owner
            .with_completed("outer", &mut control, |_, _| Ok(()))
            .unwrap();
        domains.map(|d| control.consumed(d))
    });
    for (domain, exact) in domains.into_iter().zip(counts) {
        assert!(exact > 0, "{domain:?}");
        for (limit, cancel) in [(exact, false), (exact - 1, false), (exact, true)] {
            with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::ResultNode => limits.result_nodes = limit,
                    WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
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
                    let messages = control.messages().to_vec();
                    assert_eq!(
                        owner.read_legacy_string("outer", &mut control).unwrap_err(),
                        error
                    );
                    assert_eq!(control.messages(), messages);
                }
                assert_eq!(owner.is_complete("outer"), succeeds);
                assert_eq!(owner.depth.get(), 0);
            });
        }
    }
}

#[test]
fn shared_constructor_iteration_keeps_outer_locals_and_iteration_frames_isolated() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="iterate"/></xsl:variable>
      <xsl:template name="iterate"><xsl:variable name="local" select="'outer'"/>
        <xsl:for-each select="r/b"><xsl:value-of select="$local"/>
          <xsl:variable name="local" select="'inner'"/><xsl:value-of select="$local"/>
        </xsl:for-each><xsl:value-of select="$local"/>
      </xsl:template>"#,
    );
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|count| {
                let program = &program;
                scope.spawn(move || {
                    let source = document(&format!("<r>{}</r>", "<b/>".repeat(count)));
                    with_owner(program, Some(&source), &BTreeMap::new(), |owner| {
                        value(&owner, "outer", &mut InvocationControl::unbounded())
                    })
                })
            })
            .collect::<Vec<_>>();
        for (count, handle) in handles.into_iter().enumerate() {
            assert_eq!(
                handle.join().unwrap(),
                format!("{}outer", "outerinner".repeat(count))
            );
        }
    });
}

#[test]
fn shared_constructor_iteration_uses_effective_view_and_retains_result() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
    for strip in [false, true] {
        let style = document(&format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            {}<xsl:output method="text"/>
            <xsl:variable name="outer"><xsl:call-template name="iterate"/></xsl:variable>
            <xsl:template name="iterate"><xsl:for-each select="r/node()"><xsl:value-of select="position()"/><xsl:value-of select="last()"/></xsl:for-each></xsl:template>
            <xsl:template match="/"><xsl:value-of select="$outer"/></xsl:template>
          </xsl:stylesheet>"#,
            if strip {
                r#"<xsl:strip-space elements="*"/>"#
            } else {
                ""
            }
        ));
        let compiled = compile_stylesheet(&style).unwrap();
        let source = document("<r> <b/> <b/> </r>");
        let mut control = InvocationControl::unbounded();
        let result = execute_program(&compiled, &source, "iteration-view", &mut control).unwrap();
        let output = compiled.output.clone();
        drop(compiled);
        drop(style);
        drop(source);
        assert_eq!(
            serialize_xml(&result, &output, "iteration-view", 1024, &mut control).unwrap(),
            if strip { "1222" } else { "1525354555" }
        );
    }
}
