//! Condition sites are whole instructions, not operand-token spans.
use super::super::failure_observations::driver;
use super::*;

#[test]
fn demand_string_literal_value_of_retains_read_site_and_origin() {
    for expression in ["$second = 'x'", "$second != 'x'", "not($second = 'x')"] {
        let mut control = InvocationControl::unbounded();
        let (driver, xml) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:value-of select="{expression}"/></xsl:template>
          <xsl:template name="first_constructor"/>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, "XTMM9000");
        let captured = driver
            .resolver
            .failed_context(1, &[], 0, &mut control)
            .unwrap();
        let read = captured.dependencies.last().unwrap().read.as_ref().unwrap();
        assert!(xml[read.span.clone()].starts_with("<xsl:value-of"));
        assert!(xml[read.span.clone()].contains(expression));
        assert_ne!(Some(read.as_ref()), error.location.as_ref());
        assert_eq!(driver.run(&mut control).unwrap_err(), error);
        assert_eq!(control.messages(), &["failure"]);
        assert!(driver.resolver.stack.borrow().is_empty());
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_boolean_value_of_retains_actual_read_site() {
    for expression in [
        "boolean($second)",
        "$second = true()",
        "$second != false()",
        "$first and $second",
    ] {
        let mut control = InvocationControl::unbounded();
        let (driver, xml) = driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:value-of select="{expression}"/></xsl:template>
          <xsl:template name="first_constructor"/>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, "XTMM9000");
        let captured = driver
            .resolver
            .failed_context(1, &[], 0, &mut control)
            .unwrap();
        let read = captured.dependencies.last().unwrap().read.as_ref().unwrap();
        assert!(xml[read.span.clone()].starts_with("<xsl:value-of"));
        assert!(xml[read.span.clone()].contains(expression));
        assert_ne!(Some(read.as_ref()), error.location.as_ref());
        assert_eq!(driver.run(&mut control).unwrap_err(), error);
        assert_eq!(control.messages(), &["failure"]);
        assert!(driver.resolver.stack.borrow().is_empty());
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_boolean_value_of_short_circuits_without_a_spurious_read_edge() {
    for trace in [false, true] {
        let mut control = InvocationControl::unbounded();
        let (driver, _) = driver(
            r#"
          <xsl:template match="/"><xsl:variable name="first" select="0"/><xsl:value-of select="$first and $second"/></xsl:template>
          <xsl:template name="first_constructor"/>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">unreached</xsl:message></xsl:template>"#,
            &mut control,
        );
        driver.provenance.enabled.set(trace);
        assert_eq!(driver.run(&mut control).unwrap(), "false");
        assert_eq!(driver.evaluations("first"), 0);
        assert_eq!(driver.evaluations("second"), 0);
        assert!(driver.provenance.path.borrow().is_empty());
        assert!(driver.provenance.first_cycle.get().is_none());
        assert!(control.messages().is_empty());
    }
}

#[test]
fn demand_boolean_failure_reuse_does_not_replay_messages() {
    let mut control = InvocationControl::unbounded();
    let (driver, xml) = driver(
        r#"
      <xsl:template match="/"><xsl:if test="$first">wrong</xsl:if></xsl:template>
      <xsl:template name="first_constructor"><xsl:message terminate="yes">failed</xsl:message></xsl:template>
      <xsl:template name="second_constructor"/>"#,
        &mut control,
    );
    driver.provenance.enabled.set(true);
    let first = driver.run(&mut control).unwrap_err();
    assert_eq!(first.code, "XTMM9000");
    assert_eq!(driver.run(&mut control).unwrap_err(), first);
    assert_eq!(driver.evaluations("first"), 1);
    assert_eq!(control.messages(), &["failed"]);
    let context = driver
        .resolver
        .failed_context(0, &[], 0, &mut control)
        .unwrap();
    assert_eq!(context.dependencies.len(), 1);
    let read = context.dependencies[0].read.as_ref().unwrap();
    assert!(xml[read.span.clone()].starts_with("<xsl:if"));
    assert_ne!(Some(read.as_ref()), first.location.as_ref());
    assert!(first.location.is_some());
    assert!(driver.provenance.path.borrow().is_empty());
    assert!(driver.resolver.stack.borrow().is_empty());
}

#[test]
fn demand_nested_boolean_preserves_if_and_when_sites() {
    for expression in [
        "not($second)",
        "$first and $second",
        "not($first) or $second",
        "not(string-length($second))",
        "string-length($first) and string-length($second)",
        "not(string-length($first)) or string-length($second)",
    ] {
        for choose in [false, true] {
            let condition = if choose {
                format!(
                    r#"<xsl:choose><xsl:when test="{expression}">unused</xsl:when></xsl:choose>"#
                )
            } else {
                format!(r#"<xsl:if test="{expression}">unused</xsl:if>"#)
            };
            let mut control = InvocationControl::unbounded();
            let (driver, xml) = driver(
                &format!(
                    r#"
              <xsl:template match="/">{condition}</xsl:template>
              <xsl:template name="first_constructor">A</xsl:template>
              <xsl:template name="second_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#
                ),
                &mut control,
            );
            driver.provenance.enabled.set(true);
            let failure = driver.run(&mut control).unwrap_err();
            assert_eq!(failure.code, "XTMM9000", "{expression}");
            let context = driver
                .resolver
                .failed_context(1, &[], 0, &mut control)
                .unwrap();
            let read = context.dependencies[0].read.as_ref().unwrap();
            assert!(xml[read.span.clone()].starts_with(if choose {
                "<xsl:when"
            } else {
                "<xsl:if"
            }));
            assert!(xml[read.span.clone()].contains(expression));
            assert_ne!(Some(read.as_ref()), failure.location.as_ref());
            assert!(driver.provenance.path.borrow().is_empty());
        }
    }
}

#[test]
fn demand_direct_boolean_cycle_preserves_closing_when_site() {
    let mut control = InvocationControl::unbounded();
    let (driver, xml) = driver(
        r#"
      <xsl:template match="/"><xsl:if test="$first">unused</xsl:if></xsl:template>
      <xsl:template name="first_constructor"><xsl:if test="$second">unused</xsl:if></xsl:template>
      <xsl:template name="second_constructor"><xsl:choose><xsl:when test="$first">unused</xsl:when></xsl:choose></xsl:template>"#,
        &mut control,
    );
    driver.provenance.enabled.set(true);
    assert_eq!(driver.run(&mut control).unwrap_err().code, "FXAR0030");
    let observed = driver.provenance.first_cycle.get().unwrap();
    assert_eq!(
        observed
            .dependencies
            .iter()
            .map(|edge| edge.binding)
            .collect::<Vec<_>>(),
        [0, 1, 0]
    );
    for (index, edge) in observed.dependencies.iter().enumerate() {
        assert!(
            xml[edge.read.as_ref().unwrap().span.clone()].starts_with(if index == 2 {
                "<xsl:when"
            } else {
                "<xsl:if"
            })
        );
    }
    assert!(driver.provenance.path.borrow().is_empty());
    assert!(driver.resolver.stack.borrow().is_empty());
}

#[test]
fn demand_boolean_short_circuit_never_evaluates_skipped_failure() {
    for (operator, first, expected) in [("and", "", ""), ("or", "A", "yes")] {
        let mut work = Vec::new();
        for trace in [false, true] {
            let mut control = InvocationControl::unbounded();
            let (driver, _) = driver(
                &format!(
                    r#"
              <xsl:template match="/"><xsl:if test="string-length($first) {operator} string-length($second)">yes</xsl:if></xsl:template>
              <xsl:template name="first_constructor"><xsl:text>{first}</xsl:text></xsl:template>
              <xsl:template name="second_constructor"><xsl:message terminate="yes">unreached</xsl:message></xsl:template>"#
                ),
                &mut control,
            );
            driver.provenance.enabled.set(trace);
            assert_eq!(driver.run(&mut control).unwrap(), expected);
            assert_eq!(driver.evaluations("first"), 1);
            assert_eq!(driver.evaluations("second"), 0);
            assert!(control.messages().is_empty());
            assert!(driver.provenance.first_cycle.get().is_none());
            assert!(driver.provenance.path.borrow().is_empty());
            work.push(control.consumed(WorkDomain::XPathOperation));
        }
        assert_eq!(work[0], work[1]);
    }
}

#[test]
fn demand_nested_boolean_cycle_keeps_closing_condition_site() {
    let mut control = InvocationControl::unbounded();
    let (driver, xml) = driver(
        r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:if test="not(string-length($second))">unused</xsl:if></xsl:template>
      <xsl:template name="second_constructor"><xsl:choose><xsl:when test="not(string-length($first))">unused</xsl:when></xsl:choose></xsl:template>"#,
        &mut control,
    );
    driver.provenance.enabled.set(true);
    assert_eq!(driver.run(&mut control).unwrap_err().code, "FXAR0030");
    let observed = driver.provenance.first_cycle.get().unwrap();
    assert_eq!(
        observed
            .dependencies
            .iter()
            .map(|edge| edge.binding)
            .collect::<Vec<_>>(),
        [0, 1, 0]
    );
    let read = observed.dependencies[2].read.as_ref().unwrap();
    assert!(xml[read.span.clone()].starts_with("<xsl:when"));
    assert_ne!(observed.dependencies[0].read, observed.dependencies[2].read);
    assert!(driver.provenance.path.borrow().is_empty());
    assert!(driver.resolver.stack.borrow().is_empty());
}
