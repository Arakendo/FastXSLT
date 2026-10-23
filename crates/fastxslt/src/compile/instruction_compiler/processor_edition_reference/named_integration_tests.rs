use super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

pub(super) fn document(identity: &str, bytes: &[u8]) -> Document {
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

pub(super) fn sheet(version: &str, declarations: &str) -> Document {
    document("urn:selected-named", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:templates" xmlns:q="urn:templates" version="{version}"><xsl:output method="text"/>{declarations}</xsl:stylesheet>"#).as_bytes())
}

#[test]
fn shared_namespace_exclusion_must_remove_other_prefixes_for_the_same_uri() {
    let style = document("urn:exclusion", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:excluded" xmlns:q="urn:excluded" version="1.0" exclude-result-prefixes="p"><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#);
    let program = compiler::compile_stylesheet(&style).unwrap();
    let actual = execute(
        &program,
        &document("urn:source", b"<doc/>"),
        "exclusion",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert!(
        !actual.contains("urn:excluded"),
        "exclusion must apply to the namespace URI: {actual}"
    );
}

#[test]
fn shared_namespace_exclusion_resolves_at_declaration_and_preserves_required_names() {
    for (root_attributes, literal, present, absent) in [
        (
            r#"xmlns:p="urn:old" xmlns:q="urn:old" exclude-result-prefixes="p""#,
            r#"<out xmlns:p="urn:new"/>"#,
            "urn:new",
            "urn:old",
        ),
        (
            r#"xmlns:p="urn:old" xmlns:q="urn:new" exclude-result-prefixes="q""#,
            r#"<out xmlns:p="urn:new"/>"#,
            "",
            "urn:old",
        ),
        (
            r#"xmlns:p="urn:old" xmlns:q="urn:old""#,
            r#"<out xmlns:p="urn:new" xsl:exclude-result-prefixes="p"/>"#,
            "urn:old",
            "urn:new",
        ),
        (
            r#"xmlns:p="urn:required" xmlns:q="urn:required" exclude-result-prefixes="p""#,
            "<p:out/>",
            "xmlns:p=\"urn:required\"",
            "xmlns:q=",
        ),
        (
            r#"xmlns:p="urn:required" xmlns:q="urn:required" exclude-result-prefixes="p""#,
            r#"<out p:a="v"/>"#,
            "xmlns:p=\"urn:required\"",
            "xmlns:q=",
        ),
        (
            r#"xmlns:p="urn:extension" xmlns:q="urn:extension" extension-element-prefixes="p""#,
            "<out/>",
            "",
            "urn:extension",
        ),
        (
            r##"xmlns="urn:old" exclude-result-prefixes="#default""##,
            r#"<out xmlns=""/>"#,
            "",
            "urn:old",
        ),
    ] {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0" {root_attributes}><xsl:template match="/">{literal}</xsl:template></xsl:stylesheet>"#
        );
        let program =
            compiler::compile_stylesheet(&document("urn:exclusion", xml.as_bytes())).unwrap();
        let actual = execute(
            &program,
            &document("urn:source", b"<doc/>"),
            "exclusion",
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert!(actual.contains(present), "required {present}: {actual}");
        assert!(!actual.contains(absent), "forbidden {absent}: {actual}");
    }
}

#[test]
fn selected_root_exclusion_is_uri_based_without_changing_template_qnames() {
    for version in ["1.0", "2.0"] {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:named" xmlns:q="urn:named" version="{version}" exclude-result-prefixes="p"><xsl:template match="/"><out><xsl:call-template name="q:n"/></out></xsl:template><xsl:template name="p:n">correct</xsl:template><xsl:template name="n">wrong</xsl:template></xsl:stylesheet>"#
        );
        let style = document("urn:selected-exclusion", xml.as_bytes());
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(style);
        let actual = execute(
            &program,
            &document("urn:source", b"<doc/>"),
            "selected",
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert!(actual.contains("correct"), "{actual}");
        assert!(!actual.contains("wrong"), "{actual}");
        assert!(!actual.contains("urn:named"), "{actual}");
    }
}

#[test]
fn selected_exclusion_rejects_unbound_names_and_unintegrated_controls() {
    for (attributes, expected) in [
        ("exclude-result-prefixes='missing'", "XTSE0808"),
        ("exclude-result-prefixes='#all'", "FXAR0031"),
        ("exclude-result-prefixes='#default'", "FXAR0031"),
        ("extension-element-prefixes='p'", "FXAR0031"),
    ] {
        for version in ["1.0", "2.0"] {
            let xml = format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:p" version="{version}" {attributes}><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#
            );
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &document("urn:exclusion", xml.as_bytes()),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("invalid/unintegrated exclusion admitted");
            };
            assert_eq!(failure.code, expected);
        }
    }
}

#[test]
fn selected_number_format_parameter_preserves_legacy_conversion_and_controls() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let declarations = r#"<xsl:template match="doc"><xsl:apply-templates/></xsl:template><xsl:template match="item"><xsl:call-template name="n"><xsl:with-param name="format">A. </xsl:with-param></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="format">1. </xsl:param><xsl:number format="{$format}"/></xsl:template>"#;
    let source = document("urn:numbering", b"<doc><item/><item/></doc>");
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "reference", &mut baseline).unwrap(),
        "A. B. "
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            "A. B. "
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
                baseline.consumed(domain),
                "{version}: {domain:?}"
            );
        }
        let mut limits = WorkLimits::unbounded();
        limits.xslt_instructions = actual.consumed(WorkDomain::XsltInstruction) - 1;
        assert!(
            execute(
                &program,
                &source,
                "budget",
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap_err()
            .contains("FXCT0002")
        );
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XsltInstruction, 0);
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
            "A. B. "
        );
    }
}

#[test]
fn selected_numbering_does_not_bypass_static_scope_or_admit_other_options() {
    for version in ["1.0", "2.0"] {
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(
                version,
                r#"<xsl:template match="doc"><xsl:if test="false()"><xsl:number format="{$missing}"/></xsl:if></xsl:template>"#,
            ),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("unbound numbering format admitted");
        };
        assert_eq!(failure.code, "XPST0008");
        for attribute in [
            "value='1'",
            "level='any'",
            "count='item'",
            "from='doc'",
            "lang='en'",
            "letter-value='alphabetic'",
            "grouping-size='3'",
            "version='1.0'",
        ] {
            let declarations =
                format!("<xsl:template match='doc'><xsl:number {attribute}/></xsl:template>");
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &sheet(version, &declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("numbering option admitted: {attribute}");
            };
            assert_eq!(failure.code, "FXAR0031");
        }
    }
}

#[test]
fn selected_named_template_retains_processor_policy_and_qualified_name() {
    let style = sheet(
        "2.0",
        r#"<xsl:template match="/"><xsl:call-template name="q:n"/></xsl:template><xsl:template name="p:n"><xsl:sequence><xsl:fallback><xsl:value-of select="system-property('xsl:version')"/></xsl:fallback></xsl:sequence></xsl:template><xsl:template name="n"><xsl:text>wrong</xsl:text></xsl:template>"#,
    );
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    drop(style);
    assert_eq!(
        execute(
            &program,
            &document("urn:source", b"<r/>"),
            "named",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "1"
    );
}

#[test]
fn selected_named_and_matched_template_roles_preserve_focus_and_work() {
    use crate::execution_control_experiment::WorkDomain;
    let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"/><xsl:apply-templates/></xsl:template><xsl:template name="n" match="item"><xsl:value-of select="@v"/></xsl:template>"#;
    let program = compile(
        &sheet("2.0", declarations),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let source = document("urn:source", b"<doc v='caller'><item v='matched'/></doc>");
    let mut actual = InvocationControl::unbounded();
    let mut expected = InvocationControl::unbounded();
    assert_eq!(
        execute(&program, &source, "selected", &mut actual).unwrap(),
        "callermatched"
    );
    assert_eq!(
        execute(&reference, &source, "reference", &mut expected).unwrap(),
        "callermatched"
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
fn selected_named_references_preserve_static_failures_and_lexical_scope() {
    for declarations in [
        r#"<xsl:template name="p:n"/><xsl:template name="q:n"/>"#,
        r#"<xsl:template match="/"><xsl:call-template name="missing"/></xsl:template>"#,
        r#"<xsl:template match="/"><xsl:call-template name="unknown:n"/></xsl:template>"#,
        r#"<xsl:template name="{broken}"/>"#,
    ] {
        let Err(StrictPreprocessingFailure::Compile(actual)) = compile(
            &sheet("2.0", declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("expected static failure: {declarations}")
        };
        let expected = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap_err();
        assert_eq!(actual.code, expected.code);
        assert_eq!(actual.category, expected.category);
        assert_eq!(actual.location.resource, "urn:selected-named");
        assert!(!actual.location.span.is_empty());
    }
    let style = sheet(
        "2.0",
        r#"<xsl:template match="/"><xsl:variable name="caller" select="'secret'"/><xsl:call-template name="n"/></xsl:template><xsl:template name="n"><xsl:value-of select="$caller"/></xsl:template>"#,
    );
    let Err(StrictPreprocessingFailure::Compile(failure)) =
        compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
    else {
        panic!("caller local leaked")
    };
    assert_eq!(failure.code, "XPST0008");
}

#[test]
fn selected_named_templates_keep_parameters_and_modern_readers_guarded() {
    for declarations in [
        r#"<xsl:template name="n"><xsl:param name="p" select="system-property('xsl:version')"/></xsl:template>"#,
        r#"<xsl:template match="/"><xsl:call-template name="n"><xsl:with-param name="p" select="42.5"/></xsl:call-template></xsl:template><xsl:template name="n"/>"#,
        r#"<xsl:template name="n" mode="m"/>"#,
        r#"<xsl:template match="/"><xsl:call-template name="n" unknown="x"/></xsl:template><xsl:template name="n"/>"#,
    ] {
        assert!(matches!(
            compile(
                &sheet("2.0", declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            ),
            Err(StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Unsupported,
                ..
            }))
        ));
    }
    assert!(
        compile(
            &sheet("3.0", r#"<xsl:template name="n"/>"#),
            Edition::Modern,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}

#[test]
fn selected_named_body_defers_only_reached_call_errors() {
    let source = document("urn:source", b"<r/>");
    for (root_body, fails) in [
        ("<xsl:text>ok</xsl:text>", false),
        ("<xsl:call-template name='n'/>", true),
    ] {
        let declarations = format!(
            r#"<xsl:template match="/">{root_body}</xsl:template><xsl:template name="n"><xsl:value-of select="system-property()"/></xsl:template>"#
        );
        let program = compile(
            &sheet("2.0", &declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let result = execute(
            &program,
            &source,
            "timing",
            &mut InvocationControl::unbounded(),
        );
        if fails {
            let failure = result.unwrap_err();
            assert!(failure.contains("XPST0017") && failure.contains("urn:selected-named"));
        } else {
            assert_eq!(result.unwrap(), "ok");
        }
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet("1.0", &declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("normal context must retain static arity failure")
        };
        assert_eq!(failure.code, "XPST0017");
    }
}

#[test]
fn selected_named_calls_preserve_failure_cleanup_and_reuse() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let program = compile(&sheet("2.0", r#"<xsl:template match="/"><xsl:call-template name="n"/></xsl:template><xsl:template name="n"><xsl:text>ok</xsl:text></xsl:template>"#), Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let source = document("urn:source", b"<r/>");
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
                assert_eq!(result.unwrap(), "ok");
            }
        }
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0);
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
        "ok"
    );
    let recursive = compile(&sheet("2.0", r#"<xsl:template match="/"><xsl:call-template name="n"/></xsl:template><xsl:template name="n"><xsl:call-template name="n"/></xsl:template>"#), Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let mut limits = WorkLimits::unbounded();
    limits.xslt_instructions = 16;
    let failure = execute(
        &recursive,
        &source,
        "recursive",
        &mut InvocationControl::new(CancellationToken::new(), limits),
    )
    .unwrap_err();
    assert!(failure.contains("FXCT0002") && failure.contains("xslt-instruction"));
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_named_templates_replay_original_qname_cases() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    for case in [
        "namedtemplate01",
        "namedtemplate02",
        "namedtemplate03",
        "namedtemplate04",
        "namedtemplate05",
        "namedtemplate06",
        "namedtemplate07",
        "namedtemplate08",
        "namedtemplate09",
        "namedtemplate10",
        "namedtemplate11",
        "namedtemplate12",
        "namedtemplate13",
        "namedtemplate14",
        "namedtemplate15",
        "namedtemplate16",
    ] {
        let style = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("Xalan_Conformance_Tests/namedtemplate/{case}.xsl")))
                .unwrap(),
        );
        let source = document(
            "urn:oasis:source",
            &std::fs::read(root.join(format!("Xalan_Conformance_Tests/namedtemplate/{case}.xml")))
                .unwrap(),
        );
        let expected = std::fs::read(root.join(format!(
            "Xalan_Conformance_Tests/REF_OUT/namedtemplate/{case}.out"
        )))
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

#[test]
fn selected_named_literal_defaults_and_temporary_arguments_preserve_shared_work() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let declarations = r#"<xsl:template match="doc"><xsl:variable name="v"><xsl:value-of select="a"/></xsl:variable><xsl:call-template name="n"><xsl:with-param name="p" select="$v"/></xsl:call-template><xsl:call-template name="n"/><xsl:value-of select="$v"/></xsl:template><xsl:template name="n"><xsl:param name="p">default</xsl:param><xsl:param name="q">!</xsl:param><xsl:value-of select="$p"/><xsl:value-of select="$q"/></xsl:template>"#;
    let source = document("urn:source", b"<doc><a>supplied</a></doc>");
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let mut expected = InvocationControl::unbounded();
    let expected_text = execute(&reference, &source, "reference", &mut expected).unwrap();
    assert_eq!(expected_text, "supplied!default!supplied");
    for version in ["1.0", "2.0"] {
        let style = sheet(version, declarations);
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(style);
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            expected_text
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
                "{version}: {domain:?}"
            );
        }
        let run = || {
            for allowance in [
                actual.consumed(WorkDomain::ResultTextByte) - 1,
                actual.consumed(WorkDomain::ResultTextByte),
            ] {
                let mut limits = WorkLimits::unbounded();
                limits.result_text_bytes = allowance;
                let result = execute(
                    &program,
                    &source,
                    "budget",
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                );
                if allowance < actual.consumed(WorkDomain::ResultTextByte) {
                    assert!(result.unwrap_err().contains("FXCT0002"));
                } else {
                    assert_eq!(result.unwrap(), expected_text);
                }
            }
            let mut cancelled =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0);
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
            expected_text
        );
    }
}

#[test]
fn selected_named_parameter_validation_does_not_leak_or_widen_policy() {
    for (declarations, code) in [
        (
            r#"<xsl:template name="n"><xsl:param name="p"/><xsl:param name="p"/></xsl:template>"#,
            "FXST0012",
        ),
        (
            r#"<xsl:template name="n">body<xsl:param name="p"/></xsl:template>"#,
            "FXST0011",
        ),
        (
            r#"<xsl:template match="/"><xsl:call-template name="n"><xsl:with-param name="p" select="$absent"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p"/></xsl:template>"#,
            "XPST0008",
        ),
        (
            r#"<xsl:template match="/"><xsl:call-template name="n"/><xsl:value-of select="$p"/></xsl:template><xsl:template name="n"><xsl:param name="p"/></xsl:template>"#,
            "XPST0008",
        ),
    ] {
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet("2.0", declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("expected {code}");
        };
        assert_eq!(failure.code, code);
    }
    for parameter in [
        r#"<xsl:param name="p" select="system-property('xsl:version')"/>"#,
        r#"<xsl:param name="p"><xsl:value-of select="system-property('xsl:version')"/></xsl:param>"#,
        r#"<xsl:param name="p" tunnel="yes"/>"#,
    ] {
        assert!(
            compile(
                &sheet(
                    "2.0",
                    &format!(r#"<xsl:template name="n">{parameter}</xsl:template>"#)
                ),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
    // Expression defaults remain guarded in both named and matched roles.
    assert!(
        compile(
            &sheet(
                "2.0",
                r#"<xsl:template name="n" match="doc"><xsl:param name="p" select="system-property('xsl:version')"/></xsl:template>"#
            ),
            Edition::Legacy,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}

#[test]
fn selected_named_and_matched_parameters_preserve_caller_and_selected_focus() {
    use crate::execution_control_experiment::WorkDomain;
    let declarations = r#"<xsl:template match="doc"><xsl:variable name="v"><xsl:value-of select="a"/></xsl:variable><xsl:call-template name="n"><xsl:with-param name="kind" select="'named:'"/><xsl:with-param name="p" select="$v"/></xsl:call-template><xsl:apply-templates select="items/item"><xsl:with-param name="kind" select="'matched:'"/><xsl:with-param name="p" select="$v"/></xsl:apply-templates><xsl:apply-templates select="items/item"/></xsl:template><xsl:template name="n" match="item"><xsl:param name="kind">default:</xsl:param><xsl:param name="p">none</xsl:param><xsl:value-of select="$kind"/><xsl:value-of select="$p"/><xsl:value-of select="@v"/><xsl:text>;</xsl:text></xsl:template>"#;
    let source = document(
        "urn:focus",
        b"<doc v='root'><a>arg</a><items><item v='A'/><item v='B'/></items></doc>",
    );
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    let expected = execute(&reference, &source, "baseline", &mut baseline).unwrap();
    assert_eq!(
        expected,
        "named:argroot;matched:argA;matched:argB;default:noneA;default:noneB;"
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            expected
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
                baseline.consumed(domain),
                "{version}: {domain:?}"
            );
        }
    }
}

#[test]
fn selected_temporary_argument_retains_selected_value_policy_and_guards_deferred_constructors() {
    let source = document("urn:source", b"<doc/>");
    for (expression, expected) in [
        ("system-property('xsl:version')", Some("1")),
        ("system-property()", None),
    ] {
        let declarations = format!(
            r#"<xsl:template match="doc"><xsl:variable name="v"><xsl:value-of select="{expression}"/></xsl:variable><xsl:call-template name="n"><xsl:with-param name="p" select="$v"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p">fallback</xsl:param><xsl:value-of select="$p"/></xsl:template>"#
        );
        let compiled = compile(
            &sheet("2.0", &declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        );
        if let Some(expected) = expected {
            let program = compiled.unwrap();
            assert_eq!(
                execute(
                    &program,
                    &source,
                    "selected",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        } else {
            // A deferred instruction is not a ValueExpression; do not flatten
            // it into a successful temporary value or fall back to old policy.
            assert!(matches!(
                compiled,
                Err(StrictPreprocessingFailure::Compile(CompileFailure {
                    category: CompileCategory::Unsupported,
                    ..
                }))
            ));
            assert!(
                compile(
                    &sheet("1.0", &declarations),
                    Edition::Legacy,
                    &mut InvocationControl::unbounded()
                )
                .is_err()
            );
        }
    }
}

#[test]
fn selected_recursive_name_arguments_preserve_focus_and_bounded_execution() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let declarations = r#"<xsl:template match="/doc"><xsl:variable name="v" select="name(.)"/><xsl:call-template name="n"><xsl:with-param name="p" select="$v"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p">default</xsl:param><xsl:value-of select="$p"/><xsl:text>;</xsl:text><xsl:apply-templates select="*"/></xsl:template><xsl:template match="*"><xsl:call-template name="n"><xsl:with-param name="p" select="name(.)"/></xsl:call-template></xsl:template>"#;
    let source = document("urn:recursive", b"<doc><a><b/></a><c/></doc>");
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "baseline", &mut baseline).unwrap(),
        "doc;a;b;c;"
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            "doc;a;b;c;"
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
                baseline.consumed(domain),
                "{version}: {domain:?}"
            );
        }
        let mut limits = WorkLimits::unbounded();
        limits.xslt_instructions = actual.consumed(WorkDomain::XsltInstruction) - 1;
        assert!(
            execute(
                &program,
                &source,
                "budget",
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap_err()
            .contains("FXCT0002")
        );
        let mut cancelled = InvocationControl::unbounded()
            .cancelling_on_charge(WorkDomain::XsltTemplateCandidate, 0);
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
            "doc;a;b;c;"
        );
    }
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_named_import_cases_require_supplied_graph() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    for number in 17..=19 {
        let case = format!("namedtemplate{number:02}");
        let style = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("Xalan_Conformance_Tests/namedtemplate/{case}.xsl")))
                .unwrap(),
        );
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("reclassify {case}");
        };
        assert_eq!(
            failure.category,
            CompileCategory::Unsupported,
            "{case}: {failure:?}"
        );
        assert_eq!(failure.code, "FXAR0031", "{case}: {failure:?}");
    }
}

#[test]
fn selected_string_literal_defaults_are_not_general_expression_admission() {
    let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"/><xsl:apply-templates select="item"/></xsl:template><xsl:template name="n" match="item"><xsl:param name="p" select="'literal'"/><xsl:value-of select="$p"/></xsl:template>"#;
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(
            execute(
                &program,
                &document("urn:defaults", b"<doc><item/></doc>"),
                "defaults",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "literalliteral"
        );
        let invalid = r#"<xsl:template name="n"><xsl:param name="p" select="'literal'">content</xsl:param></xsl:template>"#;
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(version, invalid),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("select and content must fail");
        };
        assert_eq!(failure.category, CompileCategory::Invalid);
    }
}

#[test]
fn selected_named_metadata_does_not_filter_calls_or_reinterpret_source_arguments() {
    use crate::execution_control_experiment::WorkDomain;
    let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p" select="a"/></xsl:call-template><xsl:apply-templates select="a"/></xsl:template><xsl:template name="n" match="a" mode="other" priority="2147483647"><xsl:param name="p"/><xsl:value-of select="$p"/><xsl:value-of select="@v"/><xsl:text>;</xsl:text></xsl:template><xsl:template match="a">plain;</xsl:template>"#;
    let source = document(
        "urn:source-argument",
        b"<doc v='caller'><a>first</a><a>second</a></doc>",
    );
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "baseline", &mut baseline).unwrap(),
        "firstcaller;plain;plain;"
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            "firstcaller;plain;plain;"
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
                baseline.consumed(domain),
                "{version}: {domain:?}"
            );
        }
        for priority in ["-2147483648", "2147483647"] {
            let declarations = format!(
                r#"<xsl:template match="doc"><xsl:apply-templates select="a"/></xsl:template><xsl:template match="a">ordinary</xsl:template><xsl:template match="a" priority="{priority}">explicit</xsl:template>"#
            );
            let program = compile(
                &sheet(version, &declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(
                execute(
                    &program,
                    &source,
                    "priority",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                if priority.starts_with('-') {
                    "ordinaryordinary"
                } else {
                    "explicitexplicit"
                }
            );
        }
    }
    for attributes in [
        r##"mode="#all""##,
        r#"mode="one two""#,
        r#"priority="0.1""#,
        r#"priority="9007199254740993""#,
    ] {
        assert!(matches!(
            compile(
                &sheet("2.0", &format!(r#"<xsl:template match="a" {attributes}/>"#)),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            ),
            Err(StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Unsupported,
                ..
            }))
        ));
    }
}

#[test]
fn selected_source_arguments_release_owners_after_selection_failure() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let style = sheet(
        "2.0",
        r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p" select="items/item"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p"/><xsl:value-of select="$p"/></xsl:template>"#,
    );
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    drop(style);
    let run = || {
        let source = document(
            "urn:retained-argument",
            b"<doc><items><item>first</item><item>second</item></items></doc>",
        );
        let mut baseline = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "baseline", &mut baseline).unwrap(),
            "first"
        );
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = baseline.consumed(WorkDomain::XPathNodeVisit) - 1;
        assert!(
            execute(
                &program,
                &source,
                "budget",
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap_err()
            .contains("FXCT0002")
        );
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 0);
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
            "first"
        );
    };
    run();
    #[cfg(feature = "allocation-observation")]
    {
        let observed = allocation_counter::measure(run);
        assert_eq!(observed.bytes_current, 0);
        assert_eq!(observed.count_current, 0);
    }
}

#[test]
fn selected_content_arguments_and_variable_predicates_preserve_shared_controls() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p">hit</xsl:with-param></xsl:call-template><xsl:call-template name="n"><xsl:with-param name="p">miss</xsl:with-param></xsl:call-template><xsl:call-template name="n"><xsl:with-param name="p"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p">default</xsl:param><xsl:if test="./item[@kind=string($p)]"><xsl:value-of select="$p"/><xsl:text>!</xsl:text></xsl:if></xsl:template>"#;
    let source = document("urn:predicates", b"<doc xmlns:p='urn:other'><item kind='hit'/><item p:kind='miss'/><item/><item kind=''/></doc>");
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "baseline", &mut baseline).unwrap(),
        "hit!!"
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            "hit!!"
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
                baseline.consumed(domain),
                "{version}: {domain:?}"
            );
        }
        let run = || {
            let mut limits = WorkLimits::unbounded();
            limits.xpath_node_visits = actual.consumed(WorkDomain::XPathNodeVisit) - 1;
            assert!(
                execute(
                    &program,
                    &source,
                    "budget",
                    &mut InvocationControl::new(CancellationToken::new(), limits)
                )
                .unwrap_err()
                .contains("FXCT0002")
            );
            let mut cancelled =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 0);
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
                "hit!!"
            );
        };
        run();
        #[cfg(feature = "allocation-observation")]
        {
            let observed = allocation_counter::measure(run);
            assert_eq!(observed.bytes_current, 0);
            assert_eq!(observed.count_current, 0);
        }
    }
}

#[test]
fn selected_variable_predicate_checks_dormant_bindings_and_constructor_limits() {
    for version in ["1.0", "2.0"] {
        for condition in [
            "./item[@kind=string($missing)]",
            "false() and ./item[@kind=string($missing)]",
            "true() or ./item[@kind=string($missing)]",
        ] {
            let declarations = format!(
                r#"<xsl:template match="doc"><xsl:if test="{condition}">bad</xsl:if></xsl:template>"#
            );
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &sheet(version, &declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("unbound predicate admitted");
            };
            assert_eq!(failure.code, "XPST0008");
        }
        let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p"><xsl:value-of select="system-property('xsl:version')"/></xsl:with-param></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p"/></xsl:template>"#;
        assert!(matches!(
            compile(
                &sheet(version, declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            ),
            Err(StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Unsupported,
                ..
            }))
        ));
    }
}

#[test]
fn selected_integer_arguments_preserve_numeric_truth_and_lexical_limits() {
    let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p" select="0"/></xsl:call-template><xsl:call-template name="n"><xsl:with-param name="p" select="-2147483648"/></xsl:call-template><xsl:call-template name="n"><xsl:with-param name="p" select="2147483647"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p">default</xsl:param><xsl:value-of select="$p"/><xsl:choose><xsl:when test="$p">T;</xsl:when><xsl:otherwise>F;</xsl:otherwise></xsl:choose></xsl:template>"#;
    let source = document("urn:integers", b"<doc/>");
    let expected = execute(
        &compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap(),
        &source,
        "baseline",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(expected, "0F;-2147483648T;2147483647T;");
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(
            execute(
                &program,
                &source,
                "selected",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
        for select in ["+1", "1e2", "2147483648", "0.5"] {
            let declarations = format!(
                r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p" select="{select}"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p"/></xsl:template>"#
            );
            assert!(matches!(
                compile(
                    &sheet(version, &declarations),
                    Edition::Legacy,
                    &mut InvocationControl::unbounded()
                ),
                Err(StrictPreprocessingFailure::Compile(CompileFailure {
                    category: CompileCategory::Unsupported,
                    ..
                }))
            ));
        }
    }
}

#[test]
fn selected_parameter_numeric_composition_preserves_legacy_conversion() {
    use crate::execution_control_experiment::WorkDomain;
    let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p" select="a"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p"/><xsl:variable name="v"><xsl:value-of select="number($p)"/></xsl:variable><xsl:value-of select="$v"/><xsl:text>|</xsl:text><xsl:value-of select="1 + $v"/><xsl:text>;</xsl:text></xsl:template>"#;
    let source = document("urn:numeric", b"<doc><a> 12 </a><a>99</a></doc>");
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "baseline", &mut baseline).unwrap(),
        "12|13;"
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            "12|13;"
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
                baseline.consumed(domain),
                "{version}: {domain:?}"
            );
        }
        let nonnumeric = document("urn:nonnumeric", b"<doc><a>bad</a></doc>");
        for candidate in [&reference, &program] {
            let failure = execute(
                candidate,
                &nonnumeric,
                "unsupported-arithmetic",
                &mut InvocationControl::unbounded(),
            )
            .unwrap_err();
            assert!(failure.contains("FXRT1022"), "{failure}");
            assert_eq!(
                execute(
                    candidate,
                    &source,
                    "reuse",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "12|13;"
            );
        }
        for expression in ["number($missing)", "1 + $missing"] {
            let declarations = format!(
                r#"<xsl:template match="doc"><xsl:if test="false()"><xsl:value-of select="{expression}"/></xsl:if></xsl:template>"#
            );
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &sheet(version, &declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("unbound numeric variable admitted");
            };
            assert_eq!(failure.code, "XPST0008");
        }
    }
}

#[test]
fn selected_iteration_argument_preserves_order_empty_content_and_controls() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let declarations = r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p"><xsl:for-each select="a"><xsl:value-of select="."/></xsl:for-each></xsl:with-param></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p">default</xsl:param><xsl:value-of select="$p"/><xsl:value-of select="$p"/></xsl:template>"#;
    let reference = compiler::compile_stylesheet(&sheet("1.0", declarations)).unwrap();
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        for (xml, expected) in [
            (
                "<doc><a>first<b>nested</b>end</a><a>second</a></doc>",
                "firstnestedendsecondfirstnestedendsecond",
            ),
            ("<doc><a xmlns='urn:other'>ignored</a></doc>", ""),
            ("<doc/>", ""),
        ] {
            let source = document("urn:iteration", xml.as_bytes());
            let mut baseline = InvocationControl::unbounded();
            let mut actual = InvocationControl::unbounded();
            assert_eq!(
                execute(&reference, &source, "reference", &mut baseline).unwrap(),
                expected
            );
            assert_eq!(
                execute(&program, &source, "selected", &mut actual).unwrap(),
                expected
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
                    baseline.consumed(domain),
                    "{version}: {domain:?}"
                );
            }
        }
        let source = document("urn:iteration", b"<doc><a>one</a><a>two</a></doc>");
        let mut baseline = InvocationControl::unbounded();
        execute(&program, &source, "baseline", &mut baseline).unwrap();
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = baseline.consumed(WorkDomain::XPathNodeVisit) - 1;
        assert!(
            execute(
                &program,
                &source,
                "budget",
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap_err()
            .contains("FXCT0002")
        );
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 0);
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
            "onetwoonetwo"
        );
    }
}

#[test]
fn selected_iteration_argument_does_not_admit_general_constructor_semantics() {
    for constructor in [
        r#"<xsl:for-each select="a" version="1.0"><xsl:value-of select="."/></xsl:for-each>"#,
        r#"<xsl:for-each select="a"><xsl:value-of select="." disable-output-escaping="yes"/></xsl:for-each>"#,
        r#"<xsl:for-each select="a"><xsl:value-of select="position()"/></xsl:for-each>"#,
        r#"<xsl:for-each select="a[1]"><xsl:value-of select="."/></xsl:for-each>"#,
        r#"<xsl:for-each select="a"><xsl:sequence select="."/></xsl:for-each>"#,
        r#"<xsl:for-each select="a"><xsl:text>prefix</xsl:text><xsl:value-of select="."/></xsl:for-each>"#,
    ] {
        for version in ["1.0", "2.0"] {
            let declarations = format!(
                r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p">{constructor}</xsl:with-param></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p"/><xsl:value-of select="$p"/></xsl:template>"#
            );
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &sheet(version, &declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("general constructor admitted: {constructor}");
            };
            assert_eq!(failure.code, "FXAR0031");
        }
    }
}

#[test]
fn selected_iteration_argument_uses_caller_focus_for_apply_templates() {
    let declarations = r#"<xsl:template match="doc"><xsl:apply-templates select="target"><xsl:with-param name="p"><xsl:for-each select="a"><xsl:value-of select="."/></xsl:for-each></xsl:with-param></xsl:apply-templates></xsl:template><xsl:template match="target"><xsl:param name="p">default</xsl:param><xsl:value-of select="$p"/><xsl:text>;</xsl:text></xsl:template>"#;
    let source = document(
        "urn:caller",
        b"<doc><a>caller</a><target><a>wrong</a></target><target/></doc>",
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(
            execute(
                &program,
                &source,
                "apply",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "caller;caller;"
        );
    }
}
