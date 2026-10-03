//! Namespace count arguments retain only an atomic integer, never occurrences.

use super::{
    CancellationToken, InvocationControl, WhitespaceRepresentation, WorkDomain, WorkLimits,
    compile_stylesheet, document, run,
};

fn program(version: &str, select: &str) -> super::StylesheetProgram {
    compile_stylesheet(&document("urn:namespace:argument-style", &format!(r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:strip-space elements="*"/><xsl:template match="/"><xsl:call-template name="emit"><xsl:with-param name="max" select="{select}"/></xsl:call-template><xsl:text>|</xsl:text><xsl:value-of select="{select}"/></xsl:template><xsl:template name="emit"><xsl:param name="max"/><xsl:value-of select="$max"/></xsl:template></xsl:stylesheet>"#))).unwrap()
}

#[test]
fn namespace_count_argument_matches_scalar_with_owner_relative_identity_and_views() {
    let source = document(
        "urn:namespace:arguments",
        r#"<r xmlns:p="urn:p"> <child xmlns:q="urn:q"/> <child/> </r>"#,
    );
    for version in ["1.0", "3.0"] {
        for (select, expected) in [
            ("count(descendant-or-self::*/namespace::*)", "7|7"),
            ("count(descendant-or-self::*/namespace::p)", "3|3"),
            ("count(descendant-or-self::*/namespace::missing)", "0|0"),
        ] {
            let program = program(version, select);
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
                    expected
                );
            }
        }
    }
}

#[test]
fn namespace_count_argument_exhaustion_and_cancellation_leave_prepared_state_reusable() {
    let source = document(
        "urn:namespace:arguments",
        r#"<r xmlns:p="urn:p"><child/></r>"#,
    );
    let program = program("1.0", "count(descendant-or-self::*/namespace::*)");
    let representation = WhitespaceRepresentation::VisibilityView;
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 3);
    let failure = run(&program, &source, representation, &mut cancelled).unwrap_err();
    assert_eq!(failure.code, "FXCT0001");
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        run(&program, &source, representation, &mut measured).unwrap(),
        "4|4"
    );
    let charge = measured.consumed(WorkDomain::XPathNodeVisit);
    for allowance in [charge - 1, charge] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = allowance;
        let result = run(
            &program,
            &source,
            representation,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowance == charge {
            assert_eq!(result.unwrap(), "4|4");
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
}
