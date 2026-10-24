use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

fn program(version: &str, expression: &str) -> Result<StylesheetProgram, ExecutionFailure> {
    selected(&[(
        PRINCIPAL,
        &format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="{expression}"/></xsl:template></xsl:stylesheet>"#
        ),
    )])
}

fn source() -> Document {
    Document::from_parsed(
        parse_document(
            "urn:value-source",
            b"<r><group><a id='first'>one</a><a id='second'>two</a></group></r>",
            super::super::super::XML_LIMITS,
        )
        .unwrap(),
    )
    .unwrap()
}

fn controls(expression: &str) {
    let compiled = program("2.0", expression).unwrap();
    let input = source();
    let mut measured = InvocationControl::unbounded();
    let expected = execute(&compiled, &input, "measure", &mut measured).unwrap();
    let required = measured.consumed(WorkDomain::ResultNode);
    assert!(required > 0);
    for budget in [required - 1, required] {
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = budget;
        let result = execute(
            &compiled,
            &input,
            "limited",
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if budget == required {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().contains("FXCT0002"));
        }
    }
    let cancellation_domain = if expression.starts_with("./") {
        WorkDomain::XPathNodeVisit
    } else {
        WorkDomain::XsltInstruction
    };
    assert!(measured.consumed(cancellation_domain) > 1);
    let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(cancellation_domain, 1);
    assert!(
        execute(&compiled, &input, "cancel", &mut cancelled)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        execute(
            &compiled,
            &input,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn selected_value_explicit_relative_paths_preserve_first_node_and_views() {
    let input = Document::from_parsed(
        parse_document(
            "urn:relative-source",
            b"<r><group>  <a>one</a><a>two</a>tail</group><group><a>three</a></group></r>",
            super::super::super::XML_LIMITS,
        )
        .unwrap(),
    )
    .unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            let whitespace = if strip {
                r#"<xsl:strip-space elements="*"/>"#
            } else {
                ""
            };
            for (expression, expected) in [
                ("./a", "one|three|"),
                ("./a/text()", "one|three|"),
                ("./missing", "||"),
                ("./text()", if strip { "tail||" } else { "  ||" }),
            ] {
                let xml = format!(
                    r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{whitespace}<xsl:output method="text"/><xsl:template match="/"><xsl:for-each select="r/group"><xsl:value-of select="{expression}"/><xsl:text>|</xsl:text></xsl:for-each></xsl:template></xsl:stylesheet>"#
                );
                assert_eq!(
                    execute(
                        &selected(&[(PRINCIPAL, &xml)]).unwrap(),
                        &input,
                        "relative-value",
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
        for expression in ["./a[1]", "./*", "./a/@*", "./a//b"] {
            assert_eq!(
                program(version, expression).unwrap_err().category,
                FailureCategory::Unsupported
            );
        }
    }
    controls("./r/group/a");
}

#[test]
fn selected_value_constant_booleans_use_shared_semantics() {
    for version in ["1.0", "2.0", "3.0"] {
        for (expression, expected) in [
            ("true()", "true"),
            ("false()", "false"),
            ("true() and false()", "false"),
            ("true() or false()", "true"),
            ("not(true())", "false"),
            ("boolean(false())", "false"),
            ("(false() or true()) and not(false())", "true"),
            ("false() and true() or true()", "true"),
        ] {
            assert_eq!(
                execute(
                    &program(version, expression).unwrap(),
                    &source(),
                    "value",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
        for expression in [
            "boolean('x')",
            "not(0)",
            "true(1)",
            "true() and r",
            "false() or $missing",
        ] {
            assert_eq!(
                program(version, expression).unwrap_err().category,
                FailureCategory::Unsupported
            );
        }
    }
    controls("true()");
    controls("not(false()) and boolean(true())");
}

#[test]
fn selected_value_child_attribute_paths_are_read_only_and_first_node() {
    let input = Document::from_parsed(parse_document("urn:attribute-values",
        br#"<r><top id="root"/><group><a xmlns:p="urn:p" p:id="omit"/><a id="one&amp;two"/><a id="later"/></group></r>"#,
        super::super::super::XML_LIMITS).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for (expression, expected) in [
            ("a/@id", "one&amp;two"),
            ("./a/@id", "one&amp;two"),
            ("/r/top/@id", "root"),
            ("/r/group/a/@id", "one&amp;two"),
            ("missing/@id", ""),
            ("r/top/@id", ""),
        ] {
            let xml = format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="r/group"><xsl:value-of select="{expression}"/></xsl:for-each></out></xsl:template></xsl:stylesheet>"#
            );
            assert_eq!(
                execute(
                    &selected(&[(PRINCIPAL, &xml)]).unwrap(),
                    &input,
                    "attribute-value",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                format!("<out>{expected}</out>")
            );
        }
        for expression in ["a/@p:id", "a/@*", "a[1]/@id", "a//@id", "a/@id | b/@id"] {
            assert_eq!(
                program(version, expression).unwrap_err().category,
                FailureCategory::Unsupported
            );
        }
    }
    controls("./r/group/a/@id");
}
