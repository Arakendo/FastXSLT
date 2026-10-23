use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

fn stylesheet(version: &str, pattern: &str) -> Document {
    document("urn:selected-pattern", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="r"><xsl:apply-templates/></xsl:template><xsl:template match="*" priority="-10"/><xsl:template match="text()"/><xsl:template match="{pattern}"><xsl:value-of select="name(.)"/></xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_match_predicates_preserve_focus_and_charged_execution() {
    let source = document("urn:pattern-source", b"<r>text<a val='yes'>117</a><!-- gap --><a val='no'>2</a><b>117</b><?gap x?><c foo='nope'>3</c></r>");
    for (pattern, output) in [
        ("//a", "aa"),
        ("a[@val]", "aa"),
        ("*[.=117]", "ab"),
        (
            "*[(not(.=117) and ((position() &gt; 1) and (position() &lt; 4))) and ((@val='no') or (@foo='nope'))]",
            "a",
        ),
        ("*[(. = 117 or @foo='nope') and not(position() = 1)]", "bc"),
    ] {
        let reference = compiler::compile_stylesheet(&stylesheet("1.0", pattern)).unwrap();
        let mut reference_control = InvocationControl::unbounded();
        assert_eq!(
            execute(&reference, &source, "reference", &mut reference_control).unwrap(),
            output
        );
        for version in ["1.0", "2.0", "3.0"] {
            let style = stylesheet(version, pattern);
            let selected =
                compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
            drop(style);
            let mut control = InvocationControl::unbounded();
            assert_eq!(
                execute(&selected, &source, "selected", &mut control).unwrap(),
                output
            );
            for domain in [
                WorkDomain::XPathNodeVisit,
                WorkDomain::XsltInstruction,
                WorkDomain::ResultTextByte,
            ] {
                assert_eq!(control.consumed(domain), reference_control.consumed(domain));
            }
            let required = control.consumed(WorkDomain::XPathNodeVisit);
            assert!(required > 0);
            let mut limits = WorkLimits::unbounded();
            limits.xpath_node_visits = required;
            assert_eq!(
                execute(
                    &selected,
                    &source,
                    "exact",
                    &mut InvocationControl::new(CancellationToken::new(), limits)
                )
                .unwrap(),
                output
            );
            limits.xpath_node_visits -= 1;
            assert!(
                execute(
                    &selected,
                    &source,
                    "short",
                    &mut InvocationControl::new(CancellationToken::new(), limits)
                )
                .unwrap_err()
                .contains("FXCT0002")
            );
            let mut cancelled =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 1);
            assert!(
                execute(&selected, &source, "cancel", &mut cancelled)
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
                output
            );
        }
    }
}

#[test]
fn selected_match_predicates_do_not_admit_other_pattern_families() {
    for pattern in [
        "*[@val &gt; 1]",
        "a[position() &gt; 1]",
        "node()",
        "@val",
        "//a/b",
    ] {
        let Err(StrictPreprocessingFailure::Compile(error)) = compile(
            &stylesheet("2.0", pattern),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("must reject {pattern}")
        };
        assert_eq!(error.code, "FXAR0031");
        assert_eq!(error.location.resource, "urn:selected-pattern");
    }
    assert!(
        compile(
            &stylesheet("3.0", "*[.=117]"),
            Edition::Modern,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}
