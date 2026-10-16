//! Constant attributes retain ordinary result construction and namespace fixup.
use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};

#[test]
fn shared_constructor_literal_attributes_match_ordinary_and_outlive_owners() {
    let mut results = Vec::new();
    for global in [false, true] {
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
        let style = document(&format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            <xsl:output omit-xml-declaration="yes"/>{outer}
            <xsl:variable name="text" select="'body'"/>
            <xsl:template name="build"><e xmlns:p="urn:p" p:a="é🙂" plain="{{{{literal}}}}" empty=""><xsl:value-of select="$text"/><child xml:lang="en"/></e></xsl:template>
            <xsl:template match="/">{entry}</xsl:template>
          </xsl:stylesheet>"#
        ));
        let compiled = compile_stylesheet(&style).unwrap();
        let source = document("<r/>");
        let mut control = InvocationControl::unbounded();
        let result = execute_program(&compiled, &source, "attributes", &mut control).unwrap();
        let output = compiled.output.clone();
        drop(compiled);
        drop(style);
        drop(source);
        results.push(serialize_xml(&result, &output, "attributes", 4096, &mut control).unwrap());
    }
    assert_eq!(results[0], results[1]);
    assert!(results[1].contains("p:a=\"é🙂\""));
    assert!(results[1].contains("plain=\"{literal}\""));
    assert!(results[1].contains("xml:lang=\"en\""));
}

#[test]
fn shared_constructor_literal_attributes_leave_computed_values_guarded() {
    for body in [
        r#"<e><xsl:attribute name="a">constant</xsl:attribute></e>"#,
        r#"<e><xsl:attribute name="a"><xsl:value-of select="$text"/></xsl:attribute></e>"#,
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
          <xsl:variable name="text" select="'value'"/>
          <xsl:template name="build">{body}</xsl:template>"#
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

#[test]
fn shared_constructor_variable_attributes_match_ordinary_value_kinds() {
    for (declaration, expected) in [
        (r#"<xsl:variable name="text" select="r/b"/>"#, "é🙂"),
        (r#"<xsl:variable name="text" select="r/missing"/>"#, ""),
        (r#"<xsl:variable name="text" select="false()"/>"#, "false"),
        (r#"<xsl:variable name="text" select="12"/>"#, "12"),
        (
            r#"<xsl:variable name="text"><xsl:call-template name="fragment"/></xsl:variable><xsl:template name="fragment"><a>é</a><b>🙂</b></xsl:template>"#,
            "é🙂",
        ),
    ] {
        let mut results = Vec::new();
        for global in [false, true] {
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
            let style = document(&format!(
                r#"
              <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
                <xsl:output omit-xml-declaration="yes"/>{outer}{declaration}
                <xsl:template name="build"><e a="{{$text}}"/></xsl:template>
                <xsl:template match="/">{entry}</xsl:template>
              </xsl:stylesheet>"#
            ));
            let compiled = compile_stylesheet(&style).unwrap();
            let source = document("<r><b>é🙂</b><b>not first</b></r>");
            let mut control = InvocationControl::unbounded();
            let result =
                execute_program(&compiled, &source, "variable-attribute", &mut control).unwrap();
            let output = compiled.output.clone();
            drop(compiled);
            drop(style);
            drop(source);
            results.push(
                serialize_xml(&result, &output, "variable-attribute", 4096, &mut control).unwrap(),
            );
        }
        assert_eq!(results[0], results[1]);
        assert!(
            results[1].contains(&format!("a=\"{expected}\"")),
            "{}",
            results[1]
        );
    }
}

#[test]
fn shared_constructor_variable_attributes_resolve_once_without_caller_local_leakage() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
      <xsl:variable name="text"><xsl:call-template name="read"/></xsl:variable>
      <xsl:variable name="local" select="'global'"/>
      <xsl:template name="build"><xsl:variable name="local" select="'caller'"/><e a="{$text}" b="{$text}"/></xsl:template>
      <xsl:template name="read"><xsl:message>once</xsl:message><xsl:value-of select="$local"/></xsl:template>"#,
    );
    let source = document("<r/>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        owner
            .with_completed("outer", &mut control, |_, _| Ok(()))
            .unwrap();
        assert_eq!(value(&owner, "text", &mut control), "global");
        assert_eq!(control.messages(), &["once"]);
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_variable_attributes_pin_cycles_and_local_shadowing() {
    for local in ["", r#"<xsl:variable name="outer" select="'local'"/>"#] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
          <xsl:template name="build">{local}<e a="{{$outer}}"/></xsl:template>"#
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let result = owner.with_completed("outer", &mut control, |_, _| Ok(()));
            if local.is_empty() {
                let error = result.unwrap_err();
                assert_eq!(error.code, "XTDE0640");
                assert_eq!(
                    owner.read_legacy_string("outer", &mut control).unwrap_err(),
                    error
                );
            } else {
                result.unwrap();
            }
            assert_eq!(owner.is_complete("outer"), !local.is_empty());
            assert_eq!(owner.depth.get(), 0);
        });
    }
}

#[test]
fn shared_constructor_variable_attributes_preserve_failure_origin_and_read_location() {
    for terminating in [false, true] {
        let declaration = if terminating {
            r#"<xsl:variable name="text"><xsl:call-template name="fail"/></xsl:variable><xsl:template name="fail"><xsl:message terminate="yes">stop</xsl:message></xsl:template>"#
        } else {
            ""
        };
        let xml = format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
            {declaration}<xsl:template name="build"><e a="{{$text}}"/></xsl:template>
            <xsl:template match="/"/>
          </xsl:stylesheet>"#
        );
        let style = document(&xml);
        let compiled = compile_stylesheet(&style).unwrap();
        let source = document("<r/>");
        with_owner(&compiled, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
            assert_eq!(
                error.code,
                if terminating { "XTMM9000" } else { "FXRT0002" }
            );
            let location = error.location.as_ref().unwrap();
            assert_eq!(location.resource, "urn:initializer-handoff");
            let located = &xml[location.span.clone()];
            assert!(
                located.contains(if terminating {
                    "terminate=\"yes\""
                } else {
                    "a=\"{$text}\""
                }),
                "{located}"
            );
            let messages = control.messages().to_vec();
            assert_eq!(
                owner.read_legacy_string("outer", &mut control).unwrap_err(),
                error
            );
            assert_eq!(control.messages(), messages);
            assert!(!owner.is_complete("outer"));
        });
    }
}

#[test]
fn shared_constructor_variable_attributes_bound_conversion_and_materialization() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
      <xsl:variable name="text" select="r/b"/>
      <xsl:template name="build"><e a="{$text}" b="{$text}"/></xsl:template>"#,
    );
    let source = document("<r><b>é🙂</b></r>");
    let domains = [
        WorkDomain::ResultNode,
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
        assert!(exact > 0, "{domain:?}");
        for (limit, cancel) in [(exact, false), (exact - 1, false), (exact, true)] {
            with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::ResultNode => limits.result_nodes = limit,
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

#[test]
fn shared_constructor_variable_attributes_isolate_supplied_parameters() {
    use crate::xdm::atomic_value_experiment::AtomicValue;
    let style = document(
        r#"
      <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
        <xsl:output omit-xml-declaration="yes"/>
        <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
        <xsl:param name="text"><xsl:call-template name="fail"/></xsl:param>
        <xsl:template name="fail"><xsl:message terminate="yes">unused</xsl:message></xsl:template>
        <xsl:template name="build"><e a="{$text}"/></xsl:template>
        <xsl:template match="/"><xsl:copy-of select="$outer"/></xsl:template>
      </xsl:stylesheet>"#,
    );
    let compiled = compile_stylesheet(&style).unwrap();
    std::thread::scope(|scope| {
        let handles = (0..8).map(|index| {
            let compiled = &compiled;
            scope.spawn(move || {
                let source = document("<r/>");
                let supplied = BTreeMap::from([("text".into(), InvocationParameter {
                    value: InvocationParameterValue::Atomic(AtomicValue::string(index.to_string())), tunnel: false,
                })]);
                with_owner(compiled, Some(&source), &supplied, |owner| {
                    let mut control = InvocationControl::unbounded();
                    owner.with_completed("outer", &mut control, |globals, _| {
                        let tree = globals.temporary_trees.get("outer").unwrap();
                        assert!(tree.nodes.iter().any(|node| matches!(&node.kind,
                            crate::runtime::golden_runtime_experiment::runtime_context::TemporaryNodeKind::Attribute { value, .. }
                            if value == &index.to_string())));
                        Ok(())
                    }).unwrap();
                    owner.initialize_all(&mut control).unwrap();
                    assert!(control.messages().is_empty());
                });
            })
        }).collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn shared_constructor_variable_attributes_use_effective_source_strings() {
    for strip in [false, true] {
        let style = document(&format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            {}<xsl:output omit-xml-declaration="yes"/>
            <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
            <xsl:variable name="text" select="r/b"/>
            <xsl:template name="build"><e a="{{$text}}"/></xsl:template>
            <xsl:template match="/"><xsl:copy-of select="$outer"/></xsl:template>
          </xsl:stylesheet>"#,
            if strip {
                r#"<xsl:strip-space elements="*"/>"#
            } else {
                ""
            }
        ));
        let compiled = compile_stylesheet(&style).unwrap();
        let source = document("<r><b><x>A</x> <x>B</x></b></r>");
        let mut control = InvocationControl::unbounded();
        let result = execute_program(&compiled, &source, "attribute-view", &mut control).unwrap();
        let output = compiled.output.clone();
        drop(compiled);
        drop(source);
        let text = serialize_xml(&result, &output, "attribute-view", 4096, &mut control).unwrap();
        assert!(
            text.contains(if strip { "a=\"AB\"" } else { "a=\"A B\"" }),
            "{text}"
        );
    }
}

#[test]
fn shared_constructor_literal_attributes_charge_before_publication() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
      <xsl:variable name="text" select="'body'"/>
      <xsl:template name="build"><e a="é🙂" b="second"><xsl:value-of select="$text"/></e></xsl:template>"#,
    );
    assert!(matches!(
        program
            .global_bindings
            .iter()
            .find(|binding| binding.name == "outer")
            .unwrap()
            .default,
        GlobalBindingDefault::SharedConstructor { .. }
    ));
    let source = document("<r/>");
    let domains = [
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::XdmNode,
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
                    WorkDomain::ResultNode => limits.result_nodes = limit,
                    WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
                    WorkDomain::XdmNode => limits.xdm_nodes = limit,
                    _ => unreachable!(),
                }
                let mut control = InvocationControl::new(CancellationToken::new(), limits);
                if cancel {
                    control = control.cancelling_on_charge(domain, 0);
                }
                let result = owner.with_completed("outer", &mut control, |_, _| Ok(()));
                let succeeds = limit == exact && !cancel;
                assert_eq!(
                    result.is_ok(),
                    succeeds,
                    "{domain:?}: limit={limit}, exact={exact}, cancel={cancel}"
                );
                if let Err(error) = result {
                    assert_eq!(error.code, if cancel { "FXCT0001" } else { "FXCT0002" });
                }
                assert_eq!(owner.is_complete("outer"), succeeds);
                assert_eq!(owner.depth.get(), 0);
            });
        }
    }
}
