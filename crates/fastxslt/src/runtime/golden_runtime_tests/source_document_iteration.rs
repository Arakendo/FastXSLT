//! Zero/single resolved documents, lexical ownership and source-specific keys.

use super::*;

fn source_document_fixture(references: &str) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let style = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:key name="which" match="value" use="@tag"/><xsl:template match="/"><xsl:variable name="lookup" select="external/value"/><xsl:variable name="alias" select="$lookup"/><out><xsl:for-each select="document(external/ref)"><entry><xsl:value-of select="key('which',$alias)"/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/><xsl:for-each select="$alias"><original><xsl:value-of select="."/></original></xsl:for-each></entry></xsl:for-each><after><xsl:value-of select="$alias"/></after></out></xsl:template></xsl:stylesheet>"#;
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(3, 8192, 24576));
    builder.admit(STYLE, style.to_vec()).unwrap();
    let external = br#"<external> <value tag="one">secondary</value> </external>"#;
    builder.admit(EXTRA, external.to_vec()).unwrap();
    builder
        .admit("https://example.test/bound/other.xml", external.to_vec())
        .unwrap();
    let snapshot = builder.seal();
    let source = Document::from_parsed(
        parse_document(
            "https://example.test/bound/source.xml",
            format!(r#"<external> <value tag="one">one</value>{references} </external>"#)
                .as_bytes(),
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
fn source_document_iteration_preserves_lexical_owners_keys_and_cardinality() {
    let fixture = source_document_fixture("<ref>extra.xml</ref><ref>extra.xml</ref>");
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
                "<out><entry>secondary:1/1<original>one</original></entry><after>one</after></out>"
            );
        }
    }
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded().without_key_index_cache()
        )
        .unwrap(),
        "<out><entry>secondary:1/1<original>one</original></entry><after>one</after></out>"
    );
    let empty = source_document_fixture("");
    let denied = HashSet::from([EXTRA.to_owned()]);
    assert_eq!(
        run(
            &empty,
            WhitespaceRepresentation::VisibilityView,
            Some(&denied),
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out><after>one</after></out>"
    );
    let two = source_document_fixture("<ref>extra.xml</ref><ref>other.xml</ref>");
    assert_eq!(
        run(
            &two,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT1017"
    );
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| { assert_eq!(run(&fixture, WhitespaceRepresentation::VisibilityView, None, &mut InvocationControl::unbounded()).unwrap(), "<out><entry>secondary:1/1<original>one</original></entry><after>one</after></out>"); });
        }
    });
}

#[test]
fn source_document_iteration_keeps_authority_and_exact_controls() {
    let fixture = source_document_fixture("<ref>extra.xml</ref>");
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
    let missing = source_document_fixture("<ref>missing.xml</ref>");
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
    let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(
        WorkDomain::XPathNodeVisit,
        baseline.consumed(WorkDomain::XPathNodeVisit) - 1,
    );
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
fn source_document_iteration_releases_empty_single_rejected_and_failed_attempts() {
    for references in [
        "",
        "<ref>extra.xml</ref>",
        "<ref>extra.xml</ref><ref>other.xml</ref>",
        "<ref>missing.xml</ref>",
    ] {
        let fixture = source_document_fixture(references);
        let info = allocation_counter::measure(|| {
            let _outcome = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::unbounded(),
            );
        });
        assert_eq!(info.bytes_current, 0);
        assert_eq!(info.count_current, 0);
    }
    let fixture = source_document_fixture("<ref>extra.xml</ref>");
    let mut baseline = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    for mut control in [
        InvocationControl::unbounded().cancelling_on_charge(
            WorkDomain::XPathNodeVisit,
            baseline.consumed(WorkDomain::XPathNodeVisit) - 1,
        ),
        InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xdm_nodes: baseline.consumed(WorkDomain::XdmNode) - 1,
                ..WorkLimits::unbounded()
            },
        ),
    ] {
        let info = allocation_counter::measure(|| {
            assert!(
                run(
                    &fixture,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut control
                )
                .is_err()
            );
        });
        assert_eq!(info.bytes_current, 0);
        assert_eq!(info.count_current, 0);
    }
}
