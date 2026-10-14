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
