use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

fn static_sets(version: &str, sets: &str) -> Document {
    document("urn:selected-sets", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:attribute-set name="a"><xsl:attribute name="color">red</xsl:attribute><xsl:attribute name="empty"/></xsl:attribute-set><xsl:attribute-set name="b"><xsl:attribute name="color">blue</xsl:attribute></xsl:attribute-set><xsl:template match="/"><xsl:element name="test" namespace="urn:test" use-attribute-sets="{sets}">body</xsl:element></xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_static_attribute_sets_preserve_shared_order_namespaces_and_charges() {
    let source = document("urn:source", b"<r/>");
    for sets in ["a b", "b a"] {
        let baseline = compiler::compile_stylesheet(&static_sets("1.0", sets)).unwrap();
        let mut reference = InvocationControl::unbounded();
        let expected = execute(&baseline, &source, "same", &mut reference).unwrap();
        assert!(expected.contains("empty=\"\""));
        assert!(expected.contains("xmlns=\"urn:test\""));
        assert!(expected.contains(if sets == "a b" {
            "color=\"blue\""
        } else {
            "color=\"red\""
        }));
        for version in ["1.0", "2.0", "3.0"] {
            let style = static_sets(version, sets);
            let program =
                compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
            drop(style);
            let mut actual = InvocationControl::unbounded();
            assert_eq!(
                execute(&program, &source, "same", &mut actual).unwrap(),
                expected
            );
            for domain in [
                WorkDomain::XsltInstruction,
                WorkDomain::ResultNode,
                WorkDomain::ResultTextByte,
            ] {
                assert_eq!(actual.consumed(domain), reference.consumed(domain));
            }
            let required = actual.consumed(WorkDomain::ResultTextByte);
            assert!(required > 0);
            let mut limits = WorkLimits::unbounded();
            limits.result_text_bytes = required;
            assert_eq!(
                execute(
                    &program,
                    &source,
                    "exact",
                    &mut InvocationControl::new(CancellationToken::new(), limits)
                )
                .unwrap(),
                expected
            );
            limits.result_text_bytes -= 1;
            assert!(
                execute(
                    &program,
                    &source,
                    "short",
                    &mut InvocationControl::new(CancellationToken::new(), limits)
                )
                .unwrap_err()
                .contains("FXCT0002")
            );
            let mut cancelled =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultNode, 0);
            assert!(
                execute(&program, &source, "cancel", &mut cancelled)
                    .unwrap_err()
                    .contains("FXCT0001")
            );
            assert_eq!(
                execute(
                    &program,
                    &source,
                    "reuse",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn selected_static_sets_preserve_missing_set_diagnostics_and_guard_dynamic_content() {
    let reference = compiler::compile_stylesheet(&static_sets("1.0", "missing")).unwrap_err();
    for version in ["1.0", "2.0", "3.0"] {
        let Err(StrictPreprocessingFailure::Compile(error)) = compile(
            &static_sets(version, "missing"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("undefined set must fail")
        };
        assert_eq!(error.code, reference.code);
        assert_eq!(error.category, reference.category);
        assert_eq!(error.location.resource, reference.location.resource);
    }
    for content in [
        r#"<xsl:attribute-set name="a" use-attribute-sets="b"/>"#,
        r#"<xsl:attribute-set name="a"><xsl:attribute name="x"><xsl:value-of select="."/></xsl:attribute></xsl:attribute-set>"#,
        r#"<xsl:attribute-set name="a"><xsl:attribute name="{@name}">x</xsl:attribute></xsl:attribute-set>"#,
    ] {
        let style = document("urn:guarded-sets", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">{content}<xsl:template match="/"/></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(error)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("must stay guarded")
        };
        assert_eq!(error.code, "FXAR0031");
    }
}

fn style(version: &str, name: &str, extras: &str) -> Document {
    document("urn:computed-selected", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:result" version="{version}"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="r"><xsl:element name="{name}" {extras}><xsl:text>body</xsl:text></xsl:element></xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_attribute_element_names_preserve_shared_results_failures_and_charges() {
    let baseline = compiler::compile_stylesheet(&style("1.0", "{@x}", "")).unwrap();
    for xml in [
        "<r x='ok'/>",
        "<r x='p:ok'/>",
        "<r/>",
        "<r x='bad name'/>",
        "<r x='missing:ok'/>",
    ] {
        let source = document("urn:source", xml.as_bytes());
        let mut reference_control = InvocationControl::unbounded();
        let reference = execute(&baseline, &source, "same-request", &mut reference_control);
        for version in ["1.0", "2.0", "3.0"] {
            let program = compile(
                &style(version, "{@x}", ""),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let mut actual_control = InvocationControl::unbounded();
            let actual = execute(&program, &source, "same-request", &mut actual_control);
            match (&reference, actual) {
                (Ok(expected), Ok(actual)) => assert_eq!(&actual, expected),
                (Err(expected), Err(actual)) => {
                    assert_eq!(&actual, expected);
                }
                (expected, actual) => panic!("parity failure: {expected:?} / {actual:?}"),
            }
            for domain in [
                WorkDomain::XPathNodeVisit,
                WorkDomain::XsltInstruction,
                WorkDomain::ResultNode,
                WorkDomain::ResultTextByte,
            ] {
                assert_eq!(
                    actual_control.consumed(domain),
                    reference_control.consumed(domain)
                );
            }
        }
    }
}

#[test]
fn selected_attribute_element_names_preserve_control_limits_and_reuse() {
    let program = compile(
        &style("2.0", "{@x}", ""),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document("urn:source", b"<r x='ok'/>");
    let mut measure = InvocationControl::unbounded();
    let expected = execute(&program, &source, "measure", &mut measure).unwrap();
    let required = measure.consumed(WorkDomain::ResultNode);
    assert!(required > 0);
    let mut limits = WorkLimits::unbounded();
    limits.result_nodes = required;
    assert_eq!(
        execute(
            &program,
            &source,
            "exact",
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap(),
        expected
    );
    limits.result_nodes -= 1;
    assert!(
        execute(
            &program,
            &source,
            "short",
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap_err()
        .contains("FXCT0002")
    );
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultNode, 1);
    assert!(
        execute(&program, &source, "cancel", &mut cancelled)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        execute(
            &program,
            &source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn selected_computed_name_admission_does_not_open_general_avts() {
    for name in [
        "{@p:x}",
        "{@x[1]}",
        "{name()}",
        "prefix{@x}",
        "{$name}",
        "{child/@x}",
    ] {
        assert!(
            compile(
                &style("1.0", name, ""),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err(),
            "{name}"
        );
    }
    for extras in [r#"namespace="urn:other""#, r#"use-attribute-sets="s""#] {
        assert!(
            compile(
                &style("1.0", "{@x}", extras),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
    assert!(
        compile(
            &style("3.0", "{@x}", ""),
            Edition::Modern,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}
