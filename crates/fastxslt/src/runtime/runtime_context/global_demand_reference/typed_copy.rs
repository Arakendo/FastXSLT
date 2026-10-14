//! Compiled copy consumers demanding owned temporary values through one resolver.

use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn document(xml: &str) -> Arc<Document> {
    Arc::new(
        Document::from_parsed(
            parse_document(
                "urn:typed-demand",
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

fn driver(body: &str, complete: bool, control: &mut InvocationControl) -> Rc<Driver> {
    let stylesheet = document(&format!(
        r#"
      <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
        <xsl:variable name="first" select="'placeholder'"/>
        <xsl:variable name="second" select="'placeholder'"/>
        {body}
      </xsl:stylesheet>"#
    ));
    Driver::new(
        Arc::new(compile_stylesheet(&stylesheet).unwrap()),
        document("<r/>"),
        &BTreeMap::new(),
        complete,
        control,
    )
    .unwrap()
}

const FORWARD: &str = r#"
  <xsl:template match="/"><xsl:copy-of select="$first"/><xsl:copy-of select="$first"/></xsl:template>
  <xsl:template name="first_constructor"><outer><xsl:copy-of select="$second"/></outer></xsl:template>
  <xsl:template name="second_constructor"><inner xmlns:p="urn:p" p:a="v">B</inner></xsl:template>"#;

#[test]
fn demand_typed_copy_recursively_resolves_once_and_preserves_result_structure() {
    for complete in [false, true] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(FORWARD, complete, &mut control);
        let result = driver
            .execute(
                &driver.program.root_template.as_ref().unwrap().body,
                &mut control,
            )
            .unwrap();
        assert_eq!(result.roots.len(), 2);
        assert_eq!(
            temporary_tree_string_value(&result, "result", &mut control).unwrap(),
            "BB"
        );
        for root in &result.roots {
            let node = &result.nodes[*root];
            let super::super::super::TemporaryNodeKind::Element { name, .. } = &node.kind else {
                panic!("outer element");
            };
            assert_eq!(name.local, "outer");
            let inner = &result.nodes[node.children[0]];
            let super::super::super::TemporaryNodeKind::Element {
                name,
                attributes,
                namespaces,
            } = &inner.kind
            else {
                panic!("inner element");
            };
            assert_eq!(name.local, "inner");
            assert_eq!(attributes.len(), 1);
            assert!(
                namespaces
                    .iter()
                    .any(|binding| binding.namespace == "urn:p")
            );
        }
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
        let value = driver.read_value("first", &mut control).unwrap().unwrap();
        let OwnedValue::Temporary(tree) = value.as_ref() else {
            panic!("temporary");
        };
        let weak = Arc::downgrade(tree);
        drop(value);
        drop(driver);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            temporary_tree_string_value(&result, "after-drop", &mut control).unwrap(),
            "BB"
        );
    }
}

#[test]
fn demand_typed_copy_local_shadow_does_not_evaluate_the_global() {
    let body = r#"
      <xsl:template match="/">
        <xsl:variable name="first"><local>L</local></xsl:variable>
        <xsl:copy-of select="$first"/>
        <xsl:value-of select="$second"/>
      </xsl:template>
      <xsl:template name="first_constructor"><xsl:message terminate="yes">wrong global</xsl:message></xsl:template>
      <xsl:template name="second_constructor">S</xsl:template>"#;
    for complete in [false, true] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(body, complete, &mut control);
        assert_eq!(driver.run(&mut control).unwrap(), "LS");
        assert_eq!(driver.evaluations("first"), 0);
        assert_eq!(driver.evaluations("second"), 1);
    }
}

#[test]
fn demand_typed_copy_and_string_reads_share_cycle_and_completed_state() {
    for cycle in [false, true] {
        let tail = if cycle {
            "<xsl:value-of select=\"$first\"/>"
        } else {
            "B"
        };
        let body = format!(
            r#"
          <xsl:template match="/"><xsl:copy-of select="$first"/><xsl:value-of select="$second"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:copy-of select="$second"/></xsl:template>
          <xsl:template name="second_constructor">{tail}</xsl:template>"#
        );
        let mut control = InvocationControl::unbounded();
        let driver = driver(&body, false, &mut control);
        let result = driver.run(&mut control);
        if cycle {
            let error = result.unwrap_err();
            assert_eq!(error.code, "FXAR0030");
            assert_eq!(driver.run(&mut control).unwrap_err(), error);
        } else {
            assert_eq!(result.unwrap(), "BB");
        }
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_typed_copy_limits_and_cancellation_leave_no_active_markers() {
    let mut baseline = InvocationControl::unbounded();
    let reference = driver(FORWARD, false, &mut baseline);
    assert_eq!(reference.run(&mut baseline).unwrap(), "BB");
    let exact = baseline.consumed(WorkDomain::XdmNode);
    for (limit, cancel, success) in [
        (exact, false, true),
        (exact - 1, false, false),
        (exact, true, false),
    ] {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        let driver = driver(FORWARD, false, &mut control);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::XdmNode, 1);
        }
        let result = driver.run(&mut control);
        if success {
            assert_eq!(result.unwrap(), "BB");
        } else {
            assert_eq!(
                result.unwrap_err().code,
                if cancel { "FXCT0001" } else { "FXCT0002" }
            );
        }
        assert!(driver.resolver.stack.borrow().is_empty());
        assert!(
            !driver
                .resolver
                .states
                .borrow()
                .iter()
                .any(|state| matches!(state, super::super::State::Evaluating))
        );
    }
}

#[test]
fn demand_typed_copy_failure_does_not_poison_an_already_completed_global() {
    let mut setup = InvocationControl::unbounded();
    let driver = driver(FORWARD, false, &mut setup);
    let first = driver.read_value("first", &mut setup).unwrap().unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(driver.run(&mut baseline).unwrap(), "BB");
    let exact = baseline.consumed(WorkDomain::ResultNode);
    assert!(exact > 1);
    for (limit, cancel) in [(exact, false), (exact - 1, false), (exact, true)] {
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::ResultNode, 1);
        }
        let result = driver.run(&mut control);
        if limit == exact && !cancel {
            assert_eq!(result.unwrap(), "BB");
        } else {
            assert_eq!(
                result.unwrap_err().code,
                if cancel { "FXCT0001" } else { "FXCT0002" }
            );
        }
        // A consumer failed after resolution; it must not rewrite the completed
        // cell as a failed constructor. A fresh control is a test inspection,
        // not a promise that a cancelled production invocation can be resumed.
        let mut inspect = InvocationControl::unbounded();
        let retained = driver.read_value("first", &mut inspect).unwrap().unwrap();
        assert!(Arc::ptr_eq(&first, &retained));
        assert_eq!(retained.string_value(&mut inspect).unwrap(), "B");
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_typed_copy_cow_and_complete_frames_conserve_work_totals() {
    let domains = [
        WorkDomain::XdmNode,
        WorkDomain::XdmStringValueNode,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XPathOperation,
        WorkDomain::XsltInstruction,
        WorkDomain::XsltTemplateCandidate,
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
    ];
    let mut observations = Vec::new();
    for complete in [false, true] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(FORWARD, complete, &mut control);
        assert_eq!(driver.run(&mut control).unwrap(), "BB");
        observations.push(domains.map(|domain| control.consumed(domain)));
    }
    assert_eq!(observations[0], observations[1]);
}
