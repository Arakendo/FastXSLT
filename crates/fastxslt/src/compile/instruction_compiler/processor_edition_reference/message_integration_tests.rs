use super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn document(identity: &str, bytes: &[u8]) -> Document {
    Document::from_parsed(
        parse_document(
            identity,
            bytes,
            ParseLimits {
                max_events: 10000,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

fn sheet(body: &str) -> Document {
    versioned_sheet(body, "2.0")
}

fn versioned_sheet(body: &str, version: &str) -> Document {
    document("urn:selected-message", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_message_body_preserves_processor_policy_and_observation() {
    let style = sheet(
        r#"<xsl:message><xsl:sequence><xsl:fallback><xsl:value-of select="system-property('xsl:version')"/></xsl:fallback></xsl:sequence></xsl:message><xsl:text>result</xsl:text>"#,
    );
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    drop(style);
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        execute(
            &program,
            &document("urn:source", b"<r/>"),
            "message",
            &mut control
        )
        .unwrap(),
        "result"
    );
    assert_eq!(control.messages(), ["1"]);
}

#[test]
fn selected_message_termination_and_body_errors_keep_shared_timing() {
    let source = document("urn:source", b"<r/>");
    for (condition, fails) in [("false()", false), ("true()", true)] {
        let style = sheet(&format!(
            r#"<xsl:if test="{condition}"><xsl:message terminate="yes">stop</xsl:message></xsl:if><xsl:text>ok</xsl:text>"#
        ));
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let mut control = InvocationControl::unbounded();
        let result = execute(&program, &source, "termination", &mut control);
        if fails {
            let failure = result.unwrap_err();
            assert!(failure.contains("XTMM9000") && failure.contains("urn:selected-message"));
            assert_eq!(control.messages(), ["stop"]);
        } else {
            assert_eq!(result.unwrap(), "ok");
            assert!(control.messages().is_empty());
        }
    }
    let style = sheet(r#"<xsl:message><xsl:value-of select="system-property()"/></xsl:message>"#);
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let mut control = InvocationControl::unbounded();
    assert!(
        execute(&program, &source, "body-failure", &mut control)
            .unwrap_err()
            .contains("XPST0017")
    );
    assert!(control.messages().is_empty());
    for attribute in [
        "terminate='true'",
        "terminate='invalid'",
        "select=\"'hidden'\"",
        "unknown='value'",
        "xsl:version='1.0'",
    ] {
        let style = sheet(&format!("<xsl:message {attribute}>text</xsl:message>"));
        assert!(matches!(
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()),
            Err(StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Unsupported,
                ..
            }))
        ));
    }
}

#[test]
fn selected_messages_preserve_lexical_bindings_and_local_version_resets() {
    let style = sheet(
        r#"<xsl:variable name="v" select="'bound'"/><xsl:message><xsl:value-of select="$v"/><out xsl:version="1.0"><xsl:value-of select="system-property('xsl:version')"/></out></xsl:message><xsl:value-of select="$v"/>"#,
    );
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        execute(
            &program,
            &document("urn:source", b"<r/>"),
            "binding",
            &mut control
        )
        .unwrap(),
        "bound"
    );
    assert_eq!(control.messages(), ["bound1"]);
    let invalid = sheet(
        r#"<xsl:message><out xsl:version="1.0"><xsl:sequence><xsl:fallback/></xsl:sequence></out></xsl:message>"#,
    );
    assert!(matches!(
        compile(
            &invalid,
            Edition::Legacy,
            &mut InvocationControl::unbounded()
        ),
        Err(StrictPreprocessingFailure::Compile(CompileFailure {
            category: CompileCategory::Invalid,
            ..
        }))
    ));
}

#[test]
fn selected_messages_preserve_work_limits_cancellation_and_cleanup() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let body = "<xsl:message>note</xsl:message><xsl:text>ok</xsl:text>";
    let program = compile(
        &sheet(body),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let reference = compiler::compile_stylesheet(&versioned_sheet(body, "1.0")).unwrap();
    let source = document("urn:source", b"<r/>");
    let mut actual = InvocationControl::unbounded();
    let mut expected = InvocationControl::unbounded();
    assert_eq!(
        execute(&program, &source, "selected", &mut actual).unwrap(),
        "ok"
    );
    assert_eq!(
        execute(&reference, &source, "reference", &mut expected).unwrap(),
        "ok"
    );
    assert_eq!(actual.messages(), expected.messages());
    for domain in [
        WorkDomain::XPathNodeVisit,
        WorkDomain::XPathOperation,
        WorkDomain::XsltInstruction,
        WorkDomain::XsltTemplateCandidate,
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::SerializedByte,
    ] {
        assert_eq!(
            actual.consumed(domain),
            expected.consumed(domain),
            "{domain:?}"
        );
    }
    let run = || {
        let mut limits = WorkLimits::unbounded();
        limits.result_text_bytes = 3;
        let mut limited = InvocationControl::new(CancellationToken::new(), limits);
        assert!(
            execute(&program, &source, "limit", &mut limited)
                .unwrap_err()
                .contains("FXCT0002")
        );
        assert!(limited.messages().is_empty());
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0);
        assert!(
            execute(&program, &source, "cancel", &mut cancelled)
                .unwrap_err()
                .contains("FXCT0001")
        );
        assert!(cancelled.messages().is_empty());
    };
    run();
    #[cfg(feature = "allocation-observation")]
    {
        let observed = allocation_counter::measure(run);
        assert_eq!(observed.bytes_current, 0);
        assert_eq!(observed.count_current, 0);
    }
    assert_eq!(
        execute(
            &program,
            &source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "ok"
    );
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_message_replays_original_ver01() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let style = document(
        "urn:oasis:ver01",
        &std::fs::read(root.join("Xalan_Conformance_Tests/ver/ver01.xsl")).unwrap(),
    );
    let source = document(
        "urn:oasis:source",
        &std::fs::read(root.join("Xalan_Conformance_Tests/ver/ver01.xml")).unwrap(),
    );
    let expected =
        std::fs::read(root.join("Xalan_Conformance_Tests/REF_OUT/ver/ver01.out")).unwrap();
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    drop(style);
    let mut control = InvocationControl::unbounded();
    let actual = execute(&program, &source, "ver01", &mut control).unwrap();
    assert_eq!(
        crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
            &actual,
            &expected,
            false,
            None,
            Some("xml")
        ),
        Ok(true)
    );
    assert_eq!(
        control.messages(),
        ["This stylesheet requires XSLT 8.5 or higher"]
    );
}
