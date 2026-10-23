use super::super::named_tests::{document, sheet};
use super::super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

const RECURSIVE: &str = r#"<xsl:template match="/doc/*"><xsl:call-template name="n"><xsl:with-param name="stop"><xsl:value-of select="@repeat"/></xsl:with-param></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="start">1</xsl:param><xsl:param name="stop">1</xsl:param><xsl:param name="step">1</xsl:param><xsl:value-of select="name()"/><xsl:text>;</xsl:text><xsl:if test="$start &lt; $stop"><xsl:call-template name="n"><xsl:with-param name="stop"><xsl:value-of select="$stop"/></xsl:with-param><xsl:with-param name="start"><xsl:value-of select="$start + $step"/></xsl:with-param></xsl:call-template></xsl:if></xsl:template>"#;

#[test]
fn selected_child_path_terminal_wildcard_preserves_dispatch_and_charges() {
    let body = r#"<xsl:template match="/"><xsl:apply-templates select="doc/*"/></xsl:template><xsl:template match="*"><xsl:value-of select="name()"/><xsl:text>;</xsl:text></xsl:template>"#;
    for xml in ["<doc/>", "<doc>text<a/><!--comment--><b/><a/></doc>"] {
        let source = document("source", xml.as_bytes());
        let reference = compiler::compile_stylesheet(&sheet("1.0", body)).unwrap();
        let mut baseline = InvocationControl::unbounded();
        let expected = execute(&reference, &source, "reference", &mut baseline).unwrap();
        for version in ["1.0", "2.0"] {
            let program = compile(
                &sheet(version, body),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let mut actual = InvocationControl::unbounded();
            assert_eq!(
                execute(&program, &source, "selected", &mut actual).unwrap(),
                expected
            );
            for domain in [WorkDomain::XPathNodeVisit, WorkDomain::XsltInstruction] {
                assert_eq!(actual.consumed(domain), baseline.consumed(domain));
            }
        }
    }
    for path in ["doc//*", "doc/*/a", "doc/*[1]", "/doc/*", "doc/@*"] {
        let style = sheet(
            "1.0",
            &format!(
                r#"<xsl:template match="/"><xsl:apply-templates select="{path}"/></xsl:template>"#
            ),
        );
        assert!(
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).is_err(),
            "{path}"
        );
    }
}

#[test]
fn selected_variable_relations_preserve_existential_node_set_semantics() {
    for (operator, xml, expected) in [
        ("&lt;", "<doc><a>9</a><a>1</a><b>5</b></doc>", "yes"),
        ("&gt;", "<doc><a>1</a><a>9</a><b>5</b></doc>", "yes"),
        ("&lt;=", "<doc><a>9</a><a>5</a><b>5</b></doc>", "yes"),
        ("&gt;=", "<doc><a>1</a><a>5</a><b>5</b></doc>", "yes"),
        ("&lt;", "<doc><a>bad</a><b>5</b></doc>", ""),
        ("&lt;", "<doc><b>5</b></doc>", ""),
    ] {
        let declarations = format!(
            r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="left" select="a"/><xsl:with-param name="right" select="b"/></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="left"/><xsl:param name="right"/><xsl:if test="$left {operator} $right">yes</xsl:if></xsl:template>"#
        );
        let source = document("urn:relations", xml.as_bytes());
        for version in ["1.0", "2.0"] {
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
                    "relation",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn recursive_arguments_preserve_selected_policy_focus_and_work() {
    let source = document(
        "urn:recursive",
        b"<doc><a repeat='3'/><b repeat='2'/></doc>",
    );
    let reference = compiler::compile_stylesheet(&sheet("1.0", RECURSIVE)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "reference", &mut baseline).unwrap(),
        "a;a;a;b;b;"
    );
    for version in ["1.0", "2.0"] {
        let program = compile(
            &sheet(version, RECURSIVE),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            "a;a;a;b;b;"
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
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 0);
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
            "a;a;a;b;b;"
        );
    }
}

#[test]
fn argument_binding_errors_remain_static_in_dormant_bodies() {
    for expression in ["$missing", "$known + $missing", "$missing + $known"] {
        for version in ["1.0", "2.0"] {
            let declarations = format!(
                r#"<xsl:template match="doc"><xsl:variable name="known">1</xsl:variable><xsl:if test="false()"><xsl:call-template name="n"><xsl:with-param name="p"><xsl:value-of select="{expression}"/></xsl:with-param></xsl:call-template></xsl:if></xsl:template><xsl:template name="n"><xsl:param name="p"/></xsl:template>"#
            );
            let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
                &sheet(version, &declarations),
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            ) else {
                panic!("unbound argument admitted");
            };
            assert_eq!(failure.code, "XPST0008");
        }
    }
}

#[test]
fn comparison_binding_errors_remain_static_across_short_circuit() {
    for condition in [
        "$known &lt; $missing",
        "$missing &lt; $known",
        "false() and $known &lt; $missing",
        "true() or $missing &lt; $known",
    ] {
        let declarations = format!(
            r#"<xsl:template match="doc"><xsl:variable name="known">1</xsl:variable><xsl:if test="{condition}">bad</xsl:if></xsl:template>"#
        );
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet("2.0", &declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("unbound comparison admitted");
        };
        assert_eq!(failure.code, "XPST0008");
    }
}

#[test]
fn value_argument_keeps_unintegrated_constructor_boundaries() {
    for body in [
        r#"<xsl:value-of select="@v" disable-output-escaping="yes"/>"#,
        r#"<xsl:value-of select="@v" version="1.0"/>"#,
        r#"<xsl:value-of select="@v"/><xsl:text>suffix</xsl:text>"#,
        r#"<xsl:value-of select="system-property()"/>"#,
        r#"<xsl:value-of select="$a * $b"/>"#,
    ] {
        let declarations = format!(
            r#"<xsl:template match="doc"><xsl:call-template name="n"><xsl:with-param name="p">{body}</xsl:with-param></xsl:call-template></xsl:template><xsl:template name="n"><xsl:param name="p"/></xsl:template>"#
        );
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet("2.0", &declarations),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("constructor admitted: {body}");
        };
        assert_eq!(failure.code, "FXAR0031");
    }
}
#[test]
fn selected_apply_mode_keeps_ncname_boundary() {
    for mode in ["#current", "#all", "#default", "", "p:mode", "one two"] {
        let style = sheet(
            "1.0",
            &format!(
                "<xsl:template match=\"/\"><xsl:apply-templates mode=\"{mode}\"/></xsl:template>"
            ),
        );
        assert!(
            super::super::compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
                .is_err(),
            "{mode}"
        );
    }
    for version in ["1.0", "2.0"] {
        let style = sheet(
            version,
            "<xsl:template match=\"/\"><xsl:apply-templates select=\"r\" mode=\"chosen\"/></xsl:template><xsl:template match=\"r\" mode=\"chosen\">selected</xsl:template><xsl:template match=\"r\">wrong</xsl:template>",
        );
        let program =
            super::super::compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
                .unwrap();
        assert_eq!(
            crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test(
                &program,
                &document("urn:source", b"<r/>"),
                "mode"
            )
            .unwrap(),
            "selected"
        );
    }
}
