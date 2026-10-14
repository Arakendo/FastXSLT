//! Failure provenance and message observations across recursive demand reads.

use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
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

pub(super) fn driver(body: &str, control: &mut InvocationControl) -> (Rc<Driver>, String) {
    let xml = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
      <xsl:variable name="first" select="'placeholder'"/>
      <xsl:variable name="second" select="'placeholder'"/>
      {body}</xsl:stylesheet>"#
    );
    let program = Arc::new(compile_stylesheet(&document("urn:failure:stylesheet", &xml)).unwrap());
    (
        Driver::new(
            program,
            document("urn:failure:source", "<r/>"),
            &BTreeMap::new(),
            false,
            control,
        )
        .unwrap(),
        xml,
    )
}

#[test]
fn demand_failure_nested_copy_preserves_exact_origin_and_memoized_message() {
    let body = r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:message>outer</xsl:message><xsl:copy-of select="$second"/></xsl:template>
      <xsl:template name="second_constructor"><xsl:message terminate="yes">inner failure</xsl:message></xsl:template>"#;
    let mut control = InvocationControl::unbounded();
    let (driver, xml) = driver(body, &mut control);
    let error = driver.run(&mut control).unwrap_err();
    assert_eq!(error.code, "XTMM9000");
    assert_eq!(error.category, FailureCategory::Invalid);
    assert_eq!(error.request_id.as_deref(), Some("demand-shared-reference"));
    assert_eq!(error.work_domain, None);
    let location = error.location.as_ref().unwrap();
    assert_eq!(location.resource, "urn:failure:stylesheet");
    assert!(xml[location.span.clone()].starts_with("<xsl:message terminate=\"yes\""));
    assert_eq!(control.messages(), &["outer", "inner failure"]);
    assert_eq!(
        driver.read_string("first", &mut control).unwrap_err(),
        error
    );
    assert_eq!(
        driver.read_string("second", &mut control).unwrap_err(),
        error
    );
    assert_eq!(control.messages(), &["outer", "inner failure"]);
    assert_eq!(driver.evaluations("first"), 1);
    assert_eq!(driver.evaluations("second"), 1);
    assert!(driver.resolver.stack.borrow().is_empty());
}

#[test]
fn demand_failure_success_messages_follow_first_demand_not_repeated_reads() {
    let body = r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/><xsl:value-of select="$second"/><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:message>before</xsl:message><xsl:copy-of select="$second"/><xsl:message>after</xsl:message></xsl:template>
      <xsl:template name="second_constructor"><xsl:message>dependency</xsl:message>B</xsl:template>"#;
    let mut control = InvocationControl::unbounded();
    let (driver, _) = driver(body, &mut control);
    assert_eq!(driver.run(&mut control).unwrap(), "BBB");
    assert_eq!(control.messages(), &["before", "dependency", "after"]);
    assert_eq!(driver.evaluations("first"), 1);
    assert_eq!(driver.evaluations("second"), 1);
}

#[test]
fn demand_failure_control_fields_survive_recursive_evaluation() {
    let body = r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:copy-of select="$second"/></xsl:template>
      <xsl:template name="second_constructor"><box>B</box></xsl:template>"#;
    for cancel in [false, true] {
        let mut setup = InvocationControl::unbounded();
        let (driver, _) = driver(body, &mut setup);
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = if cancel { usize::MAX } else { 0 };
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::ResultNode, 1);
        }
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, if cancel { "FXCT0001" } else { "FXCT0002" });
        assert_eq!(
            error.category,
            if cancel {
                FailureCategory::Cancelled
            } else {
                FailureCategory::Limit
            }
        );
        assert_eq!(error.work_domain, Some(WorkDomain::ResultNode));
        assert_eq!(error.request_id.as_deref(), Some("demand-shared-reference"));
        // Current control failures do not carry an expression location. Do not
        // relabel them with the outer variable-read site or claim richer traces.
        assert_eq!(error.location, None);
        let mut inspect = InvocationControl::unbounded();
        assert_eq!(
            driver.read_string("first", &mut inspect).unwrap_err(),
            error
        );
        assert_eq!(
            driver.read_string("second", &mut inspect).unwrap_err(),
            error
        );
        assert!(driver.resolver.stack.borrow().is_empty());
        assert!(
            !driver
                .resolver
                .states
                .borrow()
                .iter()
                .any(|state| matches!(state, super::super::State::Complete(_)))
        );
    }
}

#[test]
fn demand_failure_actual_cycle_retains_binding_order_without_inventing_locations() {
    let body = r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:copy-of select="$second"/></xsl:template>
      <xsl:template name="second_constructor"><xsl:value-of select="$first"/></xsl:template>"#;
    let mut control = InvocationControl::unbounded();
    let (driver, _) = driver(body, &mut control);
    let error = driver.run(&mut control).unwrap_err();
    assert_eq!(error.code, "FXAR0030");
    assert_eq!(error.location, None);
    assert_eq!(error.work_domain, None);
    assert_eq!(error.category, FailureCategory::Invalid);
    let states = driver.resolver.states.borrow();
    // The inner resolver owns the structured ordinal path. Translation presently
    // flattens it into an experimental failure; a related-location contract is
    // still missing and must not be inferred from this test.
    let inner = driver
        .program
        .global_bindings
        .iter()
        .position(|binding| binding.name == "second")
        .unwrap();
    assert!(matches!(states[inner], super::super::State::Failed(_)));
    assert!(error.detail.contains("[0, 1, 0]"));
    assert!(driver.resolver.stack.borrow().is_empty());
}
