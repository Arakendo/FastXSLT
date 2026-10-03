//! Mixed qualified selections share the ordinary sequence-focus executor.

use super::{
    CompileCategory, InvocationControl, WhitespaceRepresentation, WorkDomain, compile_stylesheet,
    document, run, stylesheet,
};

fn body(select: &str) -> String {
    format!(
        r#"<xsl:for-each select="{select}"><xsl:value-of select="name()"/><xsl:text>:</xsl:text><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>|</xsl:text></xsl:for-each>"#
    )
}

#[test]
fn mixed_focus_normalizes_identity_and_order_before_positions_in_both_profiles() {
    let source = document(
        "urn:namespace:mixed",
        r#"<r xmlns:p="urn:p" a="attr"> <child> text </child> </r>"#,
    );
    for version in ["1.0", "3.0"] {
        for select in [
            "r/@* | r/child | r/namespace::* | r | r/namespace::p",
            "r/namespace::p | r | r/namespace::* | r/child | r/@*",
        ] {
            let program = compile_stylesheet(&stylesheet(version, &body(select))).unwrap();
            for representation in [
                WhitespaceRepresentation::VisibilityView,
                WhitespaceRepresentation::CompleteReference,
            ] {
                assert_eq!(
                    run(
                        &program,
                        &source,
                        representation,
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    "r:1/5|p:2/5|xml:3/5|a:4/5|child:5/5|"
                );
            }
        }
    }
}

#[test]
fn mixed_focus_cancel_discards_partial_selection_and_prepared_input_remains_reusable() {
    let source = document(
        "urn:namespace:mixed",
        r#"<r xmlns:p="urn:p" a="attr"><child/></r>"#,
    );
    let program =
        compile_stylesheet(&stylesheet("1.0", &body("r/@* | r/namespace::* | r"))).unwrap();
    let mut control =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 5);
    assert!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut control
        )
        .is_err()
    );
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "r:1/4|p:2/4|xml:3/4|a:4/4|"
    );
}

#[test]
fn mixed_focus_sort_and_unproved_conditional_consumers_remain_explicitly_unsupported() {
    for version in ["1.0", "3.0"] {
        for consumer in [
            r#"<xsl:sort select="name()"/>"#,
            r#"<xsl:if test="name() = 'p'"><xsl:text>first</xsl:text></xsl:if>"#,
        ] {
            let style = stylesheet(
                version,
                &format!(
                    r#"<xsl:for-each select="r/namespace::* | r/@*">{consumer}</xsl:for-each>"#
                ),
            );
            let failure = compile_stylesheet(&style).unwrap_err();
            assert_eq!(failure.category, CompileCategory::Unsupported);
        }
    }
}
