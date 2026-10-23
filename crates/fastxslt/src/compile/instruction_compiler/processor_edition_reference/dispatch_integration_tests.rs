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

fn sheet(version: &str, declarations: &str) -> Document {
    document("urn:selected-dispatch", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/>{declarations}</xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_dispatch_retains_policy_inside_explicit_template_calls() {
    let style = sheet(
        "2.0",
        r#"<xsl:template match="/"><xsl:apply-templates/></xsl:template><xsl:template match="doc"><xsl:apply-templates/></xsl:template><xsl:template match="doc/item"><xsl:sequence><xsl:fallback><xsl:value-of select="./@v"/><xsl:value-of select="system-property('xsl:version')"/></xsl:fallback></xsl:sequence></xsl:template>"#,
    );
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    drop(style);
    assert_eq!(
        execute(
            &program,
            &document("urn:source", b"<doc><item v='a'/><item v='b'/></doc>"),
            "dispatch",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "a1b1"
    );
}

#[test]
fn selected_dispatch_preserves_child_pattern_priority_and_source_focus() {
    use crate::execution_control_experiment::WorkDomain;
    let declarations = r#"<xsl:template match="/"><xsl:apply-templates/></xsl:template><xsl:template match="doc/item"><xsl:value-of select="@v"/></xsl:template><xsl:template match="item"><xsl:text>other</xsl:text></xsl:template><xsl:template match="text()"><xsl:apply-templates/></xsl:template>"#;
    let program = compile(
        &sheet("2.0", declarations),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let source = document(
        "urn:source",
        b"<doc>ignore<item v='first'/><nested><item v='not-first'/></nested><item v='last'/></doc>",
    );
    let mut actual = InvocationControl::unbounded();
    let mut expected = InvocationControl::unbounded();
    assert_eq!(
        execute(&program, &source, "selected", &mut actual).unwrap(),
        "firstotherlast"
    );
    assert_eq!(
        execute(&reference, &source, "reference", &mut expected).unwrap(),
        "firstotherlast"
    );
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
}

#[test]
fn selected_dispatch_does_not_admit_argument_mode_or_selection_readers() {
    for instruction in [
        r#"<xsl:apply-templates select="item[position()=2]"/>"#,
        r##"<xsl:apply-templates mode="#current"/>"##,
        r#"<xsl:apply-templates unknown="x"/>"#,
        r#"<xsl:apply-templates><xsl:with-param name="p" select="42.5"/></xsl:apply-templates>"#,
        r#"<xsl:apply-templates><xsl:sort select="text()"/></xsl:apply-templates>"#,
        r#"<xsl:value-of select="@*"/>"#,
        r#"<xsl:value-of select="./@*[1]"/>"#,
    ] {
        let style = sheet(
            "2.0",
            &format!(r#"<xsl:template match="/">{instruction}</xsl:template>"#),
        );
        assert!(
            matches!(
                compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()),
                Err(StrictPreprocessingFailure::Compile(CompileFailure {
                    category: CompileCategory::Unsupported,
                    ..
                }))
            ),
            "{instruction}"
        );
    }
    let style = sheet(
        "2.0",
        r#"<xsl:template match="/"><xsl:variable name="caller" select="'secret'"/><xsl:apply-templates/></xsl:template><xsl:template match="item"><xsl:value-of select="$caller"/></xsl:template>"#,
    );
    let Err(StrictPreprocessingFailure::Compile(failure)) =
        compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
    else {
        panic!("caller local leaked into callee")
    };
    assert_eq!(failure.code, "XPST0008");
    let modern = sheet(
        "3.0",
        r#"<xsl:template match="/"><xsl:apply-templates/></xsl:template>"#,
    );
    assert!(
        compile(
            &modern,
            Edition::Modern,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
    assert!(!admitted_value(
        "@v",
        LocalVersion::ModernWitness(DeclaredVersion::Three)
    ));
}

#[test]
fn selected_stylesheet_id_does_not_choose_processor_policy() {
    for tag in ["stylesheet", "transform"] {
        let xml = format!(
            r#"<xsl:{tag} xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0" id="modern"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="system-property('xsl:version')"/></xsl:template></xsl:{tag}>"#
        );
        let style = document("urn:id-metadata", xml.as_bytes());
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(style);
        assert_eq!(
            execute(
                &program,
                &document("urn:source", b"<r/>"),
                "id",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "1"
        );
    }
}

#[test]
fn selected_dispatch_preserves_budgets_cancellation_and_reuse() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let program = compile(&sheet("2.0", r#"<xsl:template match="/"><xsl:apply-templates/></xsl:template><xsl:template match="doc/item"><xsl:value-of select="./@v"/></xsl:template>"#), Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let source = document("urn:source", b"<doc><item v='ab'/></doc>");
    let run = || {
        for allowance in [1, 2] {
            let mut limits = WorkLimits::unbounded();
            limits.result_text_bytes = allowance;
            let result = execute(
                &program,
                &source,
                "limit",
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance == 1 {
                assert!(result.unwrap_err().contains("FXCT0002"));
            } else {
                assert_eq!(result.unwrap(), "ab");
            }
        }
        let mut cancelled = InvocationControl::unbounded()
            .cancelling_on_charge(WorkDomain::XsltTemplateCandidate, 0);
        assert!(
            execute(&program, &source, "cancel", &mut cancelled)
                .unwrap_err()
                .contains("FXCT0001")
        );
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
        "ab"
    );
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_dispatch_replays_original_version_cases() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    for case in ["ver02", "ver03", "ver04"] {
        let style = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("Xalan_Conformance_Tests/ver/{case}.xsl"))).unwrap(),
        );
        let source = document(
            "urn:oasis:source",
            &std::fs::read(root.join(format!("Xalan_Conformance_Tests/ver/{case}.xml"))).unwrap(),
        );
        let expected =
            std::fs::read(root.join(format!("Xalan_Conformance_Tests/REF_OUT/ver/{case}.out")))
                .unwrap();
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(style);
        let actual = execute(&program, &source, case, &mut InvocationControl::unbounded()).unwrap();
        assert_eq!(
            crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                &actual,
                &expected,
                false,
                None,
                Some("xml")
            ),
            Ok(true),
            "{case}: {actual}"
        );
    }
}
