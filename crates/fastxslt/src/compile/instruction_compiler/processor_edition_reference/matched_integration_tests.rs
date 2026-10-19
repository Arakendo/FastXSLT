use super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus; fragment evidence only"]
fn selected_text_replays_original_bvt033_instruction_without_case_credit() {
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
    let result_element = meaningful_children(&style, template)
        .into_iter()
        .find(|node| {
            style
                .name(*node)
                .is_some_and(|name| name.namespace.is_none() && name.local == "test16")
        })
        .unwrap();
    let text = meaningful_children(&style, result_element)
        .into_iter()
        .find(|node| is_xslt_element(&style, *node, "text"))
        .unwrap();
    let instruction = selected_text(&style, text, LocalVersion::LegacyForward).unwrap();
    let Instruction::Text {
        value,
        disable_output_escaping,
        location,
    } = instruction
    else {
        panic!("shared text instruction")
    };
    assert_eq!(value, "text");
    assert!(!disable_output_escaping);
    assert_eq!(location, *style.location(text));
    // The complete case remains blocked by other owners; this is neither a
    // rewritten fixture nor a replacement expected result or corpus pass.
    assert!(compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).is_err());
}

#[test]
fn selected_text_attributes_use_policy_without_changing_text_construction() {
    let source = document("urn:source", b"<r/>");
    for version in ["1.0", "1.5", "2.0", "3.0"] {
        for escaping in ["yes", "no", "maybe", "true"] {
            let style = document("urn:text-policy", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:text disable-output-escaping="{escaping}">&lt;tag/&gt;<!-- ignored -->&amp;</xsl:text></out></xsl:template></xsl:stylesheet>"#).as_bytes());
            let result = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
            if version == "1.0" && !matches!(escaping, "yes" | "no") {
                let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                    panic!("invalid normal text Boolean");
                };
                assert_eq!(failure.code, "XTSE0020");
                assert_eq!(failure.category, CompileCategory::Invalid);
                assert_eq!(failure.location.resource, "urn:text-policy");
            } else {
                let program = result.unwrap();
                drop(style);
                assert_eq!(
                    execute_compiled_stylesheet_for_test(&program, &source, "text").unwrap(),
                    if escaping == "yes" {
                        "<out><tag/>&</out>"
                    } else {
                        "<out>&lt;tag/&gt;&amp;</out>"
                    }
                );
            }
        }
        let style = document("urn:unknown-text", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="/"><xsl:text unknown="{{ $unbound">text</xsl:text></xsl:template></xsl:stylesheet>"#).as_bytes());
        let result = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
        if version == "1.0" {
            let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                panic!("unknown normal text attribute");
            };
            assert_eq!(failure.code, "XTSE0090");
        } else {
            assert_eq!(
                execute_compiled_stylesheet_for_test(&result.unwrap(), &source, "unknown-text")
                    .unwrap(),
                "text"
            );
        }
    }
}

#[test]
fn processor_policy_literal_binding_and_nested_fallback_space_are_shared() {
    let sheet = document("urn:binding-space", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:output method="xml" encoding="UTF-8" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="label" select="'outer'"/><out><xsl:new xml:space="preserve"> ignored <xsl:fallback xml:space="default"> <xsl:new><xsl:fallback xml:space="preserve"> <xsl:value-of select="$label"/> </xsl:fallback></xsl:new> </xsl:fallback><xsl:fallback><xsl:variable name="inner" select="'sibling'"/><xsl:value-of select="$inner"/></xsl:fallback></xsl:new><xsl:fallback xml:space="preserve">not-run</xsl:fallback><xsl:value-of select="$label"/></out></xsl:template></xsl:stylesheet>"#);
    let source = document("urn:binding-source", b"<root/>");
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    for id in ["first", "reuse"] {
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &source, id).unwrap(),
            "<out> outer siblingouter</out>"
        );
    }
}

#[test]
fn selected_text_preserves_local_scope_content_errors_and_unintegrated_guards() {
    for (root_version, local_version, succeeds) in [("1.0", "2.0", true), ("2.0", "1.0", false)] {
        let style = document("urn:text-scope", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{root_version}"><xsl:output method="text"/><xsl:template match="/"><out xsl:version="{local_version}"><xsl:text disable-output-escaping="maybe">text</xsl:text></out></xsl:template></xsl:stylesheet>"#).as_bytes());
        let result = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
        if succeeds {
            assert_eq!(
                execute_compiled_stylesheet_for_test(
                    &result.unwrap(),
                    &document("urn:source", b"<r/>"),
                    "local"
                )
                .unwrap(),
                "text"
            );
        } else {
            let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                panic!("local normal scope must reject invalid Boolean")
            };
            assert_eq!(failure.code, "XTSE0020");
        }
    }
    for (text, code) in [
        (
            r#"<xsl:text unknown="ignored"><child/></xsl:text>"#,
            "FXST0026",
        ),
        (
            r#"<xsl:text default-collation="urn:unknown">x</xsl:text>"#,
            "FXAR0031",
        ),
        (r#"<xsl:text xml:space="preserve">x</xsl:text>"#, "FXAR0031"),
        (r#"<xsl:text version="1.0">x</xsl:text>"#, "FXAR0031"),
    ] {
        let style = document("urn:text-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/">{text}</xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("text guard")
        };
        assert_eq!(failure.code, code);
        assert_eq!(failure.location.resource, "urn:text-guard");
        assert!(!failure.location.span.is_empty());
        assert!(compile(&style, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
    }
}

#[test]
fn selected_text_retains_shared_execution_charges_and_cleanup() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
    let source = document("urn:source", b"<r/>");
    for escaping in ["yes", "no"] {
        let stylesheet = |version: &str| {
            document("urn:text-charges", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:text disable-output-escaping="{escaping}">&lt;a/&gt;Ω</xsl:text></out></xsl:template></xsl:stylesheet>"#).as_bytes())
        };
        let selected = compile(
            &stylesheet("3.0"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let reference = compiler::compile_stylesheet(&stylesheet("1.0")).unwrap();
        let mut selected_control = InvocationControl::unbounded();
        let mut reference_control = InvocationControl::unbounded();
        let expected = execute(&reference, &source, "reference", &mut reference_control).unwrap();
        assert_eq!(
            execute(&selected, &source, "selected", &mut selected_control).unwrap(),
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
                selected_control.consumed(domain),
                reference_control.consumed(domain),
                "{domain:?}"
            );
        }
        let bytes = selected_control.consumed(WorkDomain::ResultTextByte);
        assert!(bytes > 0);
        for allowance in [bytes - 1, bytes] {
            let mut limits = WorkLimits::unbounded();
            limits.result_text_bytes = allowance;
            let run = || {
                let result = execute(
                    &selected,
                    &source,
                    "bounded",
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                );
                if allowance == bytes {
                    assert_eq!(result.unwrap(), expected);
                } else {
                    assert!(result.unwrap_err().contains("FXCT0002"));
                }
            };
            run();
            #[cfg(feature = "allocation-observation")]
            {
                let observed = allocation_counter::measure(run);
                assert_eq!(observed.bytes_current, 0);
                assert_eq!(observed.count_current, 0);
            }
        }
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0);
        assert!(
            execute(&selected, &source, "cancelled", &mut cancelled)
                .unwrap_err()
                .contains("FXCT0001")
        );
        assert_eq!(
            execute(
                &selected,
                &source,
                "reuse",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn processor_policy_literal_binding_and_space_keep_adjacent_guards() {
    for content in [
        r#"<xsl:variable name="v" select="'x'"><xsl:text>illegal</xsl:text></xsl:variable>"#,
        r#"<xsl:variable name="v" select="'x'"/><xsl:variable name="v" select="'y'"/>"#,
        r#"<xsl:new><xsl:fallback><xsl:variable name="inner" select="'x'"/></xsl:fallback><xsl:fallback><xsl:value-of select="$inner"/></xsl:fallback></xsl:new>"#,
        r#"<xsl:new><xsl:fallback xml:space="invalid">x</xsl:fallback></xsl:new>"#,
    ] {
        let sheet = document("urn:binding-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/">{content}</xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("invalid content admitted")
        };
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.location.resource, "urn:binding-guard");
    }
}

#[test]
fn processor_policy_direct_call_arity_has_selected_error_timing() {
    for expression in ["system-property()", "system-property('a', 'b')"] {
        for (edition, version, deferred) in [
            (Edition::Legacy, "2.0", true),
            (Edition::Legacy, "1.0", false),
            (Edition::Modern, "2.0", false),
        ] {
            let sheet = document("urn:call-arity", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:template match="item"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
            let compiled = compile(&sheet, edition, &mut InvocationControl::unbounded());
            if deferred {
                let program = compiled.unwrap();
                let dormant = document("urn:call-dormant", b"<other/>");
                let reached = document("urn:call-reached", b"<item/>");
                assert!(
                    execute_compiled_stylesheet_for_test(&program, &dormant, "dormant").is_ok()
                );
                let failure = execute_compiled_stylesheet_for_test(&program, &reached, "reached")
                    .unwrap_err();
                assert!(
                    failure.contains("XPST0017") && failure.contains("urn:call-arity"),
                    "{failure}"
                );
                assert!(execute_compiled_stylesheet_for_test(&program, &dormant, "reuse").is_ok());
            } else {
                let Err(StrictPreprocessingFailure::Compile(failure)) = compiled else {
                    panic!("arity must remain static")
                };
                assert_eq!(failure.code, "XPST0017");
                assert_eq!(failure.category, CompileCategory::Invalid);
            }
        }
    }
}

#[test]
fn processor_policy_direct_call_does_not_admit_nested_or_general_call_semantics() {
    for expression in [
        "false() and unknown-function()",
        "true() or system-property('xsl:version')",
        "system-property($missing, 'x')",
        "system-property('xsl:version')",
        "unknown-function()",
        "system-property(,)",
        "false() and system-property($missing, 'x')",
        "false() and (system-property()",
        "true() or system-property())",
        "false() and system-property(,)",
        "false() and not()",
        "false() and p:system-property()",
        "false() and flag[system-property()]",
    ] {
        let sheet = document("urn:call-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="never"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("unproved call shape admitted: {expression}")
        };
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }
}

#[test]
fn processor_policy_nested_calls_use_shared_short_circuit_evaluation() {
    for (expression, expected) in [
        ("false() and system-property()", Some("false")),
        ("true() or system-property()", Some("true")),
        ("not(false() and system-property())", Some("true")),
        ("boolean(true() or system-property())", Some("true")),
        (
            "false() and true() or true() or system-property()",
            Some("true"),
        ),
        ("true() and system-property()", None),
        ("false() or system-property()", None),
        ("system-property() and false()", None),
        ("system-property() or true()", None),
        ("not(system-property())", None),
        ("boolean(system-property())", None),
        ("true() and (false() or system-property())", None),
    ] {
        for (edition, version, deferred) in [
            (Edition::Legacy, "2.0", true),
            (Edition::Legacy, "1.0", false),
            (Edition::Modern, "2.0", false),
        ] {
            let sheet = document("urn:nested-call", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="item"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
            let compiled = compile(&sheet, edition, &mut InvocationControl::unbounded());
            if !deferred {
                let Err(StrictPreprocessingFailure::Compile(failure)) = compiled else {
                    panic!("must remain static: {expression}")
                };
                assert_eq!(failure.code, "XPST0017");
                continue;
            }
            let program = compiled.unwrap();
            let dormant = document("urn:dormant", b"<other/>");
            let reached = document("urn:reached", b"<item/>");
            assert_eq!(
                execute_compiled_stylesheet_for_test(&program, &dormant, "dormant").unwrap(),
                ""
            );
            let result = execute_compiled_stylesheet_for_test(&program, &reached, "reached");
            if let Some(expected) = expected {
                assert_eq!(result.unwrap(), expected, "{expression}");
            } else {
                let failure = result.unwrap_err();
                assert!(
                    failure.contains("XPST0017") && failure.contains("urn:nested-call"),
                    "{failure}"
                );
            }
            assert!(execute_compiled_stylesheet_for_test(&program, &dormant, "reuse").is_ok());
        }
    }
}

#[test]
fn processor_policy_nested_call_reachability_depends_on_each_source() {
    for (expression, skip_source, fail_source, expected) in [
        (
            "flag and system-property()",
            "<item/>",
            "<item><flag/></item>",
            "false",
        ),
        (
            "flag or system-property()",
            "<item><flag/></item>",
            "<item/>",
            "true",
        ),
    ] {
        let sheet = document("urn:dynamic-call", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:output method="text"/><xsl:template match="item"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        for _ in 0..2 {
            assert_eq!(
                execute_compiled_stylesheet_for_test(
                    &program,
                    &document("urn:skip", skip_source.as_bytes()),
                    "skip"
                )
                .unwrap(),
                expected
            );
            let failure = execute_compiled_stylesheet_for_test(
                &program,
                &document("urn:fail", fail_source.as_bytes()),
                "fail",
            )
            .unwrap_err();
            assert!(
                failure.contains("XPST0017") && failure.contains("urn:dynamic-call"),
                "{failure}"
            );
        }
    }
}

#[test]
fn processor_policy_boolean_call_admission_is_bounded() {
    let location = SourceLocation {
        resource: "urn:bounded-call".to_owned(),
        span: 0..1,
    };
    for expression in [
        format!("{}system-property(){}", "not(".repeat(32), ")".repeat(32)),
        format!("{}false() and system-property()", " ".repeat(4096)),
        "false() and unknown-function()".to_owned(),
        "true() or system-property('xsl:version')".to_owned(),
    ] {
        assert!(
            boolean_call_timing::compile(&expression, LocalVersion::LegacyForward, &location, &[])
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn processor_policy_boolean_calls_keep_unbound_reads_static() {
    for expression in [
        "false() and ($missing or system-property())",
        "true() or ($missing and system-property())",
        "system-property() and $missing",
        "boolean($missing) or system-property()",
        "not($missing) and system-property()",
    ] {
        for (edition, version) in [
            (Edition::Legacy, "2.0"),
            (Edition::Legacy, "1.0"),
            (Edition::Modern, "2.0"),
        ] {
            let sheet = document("urn:static-variable-call", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:template match="never"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
            let Err(StrictPreprocessingFailure::Compile(failure)) =
                compile(&sheet, edition, &mut InvocationControl::unbounded())
            else {
                panic!("unbound read admitted: {expression}")
            };
            assert_eq!(failure.code, "XPST0008");
            assert_eq!(failure.category, CompileCategory::Invalid);
            assert_eq!(failure.location.resource, "urn:static-variable-call");
        }
    }
}

#[test]
fn processor_policy_boolean_calls_reuse_typed_local_values() {
    for (binding, expression, expected) in [
        (r#"select="''""#, "$v and system-property()", Some("false")),
        (r#"select="'0'""#, "$v or system-property()", Some("true")),
        (
            r#"select="'false'""#,
            "boolean($v) or system-property()",
            Some("true"),
        ),
        (r#"select="'0'""#, "$v and system-property()", None),
        (r#"select="''""#, "not($v) and system-property()", None),
    ] {
        let sheet = document("urn:typed-variable-call", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:output method="text"/><xsl:template match="item"><xsl:variable name="v" {binding}/><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        for request in ["first", "reuse"] {
            let result = execute_compiled_stylesheet_for_test(
                &program,
                &document("urn:typed-source", b"<item/>"),
                request,
            );
            if let Some(expected) = expected {
                assert_eq!(result.unwrap(), expected);
            } else {
                let failure = result.unwrap_err();
                assert!(
                    failure.contains("XPST0017") && failure.contains("urn:typed-variable-call"),
                    "{failure}"
                );
            }
        }
    }
}

#[test]
fn processor_policy_boolean_call_bindings_do_not_escape_lexical_scope() {
    for content in [
        r#"<xsl:value-of select="false() and ($later or system-property())"/><xsl:variable name="later" select="'x'"/>"#,
        r#"<xsl:future><xsl:fallback><xsl:variable name="inner" select="'x'"/></xsl:fallback><xsl:fallback><xsl:value-of select="true() or ($inner and system-property())"/></xsl:fallback></xsl:future>"#,
    ] {
        let sheet = document("urn:scoped-call", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="never">{content}</xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("out-of-scope read admitted")
        };
        assert_eq!(failure.code, "XPST0008");
    }
    let sheet = document("urn:inherited-call", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:output method="text"/><xsl:template match="/"><xsl:variable name="outer" select="'x'"/><xsl:future><xsl:fallback><xsl:value-of select="$outer or system-property()"/></xsl:fallback></xsl:future><xsl:value-of select="$outer or system-property()"/></xsl:template></xsl:stylesheet>"#);
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    assert_eq!(
        execute_compiled_stylesheet_for_test(
            &program,
            &document("urn:inherited-source", b"<r/>"),
            "inherited"
        )
        .unwrap(),
        "truetrue"
    );
}

#[test]
fn processor_policy_path_syntax_failure_defers_only_in_legacy_forward_context() {
    for expression in ["", "...", "item/1invalid"] {
        let sheet = document("urn:deferred-path", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let dormant = document("urn:dormant-path", b"<other/>");
        let reached = document("urn:reached-path", b"<item/>");
        assert!(execute_compiled_stylesheet_for_test(&program, &dormant, "dormant").is_ok());
        let failure =
            execute_compiled_stylesheet_for_test(&program, &reached, "reached").unwrap_err();
        assert!(
            failure.contains("XPST0003") && failure.contains("urn:deferred-path"),
            "{failure}"
        );
        assert!(execute_compiled_stylesheet_for_test(&program, &dormant, "reuse").is_ok());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded())
        else {
            panic!("modern 2.0 must retain static syntax failure")
        };
        assert_eq!(failure.code, "XPST0003");
    }
}

#[test]
fn processor_policy_path_deferral_does_not_reclassify_unsupported_expressions() {
    for expression in [".5", "12", "1 + 2", "sum(item)", "item///name", "$missing"] {
        let sheet = document("urn:expression-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="never"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("must not defer {expression}")
        };
        if expression == "$missing" {
            assert_eq!(failure.code, "XPST0008");
        } else {
            assert_eq!(failure.category, CompileCategory::Unsupported);
        }
    }
}

#[test]
fn processor_policy_computed_container_version_restores_nested_and_sibling_contexts() {
    let sheet = document("urn:container-version", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="item"><xsl:element name="old" version="1.0"><xsl:value-of select="text()"/><xsl:element name="new" version="3.0"><xsl:value-of select="text()"/></xsl:element><xsl:value-of select="text()"/></xsl:element><xsl:element name="sibling"><xsl:value-of select="text()"/></xsl:element></xsl:template></xsl:stylesheet>"#);
    let source = document("urn:container-source", b"<item>A<!--split-->B</item>");
    let program = compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).unwrap();
    let actual =
        execute_compiled_stylesheet_for_test(&program, &source, "container-version").unwrap();
    assert!(
        actual.ends_with("<old>A<new>A B</new>A</old><sibling>A B</sibling>"),
        "{actual}"
    );
    assert!(compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).is_err());
}

#[test]
fn processor_policy_computed_container_version_keeps_unproved_constructor_guards() {
    for attributes in [
        r#"name="{name()}" version="1.0""#,
        r#"name="out" version="1.0" namespace="urn:override""#,
        r#"name="out" xsl:version="1.0""#,
        r#"name="out" version="1.0" use-attribute-sets="a""#,
    ] {
        let sheet = document("urn:container-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="item"><xsl:element {attributes}><xsl:value-of select="text()"/></xsl:element></xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded())
        else {
            panic!("unproved constructor admitted")
        };
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.location.resource, "urn:container-guard");
    }
}

#[test]
fn processor_policy_value_of_local_version_applies_to_self_and_restores_siblings() {
    let source = document(
        "urn:instruction-version-source",
        b"<item>A<!--split-->B</item>",
    );
    for (root, local, expected) in [
        ("3.0", "1.0", "<out>AA B</out>"),
        ("1.0", "3.0", "<out>A BA</out>"),
    ] {
        let sheet = document("urn:instruction-version", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{root}"><xsl:template match="item"><out><xsl:value-of version="{local}" select="text()"/><xsl:value-of select="text()"/></out></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).unwrap();
        let actual =
            execute_compiled_stylesheet_for_test(&program, &source, "instruction-version").unwrap();
        assert!(actual.ends_with(expected), "{actual}");
        assert_eq!(
            program.processor,
            crate::xslt::golden_semantics_experiment::ProcessorPolicyStamp::ModernReference
        );
        assert!(compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).is_err());
    }
}

#[test]
fn processor_policy_instruction_version_does_not_bypass_attribute_guards() {
    for attributes in [
        r#"xsl:version="1.0""#,
        r#"version="1.0" xsl:version="3.0""#,
        r#"version="1.0" unexpected="yes""#,
        r#"version="1.0" separator="|""#,
        r#"version="1.0" disable-output-escaping="yes""#,
        r#"version="1e0""#,
    ] {
        let sheet = document("urn:instruction-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="item"><xsl:value-of {attributes} select="text()"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded())
        else {
            panic!("unpropagated attributes admitted: {attributes}")
        };
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.location.resource, "urn:instruction-guard");
    }
}

#[test]
fn processor_policy_modern_literal_version_controls_conversion_not_processor() {
    let sheet = document("urn:modern-local", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="item"><out><xsl:value-of select="text()"/><old xsl:version="1.0"><xsl:value-of select="text()"/><new xsl:version="3.0"><xsl:element name="computed"><xsl:value-of select="text()"/></xsl:element></new><xsl:value-of select="text()"/></old><xsl:value-of select="text()"/></out></xsl:template></xsl:stylesheet>"#);
    let source = document("urn:modern-local-source", b"<item>A<!--split-->B</item>");
    let program = compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).unwrap();
    assert_eq!(
        program.processor,
        crate::xslt::golden_semantics_experiment::ProcessorPolicyStamp::ModernReference
    );
    let actual = execute_compiled_stylesheet_for_test(&program, &source, "modern-local").unwrap();
    assert!(
        actual.ends_with("<out>A B<old>A<new><computed>A B</computed></new>A</old>A B</out>"),
        "{actual}"
    );
    let legacy = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let actual = execute_compiled_stylesheet_for_test(&legacy, &source, "legacy-control").unwrap();
    assert!(
        actual.ends_with("<out>A<old>A<new><computed>A</computed></new>A</old>A</out>"),
        "{actual}"
    );
}

#[test]
fn processor_policy_modern_root_one_inherits_compatibility_until_local_override() {
    let sheet = document("urn:modern-root-one", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="item"><out><xsl:value-of select="text()"/><new xsl:version="2.0"><xsl:value-of select="text()"/></new><xsl:value-of select="text()"/></out></xsl:template></xsl:stylesheet>"#);
    let source = document("urn:modern-root-one-source", b"<item>A<!--split-->B</item>");
    let program = compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).unwrap();
    let actual = execute_compiled_stylesheet_for_test(&program, &source, "root-one").unwrap();
    assert!(actual.ends_with("<out>A<new>A B</new>A</out>"), "{actual}");
}

#[test]
fn processor_policy_numeric_root_validation_does_not_change_baseline() {
    let source = document("urn:root-number-source", b"<item/>");
    for lexical in ["0", "0.0", ".5", "1.1", "5"] {
        let sheet = document("urn:root-number", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{lexical}"><xsl:template match="item"><xsl:future><xsl:fallback>ok</xsl:fallback></xsl:future></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        assert!(
            execute_compiled_stylesheet_for_test(&program, &source, "root-number")
                .unwrap()
                .ends_with("ok")
        );
    }
    for (lexical, accepted) in [("0", false), ("+1.0", true)] {
        let sheet = document("urn:baseline-number", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{lexical}"><xsl:template match="item"><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet>"#).as_bytes());
        let baseline =
            compiler::compile_stylesheet_at(&sheet, compiler::document_element(&sheet).unwrap());
        assert_eq!(baseline.is_ok(), accepted);
        if let Err(failure) = baseline {
            assert_eq!(failure.code, "XTSE0110");
        }
    }
}

#[test]
fn processor_policy_legacy_numeric_witnesses_preserve_root_and_local_scope() {
    let source = document("urn:numeric-source", b"<item/>");
    for one in ["1", "1.", "01.00", "0001.000", "1.00000000000000001"] {
        for future in [
            "2",
            "02.00",
            "3.",
            "0004.000",
            "0",
            ".5",
            "1.1",
            "5",
            "999999999999999999999999999999999999",
        ] {
            let sheet = document("urn:numeric-version", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{one}"><xsl:template match="item"><out xsl:version="{future}"><xsl:future><xsl:fallback>ok</xsl:fallback></xsl:future></out></xsl:template></xsl:stylesheet>"#).as_bytes());
            let program =
                compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
            assert!(
                execute_compiled_stylesheet_for_test(&program, &source, "numeric")
                    .unwrap()
                    .ends_with("<out>ok</out>")
            );
            // Swapping scope makes the unknown instruction an error; the
            // selected processor does not change with either spelling.
            let strict = document("urn:numeric-strict", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{future}"><xsl:template match="item"><out xsl:version="{one}"><xsl:future><xsl:fallback>wrong</xsl:fallback></xsl:future></out></xsl:template></xsl:stylesheet>"#).as_bytes());
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &strict,
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("strict local context must reject unknown instruction")
            };
            assert_eq!(failure.category, CompileCategory::Invalid);
        }
    }
}

#[test]
fn processor_policy_legacy_numeric_nonwitnesses_remain_explicitly_guarded() {
    for lexical in ["", ".", "+1.0", "-1.0", "1e0", "1..0", " 1.0 ", "１.0"] {
        for local in [false, true] {
            let (root_version, local_attribute) = if local {
                ("1.0", format!(r#" xsl:version="{lexical}""#))
            } else {
                (lexical, String::new())
            };
            let sheet = document("urn:numeric-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{root_version}"><xsl:template match="item"><out{local_attribute}>ok</out></xsl:template></xsl:stylesheet>"#).as_bytes());
            let Err(StrictPreprocessingFailure::Compile(failure)) =
                compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
            else {
                panic!("unguarded lexical form: {lexical:?}")
            };
            assert_eq!(failure.category, CompileCategory::Unsupported);
            assert_eq!(failure.location.resource, "urn:numeric-guard");
        }
    }
}

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

#[test]
fn processor_policy_matched_body_preserves_dispatch_and_policy() {
    let sheet = document("urn:matched", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item"><xsl:future><xsl:fallback><result><xsl:value-of select="text()"/></result></xsl:fallback></xsl:future></xsl:template><xsl:template match="text()"/></xsl:stylesheet>"#);
    let source = document(
        "urn:source",
        b"<root>skip<item>A<!--split-->not-first</item><item>B</item></root>",
    );
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    assert!(program.root_template.is_none());
    assert_eq!(program.matched_templates.len(), 2);
    let result = execute_compiled_stylesheet_for_test(&program, &source, "matched").unwrap();
    assert!(
        result.ends_with("<result>A</result><result>B</result>"),
        "{result}"
    );
    assert!(compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
    assert!(compiler::compile_stylesheet(&sheet).is_err());
}

#[test]
fn processor_policy_computed_body_preserves_nested_fallback_and_path_mode() {
    let sheet = document("urn:computed", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item[@kind='chosen']"><xsl:future><xsl:fallback><xsl:element name="result"><xsl:future><xsl:fallback><xsl:value-of select="title/text()"/></xsl:fallback></xsl:future></xsl:element></xsl:fallback><xsl:fallback><xsl:text>!</xsl:text></xsl:fallback></xsl:future></xsl:template><xsl:template match="text()"/></xsl:stylesheet>"#);
    let source = document("urn:computed-source", b"<root><item kind='other'><title>skip</title></item><item kind='chosen'><title>A<!--split-->not-first</title><title>also-not-first</title></item></root>");
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let actual = execute_compiled_stylesheet_for_test(&program, &source, "computed").unwrap();
    assert!(actual.ends_with("<result>A</result>!"), "{actual}");
    assert!(compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
}

#[test]
fn processor_policy_matched_integration_refuses_unpropagated_routes() {
    for declarations in [
        r#"<xsl:template name="n"><xsl:text>x</xsl:text></xsl:template>"#,
        r#"<xsl:template match="item"><xsl:param name="p"/></xsl:template>"#,
        r#"<xsl:template match="item[position()=2]"><xsl:text>x</xsl:text></xsl:template>"#,
        r#"<xsl:template match="item"><xsl:value-of select="system-property('xsl:version')"/></xsl:template>"#,
        r#"<xsl:template match="item"><result value="{text()}"/></xsl:template>"#,
        r#"<xsl:template match="item"><result xsl:version="1e0"/></xsl:template>"#,
        r#"<xsl:template match="document-node()"/>"#,
        r#"<xsl:template match="item"><xsl:element name="{name()}"/></xsl:template>"#,
        r#"<xsl:template match="item"><xsl:element name="result" namespace="urn:dynamic"/></xsl:template>"#,
        r#"<xsl:template match="item"><xsl:element name="result"><xsl:attribute name="a">v</xsl:attribute></xsl:element></xsl:template>"#,
        r#"<xsl:template match="item"><xsl:element name="result"><xsl:value-of select="system-property('xsl:version')"/></xsl:element></xsl:template>"#,
        r#"<xsl:include href="urn:dependency"/>"#,
        r#"<xsl:variable name="g" select="1"/>"#,
    ] {
        let sheet = document("urn:guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">{declarations}</xsl:stylesheet>"#).as_bytes());
        assert!(matches!(
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()),
            Err(StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Unsupported,
                ..
            }))
        ));
    }
}

#[test]
fn processor_policy_text_variable_retains_scope_through_fallback() {
    let sheet = document("urn:local-variable", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item"><xsl:variable name="v">A<xsl:text>B</xsl:text></xsl:variable><xsl:future><xsl:fallback><xsl:variable name="inner">C</xsl:variable><result><xsl:value-of select="$v"/><xsl:element name="inner"><xsl:value-of select="$inner"/></xsl:element></result></xsl:fallback><xsl:fallback><xsl:variable name="inner">D</xsl:variable><xsl:value-of select="$inner"/></xsl:fallback></xsl:future><xsl:value-of select="$v"/></xsl:template></xsl:stylesheet>"#);
    let source = document("urn:local-source", b"<root><item/><item/></root>");
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    for _ in 0..2 {
        let actual = execute_compiled_stylesheet_for_test(&program, &source, "locals").unwrap();
        assert!(
            actual.ends_with(
                "<result>AB<inner>C</inner></result>DAB<result>AB<inner>C</inner></result>DAB"
            ),
            "{actual}"
        );
    }
}

#[test]
fn processor_policy_text_variable_guards_scope_and_constructor_limits() {
    for content in [
        r#"<xsl:variable name="v" select="1"/>"#,
        r#"<xsl:variable name="v"><xsl:value-of select="text()"/></xsl:variable>"#,
    ] {
        let sheet = document("urn:constructor-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item">{content}</xsl:template></xsl:stylesheet>"#).as_bytes());
        assert!(matches!(
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()),
            Err(StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Unsupported,
                ..
            }))
        ));
    }
    for content in [
        r#"<xsl:value-of select="$missing"/>"#,
        r#"<xsl:value-of select="$v"/><xsl:variable name="v">late</xsl:variable>"#,
        r#"<xsl:future><xsl:fallback><xsl:variable name="inner">x</xsl:variable></xsl:fallback><xsl:fallback><xsl:value-of select="$inner"/></xsl:fallback></xsl:future>"#,
        r#"<result><xsl:variable name="inner">x</xsl:variable></result><xsl:value-of select="$inner"/>"#,
    ] {
        let sheet = document("urn:local-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item">{content}</xsl:template></xsl:stylesheet>"#).as_bytes());
        assert!(matches!(
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()),
            Err(StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Invalid,
                code: "XPST0008",
                ..
            }))
        ));
    }
    let sheet = document("urn:duplicate", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item"><xsl:variable name="v">one</xsl:variable><xsl:variable name="v">two</xsl:variable></xsl:template></xsl:stylesheet>"#);
    let Err(StrictPreprocessingFailure::Compile(failure)) =
        compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
    else {
        panic!("duplicate must fail")
    };
    assert_eq!(failure.code, "FXST0017");
}

#[test]
fn processor_policy_literal_version_scope_restores_siblings_and_processor() {
    let sheet = document("urn:local-version", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item"><outer xsl:version="1.0"><inner xsl:version="2.0"><xsl:sequence><xsl:fallback><xsl:value-of select="text()"/></xsl:fallback></xsl:sequence></inner><xsl:text>strict-sibling</xsl:text></outer><xsl:future><xsl:fallback><xsl:text>future-sibling</xsl:text></xsl:fallback></xsl:future></xsl:template></xsl:stylesheet>"#);
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    assert_eq!(
        program.processor,
        crate::xslt::golden_semantics_experiment::ProcessorPolicyStamp::Xslt10Reference
    );
    let source = document(
        "urn:local-version-source",
        b"<item>A<!--split-->not-first</item>",
    );
    let actual = execute_compiled_stylesheet_for_test(&program, &source, "scope").unwrap();
    assert!(
        actual.ends_with("<outer><inner>A</inner>strict-sibling</outer>future-sibling"),
        "{actual}"
    );
    assert!(!actual.contains("xsl:version="));
    assert!(compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
}

#[test]
fn processor_policy_literal_version_can_enable_and_disable_forward_processing() {
    let enabled = document("urn:enabled", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="item"><result xsl:version="2.0"><xsl:future><xsl:fallback>ok</xsl:fallback></xsl:future></result></xsl:template></xsl:stylesheet>"#);
    let program = compile(
        &enabled,
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document("urn:enabled-source", b"<item/>");
    assert!(
        execute_compiled_stylesheet_for_test(&program, &source, "enabled")
            .unwrap()
            .ends_with("<result>ok</result>")
    );
    for content in [
        r#"<result xsl:version="1.0"><xsl:future><xsl:fallback>wrong</xsl:fallback></xsl:future></result>"#,
        r#"<outer xsl:version="1.0"><inner xsl:version="2.0"><xsl:future><xsl:fallback>ok</xsl:fallback></xsl:future></inner><xsl:future><xsl:fallback>wrong</xsl:fallback></xsl:future></outer>"#,
    ] {
        let sheet = document("urn:disabled", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item">{content}</xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("strict local context must reject unknown instruction")
        };
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.location.resource, "urn:disabled");
    }
}

#[test]
fn processor_policy_unbound_read_is_not_forward_compatibility_deferral() {
    for version in ["1.0", "2.0"] {
        let sheet = document("urn:unbound", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:template match="never-selected"><xsl:value-of select="$missing"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        for edition in [Edition::Legacy, Edition::Modern] {
            let Err(StrictPreprocessingFailure::Compile(failure)) =
                compile(&sheet, edition, &mut InvocationControl::unbounded())
            else {
                panic!("unbound read must fail compilation")
            };
            assert_eq!(failure.code, "XPST0008");
            assert_eq!(failure.category, CompileCategory::Invalid);
            assert_eq!(failure.location.resource, "urn:unbound");
            assert!(!failure.location.span.is_empty());
        }
    }
}

#[test]
fn processor_policy_top_level_fallback_uses_selected_legacy_version_rules() {
    for version in ["1.0", "2.0"] {
        let sheet = document("urn:placement", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:fallback><xsl:value-of select="$missing"/></xsl:fallback><xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet>"#).as_bytes());
        let compiled = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded());
        if version == "1.0" {
            let Err(StrictPreprocessingFailure::Compile(failure)) = compiled else {
                panic!("invalid placement")
            };
            assert_eq!(failure.code, "FXST1002");
            assert_eq!(failure.category, CompileCategory::Invalid);
            assert_eq!(failure.location.resource, "urn:placement");
        } else {
            let source = document("urn:placement-source", b"<r/>");
            let actual = execute_compiled_stylesheet_for_test(
                &compiled.unwrap(),
                &source,
                "ignored-top-level",
            )
            .unwrap();
            assert!(actual.ends_with("ok"), "{actual}");
        }
        // No assumption about the modern top-level compiler is smuggled in.
        assert!(compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
    }
}

#[test]
fn processor_policy_standalone_fallback_does_not_execute_or_bind_content() {
    for version in ["1.0", "2.0"] {
        let sheet = document("urn:standalone", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:template match="item"><xsl:fallback><xsl:text>not-output</xsl:text><xsl:variable name="v">not-bound</xsl:variable></xsl:fallback><xsl:variable name="v">ok</xsl:variable><xsl:value-of select="$v"/></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let source = document("urn:standalone-source", b"<item/>");
        let actual = execute_compiled_stylesheet_for_test(&program, &source, "standalone").unwrap();
        assert!(actual.ends_with("ok"), "{actual}");
        assert!(!actual.contains("not-output"));
    }
}

#[test]
fn processor_policy_missing_direct_fallback_fails_only_when_template_reached() {
    let sheet = document("urn:missing-fallback", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item"><xsl:future/><xsl:fallback>not-a-child</xsl:fallback></xsl:template></xsl:stylesheet>"#);
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let dormant = document("urn:dormant", b"<other/>");
    assert!(execute_compiled_stylesheet_for_test(&program, &dormant, "dormant").is_ok());
    let reached = document("urn:reached", b"<item/>");
    let failure = execute_compiled_stylesheet_for_test(&program, &reached, "reached").unwrap_err();
    assert!(failure.contains("XTDE1450"), "{failure}");
    assert!(failure.contains("urn:missing-fallback"), "{failure}");
    assert!(execute_compiled_stylesheet_for_test(&program, &dormant, "fresh").is_ok());
    assert!(compile(&sheet, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
}

#[test]
fn processor_policy_matched_union_retains_policy_and_cancellation() {
    use crate::execution_control_experiment::WorkDomain;
    let sheet = document("urn:union", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item|other"><xsl:future><xsl:fallback><xsl:value-of select="text()"/></xsl:fallback></xsl:future></xsl:template><xsl:template match="text()"/></xsl:stylesheet>"#);
    let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
    assert!(matches!(
        compile(&sheet, Edition::Legacy, &mut cancelled),
        Err(StrictPreprocessingFailure::Control(_))
    ));
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let source = document(
        "urn:union-source",
        b"<root><item>A</item><other>B</other></root>",
    );
    assert!(
        execute_compiled_stylesheet_for_test(&program, &source, "union")
            .unwrap()
            .ends_with("AB")
    );
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn processor_policy_replays_original_bvt_fallback() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let sheet = document(
        "urn:oasis:bvt023",
        &std::fs::read(root.join("MSFT_Conformance_Tests/BVTs/fallback.xsl")).unwrap(),
    );
    let source = document(
        "urn:oasis:bvt-source",
        &std::fs::read(root.join("MSFT_Conformance_Tests/BVTs/data.xml")).unwrap(),
    );
    let expected =
        std::fs::read(root.join("MSFT_Conformance_Tests/REF_OUT/BVTs/bvt023.txt")).unwrap();
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let actual = execute_compiled_stylesheet_for_test(&program, &source, "bvt023").unwrap();
    assert_eq!(
        crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
            &actual,
            &expected,
            false,
            None,
            Some("xml")
        ),
        Ok(true),
        "{actual}"
    );
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn processor_policy_replays_original_matched_templates() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    for case in [
        "91817", "91818", "91819", "91820", "91821", "91823", "91824",
    ] {
        let sheet = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("MSFT_Conformance_Tests/Fallback/{case}.xsl")))
                .unwrap(),
        );
        let source = document(
            "urn:oasis:books",
            &std::fs::read(root.join("MSFT_Conformance_Tests/Fallback/books.xml")).unwrap(),
        );
        let expected = std::fs::read(root.join(format!(
            "MSFT_Conformance_Tests/REF_OUT/Fallback/{case}.txt"
        )))
        .unwrap();
        let program =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let actual = execute_compiled_stylesheet_for_test(&program, &source, case).unwrap();
        assert_eq!(
            crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                &actual,
                &expected,
                false,
                None,
                Some("xml"),
            ),
            Ok(true),
            "unchanged corpus comparison: {case}"
        );
    }
    // Preserve actual compilation failure, not the catalog's coarse label.
    for (case, code) in [("91822", "XPST0008"), ("91826", "FXST1002")] {
        let sheet = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("MSFT_Conformance_Tests/Fallback/{case}.xsl")))
                .unwrap(),
        );
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("expected compilation failure for {case}")
        };
        assert_eq!(failure.code, code);
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.location.resource, format!("urn:oasis:{case}"));
    }
    let sheet = document(
        "urn:oasis:91825",
        &std::fs::read(root.join("MSFT_Conformance_Tests/Fallback/91825.xsl")).unwrap(),
    );
    let source = document(
        "urn:oasis:books",
        &std::fs::read(root.join("MSFT_Conformance_Tests/Fallback/books.xml")).unwrap(),
    );
    let program = compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let failure = execute_compiled_stylesheet_for_test(&program, &source, "91825").unwrap_err();
    assert!(failure.contains("XTDE1450"), "{failure}");
    assert!(failure.contains("urn:oasis:91825"), "{failure}");
}
