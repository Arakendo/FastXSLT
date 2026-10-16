//! Schedule comparison, not a static template-reachability implementation.
//! Both schedules use the same demand owner and ordinary compiled executor.
//! Eager initialization below follows declaration order; recursive reads still
//! resolve dependencies through the single reference owner.

use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::execution_control_experiment::WorkDomain;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn document(identity: &str, xml: &str) -> Arc<Document> {
    Arc::new(
        Document::from_parsed(
            parse_document(
                identity,
                xml.as_bytes(),
                ParseLimits {
                    max_events: 1024,
                    max_depth: 64,
                },
            )
            .unwrap(),
        )
        .unwrap(),
    )
}

fn driver(
    body: &str,
    supplied: &BTreeMap<String, String>,
    control: &mut InvocationControl,
) -> Rc<Driver> {
    let stylesheet = document(
        "urn:schedule:stylesheet",
        &format!(
            r#"
      <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
        <xsl:param name="first" select="'placeholder'"/>
        <xsl:variable name="second" select="'placeholder'"/>
        <xsl:variable name="unused" select="'placeholder'"/>
        {body}
      </xsl:stylesheet>"#
        ),
    );
    Driver::new(
        Arc::new(compile_stylesheet(&stylesheet).unwrap()),
        document("urn:schedule:source", "<r><b/></r>"),
        supplied,
        false,
        control,
    )
    .unwrap()
}

fn eager(driver: &Driver, control: &mut InvocationControl) -> Result<(), ExecutionFailure> {
    for binding in &driver.program.global_bindings {
        driver.read_value(&binding.name, control)?;
    }
    Ok(())
}

#[path = "declaration_linking/mod.rs"]
mod declaration_linking;

// Compile actual declaration children, without rewriting them into templates.
// The ordinary program still uses explicit placeholder declarations: declaration
// admission/linking is deliberately not proved by this body-lowering experiment.
fn actual_bodies(
    first: &str,
    second: &str,
    templates: &str,
    strict: bool,
    complete_clones: bool,
    control: &mut InvocationControl,
) -> (Rc<Driver>, String) {
    use crate::compile::golden_stylesheet_experiment::{
        compile_global_constructor_reference, stylesheet_preprocessing::strict_stylesheet_view,
    };
    use crate::xdm::owned_tree_experiment::NodeKind;
    let xml = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
      <xsl:variable name="first">{first}</xsl:variable>
      <xsl:variable name="second">{second}</xsl:variable>
      {templates}</xsl:stylesheet>"#
    );
    let original = document("urn:actual-global:stylesheet", &xml);
    let root = original
        .children(original.document_node())
        .iter()
        .copied()
        .find(|node| original.kind(*node) == NodeKind::Element)
        .unwrap();
    let effective = if strict {
        Arc::new(strict_stylesheet_view(&original, root, control).unwrap())
    } else {
        Arc::clone(&original)
    };
    let declarations = effective
        .children(root)
        .iter()
        .copied()
        .filter(|node| {
            effective
                .name(*node)
                .is_some_and(|name| name.local == "variable")
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 2);
    let bodies = ["first", "second"]
        .into_iter()
        .zip(declarations)
        .map(|(name, declaration)| {
            (
                name.to_owned(),
                compile_global_constructor_reference(&effective, declaration).unwrap(),
            )
        })
        .collect();
    let scaffold = driver(templates, &BTreeMap::new(), control);
    let result = Driver::with_constructor_bodies(
        Arc::clone(&scaffold.program),
        Arc::clone(&scaffold.source),
        &BTreeMap::new(),
        complete_clones,
        bodies,
        control,
    )
    .unwrap();
    // No source document or parsed stylesheet is retained by the body map.
    drop(effective);
    drop(original);
    (result, xml)
}

#[test]
fn demand_actual_global_bodies_dispatch_forward_and_preserve_profile_text() {
    let templates = r#"
      <xsl:template match="/"><xsl:value-of select="$first"/><xsl:value-of select="$first"/></xsl:template>
      <xsl:template match="b"><xsl:value-of select="$second"/></xsl:template>"#;
    for strict in [false, true] {
        for clones in [false, true] {
            let mut control = InvocationControl::unbounded();
            let (driver, xml) = actual_bodies(
                r#"<xsl:apply-templates select="//b"/>"#,
                "A<!-- split -->   <!-- split -->B",
                templates,
                strict,
                clones,
                &mut control,
            );
            assert_eq!(
                driver.run(&mut control).unwrap(),
                if strict { "ABAB" } else { "A   BA   B" }
            );
            assert_eq!(driver.evaluations("first"), 1);
            assert_eq!(driver.evaluations("second"), 1);
            assert_eq!(driver.evaluations("unused"), 0);
            assert!(driver.resolver.stack.borrow().is_empty());
            let production =
                compile_stylesheet(&document("urn:actual-global:stylesheet", &xml)).unwrap();
            let actual =
                crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test(
                    &production,
                    &driver.source,
                    "production-constructor",
                )
                .unwrap();
            // This compilation uses the modern baseline, independently of the
            // reference's explicitly selected strict preprocessing above.
            assert!(actual.ends_with("A   BA   B"), "{actual}");
        }
    }
}

#[test]
fn demand_actual_global_body_failure_keeps_declaration_child_location() {
    let templates = r#"<xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>"#;
    for strict in [false, true] {
        let mut control = InvocationControl::unbounded();
        let (driver, xml) = actual_bodies(
            r#"<xsl:copy-of select="$second"/>"#,
            r#"<xsl:message terminate="yes">actual body</xsl:message>"#,
            templates,
            strict,
            false,
            &mut control,
        );
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, "XTMM9000");
        let location = error.location.as_ref().unwrap();
        assert_eq!(location.resource, "urn:actual-global:stylesheet");
        assert!(xml[location.span.clone()].starts_with("<xsl:message terminate=\"yes\""));
        assert_eq!(
            driver.read_string("first", &mut control).unwrap_err(),
            error
        );
        assert_eq!(control.messages(), &["actual body"]);
        assert_eq!(driver.evaluations("second"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_actual_global_body_cycle_unwinds_without_partial_publication() {
    let templates = r#"<xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>"#;
    for clones in [false, true] {
        let mut control = InvocationControl::unbounded();
        let (driver, _) = actual_bodies(
            r#"prefix<xsl:copy-of select="$second"/>"#,
            r#"<xsl:copy-of select="$first"/>suffix"#,
            templates,
            false,
            clones,
            &mut control,
        );
        assert_eq!(driver.run(&mut control).unwrap_err().code, "FXAR0030");
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
        assert_eq!(
            driver.read_string("first", &mut control).unwrap_err().code,
            "FXAR0030"
        );
    }
}

#[test]
fn demand_schedule_forward_dispatch_has_one_evaluator_under_both_schedules() {
    let body = r#"
      <xsl:template match="/"><xsl:value-of select="$first"/><xsl:value-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:apply-templates select="//b"/></xsl:template>
      <xsl:template match="b"><xsl:value-of select="$second"/></xsl:template>
      <xsl:template name="second_constructor">B</xsl:template>
      <xsl:template name="unused_constructor">U</xsl:template>"#;
    for initialize in [false, true] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(body, &BTreeMap::new(), &mut control);
        if initialize {
            eager(&driver, &mut control).unwrap();
        }
        assert_eq!(driver.run(&mut control).unwrap(), "BB");
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
        assert_eq!(driver.evaluations("unused"), usize::from(initialize));
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_actual_global_bodies_preserve_exact_limits_and_cancellation() {
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    let make = || {
        actual_bodies(
            r#"<xsl:apply-templates select="//b"/>"#,
            "<box>payload</box>",
            r#"<xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
               <xsl:template match="b"><xsl:copy-of select="$second"/></xsl:template>"#,
            false,
            false,
            &mut InvocationControl::unbounded(),
        )
        .0
    };
    let mut measured = InvocationControl::unbounded();
    assert_eq!(make().run(&mut measured).unwrap(), "payload");
    for domain in [
        WorkDomain::XdmNode,
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::XPathOperation,
    ] {
        let exact = measured.consumed(domain);
        assert!(exact > 0);
        for less in [false, true] {
            let mut limits = WorkLimits::unbounded();
            let ceiling = exact - usize::from(less);
            match domain {
                WorkDomain::XdmNode => limits.xdm_nodes = ceiling,
                WorkDomain::ResultNode => limits.result_nodes = ceiling,
                WorkDomain::ResultTextByte => limits.result_text_bytes = ceiling,
                WorkDomain::XPathOperation => limits.xpath_operations = ceiling,
                _ => unreachable!(),
            }
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            let driver = make();
            let result = driver.run(&mut control);
            if less {
                let error = result.unwrap_err();
                assert_eq!(error.code, "FXCT0002");
                assert_eq!(error.work_domain, Some(domain));
            } else {
                assert_eq!(result.unwrap(), "payload");
            }
            assert!(driver.resolver.stack.borrow().is_empty());
        }
        let driver = make();
        let mut control = InvocationControl::unbounded().cancelling_on_charge(domain, 1);
        assert_eq!(driver.run(&mut control).unwrap_err().code, "FXCT0001");
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_schedule_unused_terminating_failure_is_an_observable_difference() {
    let body = r#"
      <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor">A</xsl:template>
      <xsl:template name="second_constructor">B</xsl:template>
      <xsl:template name="unused_constructor"><xsl:message terminate="yes">unused</xsl:message></xsl:template>"#;
    let mut lazy_control = InvocationControl::unbounded();
    let lazy = driver(body, &BTreeMap::new(), &mut lazy_control);
    assert_eq!(lazy.run(&mut lazy_control).unwrap(), "A");
    assert_eq!(lazy.evaluations("unused"), 0);
    let mut eager_control = InvocationControl::unbounded();
    let initialized = driver(body, &BTreeMap::new(), &mut eager_control);
    assert_eq!(
        eager(&initialized, &mut eager_control).unwrap_err().code,
        "XTMM9000"
    );
    assert!(initialized.resolver.stack.borrow().is_empty());
    assert_eq!(initialized.evaluations("unused"), 1);
}

#[test]
fn demand_schedule_unselected_mode_cycle_is_not_an_executed_dependency() {
    let body = r#"
      <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:apply-templates select="//b" mode="chosen"/></xsl:template>
      <xsl:template match="b" mode="chosen">A</xsl:template>
      <xsl:template match="b" mode="other"><xsl:value-of select="$first"/></xsl:template>
      <xsl:template name="second_constructor">B</xsl:template>
      <xsl:template name="unused_constructor">U</xsl:template>"#;
    for initialize in [false, true] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(body, &BTreeMap::new(), &mut control);
        if initialize {
            eager(&driver, &mut control).unwrap();
        }
        assert_eq!(driver.run(&mut control).unwrap(), "A");
        assert_eq!(driver.evaluations("first"), 1);
    }
    // A scanner treating every matched-template read as an unconditional edge
    // would invent first -> first here. This does not falsify a mode-aware
    // conservative analysis; no such analysis is implemented by this test.
}

#[test]
fn demand_schedule_supplied_parameter_prunes_its_default_cycle() {
    let body = r#"
      <xsl:template match="/"><xsl:value-of select="$second"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:value-of select="$second"/></xsl:template>
      <xsl:template name="second_constructor"><xsl:value-of select="$first"/></xsl:template>
      <xsl:template name="unused_constructor">U</xsl:template>"#;
    for initialize in [false, true] {
        for supplied in [false, true] {
            let mut control = InvocationControl::unbounded();
            let parameters = if supplied {
                BTreeMap::from([("first".into(), "host".into())])
            } else {
                BTreeMap::new()
            };
            let driver = driver(body, &parameters, &mut control);
            let result = if initialize {
                eager(&driver, &mut control).and_then(|()| driver.run(&mut control))
            } else {
                driver.run(&mut control)
            };
            if supplied {
                assert_eq!(result.unwrap(), "host");
                assert_eq!(driver.evaluations("first"), 0);
            } else {
                assert_eq!(result.unwrap_err().code, "FXAR0030");
                assert_eq!(driver.evaluations("first"), 1);
            }
            assert_eq!(driver.evaluations("second"), 1);
            assert!(driver.resolver.stack.borrow().is_empty());
        }
    }
}

#[test]
fn demand_schedule_eager_unused_work_can_exhaust_a_demand_sufficient_budget() {
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    let body = r#"
      <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor">A</xsl:template>
      <xsl:template name="second_constructor">B</xsl:template>
      <xsl:template name="unused_constructor">U</xsl:template>"#;
    let mut baseline = InvocationControl::unbounded();
    let reference = driver(body, &BTreeMap::new(), &mut baseline);
    assert_eq!(reference.run(&mut baseline).unwrap(), "A");
    let limit = baseline.consumed(WorkDomain::XdmNode);
    for initialize in [false, true] {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        let driver = driver(body, &BTreeMap::new(), &mut control);
        let result = if initialize {
            eager(&driver, &mut control).and_then(|()| driver.run(&mut control))
        } else {
            driver.run(&mut control)
        };
        if initialize {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        } else {
            assert_eq!(result.unwrap(), "A");
        }
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}
