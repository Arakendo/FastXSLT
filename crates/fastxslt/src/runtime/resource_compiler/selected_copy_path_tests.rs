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
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{whitespace}<xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="r/group"><xsl:copy-of select="{select}"/></xsl:for-each></out></xsl:template></xsl:stylesheet>"#
        ),
    )])
}

fn source() -> Document {
    Document::from_parsed(
        parse_document(
            "urn:copy-path-source",
            b"<r><top>T</top><group><!--control--><a><b n='1'>one</b><b n='2'>two</b></a></group></r>",
            super::super::super::XML_LIMITS,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn selected_copy_path_root_relative_order_and_empty_selection() {
    for version in ["1.0", "2.0", "3.0"] {
        for (select, expected) in [
            ("a/b", r#"<out><b n="1">one</b><b n="2">two</b></out>"#),
            ("/r/top", "<out><top>T</top></out>"),
            (" /r/top ", "<out><top>T</top></out>"),
            ("/r/missing", "<out></out>"),
            ("r/top", "<out></out>"),
        ] {
            let compiled = program(version, select, false).unwrap();
            for _ in 0..2 {
                assert_eq!(
                    execute(
                        &compiled,
                        &source(),
                        "copy-path",
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
fn selected_copy_path_preserves_subtree_namespaces_and_effective_whitespace() {
    let source = Document::from_parsed(
        parse_document(
            "urn:copy-path-source",
            br#"<r><group/><a xmlns:p="urn:p" p:id="v">  <p:b>text</p:b>  </a></r>"#,
            super::super::super::XML_LIMITS,
        )
        .unwrap(),
    )
    .unwrap();
    for strip in [false, true] {
        let compiled = program("2.0", "/r/a", strip).unwrap();
        let actual = execute(
            &compiled,
            &source,
            "copy-view",
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert!(actual.contains("xmlns:p=\"urn:p\""));
        assert!(actual.contains("p:id=\"v\""));
        assert!(actual.contains("<p:b>text</p:b>"));
        assert_eq!(actual.contains("  "), !strip);
    }
}

#[test]
fn selected_copy_path_refuses_adjacent_selection_families() {
    for version in ["1.0", "2.0"] {
        for select in [
            "",
            "current()/a",
            "//a/*",
            "a//b//c",
            "a/",
            "/*",
            "a/*/b",
            "p:a",
            "a[1]",
            "@id",
            "namespace::*",
            "$v",
            "document('x')",
            "a | b",
            "./a",
            "a/../b",
        ] {
            let failure = program(version, select, false).unwrap_err();
            assert_eq!(failure.category, FailureCategory::Unsupported, "{select}");
            assert_eq!(failure.code, "FXAR0031");
            assert_eq!(failure.location.unwrap().resource, PRINCIPAL);
        }
    }
}

#[test]
fn selected_copy_path_control_failures_allow_reuse() {
    for select in [
        "a/b",
        "*",
        "'literal'",
        "a/*",
        "/r/group/a/*",
        "//a/b",
        ".",
        "/",
    ] {
        assert_copy_controls(select);
    }
}

#[test]
fn selected_copy_direct_current_and_root_remain_distinct() {
    let input = Document::from_parsed(parse_document("urn:direct-copy",
        br#"<!--top--><r><group xmlns:p="urn:p" p:id="v">  <a>A</a>  </group><outside>B</outside></r><?end x?>"#,
        super::super::super::XML_LIMITS).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            let current = execute(
                &program(version, ".", strip).unwrap(),
                &input,
                "current",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert!(
                current.contains("<group")
                    && current.contains("xmlns:p=\"urn:p\"")
                    && current.contains("p:id=\"v\"")
            );
            assert!(!current.contains("outside") && !current.contains("<!--top-->"));
            assert_eq!(
                execute(
                    &program(version, "current()", strip).unwrap(),
                    &input,
                    "current-call",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                current
            );
            let root = execute(
                &program(version, "/", strip).unwrap(),
                &input,
                "root",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert!(
                root.contains("<!--top--><r>")
                    && root.contains("<outside>B</outside>")
                    && root.contains("<?end x?>")
            );
            assert_eq!(current.contains("  "), !strip);
            assert_eq!(root.contains("  "), !strip);
        }
    }
    assert_copy_controls("current()");
    for select in ["current(1)", "current()/a", "a[current()]", "current() | ."] {
        assert_eq!(
            program("2.0", select, false).unwrap_err().category,
            FailureCategory::Unsupported
        );
    }
}

#[test]
fn selected_copy_child_comments_preserve_order_focus_and_controls() {
    let input = Document::from_parsed(parse_document("urn:comment-copy",
        br"<!--document--><r><!--root--><group> text <!--one--><a><!--nested--></a><!--two--><?omit x?></group></r>",
        super::super::super::XML_LIMITS).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            for (select, expected) in [
                ("comment()", "<out><!--one--><!--two--></out>"),
                ("a/comment()", "<out><!--nested--></out>"),
                ("/r/comment()", "<out><!--root--></out>"),
                ("/comment()", "<out><!--document--></out>"),
                ("missing/comment()", "<out></out>"),
            ] {
                assert_eq!(
                    execute(
                        &program(version, select, strip).unwrap(),
                        &input,
                        "comment-copy",
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
        for select in [
            "//comment()",
            "a//comment()",
            "comment()[1]",
            "comment('x')",
            "text()",
            "processing-instruction()",
            "a/comment()/..",
        ] {
            assert_eq!(
                program(version, select, false).unwrap_err().category,
                FailureCategory::Unsupported
            );
        }
    }
    assert_copy_controls("comment()");
    assert_copy_controls("/r/group/comment()");
}

#[test]
fn selected_copy_interior_descendants_preserve_focus_order_and_views() {
    let input = Document::from_parsed(parse_document("urn:interior-desc-copy",
        br#"<r><group><a><b>1</b><a><b>2</b></a><b>3</b><p:b xmlns:p="urn:p">omit</p:b></a></group><a><b>outside</b></a></r>"#,
        super::super::super::XML_LIMITS).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            for (select, expected) in [
                ("a//b", "<out><b>1</b><b>2</b><b>3</b></out>"),
                ("/r/group/a//b", "<out><b>1</b><b>2</b><b>3</b></out>"),
                ("a//a/b", "<out><b>2</b></out>"),
                ("/r/a//b", "<out><b>outside</b></out>"),
                ("missing//b", "<out></out>"),
            ] {
                assert_eq!(
                    execute(
                        &program(version, select, strip).unwrap(),
                        &input,
                        "interior-desc",
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
        for select in ["a//b//c", "a//*", "a//b[1]", "a//comment()", "a///b"] {
            assert_eq!(
                program(version, select, false).unwrap_err().category,
                FailureCategory::Unsupported
            );
        }
    }
    assert_copy_controls("a//b");
    assert_copy_controls("/r/group/a//b");
}

#[test]
fn selected_copy_root_descendants_preserve_order_names_and_views() {
    let input = Document::from_parsed(parse_document("urn:root-desc-copy",
        br#"<r><group/><a><b>1</b><a><b>2</b></a><b>3</b><p:b xmlns:p="urn:p">omit</p:b></a><a><b>  <c/>  </b></a></r>"#,
        super::super::super::XML_LIMITS).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            let compiled = program(version, "//a/b", strip).unwrap();
            let actual = execute(
                &compiled,
                &input,
                "root-desc",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let tail = if strip {
                "<b><c></c></b>"
            } else {
                "<b>  <c></c>  </b>"
            };
            assert_eq!(actual, format!("<out><b>1</b><b>2</b><b>3</b>{tail}</out>"));
        }
        let compiled = program(version, "//missing/b", false).unwrap();
        assert_eq!(
            execute(
                &compiled,
                &input,
                "empty",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "<out></out>"
        );
    }
}

#[test]
fn selected_copy_final_wildcard_preserves_child_order_root_focus_and_views() {
    let source = Document::from_parsed(parse_document(
        "urn:final-wildcard", br#"<r><group><a>omit<!--omit--><?omit x?><b xmlns:p="urn:p" p:id="1">  <p:c/>  </b><z/></a></group></r>"#,
        super::super::super::XML_LIMITS,
    ).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            for select in ["a/*", "/r/group/a/*", " /r/group/a/* "] {
                let compiled = program(version, select, strip).unwrap();
                let actual = execute(
                    &compiled,
                    &source,
                    "wildcard-path",
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                assert!(!actual.contains("omit"));
                assert!(actual.contains("xmlns:p=\"urn:p\"") && actual.contains("p:id=\"1\""));
                assert!(actual.find("<b ").unwrap() < actual.find("<z").unwrap());
                assert_eq!(actual.contains("  "), !strip);
            }
        }
        for select in ["missing/*", "/r/missing/*", "r/group/a/*"] {
            let compiled = program(version, select, false).unwrap();
            assert_eq!(
                execute(
                    &compiled,
                    &source,
                    "empty",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "<out></out>"
            );
        }
    }
}

#[test]
fn selected_copy_string_literals_are_text_not_markup() {
    for version in ["1.0", "2.0", "3.0"] {
        for (select, expected) in [
            ("'hello'", "<out>hello</out>"),
            ("&quot;hello&quot;", "<out>hello</out>"),
            ("''", "<out></out>"),
            ("'&lt;a&gt;&amp;é'", "<out>&lt;a&gt;&amp;é</out>"),
            ("'say &quot;yes&quot;'", "<out>say \"yes\"</out>"),
        ] {
            let compiled = program(version, select, false).unwrap();
            assert_eq!(
                execute(
                    &compiled,
                    &source(),
                    "literal",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
        for select in [
            "'a''b'",
            "'a' | 'b'",
            "'unclosed",
            "1.5",
            "true(1)",
            "1+1",
            "$v",
        ] {
            let error = program(version, select, false).unwrap_err();
            assert_eq!(error.category, FailureCategory::Unsupported, "{select}");
            assert_eq!(error.location.unwrap().resource, PRINCIPAL);
        }
    }
}

fn assert_copy_controls(select: &str) {
    let compiled = program("1.0", select, false).unwrap();
    let source = source();
    let mut measured = InvocationControl::unbounded();
    let expected = execute(&compiled, &source, "measure", &mut measured).unwrap();
    let required = measured.consumed(WorkDomain::ResultNode);
    assert!(required > 0);
    for budget in [required - 1, required] {
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = budget;
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
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XsltInstruction, 1);
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
fn selected_copy_integer_constants_use_exact_bounded_argument_domain() {
    for version in ["1.0", "2.0", "3.0"] {
        for (select, expected) in [
            ("32", "32"),
            ("0", "0"),
            ("-0", "0"),
            ("00032", "32"),
            ("-00032", "-32"),
            ("2147483647", "2147483647"),
            ("-2147483648", "-2147483648"),
            (" 32 ", "32"),
        ] {
            assert_eq!(
                execute(
                    &program(version, select, false).unwrap(),
                    &source(),
                    "integer-copy",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                format!("<out>{expected}</out>")
            );
        }
        for select in [
            "2147483648",
            "-2147483649",
            "+1",
            "1.5",
            "1e2",
            "1+1",
            "number('1')",
            "--1",
            "- 1",
            "9007199254740993",
        ] {
            assert_eq!(
                program(version, select, false).unwrap_err().category,
                FailureCategory::Unsupported,
                "{select}"
            );
        }
    }
    assert_copy_controls("32");
    assert_copy_controls("-2147483648");
}

#[test]
fn selected_copy_boolean_constants_share_atomic_copy_and_controls() {
    for version in ["1.0", "2.0", "3.0"] {
        for (select, expected) in [
            ("true()", "<out>true</out>"),
            (" false() ", "<out>false</out>"),
        ] {
            let compiled = program(version, select, false).unwrap();
            assert_eq!(
                execute(
                    &compiled,
                    &source(),
                    "boolean-copy",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
        }
        for select in [
            "true(1)",
            "false(0)",
            "not(false())",
            "true() or false()",
            "1.5",
            "$v",
            "boolean('x')",
        ] {
            let error = program(version, select, false).unwrap_err();
            assert_eq!(error.category, FailureCategory::Unsupported);
            assert_eq!(error.location.unwrap().resource, PRINCIPAL);
        }
    }
    assert_copy_controls("true()");
    assert_copy_controls("false()");
}

#[test]
fn selected_copy_wildcard_preserves_focus_order_subtrees_and_views() {
    let source = Document::from_parsed(parse_document(
        "urn:wildcard-source",
        br#"<r><group>omit<!--omit--><?omit x?><a xmlns:p="urn:p" p:id="v">  <p:b/>  <!--keep--><?keep x?></a><z/></group></r>"#,
        super::super::super::XML_LIMITS,
    ).unwrap()).unwrap();
    for version in ["1.0", "2.0", "3.0"] {
        for strip in [false, true] {
            for select in ["*", " * "] {
                let compiled = program(version, select, strip).unwrap();
                let actual = execute(
                    &compiled,
                    &source,
                    "wildcard",
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                assert!(!actual.contains("omit"));
                assert!(!actual.contains("<group"));
                assert!(actual.contains("xmlns:p=\"urn:p\"") && actual.contains("p:id=\"v\""));
                assert!(actual.contains("<!--keep--><?keep x?>"));
                assert!(actual.find("<a ").unwrap() < actual.find("<z").unwrap());
                assert_eq!(actual.contains("  "), !strip);
            }
        }
        let empty = program(version, "*", false).unwrap();
        let empty_source = Document::from_parsed(
            parse_document(
                "urn:empty",
                b"<r><group>text</group></r>",
                super::super::super::XML_LIMITS,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            execute(
                &empty,
                &empty_source,
                "empty",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "<out></out>"
        );
    }
}
