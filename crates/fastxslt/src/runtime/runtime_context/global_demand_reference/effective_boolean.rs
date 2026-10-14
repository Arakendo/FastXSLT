//! Test-only effective-boolean reads through the one demand value owner.
//! Read-site provenance is deliberately unknown until explicitly threaded.

use super::failure_observations::driver;
use super::*;

#[test]
fn demand_boolean_source_sets_use_presence_not_text() {
    for (path, expected) in [("/r", "yes"), ("/r/missing", "")] {
        let mut control = InvocationControl::unbounded();
        let (driver, _) = driver(
            &format!(
                r#"
          <xsl:variable name="source" select="{path}"/>
          <xsl:template match="/"><xsl:if test="$source">yes</xsl:if></xsl:template>
          <xsl:template name="first_constructor"/><xsl:template name="second_constructor"/>"#
            ),
            &mut control,
        );
        assert_eq!(driver.run(&mut control).unwrap(), expected);
        assert_eq!(driver.evaluations("source"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_boolean_cached_reads_preserve_budget_and_cancellation() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let mut setup = InvocationControl::unbounded();
    let (driver, _) = driver(
        r#"
      <xsl:template match="/"><xsl:if test="$first">yes</xsl:if></xsl:template>
      <xsl:template name="first_constructor"/><xsl:template name="second_constructor"/>"#,
        &mut setup,
    );
    driver.read_value("first", &mut setup).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(driver.run(&mut baseline).unwrap(), "yes");
    let exact = baseline.consumed(WorkDomain::XPathOperation);
    for (limit, cancelled, succeeds) in [
        (exact, false, true),
        (exact - 1, false, false),
        (exact, true, false),
    ] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancelled {
            control = control.cancelling_on_charge(WorkDomain::XPathOperation, 1);
        }
        let result = driver.run(&mut control);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(
                result.unwrap_err().code,
                if cancelled { "FXCT0001" } else { "FXCT0002" }
            );
        }
        assert_eq!(driver.evaluations("first"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_boolean_empty_tree_is_true_and_reuses_the_value() {
    let mut control = InvocationControl::unbounded();
    let (driver, _) = driver(
        r#"
      <xsl:template match="/"><xsl:if test="$first">yes</xsl:if><xsl:if test="$first">yes</xsl:if></xsl:template>
      <xsl:template name="first_constructor"/>
      <xsl:template name="second_constructor"><xsl:message terminate="yes">unused</xsl:message></xsl:template>"#,
        &mut control,
    );
    assert_eq!(driver.run(&mut control).unwrap(), "yesyes");
    assert_eq!(driver.evaluations("first"), 1);
    assert_eq!(driver.evaluations("second"), 0);
    assert!(driver.resolver.stack.borrow().is_empty());
}

#[test]
fn demand_boolean_short_circuit_and_cycles_share_the_resolver() {
    for (expression, succeeds) in [("$first or $second", true), ("$first and $second", false)] {
        let mut control = InvocationControl::unbounded();
        let (driver, _) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:if test="{expression}">yes</xsl:if></xsl:template>
          <xsl:template name="first_constructor"/>
          <xsl:template name="second_constructor"><xsl:if test="$second">cycle</xsl:if></xsl:template>"#
            ),
            &mut control,
        );
        let result = driver.run(&mut control);
        if succeeds {
            assert_eq!(result.unwrap(), "yes");
        } else {
            assert_eq!(result.unwrap_err().code, "FXAR0030");
        }
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), usize::from(!succeeds));
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_boolean_local_shadow_does_not_evaluate_the_global() {
    let mut control = InvocationControl::unbounded();
    let (driver, _) = driver(
        r#"
      <xsl:template match="/"><xsl:variable name="first" select="0"/><xsl:if test="$first">wrong</xsl:if></xsl:template>
      <xsl:template name="first_constructor"><xsl:message terminate="yes">unreached</xsl:message></xsl:template>
      <xsl:template name="second_constructor"/>"#,
        &mut control,
    );
    assert_eq!(driver.run(&mut control).unwrap(), "");
    assert_eq!(driver.evaluations("first"), 0);
    assert!(control.messages().is_empty());
}
