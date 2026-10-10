//! Bound document dispatch and retained principal-source ownership.

use super::{
    BTreeMap, CancellationToken, Document, HashSet, InvocationControl, MultipleMatchPolicy,
    ParseLimits, ResourceLimits, ResourceSetBuilder, WhitespaceRepresentation, WorkDomain,
    WorkLimits, compile_resource, execute_program_with_parameters_using, parse_document,
    serialize_xml,
};
use crate::resources::ResourceSnapshot;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

#[path = "bound_variable_path_tests.rs"]
mod bound_variable_path_tests;
#[path = "empty_document_union_tests.rs"]
mod empty_document_union_tests;
#[path = "retained_owner_dispatch_cost.rs"]
mod retained_owner_dispatch_cost;
#[path = "retained_source_iteration.rs"]
mod retained_source_iteration;
#[path = "source_document_iteration.rs"]
mod source_document_iteration;

const STYLE: &str = "https://example.test/bound/style.xsl";
const EXTRA: &str = "https://example.test/bound/extra.xml";

fn fixture(source_global: bool) -> (StylesheetProgram, ResourceSnapshot, Document) {
    fixture_with_external_body(
        source_global,
        r#"<out><xsl:value-of select="$label"/>:<xsl:value-of select="external/value"/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/></out>"#,
    )
}

fn fixture_with_external_body(
    source_global: bool,
    body: &str,
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    fixture_with_templates(source_global, body, "")
}

fn fixture_with_templates(
    source_global: bool,
    body: &str,
    templates: &str,
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let global = if source_global {
        r#"<xsl:variable name="principal" select="/"/>"#
    } else {
        ""
    };
    let principal = if templates.is_empty() {
        r#"<xsl:template match="/" mode="principal"><principal><xsl:value-of select="external/value"/></principal></xsl:template>"#
    } else {
        ""
    };
    let style = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/>{global}<xsl:variable name="doc" select="document('extra.xml')"/><xsl:template match="/"><xsl:variable name="alias" select="$doc"/><xsl:apply-templates select="$alias" mode="external"><xsl:with-param name="label" select="'ok'"/></xsl:apply-templates></xsl:template><xsl:template match="/" mode="external"><xsl:param name="label"/>{body}</xsl:template>{principal}{templates}</xsl:stylesheet>"#
    );
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    builder.admit(STYLE, style.into_bytes()).unwrap();
    builder
        .admit(
            EXTRA,
            b"<external> <value>right</value> </external>".to_vec(),
        )
        .unwrap();
    let snapshot = builder.seal();
    let program = compile_resource(&snapshot, STYLE).unwrap();
    let source = Document::from_parsed(
        parse_document(
            "urn:bound:principal",
            b"<external><value>wrong</value></external>",
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
fn document_variable_dispatch_confines_key_and_document_rooted_match_caches() {
    let mut fixture = fixture_with_templates(
        true,
        r#"<external><xsl:value-of select="key('which','selected')"/><xsl:apply-templates select="external/value" mode="cached"/></external><xsl:apply-templates select="$principal" mode="principal"/>"#,
        r#"<xsl:key name="which" match="value" use="'selected'"/><xsl:template match="/" mode="principal"><principal><xsl:value-of select="key('which','selected')"/><xsl:apply-templates select="external/value" mode="cached"/></principal></xsl:template><xsl:template match="value" mode="cached"><value><xsl:value-of select="."/></value></xsl:template><xsl:template match="/external/value[.='right']" mode="cached"><matched><xsl:value-of select="."/></matched></xsl:template>"#,
    );
    fixture.2 = Document::from_parsed(
        parse_document(
            "urn:bound:principal",
            b"<external> <value>wrong</value> </external>",
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap();
    for cached in [false, true] {
        let mut control = InvocationControl::unbounded();
        if !cached {
            control = control.without_document_rooted_match_cache();
        }
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control
            )
            .unwrap(),
            "<external>right<matched>right</matched></external><principal>wrong<value>wrong</value></principal>"
        );
        if cached {
            assert!(control.document_rooted_match_cache_observation().0 >= 2);
        }
    }
}

fn run(
    fixture: &(StylesheetProgram, ResourceSnapshot, Document),
    representation: WhitespaceRepresentation,
    denied: Option<&HashSet<String>>,
    control: &mut InvocationControl,
) -> Result<String, super::super::ExecutionFailure> {
    let (program, snapshot, source) = fixture;
    let result = execute_program_with_parameters_using(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "bound-document",
        representation,
        Some(snapshot),
        denied,
        control,
    )?;
    serialize_xml(&result, &program.output, "bound-document", 8192, control)
}

#[test]
fn document_variable_dispatch_preserves_bound_owner_alias_mode_and_atomic_parameters() {
    let fixture = fixture(false);
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
                "<out>ok:right:1/1</out>"
            );
        }
    }
    std::thread::scope(|scope| {
        let threads = (0..4)
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
        for thread in threads {
            assert_eq!(thread.join().unwrap(), "<out>ok:right:1/1</out>");
        }
    });
}

#[test]
fn document_variable_dispatch_keeps_unmigrated_navigation_guarded_and_authority_explicit() {
    let fixture = fixture(true);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded(),
        )
        .unwrap(),
        "<out>ok:right:1/1</out>"
    );
    let foreign_lookup =
        fixture_with_external_body(true, r#"<out><xsl:copy-of select="$principal"/></out>"#);
    let failure = run(
        &foreign_lookup,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT1017");
    let fixture = self::fixture(false);
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
    let mut cancellation =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut cancellation
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    let mut exhausted = InvocationControl::new(
        CancellationToken::new(),
        WorkLimits {
            xdm_nodes: 0,
            ..WorkLimits::unbounded()
        },
    );
    assert!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut exhausted
        )
        .is_err()
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>ok:right:1/1</out>"
    );
}

#[test]
fn document_variable_dispatch_returns_to_retained_principal_owner_through_alias() {
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:variable name="back" select="$principal"/><xsl:apply-templates select="$back" mode="principal"/>"#,
    );
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
                "<principal>wrong</principal>"
            );
        }
    }
    let newer = fixture_with_external_body(
        true,
        r#"<xsl:variable name="back" select="$principal"/><xsl:apply-templates select="$back" mode="principal"/>"#,
    );
    std::thread::scope(|scope| {
        let threads = [&fixture, &newer, &fixture, &newer].map(|generation| {
            scope.spawn(move || {
                run(
                    generation,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            })
        });
        for thread in threads {
            assert_eq!(thread.join().unwrap(), "<principal>wrong</principal>");
        }
    });
}

#[test]
fn document_variable_dispatch_retained_owner_exact_work_boundary_and_active_cancellation() {
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:variable name="back" select="$principal"/><xsl:apply-templates select="$back" mode="principal"/>"#,
    );
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut baseline
        )
        .unwrap(),
        "<principal>wrong</principal>"
    );
    let consumed = baseline.consumed(WorkDomain::XdmNode);
    assert!(consumed > 1);
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
    let mut cancellation =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XsltTemplateCandidate, 1);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut cancellation
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<principal>wrong</principal>"
    );
}

#[cfg(feature = "allocation-observation")]
#[test]
fn document_variable_dispatch_integrated_allocation_pressure() {
    for global in [false, true] {
        let fixture = fixture(global);
        let mut runs = 0;
        let info = allocation_counter::measure(|| {
            let actual = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(actual, "<out>ok:right:1/1</out>");
            runs += 1;
        });
        assert_eq!(runs, 1);
        assert_eq!(info.bytes_current, 0);
        assert_eq!(info.count_current, 0);
        println!(
            "retained-dispatch global={global} total={} peak={} allocations={}",
            info.bytes_total, info.bytes_max, info.count_total
        );
    }
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:variable name="back" select="$principal"/><xsl:apply-templates select="$back" mode="principal"/>"#,
    );
    let denied = HashSet::from([EXTRA.to_owned()]);
    for phase in ["denied", "budget", "cancelled"] {
        let info = allocation_counter::measure(|| {
            let mut control = match phase {
                "budget" => InvocationControl::new(
                    CancellationToken::new(),
                    WorkLimits {
                        xdm_nodes: 0,
                        ..WorkLimits::unbounded()
                    },
                ),
                "cancelled" => InvocationControl::unbounded()
                    .cancelling_on_charge(WorkDomain::XsltTemplateCandidate, 1),
                _ => InvocationControl::unbounded(),
            };
            let outcome = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                (phase == "denied").then_some(&denied),
                &mut control,
            );
            assert!(outcome.is_err());
        });
        assert_eq!(info.bytes_current, 0, "failure phase {phase}");
        assert_eq!(info.count_current, 0, "failure phase {phase}");
    }
}
