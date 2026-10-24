use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

fn program(
    version: &str,
    select: &str,
    strip: bool,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let whitespace = if strip {
        r#"<xsl:strip-space elements="*"/>"#
    } else {
        ""
    };
    selected(&[(
        PRINCIPAL,
        &format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{whitespace}<xsl:output method="text"/><xsl:template match="/"><xsl:for-each select="r/group"><xsl:for-each select="{select}"><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>:</xsl:text><xsl:value-of select="."/><xsl:text>;</xsl:text></xsl:for-each><xsl:text>|</xsl:text><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/></xsl:for-each></xsl:template></xsl:stylesheet>"#
        ),
    )])
}

fn source() -> Document {
    Document::from_parsed(parse_document("urn:descendant-source",
        br#"<r><a>A</a><group> <a>B</a> <nested><a>C</a></nested> </group><a>D</a><p:a xmlns:p="urn:p">EXCLUDED</p:a></r>"#,
        super::super::super::XML_LIMITS).unwrap()).unwrap()
}

#[test]
fn selected_descendant_iteration_roots_order_focus_and_views() {
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            for (select, expected) in [
                ("//a", "1/4:A;2/4:B;3/4:C;4/4:D;|1/1"),
                ("//missing", "|1/1"),
                (".//a", "1/2:B;2/2:C;|1/1"),
                (".//group", "|1/1"),
                (".//missing", "|1/1"),
                (
                    "//group",
                    if strip {
                        "1/1:BC;|1/1"
                    } else {
                        "1/1: B C ;|1/1"
                    },
                ),
            ] {
                let compiled = program(version, select, strip).unwrap();
                assert_eq!(
                    execute(
                        &compiled,
                        &source(),
                        "descendant",
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
    }
}

#[test]
fn selected_descendant_iteration_keeps_adjacent_grammar_guarded() {
    for select in [
        "//",
        "//a/b",
        "a//b",
        ".//a/b",
        ".//*",
        ".//a[1]",
        "//*",
        "//p:a",
        "//a[1]",
        "//a | //b",
        "//a/@id",
        "//a()",
        "document('x')//a",
    ] {
        let failure = program("2.0", select, false).unwrap_err();
        assert_eq!(failure.category, FailureCategory::Unsupported, "{select}");
        assert_eq!(failure.code, "FXAR0031");
        assert_eq!(failure.location.unwrap().resource, PRINCIPAL);
    }
}

#[test]
fn selected_descendant_iteration_controls_and_reuse() {
    for select in ["//a", "*", "/r/a", "nested/*", "/r/group/*", ".//a"] {
        assert_iteration_controls(select);
    }
}

#[test]
fn selected_rooted_iteration_distinguishes_absolute_paths() {
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            for (select, expected) in [
                ("/r/a", "1/2:A;2/2:D;|1/1"),
                ("/r/group/nested/a", "1/1:C;|1/1"),
                ("r/a", "|1/1"),
                ("/r/missing", "|1/1"),
            ] {
                let compiled = program(version, select, strip).unwrap();
                assert_eq!(
                    execute(
                        &compiled,
                        &source(),
                        "rooted",
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
        for select in ["/", "/r/", "/r/*/a", "/r/a[1]", "/r//a", "/p:r/a", "/r/@id"] {
            let error = program(version, select, false).unwrap_err();
            assert_eq!(error.category, FailureCategory::Unsupported);
            assert_eq!(error.location.unwrap().resource, PRINCIPAL);
        }
    }
}

fn assert_iteration_controls(select: &str) {
    let compiled = program("2.0", select, true).unwrap();
    let source = source();
    let mut measured = InvocationControl::unbounded();
    let expected = execute(&compiled, &source, "measure", &mut measured).unwrap();
    let required = measured.consumed(WorkDomain::XPathNodeVisit);
    assert!(required > 1);
    for budget in [required - 1, required] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = budget;
        let result = execute(
            &compiled,
            &source,
            "budget",
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if budget == required {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(result.unwrap_err().contains("FXCT0002"));
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 3);
    assert!(
        execute(&compiled, &source, "cancel", &mut cancelled)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        execute(
            &compiled,
            &source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn selected_wildcard_iteration_uses_element_children_and_restores_focus() {
    let input = Document::from_parsed(parse_document("urn:wildcard-iteration",
        br#"<r><group>omit<!--omit--><?omit x?><a>A</a><p:a xmlns:p="urn:p">B</p:a><nested> <a>C</a> </nested></group><group/></r>"#,
        super::super::super::XML_LIMITS).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            let compiled = program(version, "*", strip).unwrap();
            let expected = if strip {
                "1/3:A;2/3:B;3/3:C;|1/2|2/2"
            } else {
                "1/3:A;2/3:B;3/3: C ;|1/2|2/2"
            };
            assert_eq!(
                execute(
                    &compiled,
                    &input,
                    "wildcard",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
        for select in ["r/*/a", "*/a", "@*", "namespace::*", "*[1]"] {
            let error = program(version, select, false).unwrap_err();
            assert_eq!(error.category, FailureCategory::Unsupported);
            assert_eq!(error.location.unwrap().resource, PRINCIPAL);
        }
    }
}

#[test]
fn selected_final_wildcard_iteration_preserves_focus_and_effective_children() {
    let input = Document::from_parsed(parse_document("urn:final-wildcard-iteration",
        br#"<r><group><box>omit<!--omit--><?omit x?><a>A</a><p:a xmlns:p="urn:p">B</p:a><nested> <a>C</a> </nested></box></group></r>"#,
        super::super::super::XML_LIMITS).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            for select in ["box/*", "/r/group/box/*"] {
                let compiled = program(version, select, strip).unwrap();
                let expected = if strip {
                    "1/3:A;2/3:B;3/3:C;|1/1"
                } else {
                    "1/3:A;2/3:B;3/3: C ;|1/1"
                };
                assert_eq!(
                    execute(
                        &compiled,
                        &input,
                        "final-wildcard",
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
        for select in ["missing/*", "/r/missing/*", "r/group/box/*"] {
            let compiled = program(version, select, false).unwrap();
            assert_eq!(
                execute(
                    &compiled,
                    &input,
                    "empty",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "|1/1"
            );
        }
    }
}
