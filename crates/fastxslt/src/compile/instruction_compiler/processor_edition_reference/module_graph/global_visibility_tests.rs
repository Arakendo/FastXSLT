//! Complete-pair static visibility; runtime globals and precedence remain shared.
use super::super::named_tests::document;
use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

fn pair(version: &str, value: &str) -> (Document, Document) {
    let principal = document("principal", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:import href="dependency"/><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:variable name="color" select="'{value}'"/><xsl:template match="r"><div style="{{concat('border: ',$color)}}"><xsl:apply-imports/></div></xsl:template></xsl:stylesheet>"#).as_bytes());
    let dependency = document("dependency", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="r"><pre><xsl:value-of select="concat('color: ',$color)"/></pre></xsl:template></xsl:stylesheet>"#);
    (principal, dependency)
}

#[test]
fn selected_complete_pair_reads_principal_globals_from_imported_templates() {
    let source = document("source", b"<r/>");
    for version in ["1.0", "2.0", "3.0"] {
        let (principal, dependency) = pair(version, "red");
        let program = compile_selected_legacy_resolved_pair(
            &principal,
            &dependency,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        drop((principal, dependency));
        for _ in 0..2 {
            assert_eq!(
                execute(
                    &program,
                    &source,
                    "same",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                r#"<div style="border: red"><pre>color: red</pre></div>"#
            );
        }
        let (principal, dependency) = pair(version, "blue");
        let other = compile_selected_legacy_resolved_pair(
            &principal,
            &dependency,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert!(
            execute(
                &other,
                &source,
                "other",
                &mut InvocationControl::unbounded()
            )
            .unwrap()
            .contains("color: blue")
        );
        assert!(
            execute(
                &program,
                &source,
                "old",
                &mut InvocationControl::unbounded()
            )
            .unwrap()
            .contains("color: red")
        );
    }
}

#[test]
fn selected_complete_pair_global_avt_limits_cancel_and_recover() {
    let (principal, dependency) = pair("2.0", "red");
    let program = compile_selected_legacy_resolved_pair(
        &principal,
        &dependency,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document("source", b"<r/>");
    let mut measure = InvocationControl::unbounded();
    let expected = execute(&program, &source, "measure", &mut measure).unwrap();
    let required = measure.consumed(WorkDomain::ResultTextByte);
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
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0);
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
fn selected_global_avt_stays_bounded_and_rejects_unbound_names() {
    for (value, code) in [
        ("{concat('x',$missing)}", "XPST0008"),
        ("prefix{concat('x',$color)}", "FXAR0031"),
        ("{concat($color,'x')}", "FXAR0031"),
        ("{concat('x',string($color))}", "FXAR0031"),
        ("{$color}", "FXAR0031"),
    ] {
        let principal = document("principal", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:variable name="color" select="'red'"/><xsl:template match="/"><div style="{value}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(error)) = compile(
            &principal,
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("must reject {value}")
        };
        assert_eq!(error.code, code, "{value}");
        assert_eq!(error.location.resource, "principal");
    }
}
