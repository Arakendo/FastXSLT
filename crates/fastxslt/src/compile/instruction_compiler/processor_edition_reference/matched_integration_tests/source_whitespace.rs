use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

#[test]
fn selected_whitespace_focus_uses_visible_selection_and_restores_caller() {
    let source = document("urn:focus", b"<r> <item/> <item/> <item/> </r>");
    for version in ["1.0", "2.0", "3.0"] {
        let style = document("urn:focus-style", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:strip-space elements="*"/><xsl:template match="r"><xsl:apply-templates/><xsl:text>|</xsl:text><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/></xsl:template><xsl:template match="item"><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>;</xsl:text></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &source, "focus").unwrap(),
            "1/3;2/3;3/3;|1/1"
        );
    }
}

#[test]
fn selected_child_iteration_preserves_nested_focus_scope_and_controls() {
    let source = document("urn:iteration", b"<r><a><b/><b/></a><a><b/></a></r>");
    let make = |version| {
        document("urn:iteration-style", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:param name="v" select="'outer'"/><xsl:template match="/"><xsl:for-each select="r/a"><xsl:variable name="v" select="'inner'"/><xsl:value-of select="$v"/><xsl:value-of select="position()"/><xsl:for-each select="b"><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>;</xsl:text></xsl:for-each><xsl:value-of select="position()"/></xsl:for-each><xsl:value-of select="$v"/><xsl:value-of select="last()"/><xsl:for-each select="missing"><xsl:text>BAD</xsl:text></xsl:for-each></xsl:template></xsl:stylesheet>"#).as_bytes())
    };
    let reference = compiler::compile_stylesheet(&make("1.0")).unwrap();
    let mut baseline = InvocationControl::unbounded();
    let expected = execute(&reference, &source, "reference", &mut baseline).unwrap();
    assert_eq!(expected, "inner11/2;2/2;1inner21/1;2outer1");
    for version in ["1.0", "2.0", "3.0"] {
        let program = compile(
            &make(version),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&program, &source, "selected", &mut actual).unwrap(),
            expected
        );
        for domain in [
            WorkDomain::XPathNodeVisit,
            WorkDomain::XsltInstruction,
            WorkDomain::ResultNode,
            WorkDomain::ResultTextByte,
        ] {
            assert_eq!(actual.consumed(domain), baseline.consumed(domain));
        }
        let mut limits = WorkLimits::unbounded();
        limits.xslt_instructions = actual.consumed(WorkDomain::XsltInstruction);
        assert_eq!(
            execute(
                &program,
                &source,
                "exact",
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap(),
            expected
        );
        limits.xslt_instructions -= 1;
        assert!(
            execute(
                &program,
                &source,
                "short",
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap_err()
            .contains("FXCT0002")
        );
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XsltInstruction, 3);
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
            expected
        );
    }
}

#[test]
fn selected_child_iteration_keeps_other_selections_and_sorting_guarded() {
    for iteration in [
        r#"<xsl:for-each select="r/a[1]"/>"#,
        r#"<xsl:for-each select="$v"/>"#,
        r#"<xsl:for-each select="1 to 3"/>"#,
        r#"<xsl:for-each select="r/a" default-mode="m"/>"#,
        r#"<xsl:for-each select="r/a"><xsl:sort select="."/></xsl:for-each>"#,
    ] {
        let style = document("urn:iteration-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/">{iteration}</xsl:template></xsl:stylesheet>"#).as_bytes());
        assert!(compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).is_err());
    }
}

#[test]
fn selected_literal_xml_space_inherits_resets_and_retains_attributes() {
    let source = document("urn:space-source", b"<r/>");
    let expected = br#"<out xml:space="preserve"> <keep> </keep><reset xml:space="default"><empty/><kept xml:space="preserve"> </kept></reset> </out>"#;
    for version in ["1.0", "2.0", "3.0"] {
        let style = document("urn:space-style", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:template match="/"><out xml:space="preserve"> <keep> </keep><reset xml:space="default"> <empty> </empty><kept xml:space="preserve"> </kept> </reset> </out></xsl:template></xsl:stylesheet>"#).as_bytes());
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let actual = execute_compiled_stylesheet_for_test(&program, &source, "space").unwrap();
        assert_eq!(
            crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                &actual,
                expected,
                true,
                None,
                Some("xml")
            ),
            Ok(true)
        );
    }
    for attribute in [
        r#"xml:space="other""#,
        r#"xml:space="{'preserve'}""#,
        r#"xml:lang="en""#,
    ] {
        let style = document("urn:space-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/"><out {attribute}/></xsl:template></xsl:stylesheet>"#).as_bytes());
        assert!(compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).is_err());
    }
}

fn stylesheet(version: &str, declarations: &str) -> Document {
    document("urn:selected-whitespace", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output omit-xml-declaration="yes"/>{declarations}<xsl:template match="r"><out><xsl:copy-of select="strip"/><xsl:copy-of select="keep"/><xsl:copy-of select="protected"/></out></xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_source_whitespace_preserves_effective_copy_controls_and_input() {
    let source = document("urn:prepared", br#"<r><strip> <a/> </strip><keep> <b/> </keep><protected xml:space="preserve"> <c/> <reset xml:space="default"> <d/> </reset> </protected></r>"#);
    let original_value = source.string_value(source.document_node());
    let rules = r#"<xsl:strip-space elements="*"/><xsl:preserve-space elements="keep"/>"#;
    let reference = compiler::compile_stylesheet(&stylesheet("1.0", rules)).unwrap();
    let mut baseline = InvocationControl::unbounded();
    let expected = execute(&reference, &source, "reference", &mut baseline).unwrap();
    assert_eq!(crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(&expected, br#"<out><strip><a/></strip><keep> <b/> </keep><protected xml:space="preserve"> <c/> <reset xml:space="default"><d/></reset> </protected></out>"#, true, None, Some("xml")), Ok(true));
    for version in ["1.0", "2.0", "3.0"] {
        let style = stylesheet(version, rules);
        let selected =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(style);
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&selected, &source, "selected", &mut actual).unwrap(),
            expected
        );
        for domain in [
            WorkDomain::XdmNode,
            WorkDomain::XPathNodeVisit,
            WorkDomain::XsltInstruction,
            WorkDomain::ResultNode,
            WorkDomain::ResultTextByte,
        ] {
            assert_eq!(actual.consumed(domain), baseline.consumed(domain));
        }
        let required = actual.consumed(WorkDomain::XdmNode);
        assert!(required > 0);
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = required;
        assert_eq!(
            execute(
                &selected,
                &source,
                "exact",
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap(),
            expected
        );
        limits.xdm_nodes -= 1;
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
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 1);
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
            expected
        );
        assert_eq!(source.string_value(source.document_node()), original_value);
        let preserve = compile(
            &stylesheet(version, ""),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_ne!(
            execute_compiled_stylesheet_for_test(&preserve, &source, "preserve").unwrap(),
            expected
        );
    }
}

#[test]
fn selected_source_whitespace_rejects_unbound_names_and_other_controls() {
    for declaration in [
        r#"<xsl:strip-space elements="p:*"/>"#,
        r#"<xsl:preserve-space elements="p:item"/>"#,
        r#"<xsl:strip-space elements="*" version="2.0"/>"#,
    ] {
        assert!(
            compile(
                &stylesheet("2.0", declaration),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
    assert!(
        compile(
            &stylesheet("3.0", r#"<xsl:strip-space elements="*"/>"#),
            Edition::Modern,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_whitespace_mismatches_keep_original_oracles() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    )
    .join("Xalan_Conformance_Tests");
    for (index, modern_exact) in [(17, false), (21, true)] {
        let case = format!("whitespace{index:02}");
        let style = document(
            &format!("urn:{case}:style"),
            &std::fs::read(root.join(format!("whitespace/{case}.xsl"))).unwrap(),
        );
        let source = document(
            &format!("urn:{case}:source"),
            &std::fs::read(root.join(format!("whitespace/{case}.xml"))).unwrap(),
        );
        let expected = std::fs::read(root.join(format!("REF_OUT/whitespace/{case}.out"))).unwrap();
        let modern = compiler::compile_stylesheet(&style).unwrap();
        let strict = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        for (program, exact) in [(&modern, modern_exact), (&strict, false)] {
            let actual = execute_compiled_stylesheet_for_test(program, &source, &case).unwrap();
            assert_eq!(
                crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                    &actual,
                    &expected,
                    true,
                    None,
                    Some("xml")
                ),
                Ok(exact),
                "{case}: {actual:?}"
            );
        }
    }
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_replays_source_whitespace_declarations() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    )
    .join("Xalan_Conformance_Tests");
    for index in 1..=7 {
        let case = format!("whitespace{index:02}");
        let style = document(
            &format!("https://oasis.invalid/whitespace/{case}.xsl"),
            &std::fs::read(root.join(format!("whitespace/{case}.xsl"))).unwrap(),
        );
        let source = document(
            &format!("https://oasis.invalid/whitespace/{case}.xml"),
            &std::fs::read(root.join(format!("whitespace/{case}.xml"))).unwrap(),
        );
        let expected = std::fs::read(root.join(format!("REF_OUT/whitespace/{case}.out"))).unwrap();
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let actual = execute_compiled_stylesheet_for_test(&program, &source, &case).unwrap();
        assert_eq!(
            crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                &actual,
                &expected,
                true,
                None,
                Some("xml")
            ),
            Ok(true),
            "{case}"
        );
    }
}

#[test]
fn selected_source_whitespace_matches_expanded_names_not_prefix_spelling() {
    let source = document("urn:namespace-source", br#"<r xmlns:q="urn:target" xmlns:p="urn:other"><q:a> <leaf>A</leaf> </q:a><q:b> <leaf>B</leaf> </q:b><p:a> <leaf>C</leaf> </p:a></r>"#);
    for (names, expected) in [("p:a", "A B  C "), ("p:*", "AB C ")] {
        for version in ["1.0", "2.0", "3.0"] {
            let style = document("urn:namespace-style", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:target" version="{version}"><xsl:output method="text"/><xsl:strip-space elements="{names}"/><xsl:template match="r"><xsl:value-of select="."/></xsl:template></xsl:stylesheet>"#).as_bytes());
            let program =
                compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
            assert_eq!(
                execute_compiled_stylesheet_for_test(&program, &source, "namespaces").unwrap(),
                expected
            );
        }
    }
}
