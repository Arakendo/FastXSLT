//! Shared evaluator control gates; the test constructs IR, not a public profile.
use super::*;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xslt::golden_semantics_experiment::{BooleanExpression, Instruction};

#[test]
fn numeric_introspection_arguments_fail_qname_validation_only_when_reached() {
    for version in ["1.0", "1.5"] {
        for (function, code) in [
            ("system-property", "XTDE1390"),
            ("function-available", "XTDE1400"),
            ("element-available", "XTDE1440"),
        ] {
            let bytes = format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="item"><xsl:if test="flag"><xsl:value-of select="{function}(123)"/></xsl:if><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet>"#).into_bytes();
            let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4096, 4096));
            resources.admit("urn:numeric-introspection", bytes).unwrap();
            let program = compile_resource(&resources.seal(), "urn:numeric-introspection").unwrap();
            for xml in ["<item/>", "<item><flag/></item>", "<other/>", "<item/>"] {
                let source = Document::from_parsed(
                    parse_document("urn:source", xml.as_bytes(), XML_LIMITS).unwrap(),
                )
                .unwrap();
                let result = super::super::execute_compiled_stylesheet_for_test(
                    &program, &source, "numeric",
                );
                if xml.contains("<flag/>") {
                    let failure = result.unwrap_err();
                    assert!(
                        failure.contains(code) && failure.contains("urn:numeric-introspection"),
                        "{failure}"
                    );
                    assert!(!failure.contains("XPTY0004"));
                } else {
                    assert_eq!(result.unwrap(), if xml == "<other/>" { "" } else { "ok" });
                }
            }
        }
    }
}

#[test]
fn numeric_introspection_boolean_calls_preserve_short_circuit_and_controls() {
    for (condition, fails) in [
        ("true() or system-property(123)", false),
        ("false() and system-property(123)", false),
        ("flag or system-property(123)", true),
    ] {
        let bytes = format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="text"/><xsl:template match="/"><xsl:if test="{condition}"><xsl:text>ok</xsl:text></xsl:if></xsl:template></xsl:stylesheet>"#).into_bytes();
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4096, 4096));
        resources.admit("urn:numeric-control", bytes).unwrap();
        let program = compile_resource(&resources.seal(), "urn:numeric-control").unwrap();
        let source =
            Document::from_parsed(parse_document("urn:source", b"<r/>", XML_LIMITS).unwrap())
                .unwrap();
        for (cancel, allowance) in [(false, 0), (false, 1), (true, 1)] {
            let mut limits = WorkLimits::unbounded();
            limits.xpath_operations = allowance;
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            if cancel {
                control = control.cancelling_on_charge(WorkDomain::XPathOperation, 0);
            }
            let result = execute_program(&program, &source, "numeric-control", &mut control);
            if fails {
                let failure = result.unwrap_err();
                assert_eq!(
                    failure.code,
                    if cancel {
                        "FXCT0001"
                    } else if allowance == 0 {
                        "FXCT0002"
                    } else {
                        "XTDE1390"
                    }
                );
                assert_eq!(failure.request_id.as_deref(), Some("numeric-control"));
                if failure.code == "XTDE1390" {
                    assert_eq!(
                        failure.location.as_ref().unwrap().resource,
                        "urn:numeric-control"
                    );
                } else {
                    assert_eq!(failure.work_domain, Some(WorkDomain::XPathOperation));
                }
            } else {
                assert!(result.is_ok());
            }
        }
        #[cfg(feature = "allocation-observation")]
        {
            let observation = allocation_counter::measure(|| {
                drop(execute_program(
                    &program,
                    &source,
                    "cleanup",
                    &mut InvocationControl::unbounded(),
                ));
            });
            assert_eq!(observation.bytes_current, 0);
            assert_eq!(observation.count_current, 0);
        }
    }
}

#[test]
fn deferred_boolean_charges_only_reached_calls_and_preserves_failure_ownership() {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4096, 4096));
    resources.admit("urn:deferred:style", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><xsl:if test="true()"><xsl:text>unreachable</xsl:text></xsl:if></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    let mut program = compile_resource(&resources.seal(), "urn:deferred:style").unwrap();
    let source =
        Document::from_parsed(parse_document("urn:deferred:source", b"<r/>", XML_LIMITS).unwrap())
            .unwrap();
    let location = SourceLocation {
        resource: "urn:deferred:call".to_owned(),
        span: 20..37,
    };
    let call = BooleanExpression::Xslt10DeferredFailure {
        code: "XPST0017",
        detail: "system-property requires exactly one argument".to_owned(),
        location: location.clone(),
    };
    for reached in [false, true, false] {
        let Instruction::If { test, .. } = &mut program.root_template.as_mut().unwrap().body[0]
        else {
            panic!("expected conditional")
        };
        *test = BooleanExpression::And {
            left: Box::new(BooleanExpression::Constant(reached)),
            right: Box::new(call.clone()),
        };
        let mut measured = InvocationControl::unbounded();
        let result = execute_program(&program, &source, "observed", &mut measured);
        if !reached {
            assert!(result.is_ok());
            assert_eq!(measured.consumed(WorkDomain::XPathOperation), 0);
            let mut cancel_at_call =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 0);
            assert!(execute_program(&program, &source, "skipped", &mut cancel_at_call).is_ok());
            continue;
        }
        let failure = result.unwrap_err();
        assert_eq!(failure.code, "XPST0017");
        assert_eq!(failure.category, FailureCategory::Invalid);
        assert_eq!(failure.request_id.as_deref(), Some("observed"));
        assert_eq!(failure.location, Some(location.clone()));
        assert_eq!(measured.consumed(WorkDomain::XPathOperation), 1);
        for allowance in [0, 1] {
            let mut limits = WorkLimits::unbounded();
            limits.xpath_operations = allowance;
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            let failure = execute_program(&program, &source, "bounded", &mut control).unwrap_err();
            assert_eq!(
                failure.code,
                if allowance == 0 {
                    "FXCT0002"
                } else {
                    "XPST0017"
                }
            );
        }
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 0);
        assert_eq!(
            execute_program(&program, &source, "cancelled", &mut cancelled)
                .unwrap_err()
                .code,
            "FXCT0001"
        );
        #[cfg(feature = "allocation-observation")]
        for cancel in [false, true] {
            let observed = allocation_counter::measure(|| {
                let mut control = InvocationControl::unbounded();
                if cancel {
                    control = control.cancelling_on_charge(WorkDomain::XPathOperation, 0);
                }
                let failure =
                    execute_program(&program, &source, "cleanup", &mut control).unwrap_err();
                assert_eq!(failure.code, if cancel { "FXCT0001" } else { "XPST0017" });
            });
            assert_eq!(observed.bytes_current, 0);
            assert_eq!(observed.count_current, 0);
        }
    }
}
