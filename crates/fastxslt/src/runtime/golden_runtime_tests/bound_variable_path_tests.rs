//! Compiled single-owner grouped filters and retained attribute-path values.

use super::*;

#[test]
fn grouped_variable_scalars_keep_the_actual_document_owner_and_lexical_scope() {
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:variable name="alias" select="$doc"/><out><xsl:value-of select="($doc)"/>:<xsl:value-of select="$alias"/>:<xsl:value-of select="($principal)"/><nested><xsl:variable name="doc" select="'shadow'"/><xsl:value-of select="( $doc )"/></nested><xsl:value-of select="($doc)"/></out>"#,
    );
    let expected = "<out>right:right:wrong<nested>shadow</nested>right</out>";
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
                run(&fixture, representation, None, &mut control).unwrap(),
                expected
            );
        }
    }
    std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    run(
                        &fixture,
                        WhitespaceRepresentation::VisibilityView,
                        None,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}

#[test]
fn grouped_document_scalar_preserves_exact_controls_and_diagnostics() {
    let fixture =
        fixture_with_external_body(false, r#"<out><xsl:value-of select="($doc)"/></out>"#);
    let mut baseline = InvocationControl::unbounded();
    let expected = run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    assert_eq!(expected, "<out>right</out>");
    for domain in [
        WorkDomain::XdmStringValueNode,
        WorkDomain::XdmNode,
        WorkDomain::ResultTextByte,
    ] {
        let charge = baseline.consumed(domain);
        assert!(charge > 0);
        for allowance in [charge, charge - 1] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = allowance,
                WorkDomain::XdmNode => limits.xdm_nodes = allowance,
                WorkDomain::ResultTextByte => limits.result_text_bytes = allowance,
                _ => unreachable!(),
            }
            let result = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance == charge {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert_eq!(result.unwrap_err().code, "FXCT0002");
            }
        }
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::unbounded().cancelling_on_charge(domain, 0)
            )
            .unwrap_err()
            .code,
            "FXCT0001"
        );
    }
    let absent = fixture_with_external_body(false, r#"<xsl:value-of select="($missing)"/>"#);
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
    assert_eq!(
        run(
            &absent,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT0002"
    );
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
}

fn lookup_fixture(language: &str) -> (StylesheetProgram, ResourceSnapshot, Document) {
    lookup_fixture_with_select(
        language,
        "($terms/table/expand[@language=$lang]/gentext[@key=$key])[1]",
    )
}

fn lookup_fixture_with_select(
    language: &str,
    select: &str,
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let (_, _, source) = fixture(false);
    let style = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:param name="terms" select="document('extra.xml')"/><xsl:template match="/"><xsl:variable name="lang" select="{language}"/><xsl:variable name="key" select="'title'"/><xsl:variable name="lookup" select="{select}"/><xsl:variable name="text" select="$lookup/@text"/><xsl:variable name="alias" select="$text"/><out><xsl:value-of select="$alias"/>:<xsl:value-of select="$lookup/@text"/><xsl:apply-templates select="$lookup" mode="lookup"/></out></xsl:template><xsl:template match="gentext" mode="lookup"><selected><xsl:value-of select="@text"/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/></selected></xsl:template></xsl:stylesheet>"#
    );
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    builder.admit(STYLE, style.into_bytes()).unwrap();
    builder.admit(EXTRA, br#"<table> <expand><gentext key="title" text="absent"/></expand><expand language=""><gentext key="title" text="empty"/></expand><expand language="fr"><gentext key="title" text="wrong"/></expand><expand language="en"><gentext key="other" text="wrong"/><gentext key="title" text="first"/><gentext key="title" text="second"/></expand><expand language="en"><gentext key="title" text="third"/></expand> </table>"#.to_vec()).unwrap();
    let snapshot = builder.seal();
    (
        compile_resource(&snapshot, STYLE).unwrap(),
        snapshot,
        source,
    )
}

#[test]
fn bound_variable_path_selects_first_across_parents_and_retains_attributes() {
    let fixture = lookup_fixture("'en'");
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
                "<out>first:first<selected>first:1/1</selected></out>"
            );
        }
    }
    let missing = lookup_fixture("'missing'");
    assert_eq!(
        run(
            &missing,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>:</out>"
    );
    let empty = lookup_fixture("''");
    assert_eq!(
        run(
            &empty,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>empty:empty<selected>empty:1/1</selected></out>"
    );
    std::thread::scope(|scope| {
        let threads = [&fixture, &missing, &fixture, &missing].map(|fixture| {
            scope.spawn(move || {
                run(
                    fixture,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            })
        });
        for (index, thread) in threads.into_iter().enumerate() {
            assert_eq!(
                thread.join().unwrap(),
                if index % 2 == 0 {
                    "<out>first:first<selected>first:1/1</selected></out>"
                } else {
                    "<out>:</out>"
                }
            );
        }
    });
}

#[test]
fn bound_variable_path_keeps_non_string_predicates_unsupported_and_controls_exact() {
    for select in [
        "($terms/table/expand[@language=$absent]/gentext)[1]",
        "($absent/table/expand)[1]",
    ] {
        let fixture = lookup_fixture_with_select("'en'", select);
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err()
            .code,
            "FXRT0002"
        );
    }
    let numeric = lookup_fixture("1");
    assert_eq!(
        run(
            &numeric,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT1017"
    );
    let fixture = lookup_fixture("'en'");
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
fn bound_variable_path_releases_success_and_partial_failure_allocations() {
    for language in ["'en'", "1", "'missing'"] {
        let fixture = lookup_fixture(language);
        for cancelled in [false, true] {
            let info = allocation_counter::measure(|| {
                let mut control = InvocationControl::unbounded();
                if cancelled {
                    control = control.cancelling_on_charge(WorkDomain::XPathNodeVisit, 20);
                }
                let outcome = run(
                    &fixture,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut control,
                );
                if !cancelled {
                    assert_eq!(outcome.is_ok(), language != "1");
                }
            });
            assert_eq!(info.bytes_current, 0);
            assert_eq!(info.count_current, 0);
        }
    }
}
