//! Single-owner iteration, focus restoration, sorting and control conservation.

use super::*;

fn iteration_fixture() -> (StylesheetProgram, ResourceSnapshot, Document) {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    builder
        .admit(
            STYLE,
            include_bytes!("../../../../../corpus/golden/retained-source-for-each/stylesheet.xsl")
                .to_vec(),
        )
        .unwrap();
    builder
        .admit(
            EXTRA,
            include_bytes!("../../../../../corpus/golden/retained-source-for-each/secondary.xml")
                .to_vec(),
        )
        .unwrap();
    let snapshot = builder.seal();
    let source = Document::from_parsed(
        parse_document(
            "urn:retained-iteration:principal",
            include_bytes!("../../../../../corpus/golden/retained-source-for-each/source.xml"),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap();
    (
        compile_resource(&snapshot, STYLE).unwrap(),
        snapshot,
        source,
    )
}

#[test]
fn retained_source_iteration_golden_preserves_owner_sort_focus_and_frame_parity() {
    let fixture = iteration_fixture();
    let expected =
        include_str!("../../../../../corpus/golden/retained-source-for-each/expected.xml").trim();
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
                expected
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
                    expected
                );
            });
        }
    });
}

#[test]
fn retained_source_iteration_controls_and_unqualified_transfer_remain_explicit() {
    let fixture = iteration_fixture();
    let mut baseline = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    let consumed = baseline.consumed(WorkDomain::XdmNode);
    for limit in [consumed - 1, consumed] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xdm_nodes: limit,
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
    let mut control = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 3);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut control
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    // Principal globals are retained; bare lexical selections still cannot
    // cross a focus switch merely because another variable has an owner.
    let returned = fixture_with_external_body(
        true,
        r#"<xsl:variable name="bare" select="external/value"/><xsl:for-each select="$principal"><xsl:value-of select="."/></xsl:for-each>"#,
    );
    assert_eq!(
        run(
            &returned,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT1017"
    );
    let denied = HashSet::from([EXTRA.to_owned()]);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            Some(&denied),
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0003"
    );
}

#[cfg(feature = "allocation-observation")]
#[test]
fn retained_source_iteration_releases_success_cancelled_and_exhausted_ownership() {
    let fixture = iteration_fixture();
    let mut baseline = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    let late_cancel = baseline.consumed(WorkDomain::XPathNodeVisit) - 1;
    for phase in 0..3 {
        let info = allocation_counter::measure(|| {
            let mut control = match phase {
                1 => InvocationControl::unbounded()
                    .cancelling_on_charge(WorkDomain::XPathNodeVisit, late_cancel),
                2 => InvocationControl::new(
                    CancellationToken::new(),
                    WorkLimits {
                        xdm_nodes: 3,
                        ..WorkLimits::unbounded()
                    },
                ),
                _ => InvocationControl::unbounded(),
            };
            let outcome = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control,
            );
            assert_eq!(outcome.is_ok(), phase == 0);
            if phase == 1 {
                assert_eq!(outcome.unwrap_err().code, "FXCT0001");
            }
        });
        assert_eq!(info.bytes_current, 0);
        assert_eq!(info.count_current, 0);
    }
}
