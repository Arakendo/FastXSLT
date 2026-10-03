//! Namespace xsl:copy attachment through compiled execution, not a second evaluator.

use super::{
    BTreeMap, CancellationToken, InvocationControl, MultipleMatchPolicy, WhitespaceRepresentation,
    WorkDomain, WorkLimits, compile_stylesheet, document, execute_program_with_parameters_using,
    run, serialize_xml,
};

fn program(version: &str, body: &str) -> super::StylesheetProgram {
    compile_stylesheet(&document("urn:namespace:copy-style", &format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:strip-space elements="*"/><xsl:output omit-xml-declaration="yes"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
    ))).unwrap()
}

#[test]
fn namespace_copy_attaches_owned_bindings_deduplicates_and_outlives_owners() {
    for version in ["1.0", "3.0"] {
        let (result, output) = {
            let source = document(
                "urn:namespace:copy-source",
                r#"<r xmlns:p="urn:p" xmlns:q="urn:q"> <a/> </r>"#,
            );
            let program = program(
                version,
                r#"<out><xsl:for-each select="r/namespace::*"><xsl:sort select="local-name(.)"/><xsl:copy/></xsl:for-each><xsl:for-each select="r/namespace::p"><xsl:copy/></xsl:for-each><child/></out>"#,
            );
            let execute = |representation| {
                execute_program_with_parameters_using(
                    &program,
                    &source,
                    &BTreeMap::new(),
                    MultipleMatchPolicy::UseLast,
                    "namespace-copy",
                    representation,
                    None,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            };
            let result = execute(WhitespaceRepresentation::VisibilityView);
            assert_eq!(result, execute(WhitespaceRepresentation::CompleteReference));
            let super::ResultNode::Element {
                namespaces,
                attributes,
                ..
            } = &result.children[0]
            else {
                panic!("element")
            };
            assert!(attributes.is_empty());
            assert_eq!(
                namespaces
                    .iter()
                    .map(|binding| binding.prefix.as_deref())
                    .collect::<Vec<_>>(),
                [Some("p"), Some("q")]
            );
            (result, program.output.clone())
        };
        let serialized = serialize_xml(
            &result,
            &output,
            "namespace-copy",
            8192,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert!(serialized.contains("xmlns:p=\"urn:p\""));
        assert!(serialized.contains("xmlns:q=\"urn:q\""));
        assert!(!serialized.contains("xmlns:xml"));
        document("urn:namespace:copy-result", &serialized);
    }
}

#[test]
fn namespace_copy_construction_failures_preserve_copy_site_and_profile() {
    let source = document(
        "urn:namespace:copy-source",
        r#"<r xmlns="urn:default" xmlns:p="urn:p"/>"#,
    );
    for version in ["1.0", "3.0"] {
        for (body, code) in [
            (
                r#"<out>child<xsl:for-each select="/*/namespace::p"><xsl:copy/></xsl:for-each></out>"#,
                "XTDE0410",
            ),
            (
                r#"<out xmlns:p="urn:other"><xsl:for-each select="/*/namespace::p"><xsl:copy/></xsl:for-each></out>"#,
                "XTDE0430",
            ),
            (
                r#"<out><xsl:for-each select="/*/namespace::*[name()='']"><xsl:copy/></xsl:for-each></out>"#,
                "XTDE0440",
            ),
            (
                r#"<xsl:for-each select="/*/namespace::p"><xsl:copy/></xsl:for-each>"#,
                "XTDE0420",
            ),
        ] {
            let error = run(
                &program(version, body),
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded(),
            )
            .unwrap_err();
            assert_eq!(error.code, code, "{version}: {body}");
            let location = error.location.unwrap();
            assert_eq!(location.resource, "urn:namespace:copy-style");
            assert!(location.span.start < location.span.end);
        }
    }
}

#[test]
fn namespace_copy_allocation_charges_cancel_and_exhaust_without_poisoning_reuse() {
    let source = document(
        "urn:namespace:copy-source",
        r#"<r xmlns:p="urn:p" xmlns:q="urn:q"/>"#,
    );
    let program = program(
        "1.0",
        r#"<out><xsl:for-each select="r/namespace::*"><xsl:copy/></xsl:for-each></out>"#,
    );
    let mut limits = WorkLimits::unbounded();
    limits.result_text_bytes = 0;
    let mut exhausted = InvocationControl::new(CancellationToken::new(), limits);
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut exhausted,
    )
    .unwrap_err();
    assert_eq!(error.code, "FXCT0002");
    assert_eq!(error.work_domain, Some(WorkDomain::ResultTextByte));
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0);
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut cancelled
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    let restored = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert!(restored.contains("xmlns:p=\"urn:p\""));
    assert!(restored.contains("xmlns:q=\"urn:q\""));
}

#[test]
fn namespace_copy_top_level_is_rejected_before_every_output_method() {
    let source = document("urn:namespace:copy-source", r#"<r xmlns:p="urn:p"/>"#);
    for method in ["xml", "text", "html"] {
        let mut program = program(
            "1.0",
            r#"<xsl:for-each select="r/namespace::p"><xsl:copy/></xsl:for-each>"#,
        );
        program.output.method = Some(method.to_owned());
        let error = run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(error.code, "XTDE0420", "{method}");
        assert_eq!(error.location.unwrap().resource, "urn:namespace:copy-style");
    }
}
