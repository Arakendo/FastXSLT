use super::super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
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

fn sheet(content: &str, version: &str) -> Document {
    document("urn:conditional-policy", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="item">{content}</xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_descendant_condition_is_document_rooted_and_charged() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
    let content = r#"<xsl:if test="//outside and not(//absent)">found</xsl:if><xsl:choose><xsl:when test="//absent">bad</xsl:when><xsl:otherwise>default</xsl:otherwise></xsl:choose>"#;
    let source = document("urn:descendants", b"<root><outside/><item/></root>");
    let reference = compiler::compile_stylesheet(&sheet(content, "1.0")).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "baseline", &mut baseline).unwrap(),
        "founddefault"
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(content, version),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            "founddefault"
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
    }
}

#[test]
fn selected_conditional_attributes_follow_policy_without_hiding_static_errors() {
    let source = document("urn:conditional-source", b"<item/>");
    for content in [
        r#"<xsl:if unknown="{ $missing" test="true()">ok</xsl:if>"#,
        r#"<xsl:choose unknown="{ $missing"><xsl:when unknown="ignored" test="false()">bad</xsl:when><xsl:otherwise unknown="ignored">ok</xsl:otherwise></xsl:choose>"#,
    ] {
        for version in ["1.0", "1.5", "2.0", "3.0"] {
            let style = sheet(content, version);
            let compiled = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
            if version == "1.0" {
                let Err(StrictPreprocessingFailure::Compile(failure)) = compiled else {
                    panic!("normal contexts reject unknown conditional attributes");
                };
                assert_eq!(failure.code, "XTSE0090");
            } else {
                let program = compiled.unwrap();
                drop(style);
                assert_eq!(
                    execute_compiled_stylesheet_for_test(&program, &source, "conditional").unwrap(),
                    "ok"
                );
            }
        }
        assert!(
            compile(
                &sheet(content, "3.0"),
                Edition::Modern,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
    for (body, code) in [
        (
            r#"<xsl:if unknown="ignored" test="false()"><xsl:value-of select="$missing"/></xsl:if>"#,
            "XPST0008",
        ),
        (
            r#"<xsl:choose unknown="ignored"><xsl:otherwise>bad</xsl:otherwise></xsl:choose>"#,
            "XTSE0010",
        ),
        (r#"<xsl:if unknown="ignored">bad</xsl:if>"#, "XTSE0010"),
    ] {
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(body, "2.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("static conditional failure");
        };
        assert_eq!(failure.code, code);
        assert_eq!(failure.location.resource, "urn:conditional-policy");
    }
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn processor_policy_replays_original_forward_expression_timing() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let source = document(
        "urn:oasis:forward-source",
        &std::fs::read(root.join("MSFT_Conformance_Tests/ForwardComp/books.xml")).unwrap(),
    );
    for (case, failure_code) in [
        ("91849", None),
        ("91850", Some("XPST0003")),
        ("91853", None),
        ("91854", Some("XPST0017")),
        ("91855", None),
        ("91856", Some("XTDE1390")),
    ] {
        let sheet = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("MSFT_Conformance_Tests/ForwardComp/{case}.xsl")))
                .unwrap(),
        );
        let program =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let result = execute_compiled_stylesheet_for_test(&program, &source, case);
        if let Some(code) = failure_code {
            let failure = result.unwrap_err();
            assert!(
                failure.contains(code) && failure.contains(&format!("urn:oasis:{case}")),
                "{failure}"
            );
        } else {
            let expected = std::fs::read(root.join(format!(
                "MSFT_Conformance_Tests/REF_OUT/ForwardComp/{case}.txt"
            )))
            .unwrap();
            assert_eq!(
                crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                    &result.unwrap(),
                    &expected,
                    false,
                    None,
                    Some("xml")
                ),
                Ok(true)
            );
        }
    }
}

#[test]
fn selected_conditional_options_preserve_scope_guards_and_deferred_calls() {
    for attribute in [
        r#"version="2.0""#,
        r#"default-mode="missing""#,
        r#"default-collation="urn:missing""#,
        r#"xpath-default-namespace="urn:changed""#,
        r#"xml:space="preserve""#,
    ] {
        let style = sheet(
            &format!(r#"<xsl:if test="true()" {attribute}>bad</xsl:if>"#),
            "2.0",
        );
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("unintegrated static context admitted");
        };
        assert_eq!(failure.code, "FXAR0031");
    }
    for (outer, local, valid) in [("1.0", "2.0", true), ("2.0", "1.0", false)] {
        let style = sheet(
            &format!(
                r#"<out xsl:version="{local}"><xsl:if unknown="ignored" test="true()">ok</xsl:if></out>"#
            ),
            outer,
        );
        let result = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
        assert_eq!(result.is_ok(), valid);
    }
    let style = sheet(
        r#"<xsl:choose unknown="ignored"><xsl:when unknown="ignored" test="flag">ok</xsl:when><xsl:otherwise unknown="ignored"><xsl:value-of select="system-property()"/></xsl:otherwise></xsl:choose>"#,
        "2.0",
    );
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    drop(style);
    for xml in [
        b"<item><flag/></item>".as_slice(),
        b"<item/>",
        b"<item><flag/></item>",
    ] {
        let result = execute_compiled_stylesheet_for_test(
            &program,
            &document("urn:source", xml),
            "conditional",
        );
        if xml == b"<item/>" {
            let failure = result.unwrap_err();
            assert!(failure.contains("XPST0017") && failure.contains("urn:conditional-policy"));
        } else {
            assert_eq!(result.unwrap(), "ok");
        }
    }
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus; fragment evidence only"]
fn selected_conditionals_replay_original_bvt033_fragments_without_case_credit() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let style = document(
        "urn:oasis:bvt033",
        &std::fs::read(root.join("MSFT_Conformance_Tests/BVTs/fwdcompat.xsl")).unwrap(),
    );
    let stylesheet = compiler::document_element(&style).unwrap();
    let template = meaningful_children(&style, stylesheet)
        .into_iter()
        .find(|node| {
            is_xslt_element(&style, *node, "template")
                && optional_attribute(&style, *node, None, "match") == Some("/")
        })
        .unwrap();
    for (wrapper, instruction_name) in [("test5", "choose"), ("test12", "if")] {
        let result = meaningful_children(&style, template)
            .into_iter()
            .find(|node| {
                style
                    .name(*node)
                    .is_some_and(|name| name.namespace.is_none() && name.local == wrapper)
            })
            .unwrap();
        let node = meaningful_children(&style, result)
            .into_iter()
            .find(|node| is_xslt_element(&style, *node, instruction_name))
            .unwrap();
        let instruction = super::compile(
            &style,
            node,
            Edition::Legacy,
            LocalVersion::LegacyForward,
            0,
            StaticBindings {
                locals: &[],
                globals: &[],
            },
        )
        .unwrap();
        match instruction {
            Instruction::If {
                test: BooleanExpression::Constant(true),
                body,
                location,
            } => {
                assert_eq!(body.len(), 1);
                assert_eq!(location, *style.location(node));
            }
            Instruction::Choose {
                branches,
                otherwise,
                location,
            } => {
                assert_eq!(branches.len(), 1);
                assert!(matches!(
                    branches[0].test,
                    BooleanExpression::Constant(true)
                ));
                assert_eq!(branches[0].body.len(), 1);
                assert!(otherwise.is_empty());
                assert_eq!(location, *style.location(node));
            }
            _ => panic!("shared conditional IR"),
        }
    }
    assert!(compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).is_err());
}

#[test]
fn processor_policy_conditionals_select_per_source_and_preserve_deferred_calls() {
    for content in [
        r#"<xsl:if test="flag or system-property()"><xsl:text>ok</xsl:text></xsl:if>"#,
        r#"<xsl:choose><xsl:when test="flag"><xsl:text>ok</xsl:text></xsl:when><xsl:when test="system-property()"><xsl:text>bad</xsl:text></xsl:when><xsl:otherwise><xsl:text>bad</xsl:text></xsl:otherwise></xsl:choose>"#,
    ] {
        let style = sheet(content, "2.0");
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        for (xml, expected) in [
            ("<item><flag/></item>", Some("ok")),
            ("<item/>", None),
            ("<other/>", Some("")),
            ("<item><flag/></item>", Some("ok")),
        ] {
            let source = document("urn:conditional-source", xml.as_bytes());
            let result = execute_compiled_stylesheet_for_test(&program, &source, "conditional");
            if let Some(expected) = expected {
                assert_eq!(result.unwrap(), expected);
            } else {
                let failure = result.unwrap_err();
                assert!(
                    failure.contains("XPST0017") && failure.contains("urn:conditional-policy"),
                    "{failure}"
                );
            }
        }
        for (edition, version) in [(Edition::Legacy, "1.0"), (Edition::Modern, "2.0")] {
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &sheet(content, version),
                edition,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("bad call must remain static")
            };
            assert_eq!(failure.code, "XPST0017");
        }
    }
}

#[test]
fn processor_policy_conditionals_keep_lexical_errors_static_in_skipped_branches() {
    for content in [
        r#"<xsl:if test="false()"><xsl:value-of select="$missing"/></xsl:if>"#,
        r#"<xsl:choose><xsl:when test="true()"><xsl:text>ok</xsl:text></xsl:when><xsl:otherwise><xsl:value-of select="$missing"/></xsl:otherwise></xsl:choose>"#,
        r#"<xsl:if test="false() and $missing"><xsl:text>bad</xsl:text></xsl:if>"#,
        r#"<xsl:choose><xsl:when test="true()"><xsl:variable name="inside" select="'x'"/></xsl:when><xsl:otherwise><xsl:value-of select="$inside"/></xsl:otherwise></xsl:choose>"#,
    ] {
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(content, "2.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("missing binding must stay static")
        };
        assert_eq!(failure.code, "XPST0008");
        assert_eq!(failure.location.resource, "urn:conditional-policy");
    }
}

#[test]
fn processor_policy_conditionals_preserve_scope_and_first_branch_only() {
    let content = r#"<xsl:variable name="outer" select="'kept'"/><xsl:if test="$outer"><xsl:variable name="local" select="'inner'"/><xsl:value-of select="$local"/></xsl:if><xsl:choose><xsl:when test="false()"><xsl:value-of select="system-property()"/></xsl:when><xsl:when test="flag"><xsl:variable name="local" select="'chosen'"/><xsl:value-of select="$local"/></xsl:when><xsl:when test="true()"><xsl:text>other</xsl:text></xsl:when><xsl:otherwise><xsl:value-of select="system-property()"/></xsl:otherwise></xsl:choose><xsl:variable name="local" select="'after'"/><xsl:value-of select="$local"/><xsl:value-of select="$outer"/>"#;
    let program = compile(
        &sheet(content, "2.0"),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    for (xml, expected) in [
        ("<item><flag/></item>", "innerchosenafterkept"),
        ("<item/>", "innerotherafterkept"),
    ] {
        assert_eq!(
            execute_compiled_stylesheet_for_test(
                &program,
                &document("urn:scope", xml.as_bytes()),
                "scope"
            )
            .unwrap(),
            expected
        );
    }
    let leaked = r#"<xsl:if test="true()"><xsl:variable name="local" select="'hidden'"/></xsl:if><xsl:value-of select="$local"/>"#;
    let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
        &sheet(leaked, "2.0"),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    ) else {
        panic!("branch binding leaked")
    };
    assert_eq!(failure.code, "XPST0008");
}

#[test]
fn processor_policy_conditionals_do_not_change_local_version_or_profile() {
    let content = r#"<out xsl:version="2.0"><xsl:if test="false() and system-property()"><xsl:text>bad</xsl:text></xsl:if><xsl:choose><xsl:when test="true()"><xsl:new><xsl:fallback><xsl:text>ok</xsl:text></xsl:fallback></xsl:new></xsl:when><xsl:otherwise><xsl:text>bad</xsl:text></xsl:otherwise></xsl:choose></out>"#;
    let program = compile(
        &sheet(content, "1.0"),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(
        program.processor,
        crate::xslt::golden_semantics_experiment::ProcessorPolicyStamp::Xslt10Reference
    );
    assert_eq!(
        execute_compiled_stylesheet_for_test(&program, &document("urn:local", b"<item/>"), "local")
            .unwrap(),
        "ok"
    );
    let strict_child = r#"<out xsl:version="1.0"><xsl:if test="false() and system-property()"><xsl:text>bad</xsl:text></xsl:if></out>"#;
    let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
        &sheet(strict_child, "2.0"),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    ) else {
        panic!("local 1.0 must retain static error")
    };
    assert_eq!(failure.code, "XPST0017");
}

#[test]
fn processor_policy_conditionals_share_structural_validation_and_keep_guards() {
    for content in [
        r"<xsl:if/>",
        r"<xsl:choose/>",
        r"<xsl:choose><xsl:otherwise/></xsl:choose>",
        r#"<xsl:choose><xsl:when test="true()"/><xsl:otherwise/><xsl:when test="false()"/></xsl:choose>"#,
    ] {
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(content, "2.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("invalid structure admitted")
        };
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.location.resource, "urn:conditional-policy");
    }
    for content in [
        r#"<xsl:if test="true()" version="1.0"/>"#,
        r#"<xsl:if test="true()" xpath-default-namespace="urn:x"/>"#,
        r#"<xsl:if test="function-available('unknown')"/>"#,
        r#"<xsl:choose><xsl:when test="true()" xsl:version="1.0"/></xsl:choose>"#,
        r#"<xsl:choose><xsl:when test="true()"/><xsl:otherwise xsl:version="1.0"/></xsl:choose>"#,
    ] {
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(content, "2.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("unproved static context admitted")
        };
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }
    let deep = format!(
        "{}<xsl:text>deep</xsl:text>{}",
        r#"<xsl:if test="true()">"#.repeat(17),
        "</xsl:if>".repeat(17)
    );
    let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
        &sheet(&deep, "2.0"),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    ) else {
        panic!("depth guard bypassed")
    };
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn processor_policy_conditionals_match_shared_output_and_work_charges() {
    use crate::execution_control_experiment::WorkDomain;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
    let content = r#"<xsl:choose><xsl:when test="flag"><xsl:text>yes</xsl:text></xsl:when><xsl:otherwise><xsl:if test="not(flag)"><xsl:text>no</xsl:text></xsl:if></xsl:otherwise></xsl:choose>"#;
    for (edition, version) in [
        (Edition::Legacy, "1.0"),
        (Edition::Modern, "2.0"),
        (Edition::Legacy, "1.5"),
    ] {
        let style = sheet(content, version);
        let selected_style = if version == "1.5" {
            sheet(
                &content
                    .replace("<xsl:choose>", "<xsl:choose unknown=\"ignored\">")
                    .replace("<xsl:when ", "<xsl:when unknown=\"ignored\" ")
                    .replace("<xsl:otherwise>", "<xsl:otherwise unknown=\"ignored\">")
                    .replace("<xsl:if ", "<xsl:if unknown=\"ignored\" "),
                version,
            )
        } else {
            sheet(content, version)
        };
        let selected = compile(
            &selected_style,
            edition,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let reference = compiler::compile_stylesheet(&style).unwrap();
        for xml in ["<item/>", "<item><flag/></item>"] {
            let source = document("urn:differential", xml.as_bytes());
            let mut selected_control = InvocationControl::unbounded();
            let mut reference_control = InvocationControl::unbounded();
            assert_eq!(
                execute(&selected, &source, "differential", &mut selected_control).unwrap(),
                execute(&reference, &source, "differential", &mut reference_control).unwrap()
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
                    selected_control.consumed(domain),
                    reference_control.consumed(domain),
                    "{domain:?}"
                );
            }
        }
    }
}

#[test]
fn processor_policy_conditionals_pin_literal_multiargument_call_boundaries() {
    for (condition, expected) in [
        (
            "true() or system-property('xsl:version', 'other')",
            Some("ok"),
        ),
        (
            "false() and system-property('xsl:version', 'other')",
            Some(""),
        ),
        ("false() or system-property('xsl:version', 'other')", None),
    ] {
        let content = format!(r#"<xsl:if test="{condition}"><xsl:text>ok</xsl:text></xsl:if>"#);
        let program = compile(
            &sheet(&content, "2.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let actual = execute_compiled_stylesheet_for_test(
            &program,
            &document("urn:multi", b"<item/>"),
            "multi",
        );
        if let Some(expected) = expected {
            assert_eq!(actual.unwrap(), expected);
        } else {
            assert!(actual.unwrap_err().contains("XPST0017"));
        }
    }
    for condition in [
        "false() and system-property($missing, 'x')",
        "true() or system-property('xsl:vendor')",
        "false() and system-property(,)",
    ] {
        let content = format!(r#"<xsl:if test="{condition}"/>"#);
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(&content, "2.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("unproved argument behavior admitted")
        };
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }
}

#[test]
fn processor_policy_conditionals_preserve_control_and_failure_cleanup() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
    let program = compile(&sheet(r#"<xsl:choose unknown="ignored"><xsl:when unknown="ignored" test="flag"><xsl:if unknown="ignored" test="flag"><xsl:text>ok</xsl:text></xsl:if></xsl:when><xsl:otherwise unknown="ignored"><xsl:value-of select="system-property()"/></xsl:otherwise></xsl:choose>"#, "2.0"), Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let source = document("urn:control", b"<item><flag/></item>");
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        execute(&program, &source, "measure", &mut measured).unwrap(),
        "ok"
    );
    let charges = measured.consumed(WorkDomain::XPathNodeVisit);
    assert!(charges > 0);
    for allowance in [charges - 1, charges] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = allowance;
        let result = execute(
            &program,
            &source,
            "bounded",
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowance == charges {
            assert_eq!(result.unwrap(), "ok");
        } else {
            assert!(result.unwrap_err().contains("FXCT0002"));
        }
    }
    let failed_source = document("urn:failed-source", b"<item/>");
    for cancel in [false, true] {
        let run = || {
            let mut control = InvocationControl::unbounded();
            if cancel {
                control = control.cancelling_on_charge(WorkDomain::XPathOperation, 0);
            }
            let failure = execute(&program, &failed_source, "failed", &mut control).unwrap_err();
            assert!(
                failure.contains(if cancel { "FXCT0001" } else { "XPST0017" }),
                "{failure}"
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
fn processor_policy_numeric_arguments_convert_before_qname_validation() {
    for version in ["1.0", "2.0"] {
        for (function, code) in [
            ("system-property", "XTDE1390"),
            ("function-available", "XTDE1400"),
            ("element-available", "XTDE1440"),
        ] {
            for number in ["123", "-0", ".5", "-2."] {
                let content = format!(
                    r#"<xsl:if test="flag"><xsl:value-of select="{function}({number})"/></xsl:if><xsl:text>ok</xsl:text>"#
                );
                let program = compile(
                    &sheet(&content, version),
                    Edition::Legacy,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                for xml in ["<item/>", "<item><flag/></item>", "<item/>"] {
                    let result = execute_compiled_stylesheet_for_test(
                        &program,
                        &document("urn:numeric", xml.as_bytes()),
                        "numeric",
                    );
                    if xml.contains("flag") {
                        let failure = result.unwrap_err();
                        assert!(
                            failure.contains(code) && failure.contains("urn:conditional-policy"),
                            "{failure}"
                        );
                    } else {
                        assert_eq!(result.unwrap(), "ok");
                    }
                }
            }
        }
    }
    // Modern typed calls, dynamic XPath arguments and other properties remain
    // separate gates. Rust numeric spellings must not fabricate failures.
    for argument in ["NaN", "Infinity", "1e3", "+123", "$missing", "'xsl:vendor'"] {
        let content = format!(r#"<xsl:value-of select="system-property({argument})"/>"#);
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(&content, "2.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("unproved argument admitted")
        };
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }
    let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
        &sheet(r#"<xsl:value-of select="system-property(123)"/>"#, "2.0"),
        Edition::Modern,
        &mut InvocationControl::unbounded(),
    ) else {
        panic!("modern call semantics not yet admitted")
    };
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn processor_policy_numeric_qname_errors_short_circuit_without_hiding_arity_errors() {
    for version in ["1.0", "2.0"] {
        for (expression, expected) in [
            ("true() or system-property(123)", Some("true")),
            ("false() and system-property(123)", Some("false")),
            ("not(false() and system-property(123))", Some("true")),
            ("false() or system-property(123)", None),
            ("true() and system-property(123)", None),
        ] {
            let content = format!(r#"<xsl:value-of select="{expression}"/>"#);
            let program = compile(
                &sheet(&content, version),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let result = execute_compiled_stylesheet_for_test(
                &program,
                &document("urn:short", b"<item/>"),
                "short",
            );
            if let Some(expected) = expected {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert!(result.unwrap_err().contains("XTDE1390"));
            }
        }
    }
    // A dynamic QName error in one branch must not hide a static arity error.
    for condition in [
        "system-property(123) or system-property()",
        "true() or (system-property(123) and system-property())",
    ] {
        let content = format!(r#"<xsl:if test="{condition}"/>"#);
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet(&content, "1.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("static arity failure hidden")
        };
        assert_eq!(failure.code, "XPST0017");
    }
}
