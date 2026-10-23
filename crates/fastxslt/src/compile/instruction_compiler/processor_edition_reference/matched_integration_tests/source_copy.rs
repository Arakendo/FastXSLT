use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

fn style(version: &str, select: &str) -> Document {
    document("urn:selected-copy", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="r"><out><xsl:copy-of select="{select}"/></out></xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_child_copy_preserves_shared_subtrees_namespaces_and_controls() {
    let source = document("urn:copy-source", br#"<r><num xmlns:p="urn:p" a="one"><p:child/>text<!--comment--><?pi value?></num><other/><num>second</num></r>"#);
    let baseline = compiler::compile_stylesheet(&style("1.0", "num")).unwrap();
    let mut reference = InvocationControl::unbounded();
    let expected = execute(&baseline, &source, "same", &mut reference).unwrap();
    assert!(
        expected.contains("<!--comment-->")
            && expected.contains("<?pi value?>")
            && expected.contains("second")
    );
    assert!(!expected.contains("other"));
    for version in ["1.0", "2.0", "3.0"] {
        let sheet = style(version, "num");
        let selected =
            compile(&sheet, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(sheet);
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&selected, &source, "same", &mut actual).unwrap(),
            expected
        );
        for domain in [
            WorkDomain::XPathNodeVisit,
            WorkDomain::XsltInstruction,
            WorkDomain::ResultNode,
            WorkDomain::ResultTextByte,
        ] {
            assert_eq!(actual.consumed(domain), reference.consumed(domain));
        }
        let required = actual.consumed(WorkDomain::ResultNode);
        assert!(required > 0);
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = required;
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
        limits.result_nodes -= 1;
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
        let mut cancel =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultNode, 1);
        assert!(
            execute(&selected, &source, "cancel", &mut cancel)
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
fn selected_child_copy_keeps_general_copy_routes_guarded() {
    for select in [
        "@a",
        ".",
        "*",
        "p:num",
        "$v",
        "num[1]",
        "num/child",
        "namespace::*",
        "document('x')",
    ] {
        let Err(StrictPreprocessingFailure::Compile(error)) = compile(
            &style("2.0", select),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("must reject {select}")
        };
        assert_eq!(error.code, "FXAR0031");
        assert_eq!(error.location.resource, "urn:selected-copy");
    }
    assert!(
        compile(
            &style("3.0", "num"),
            Edition::Modern,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}
