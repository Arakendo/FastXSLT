//! Aggregate/format composition uses the existing charged sum and formatter.
use super::*;

fn sum_format_program() -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4096, 4096));
    resources.admit(STYLESHEET_ID, br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:value-of select="format-number(sum(r/v), '0.00')"/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    compile_resource(&resources.seal(), STYLESHEET_ID).unwrap()
}

fn document(xml: &str) -> Document {
    Document::from_parsed(parse_document(SOURCE_ID, xml.as_bytes(), XML_LIMITS).unwrap()).unwrap()
}

fn formatted_key_sum_program(
    expression: &str,
) -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
    let style = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:decimal-format name="d" decimal-separator="," grouping-separator="."/><xsl:key name="k" match="item" use="@group"/><xsl:template match="/"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#
    );
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4096, 4096));
    resources.admit(STYLESHEET_ID, style.into_bytes()).unwrap();
    compile_resource(&resources.seal(), STYLESHEET_ID).unwrap()
}

#[test]
fn xslt10_formatted_key_sum_composes_translation_and_decimal_formats() {
    for (expression, xml, expected) in [
        (
            "format-number(sum(key('k','a')/v), '0.00')",
            "<r><item group='a'><v>1.5</v><v>2.5</v></item></r>",
            "4.00",
        ),
        (
            "format-number(string(sum(key('k','a')/v)), '0,00', 'd')",
            "<r><item group='a'><v>1.5</v></item></r>",
            "1,50",
        ),
        (
            "format-number(translate(string(sum(key('k','a')/v)), 'NaN', 0), '0')",
            "<r><item group='a'><v>bad</v></item></r>",
            "0",
        ),
        (
            "format-number(translate(sum(key('k','a')/v), '112', '98'), '0')",
            "<r><item group='a'><v>12</v></item></r>",
            "9",
        ),
        (
            "format-number(translate(string(sum(key('k','a')/v)), 'NaN', 0), '0.00')",
            "<r/>",
            "0.00",
        ),
    ] {
        let program = formatted_key_sum_program(expression);
        let source = document(xml);
        for cached in [false, true] {
            let mut control = InvocationControl::unbounded();
            if !cached {
                control = control.without_key_index_cache();
            }
            let result = execute_program(&program, &source, "composed", &mut control).unwrap();
            assert_eq!(
                serialize_xml(
                    &result,
                    &program.output,
                    "composed",
                    4096,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn xslt10_formatted_key_sum_bounds_conversion_and_releases_failed_invocations() {
    let program = formatted_key_sum_program(
        "format-number(translate(string(sum(key('k','a')/v)), 'NaN', 0), '0.00')",
    );
    let source = document("<r><item group='a'><v>bad</v></item></r>");
    let mut observed = InvocationControl::unbounded();
    execute_program(&program, &source, "measure", &mut observed).unwrap();
    for domain in [WorkDomain::XPathOperation, WorkDomain::ResultTextByte] {
        let charges = observed.consumed(domain);
        assert!(charges > 0);
        for allowance in [charges - 1, charges] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XPathOperation => limits.xpath_operations = allowance,
                WorkDomain::ResultTextByte => limits.result_text_bytes = allowance,
                _ => unreachable!(),
            }
            let result = execute_program(
                &program,
                &source,
                "bounded",
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance == charges {
                assert!(result.is_ok());
            } else {
                assert_eq!(result.unwrap_err().code, "FXCT0002");
            }
        }
        let cancel = || {
            let mut control = InvocationControl::unbounded().cancelling_on_charge(domain, 0);
            assert_eq!(
                execute_program(&program, &source, "cancelled", &mut control)
                    .unwrap_err()
                    .code,
                "FXCT0001"
            );
        };
        cancel();
        #[cfg(feature = "allocation-observation")]
        {
            let measured = allocation_counter::measure(cancel);
            assert_eq!(measured.bytes_current, 0);
            assert_eq!(measured.count_current, 0);
        }
    }
    assert!(
        execute_program(
            &program,
            &source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .is_ok()
    );
}

#[test]
fn xslt10_formatted_key_sum_keeps_unproved_compositions_guarded_with_provenance() {
    for expression in [
        "format-number(sum(key('k','a')/v) div count(key('k','a')/v), '0')",
        "format-number(translate(string(sum(key('k','a')/v)), r/search, '0'), '0')",
    ] {
        let program = formatted_key_sum_program(expression);
        let source = document("<r><item group='a'><v>2</v></item></r>");
        let failure = execute_program(
            &program,
            &source,
            "guarded",
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(failure.code, "FXRT1007");
        assert_eq!(failure.category, FailureCategory::Unsupported);
        assert_eq!(failure.request_id.as_deref(), Some("guarded"));
        let location = failure.location.unwrap();
        assert_eq!(location.resource, STYLESHEET_ID);
        assert!(location.span.end > location.span.start);
    }
}

#[test]
fn xslt10_numeric_composition_does_not_admit_node_set_extensions() {
    // The corpus now reaches this boundary. Numeric composition must not turn
    // an unknown extension into an implicit result-tree-fragment conversion.
    for (prefix, namespace) in [
        ("msxsl", "urn:schemas-microsoft-com:xslt"),
        ("alias", "urn:schemas-microsoft-com:xslt"),
        ("msxsl", "urn:not-microsoft"),
    ] {
        for number in [
            format!("sum({prefix}:node-set($tree)/node())"),
            format!(
                "sum({prefix}:node-set($tree)/node()) div count({prefix}:node-set($tree)/node())"
            ),
        ] {
            let style = format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:{prefix}="{namespace}" version="1.0"><xsl:template match="/"><xsl:variable name="tree"><v>2</v><v>4</v></xsl:variable><xsl:text>before</xsl:text><xsl:value-of select="format-number(translate(string({number}), 'NaN', 0), '0')"/></xsl:template></xsl:stylesheet>"#
            );
            let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4096, 4096));
            resources
                .admit(STYLESHEET_ID, style.as_bytes().to_vec())
                .unwrap();
            let program = compile_resource(&resources.seal(), STYLESHEET_ID).unwrap();
            let source = document("<r/>");
            let run = || {
                let failure = execute_program(
                    &program,
                    &source,
                    "extension-guard",
                    &mut InvocationControl::unbounded(),
                )
                .expect_err("no partial result or silent extension admission");
                assert_eq!(failure.category, FailureCategory::Unsupported);
                assert_eq!(failure.code, "FXRT1007");
                assert_eq!(failure.request_id.as_deref(), Some("extension-guard"));
                let location = failure.location.unwrap();
                assert_eq!(location.resource, STYLESHEET_ID);
                assert!(style[location.span].starts_with("<xsl:value-of"));
            };
            run();
            run();
            #[cfg(feature = "allocation-observation")]
            {
                let measured = allocation_counter::measure(run);
                assert_eq!(measured.bytes_current, 0);
                assert_eq!(measured.count_current, 0);
            }
        }
    }
}

#[test]
fn xslt10_formatted_key_sum_distinguishes_numeric_values_from_coerced_strings() {
    let huge = "9".repeat(400);
    for (expression, value, expected) in [
        (
            "format-number(sum(key('k','a')/v), '0')",
            huge.as_str(),
            "Infinity",
        ),
        (
            "format-number(string(sum(key('k','a')/v)), '0')",
            huge.as_str(),
            "NaN",
        ),
        (
            "format-number(translate(string(sum(key('k','a')/v)), '0', 'e'), '0')",
            "102",
            "NaN",
        ),
        (
            "format-number(translate(string(sum(key('k','a')/v)), '1', '+'), '0')",
            "12",
            "NaN",
        ),
    ] {
        let program = formatted_key_sum_program(expression);
        let source = document(&format!("<r><item group='a'><v>{value}</v></item></r>"));
        let result = execute_program(
            &program,
            &source,
            "coercion",
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(
            serialize_xml(
                &result,
                &program.output,
                "coercion",
                4096,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
    }
}

fn key_sum_program() -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 4096, 4096));
    resources.admit(STYLESHEET_ID, br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:key name="k" match="item" use="@group"/><xsl:key name="k" match="item" use="@group"/><xsl:template match="/"><xsl:variable name="group" select="'a'"/><xsl:value-of select="sum(key('k', $group)/v)"/><xsl:text>|</xsl:text><xsl:value-of select="sum(key('k', 'missing')/v)"/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    compile_resource(&resources.seal(), STYLESHEET_ID).unwrap()
}

#[test]
fn xslt10_key_sum_uses_deduplicated_selection_and_double_conversion() {
    let program = key_sum_program();
    for (xml, expected) in [
        (
            "<r><item group='a'><v>1.25</v><v>2.75</v></item><item group='a'><v>-1</v></item><item group='b'><v>99</v></item></r>",
            "3|0",
        ),
        ("<r><item group='a'><v>bad</v></item></r>", "NaN|0"),
        ("<r/>", "0|0"),
    ] {
        let source = document(xml);
        for cached in [false, true] {
            let mut control = InvocationControl::unbounded();
            if !cached {
                control = control.without_key_index_cache();
            }
            let result = execute_program(&program, &source, "key-sum", &mut control).unwrap();
            assert_eq!(
                serialize_xml(
                    &result,
                    &program.output,
                    "key-sum",
                    4096,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn xslt10_key_sum_controls_release_partial_work_and_allow_reuse() {
    let program = key_sum_program();
    let source = document("<r><item group='a'><v>1</v><v>2</v></item></r>");
    for cached in [false, true] {
        let control = || {
            if cached {
                InvocationControl::unbounded()
            } else {
                InvocationControl::unbounded().without_key_index_cache()
            }
        };
        let mut observed = control();
        execute_program(&program, &source, "measure", &mut observed).unwrap();
        let charges = observed.consumed(WorkDomain::XPathOperation);
        assert!(charges >= 2);
        for allowance in [charges - 1, charges] {
            let mut limits = WorkLimits::unbounded();
            limits.xpath_operations = allowance;
            let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
            if !cached {
                bounded = bounded.without_key_index_cache();
            }
            let result = execute_program(&program, &source, "bounded", &mut bounded);
            if allowance == charges {
                assert!(result.is_ok());
            } else {
                assert_eq!(result.unwrap_err().code, "FXCT0002");
            }
        }
        let cancel = || {
            let mut cancelled =
                control().cancelling_on_charge(WorkDomain::XPathOperation, charges - 1);
            let failure =
                execute_program(&program, &source, "cancelled", &mut cancelled).unwrap_err();
            assert_eq!(failure.code, "FXCT0001");
            assert_eq!(failure.request_id.as_deref(), Some("cancelled"));
        };
        cancel();
        #[cfg(feature = "allocation-observation")]
        {
            let observed = allocation_counter::measure(cancel);
            assert_eq!(observed.bytes_current, 0);
            assert_eq!(observed.count_current, 0);
        }
        assert!(execute_program(&program, &source, "reuse", &mut control()).is_ok());
    }
}

#[test]
fn xslt10_sum_format_composes_empty_nan_finite_and_overflow_values() {
    let program = sum_format_program();
    let huge = "9".repeat(400);
    for (xml, expected) in [
        ("<r/>".to_owned(), "0.00"),
        (
            "<r><v>1.25</v><v>2.5</v><v>-0.75</v></r>".to_owned(),
            "3.00",
        ),
        ("<r><v>1</v><v>bad</v></r>".to_owned(), "NaN"),
        ("<r><v>1e2</v></r>".to_owned(), "NaN"),
        (format!("<r><v>{huge}</v></r>"), "Infinity"),
        (format!("<r><v>-{huge}</v></r>"), "-Infinity"),
        (format!("<r><v>{huge}</v><v>-{huge}</v></r>"), "NaN"),
    ] {
        let source = document(&xml);
        for _ in 0..2 {
            let result = execute_program(
                &program,
                &source,
                "sum-format",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(
                serialize_xml(
                    &result,
                    &program.output,
                    "sum-format",
                    4096,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn xslt10_sum_format_preserves_budget_cancellation_and_cleanup() {
    let program = sum_format_program();
    let source = document("<r><v>1</v><v>2</v><v>3</v></r>");
    let mut observed = InvocationControl::unbounded();
    execute_program(&program, &source, "measure", &mut observed).unwrap();
    let charges = observed.consumed(WorkDomain::XPathOperation);
    assert!(charges >= 3);
    for allowance in [charges - 1, charges] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = allowance;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        let result = execute_program(&program, &source, "bounded", &mut control);
        if allowance == charges {
            assert!(result.is_ok());
        } else {
            let failure = result.unwrap_err();
            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(failure.request_id.as_deref(), Some("bounded"));
        }
    }
    let run_cancel = || {
        let mut control = InvocationControl::unbounded()
            .cancelling_on_charge(WorkDomain::XPathOperation, charges - 1);
        assert_eq!(
            execute_program(&program, &source, "cancelled", &mut control)
                .unwrap_err()
                .code,
            "FXCT0001"
        );
    };
    run_cancel();
    #[cfg(feature = "allocation-observation")]
    {
        let measured = allocation_counter::measure(run_cancel);
        assert_eq!(measured.bytes_current, 0);
        assert_eq!(measured.count_current, 0);
    }
    assert!(
        execute_program(
            &program,
            &source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .is_ok()
    );
}
