//! Empty foreign branches earn no cross-document identity/order promise.

use super::*;

fn copy_union_fixture(
    reference: &str,
    path: &str,
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let style = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:template match="/"><out><xsl:copy-of select="document('{reference}'){path} | external/value[2] | external/value"/></out></xsl:template></xsl:stylesheet>"#
    );
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    resources.admit(STYLE, style.into_bytes()).unwrap();
    resources
        .admit(
            EXTRA,
            b"<external> <value>foreign</value> </external>".to_vec(),
        )
        .unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLE).unwrap();
    let source = Document::from_parsed(
        parse_document(
            "urn:copy-union:source",
            b"<external> <value>B</value><value>A</value> </external>",
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap();
    (program, snapshot, source)
}

#[test]
fn empty_document_copy_union_preserves_order_dedup_views_and_owned_results() {
    let fixture = copy_union_fixture("extra.xml", "/absent");
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        for complete in [false, true] {
            let mut control = InvocationControl::unbounded();
            if complete {
                control = control.with_complete_sequence_frame_clones();
            }
            assert_eq!(
                run(&fixture, representation, None, &mut control).unwrap(),
                "<out><value>B</value><value>A</value></out>"
            );
        }
    }
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                assert_eq!(
                    run(
                        &fixture,
                        WhitespaceRepresentation::VisibilityView,
                        None,
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    "<out><value>B</value><value>A</value></out>"
                );
            });
        }
    });
}

#[test]
fn copied_empty_document_union_result_outlives_source_snapshot_and_program() {
    let (result, output) = {
        let (program, snapshot, source) = copy_union_fixture("extra.xml", "/absent");
        let result = execute_program_with_parameters_using(
            &program,
            &source,
            &BTreeMap::new(),
            MultipleMatchPolicy::UseLast,
            "bound-document",
            WhitespaceRepresentation::VisibilityView,
            Some(&snapshot),
            None,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        (result, program.output.clone())
    };
    assert_eq!(
        serialize_xml(
            &result,
            &output,
            "bound-document",
            8192,
            &mut InvocationControl::unbounded(),
        )
        .unwrap(),
        "<out><value>B</value><value>A</value></out>"
    );
}

#[test]
fn empty_document_copy_union_retains_authority_foreign_guard_and_controls() {
    let fixture = copy_union_fixture("extra.xml", "/absent");
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            Some(&HashSet::from([EXTRA.to_owned()])),
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0003"
    );
    let missing = copy_union_fixture("missing.xml", "/absent");
    assert_eq!(
        run(
            &missing,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0001"
    );
    let foreign = copy_union_fixture("extra.xml", "/external/value");
    assert_eq!(
        run(
            &foreign,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT1017"
    );
    let mut baseline = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    let consumed = baseline.consumed(WorkDomain::XPathNodeVisit);
    for limit in [consumed - 1, consumed] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xpath_node_visits: limit,
                ..WorkLimits::unbounded()
            },
        );
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control
            )
            .is_ok(),
            limit == consumed
        );
    }
    let mut cancelled = InvocationControl::unbounded()
        .cancelling_on_charge(WorkDomain::XPathNodeVisit, consumed - 1);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut cancelled
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
}

#[cfg(feature = "allocation-observation")]
#[test]
fn empty_document_copy_union_releases_success_rejection_and_cancelled_state() {
    for (path, cancel) in [
        ("/absent", false),
        ("/external/value", false),
        ("/absent", true),
    ] {
        let fixture = copy_union_fixture("extra.xml", path);
        let mut control = InvocationControl::unbounded();
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::XPathNodeVisit, 4);
        }
        let info = allocation_counter::measure(|| {
            let _outcome = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control,
            );
        });
        assert_eq!(info.bytes_current, 0);
        assert_eq!(info.count_current, 0);
    }
}

#[test]
fn empty_document_copy_union_preserves_result_budget_and_error_locations() {
    let fixture = copy_union_fixture("extra.xml", "/absent");
    let mut baseline = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    let consumed = baseline.consumed(WorkDomain::ResultNode);
    assert!(consumed > 0);
    for limit in [consumed - 1, consumed] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                result_nodes: limit,
                ..WorkLimits::unbounded()
            },
        );
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control
            )
            .is_ok(),
            limit == consumed
        );
    }
    for (reference, path, code) in [
        ("missing.xml", "/absent", "FXRS0001"),
        ("extra.xml", "/external/value", "FXRT1017"),
    ] {
        let fixture = copy_union_fixture(reference, path);
        let failure = run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(failure.code, code);
        assert!(failure.location.is_some());
    }
}

fn union_fixture(path: &str) -> (StylesheetProgram, ResourceSnapshot, Document) {
    fixture_with_templates(
        false,
        &format!(
            r#"<out><xsl:apply-templates select="external/value | document('extra.xml'){path} | external/value" mode="values"/></out>"#
        ),
        r#"<xsl:template match="value" mode="values"><value><xsl:value-of select="."/></value></xsl:template>"#,
    )
}

#[test]
fn empty_document_union_preserves_local_order_and_dedup_and_rejects_nonempty_foreign_nodes() {
    let fixture = union_fixture("/absent");
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        assert_eq!(
            run(
                &fixture,
                representation,
                None,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "<out><value>right</value></out>"
        );
    }
    let nonempty = union_fixture("/external/value");
    assert_eq!(
        run(
            &nonempty,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT1017"
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out><value>right</value></out>"
    );
}

#[test]
fn empty_document_union_preserves_resolution_failure_and_work_limits() {
    let fixture = fixture_with_templates(
        false,
        r#"<xsl:apply-templates select="external/value | document('missing.xml')/absent"/>"#,
        "",
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0001"
    );
    let fixture = union_fixture("/absent");
    let mut baseline = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    let consumed = baseline.consumed(WorkDomain::XPathNodeVisit);
    for limit in [consumed - 1, consumed] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xpath_node_visits: limit,
                ..WorkLimits::unbounded()
            },
        );
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control
            )
            .is_ok(),
            limit == consumed
        );
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 4);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut cancelled
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
}
