//! Explicit instruction provenance for selected unary legacy conversions.

use super::super::failure_observations::driver;
use super::*;

const EXPRESSIONS: &[(&str, &str)] = &[
    ("string-length($second)", "4"),
    ("string-length(string($second)) * 2", "8"),
    ("normalize-space($second)", "12"),
    ("number($second)", "12"),
];

#[test]
fn demand_boolean_condition_retains_if_or_when_not_enclosing_choose() {
    for (condition, tag) in [
        (
            r#"<xsl:if test="string-length($second)">unused</xsl:if>"#,
            "<xsl:if",
        ),
        (
            r#"<xsl:choose><xsl:when test="false()">skip</xsl:when><xsl:when test="string-length($second)">unused</xsl:when><xsl:otherwise>unused</xsl:otherwise></xsl:choose>"#,
            "<xsl:when",
        ),
    ] {
        let mut control = InvocationControl::unbounded();
        let (driver, xml) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor">{condition}</xsl:template>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        let primary = driver.run(&mut control).unwrap_err();
        assert_eq!(primary.code, "XTMM9000");
        let context = driver
            .resolver
            .failed_context(1, &[], 0, &mut control)
            .unwrap();
        let read = context.dependencies[1].read.as_ref().unwrap();
        assert!(xml[read.span.clone()].starts_with(tag));
        assert!(xml[read.span.clone()].contains("string-length($second)"));
        assert_ne!(Some(read.as_ref()), primary.location.as_ref());
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_boolean_choose_skips_unreached_global_and_conserves_work() {
    for taken in [false, true] {
        let mut work = Vec::new();
        for trace in [false, true] {
            let mut control = InvocationControl::unbounded();
            let literal = if taken { "x" } else { "" };
            let (driver, _) = driver(
                &format!(
                    r#"
              <xsl:template match="/"><xsl:choose><xsl:when test="string-length($first)">yes</xsl:when><xsl:when test="string-length($second)">no</xsl:when><xsl:otherwise>empty</xsl:otherwise></xsl:choose></xsl:template>
              <xsl:template name="first_constructor"><xsl:text>{literal}</xsl:text></xsl:template>
              <xsl:template name="second_constructor">B</xsl:template>"#
                ),
                &mut control,
            );
            driver.provenance.enabled.set(trace);
            assert_eq!(
                driver.run(&mut control).unwrap(),
                if taken { "yes" } else { "no" }
            );
            assert_eq!(driver.evaluations("first"), 1);
            assert_eq!(driver.evaluations("second"), usize::from(!taken));
            assert!(driver.provenance.path.borrow().is_empty());
            work.push(control.consumed(WorkDomain::XPathOperation));
        }
        assert_eq!(work[0], work[1]);
    }
}

#[test]
fn demand_unary_string_reads_keep_compiled_site_and_primary_failure() {
    for (expression, _) in EXPRESSIONS {
        let mut control = InvocationControl::unbounded();
        let (driver, xml) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:value-of select="{expression}"/></xsl:template>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        let primary = driver.run(&mut control).unwrap_err();
        assert_eq!(primary.code, "XTMM9000", "{expression}");
        let context = driver
            .resolver
            .failed_context(1, &[], 0, &mut control)
            .unwrap();
        let site = context.dependencies[1].read.as_ref().unwrap();
        assert!(xml[site.span.clone()].contains(expression));
        assert_ne!(Some(site.as_ref()), primary.location.as_ref());
        assert_eq!(
            driver.read_string("second", &mut control).unwrap_err(),
            primary
        );
        assert_eq!(control.messages(), &["failure"]);
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_unary_string_provenance_preserves_shared_conversion_and_work() {
    for (expression, expected) in EXPRESSIONS {
        let mut costs = Vec::new();
        for trace in [false, true] {
            let mut control = InvocationControl::unbounded();
            let (driver, _) = driver(
                &format!(
                    r#"
              <xsl:template match="/"><xsl:value-of select="{expression}"/></xsl:template>
              <xsl:template name="second_constructor"><xsl:text> 12 </xsl:text></xsl:template>"#
                ),
                &mut control,
            );
            driver.provenance.enabled.set(trace);
            assert_eq!(driver.run(&mut control).unwrap(), *expected, "{expression}");
            assert_eq!(driver.run(&mut control).unwrap(), *expected);
            assert_eq!(driver.evaluations("second"), 1);
            assert!(driver.provenance.path.borrow().is_empty());
            assert!(driver.provenance.first_cycle.get().is_none());
            costs.push(control.consumed(WorkDomain::XPathOperation));
        }
        assert_eq!(costs[0], costs[1], "semantic work: {expression}");
    }
}

#[test]
fn demand_unary_string_local_shadow_does_not_evaluate_global() {
    for (expression, expected) in EXPRESSIONS {
        let mut control = InvocationControl::unbounded();
        let (driver, _) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:variable name="second" select="' 12 '"/><xsl:value-of select="{expression}"/></xsl:template>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">wrong binding</xsl:message></xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        assert_eq!(driver.run(&mut control).unwrap(), *expected, "{expression}");
        assert_eq!(driver.evaluations("second"), 0);
        assert!(control.messages().is_empty());
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_concat_reads_preserve_site_failure_order_and_completed_prefix() {
    for fail_first in [false, true] {
        let mut control = InvocationControl::unbounded();
        let first = if fail_first {
            "<xsl:message terminate=\"yes\">first failure</xsl:message>"
        } else {
            "<xsl:message>first complete</xsl:message>A"
        };
        let (driver, xml) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:value-of select="concat($first, '-', $second, $first)"/></xsl:template>
          <xsl:template name="first_constructor">{first}</xsl:template>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">second failure</xsl:message></xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, "XTMM9000");
        let failed = usize::from(!fail_first);
        let context = driver
            .resolver
            .failed_context(failed, &[], 0, &mut control)
            .unwrap();
        assert_eq!(context.dependencies.len(), 1);
        assert_eq!(context.dependencies[0].binding, failed);
        let site = context.dependencies[0].read.as_ref().unwrap();
        assert!(xml[site.span.clone()].starts_with("<xsl:value-of"));
        assert_ne!(Some(site.as_ref()), error.location.as_ref());
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), usize::from(!fail_first));
        if fail_first {
            assert_eq!(control.messages(), &["first failure"]);
        } else {
            assert_eq!(
                driver
                    .read_string("first", &mut control)
                    .unwrap()
                    .as_deref(),
                Some("A")
            );
            assert_eq!(control.messages(), &["first complete", "second failure"]);
        }
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_concat_repeated_values_share_evaluation_not_read_identity() {
    let mut costs = Vec::new();
    for trace in [false, true] {
        let mut control = InvocationControl::unbounded();
        let (driver, _) = driver(
            r#"
          <xsl:template match="/"><xsl:value-of select="concat($first, '-', $second, $first)"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:message>A</xsl:message>A</xsl:template>
          <xsl:template name="second_constructor"><xsl:message>B</xsl:message>B</xsl:template>"#,
            &mut control,
        );
        driver.provenance.enabled.set(trace);
        assert_eq!(driver.run(&mut control).unwrap(), "A-BA");
        assert_eq!(driver.run(&mut control).unwrap(), "A-BA");
        assert_eq!(control.messages(), &["A", "B"]);
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
        assert!(driver.provenance.path.borrow().is_empty());
        costs.push(control.consumed(WorkDomain::XPathOperation));
    }
    assert_eq!(costs[0], costs[1]);
}
