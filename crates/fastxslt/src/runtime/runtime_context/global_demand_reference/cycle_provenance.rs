//! Capture at cycle detection, before the re-entry read guard unwinds. This is
//! observation evidence, not a public related-location or failure-routing model.

use super::super::super::failure_context_reference::ContextEnd;
use super::super::failure_observations::driver;
use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkLimits};

#[test]
fn demand_cycle_observation_includes_closing_typed_read_and_owns_metadata() {
    let mut control = InvocationControl::unbounded();
    let (driver, xml) = driver(
        r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:copy-of select="$second"/></xsl:template>
      <xsl:template name="second_constructor"><xsl:copy-of select="$first"/></xsl:template>"#,
        &mut control,
    );
    driver.provenance.enabled.set(true);
    let error = driver.run(&mut control).unwrap_err();
    assert_eq!(error.code, "FXAR0030");
    assert!(error.location.is_none());
    let observed = Arc::clone(driver.provenance.first_cycle.get().unwrap());
    assert!(matches!(observed.primary.as_ref(), Failure::Circularity(path) if path == &[0, 1, 0]));
    assert_eq!(observed.end, ContextEnd::Complete);
    assert_eq!(
        observed
            .dependencies
            .iter()
            .map(|edge| edge.binding)
            .collect::<Vec<_>>(),
        [0, 1, 0]
    );
    for edge in &observed.dependencies {
        assert!(xml[edge.read.as_ref().unwrap().span.clone()].starts_with("<xsl:copy-of"));
    }
    assert_eq!(
        observed.dependencies[0].declaration,
        observed.dependencies[2].declaration
    );
    assert_ne!(observed.dependencies[0].read, observed.dependencies[2].read);
    assert!(driver.provenance.path.borrow().is_empty());
    assert!(driver.resolver.stack.borrow().is_empty());
    assert_eq!(
        driver.read_string("first", &mut control).unwrap_err(),
        error
    );
    assert!(Arc::ptr_eq(
        &observed,
        driver.provenance.first_cycle.get().unwrap()
    ));
    let weak = Arc::downgrade(observed.dependencies[2].read.as_ref().unwrap());
    drop(driver);
    assert!(weak.upgrade().is_some());
    drop(observed);
    assert!(weak.upgrade().is_none());
}

#[test]
fn demand_mixed_cycle_observation_does_not_invent_string_locations() {
    let mut control = InvocationControl::unbounded();
    let (driver, _) = driver(
        r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:if test="$second = 'x'">unused</xsl:if></xsl:template>
      <xsl:template name="second_constructor"><xsl:copy-of select="$first"/></xsl:template>"#,
        &mut control,
    );
    driver.provenance.enabled.set(true);
    assert_eq!(driver.run(&mut control).unwrap_err().code, "FXAR0030");
    let observed = driver.provenance.first_cycle.get().unwrap();
    assert_eq!(observed.dependencies.len(), 3);
    assert!(observed.dependencies[0].read.is_some());
    assert!(observed.dependencies[1].read.is_none());
    assert!(observed.dependencies[2].read.is_some());
}

#[test]
fn demand_direct_cycle_observation_is_not_created_for_successful_reads() {
    for recursive in [false, true] {
        let mut control = InvocationControl::unbounded();
        let content = if recursive {
            "<xsl:copy-of select=\"$first\"/>"
        } else {
            "ok"
        };
        let (driver, _) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor">{content}</xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        let result = driver.run(&mut control);
        if recursive {
            assert_eq!(result.unwrap_err().code, "FXAR0030");
            let observed = driver.provenance.first_cycle.get().unwrap();
            assert_eq!(observed.dependencies.len(), 2);
            assert!(
                matches!(observed.primary.as_ref(), Failure::Circularity(path) if path == &[0, 0])
            );
        } else {
            assert_eq!(result.unwrap(), "ok");
            assert_eq!(driver.run(&mut control).unwrap(), "ok");
            assert!(driver.provenance.first_cycle.get().is_none());
            assert_eq!(driver.evaluations("first"), 1);
        }
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_cycle_observation_secondary_exhaustion_preserves_primary_once() {
    let mut setup = InvocationControl::unbounded();
    let (driver, _) = driver("", &mut setup);
    driver.provenance.enabled.set(true);
    let _entry = driver
        .provenance
        .enter(0, None, &driver.program, &mut setup)
        .unwrap();
    let primary = Arc::new(Failure::Circularity(vec![0, 0]));
    let mut limits = WorkLimits::unbounded();
    limits.xdm_nodes = 0;
    let mut limited = InvocationControl::new(CancellationToken::new(), limits);
    driver.provenance.observe_cycle(&primary, &mut limited);
    let observed = driver.provenance.first_cycle.get().unwrap();
    assert!(Arc::ptr_eq(&observed.primary, &primary));
    assert!(observed.dependencies.is_empty());
    assert!(matches!(observed.end, ContextEnd::Control(_)));
    driver
        .provenance
        .observe_cycle(&Arc::new(Failure::Circularity(vec![1, 1])), &mut setup);
    assert!(Arc::ptr_eq(&observed.primary, &primary));
    assert!(observed.dependencies.is_empty());
}
