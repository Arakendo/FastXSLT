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
fn namespace_copy_of_keeps_mixed_and_explicit_base_selections_unsupported() {
    for select in [
        "r/namespace::* | r",
        "document('extra.xml', /)/r/namespace::*",
    ] {
        let stylesheet = document(
            "urn:namespace:copy-style",
            &format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><out><xsl:copy-of select="{select}"/></out></xsl:template></xsl:stylesheet>"#,
            ),
        );
        let failure = compile_stylesheet(&stylesheet).unwrap_err();
        assert_eq!(
            failure.category,
            crate::compile::golden_stylesheet_experiment::CompileCategory::Unsupported
        );
        assert_eq!(failure.location.resource, "urn:namespace:copy-style");
    }
    let literal = program(
        "1.0",
        r#"<out><xsl:copy-of select="'namespace::p'"/></out>"#,
    );
    assert_eq!(
        run(
            &literal,
            &document("urn:namespace:copy-source", "<r/>"),
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap(),
        "<out>namespace::p</out>"
    );
}

#[test]
fn namespace_document_copy_reuses_owner_switch_and_attachment_controls() {
    use std::collections::HashSet;
    const STYLE: &str = "https://example.test/ns/style.xsl";
    const EXTRA: &str = "https://example.test/ns/extra.xml";
    let source = document("urn:namespace:principal", r#"<r xmlns:p="urn:wrong"/>"#);
    for (selection, expected) in [
        (
            "document('extra.xml')/r/namespace::p",
            "<out xmlns:p=\"urn:right\"></out>",
        ),
        (
            "document('extra.xml')/r/a/namespace::p",
            "<out xmlns:p=\"urn:shadow\"></out>",
        ),
        (
            "document('extra.xml')/r/namespace::p[name()='p']",
            "<out xmlns:p=\"urn:right\"></out>",
        ),
        ("document('extra.xml')/r/namespace::missing", "<out></out>"),
        ("document('extra.xml')/missing/namespace::p", "<out></out>"),
        (
            "document('')/xsl:stylesheet/namespace::xsl",
            "<out xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\"></out>",
        ),
    ] {
        let (program, snapshot) = document_copy_fixture(STYLE, EXTRA, selection);
        let execute =
            |representation, denied: Option<&HashSet<String>>, control: &mut InvocationControl| {
                let result = execute_program_with_parameters_using(
                    &program,
                    &source,
                    &BTreeMap::new(),
                    MultipleMatchPolicy::UseLast,
                    "namespace-document-copy",
                    representation,
                    Some(&snapshot),
                    denied,
                    control,
                )?;
                serialize_xml(
                    &result,
                    &program.output,
                    "namespace-document-copy",
                    8192,
                    control,
                )
            };
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            for complete in [false, true] {
                let mut control = InvocationControl::unbounded();
                if complete {
                    control = control
                        .with_complete_sequence_frame_clones()
                        .with_complete_atomic_frame_clones();
                }
                assert_eq!(
                    execute(representation, None, &mut control).unwrap(),
                    expected
                );
            }
        }
        check_document_copy_controls(
            |control| execute(WhitespaceRepresentation::VisibilityView, None, control),
            expected,
        );
        let denied = HashSet::from([if selection.contains("document('')") {
            STYLE
        } else {
            EXTRA
        }
        .to_owned()]);
        assert_eq!(
            execute(
                WhitespaceRepresentation::VisibilityView,
                Some(&denied),
                &mut InvocationControl::unbounded()
            )
            .unwrap_err()
            .code,
            "FXRS0003"
        );
        assert_eq!(
            execute(
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
    }
}

fn document_copy_fixture(
    style_id: &str,
    extra_id: &str,
    selection: &str,
) -> (super::StylesheetProgram, crate::resources::ResourceSnapshot) {
    use crate::resources::{ResourceLimits, ResourceSetBuilder};
    let style = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:strip-space elements="*"/><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:copy-of select="{selection}"/></out></xsl:template></xsl:stylesheet>"#
    );
    let program = compile_stylesheet(&document(style_id, &style)).unwrap();
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 4096, 8192));
    builder.admit(style_id, style.into_bytes()).unwrap();
    builder
        .admit(
            extra_id,
            br#"<r xmlns:p="urn:right"> <a xmlns:p="urn:shadow"/> </r>"#.to_vec(),
        )
        .unwrap();
    (program, builder.seal())
}

#[test]
fn namespace_document_copy_results_outlive_concurrent_invocations_and_resources() {
    let (program, snapshot) = document_copy_fixture(
        "https://example.test/ns/style.xsl",
        "https://example.test/ns/extra.xml",
        "document('extra.xml')/r/namespace::p",
    );
    let source = document("urn:namespace:principal", r#"<r xmlns:p="urn:wrong"/>"#);
    let results = std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    execute_program_with_parameters_using(
                        &program,
                        &source,
                        &BTreeMap::new(),
                        MultipleMatchPolicy::UseLast,
                        "namespace-document-copy",
                        WhitespaceRepresentation::VisibilityView,
                        Some(&snapshot),
                        None,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    let output = program.output.clone();
    drop((program, source, snapshot));
    for result in results {
        assert_eq!(
            serialize_xml(
                &result,
                &output,
                "after-release",
                8192,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "<out xmlns:p=\"urn:right\"></out>"
        );
    }
}

fn check_document_copy_controls(
    mut execute: impl FnMut(&mut InvocationControl) -> Result<String, super::ExecutionFailure>,
    expected: &str,
) {
    let mut measured = InvocationControl::unbounded();
    assert_eq!(execute(&mut measured).unwrap(), expected);
    for domain in [WorkDomain::XPathNodeVisit, WorkDomain::ResultTextByte] {
        let charge = measured.consumed(domain);
        if charge == 0 {
            assert_eq!(domain, WorkDomain::ResultTextByte);
            assert_eq!(expected, "<out></out>");
            continue;
        }
        for allowance in [charge, charge - 1] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XPathNodeVisit => limits.xpath_node_visits = allowance,
                WorkDomain::ResultTextByte => limits.result_text_bytes = allowance,
                _ => unreachable!(),
            }
            let result = execute(&mut InvocationControl::new(
                CancellationToken::new(),
                limits,
            ));
            if allowance == charge {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert_eq!(result.unwrap_err().code, "FXCT0002");
            }
        }
        assert_eq!(
            execute(&mut InvocationControl::unbounded().cancelling_on_charge(domain, 0))
                .unwrap_err()
                .code,
            "FXCT0001"
        );
    }
}

#[test]
fn namespace_copy_of_reuses_qualified_copy_and_preserves_control() {
    let source = document(
        "urn:namespace:copy-source",
        r#"<r xmlns:p="urn:p" xmlns:q="urn:q"> <a/> </r>"#,
    );
    for version in ["1.0", "3.0"] {
        let direct = program(
            version,
            r#"<out><xsl:copy-of select="r/namespace::*"/><child/></out>"#,
        );
        let reference = program(
            version,
            r#"<out><xsl:for-each select="r/namespace::*"><xsl:copy/></xsl:for-each><child/></out>"#,
        );
        let mut measured = InvocationControl::unbounded();
        let actual = run(
            &direct,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut measured,
        )
        .unwrap();
        assert_eq!(
            actual,
            run(
                &reference,
                &source,
                WhitespaceRepresentation::CompleteReference,
                &mut InvocationControl::unbounded(),
            )
            .unwrap()
        );
        for domain in [WorkDomain::XPathNodeVisit, WorkDomain::ResultTextByte] {
            let charge = measured.consumed(domain);
            assert!(charge > 0);
            for allowance in [charge - 1, charge] {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::XPathNodeVisit => limits.xpath_node_visits = allowance,
                    WorkDomain::ResultTextByte => limits.result_text_bytes = allowance,
                    _ => unreachable!("bounded namespace copy control domains"),
                }
                let outcome = run(
                    &direct,
                    &source,
                    WhitespaceRepresentation::VisibilityView,
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                );
                if allowance == charge {
                    assert_eq!(outcome.unwrap(), actual);
                } else {
                    assert_eq!(outcome.unwrap_err().code, "FXCT0002");
                }
            }
        }
        let error = run(
            &direct,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0),
        )
        .unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert_eq!(
            run(
                &direct,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded(),
            )
            .unwrap(),
            actual
        );
        let late = program(
            version,
            r#"<out>child<xsl:copy-of select="r/namespace::p"/></out>"#,
        );
        let error = run(
            &late,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(error.code, "XTDE0410");
        assert_eq!(error.location.unwrap().resource, "urn:namespace:copy-style");
    }
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
