//! Source `current()` is scalarized before entering a sealed document lookup.

use super::{
    BTreeMap, CancellationToken, Document, HashSet, InvocationControl, MultipleMatchPolicy,
    ParseLimits, ResourceLimits, ResourceSetBuilder, WhitespaceRepresentation, WorkDomain,
    WorkLimits, compile_resource, execute_program_with_parameters_using, parse_document,
    serialize_xml,
};
use crate::resources::ResourceSnapshot;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

const STYLE: &str = "https://example.test/current/style.xsl";
const LOOKUP: &str = "https://example.test/current/table.xml";

fn fixture(reference: &str) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let stylesheet = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:l="urn:lookup" xmlns:a="urn:attribute" exclude-result-prefixes="l a"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><l:table><l:item a:key="x">first</l:item><l:item a:key="x">second</l:item><l:item a:key="">empty</l:item><l:item>missing</l:item></l:table><xsl:template match="/"><out><xsl:for-each select="r/item"><xsl:value-of select="document('{reference}')//l:item[@a:key=current()/@a:key]"/><xsl:text>|</xsl:text></xsl:for-each></out></xsl:template></xsl:stylesheet>"#
    );
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    resources.admit(STYLE, stylesheet.into_bytes()).unwrap();
    resources.admit(LOOKUP, br#"<l:table xmlns:l="urn:lookup" xmlns:a="urn:attribute"><l:item a:key="x">first</l:item><l:item a:key="x">second</l:item><l:item a:key="">empty</l:item><l:item>missing</l:item></l:table>"#.to_vec()).unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLE).unwrap();
    let source = Document::from_parsed(parse_document(
        "urn:current:source", br#"<r xmlns:a="urn:attribute"> <item a:key="x"/><item a:key=""/><item/><item a:key="z"/> </r>"#,
        ParseLimits { max_events: 64, max_depth: 8 },
    ).unwrap()).unwrap();
    (program, snapshot, source)
}

fn run(
    fixture: &(StylesheetProgram, ResourceSnapshot, Document),
    representation: WhitespaceRepresentation,
    denied: Option<&HashSet<String>>,
    control: &mut InvocationControl,
) -> Result<String, super::super::ExecutionFailure> {
    let (program, resources, source) = fixture;
    let result = execute_program_with_parameters_using(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "current-lookup",
        representation,
        Some(resources),
        denied,
        control,
    )?;
    serialize_xml(&result, &program.output, "current-lookup", 8192, control)
}

#[test]
fn current_document_lookup_preserves_owner_names_empty_sets_and_first_match() {
    for reference in ["", "table.xml"] {
        let fixture = fixture(reference);
        for view in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            assert_eq!(
                run(&fixture, view, None, &mut InvocationControl::unbounded()).unwrap(),
                "<out>first|empty|||</out>"
            );
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
                assert_eq!(handle.join().unwrap(), "<out>first|empty|||</out>");
            }
        });
    }
}

#[test]
fn current_document_lookup_preserves_exact_work_limits_cancellation_and_recovery() {
    let fixture = fixture("table.xml");
    for view in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        let mut baseline = InvocationControl::unbounded();
        run(&fixture, view, None, &mut baseline).unwrap();
        for domain in [WorkDomain::XPathNodeVisit, WorkDomain::XdmStringValueNode] {
            let consumed = baseline.consumed(domain);
            assert!(consumed > 0);
            for (limit, succeeds) in [(consumed, true), (consumed - 1, false)] {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::XPathNodeVisit => limits.xpath_node_visits = limit,
                    WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
                    _ => unreachable!(),
                }
                let result = run(
                    &fixture,
                    view,
                    None,
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                );
                if succeeds {
                    assert!(result.is_ok());
                } else {
                    let error = result.unwrap_err();
                    assert_eq!(error.code, "FXCT0002");
                    assert_eq!(error.work_domain, Some(domain));
                }
            }
        }
        let token = CancellationToken::new();
        token.cancel();
        let error = run(
            &fixture,
            view,
            None,
            &mut InvocationControl::new(token, WorkLimits::unbounded()),
        )
        .unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert_eq!(
            run(&fixture, view, None, &mut InvocationControl::unbounded()).unwrap(),
            "<out>first|empty|||</out>"
        );
    }
}

#[test]
fn current_document_lookup_obeys_sealed_authority() {
    let fixture = fixture("table.xml");
    let denied = HashSet::from([LOOKUP.to_owned()]);
    let error = run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        Some(&denied),
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.category, super::FailureCategory::Denied);
    assert_eq!(error.request_id.as_deref(), Some("current-lookup"));
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>first|empty|||</out>"
    );
}

#[test]
fn current_document_lookup_keeps_modern_and_unbound_prefix_boundaries_explicit() {
    for (version, select, code) in [
        (
            "1.0",
            "document('')/xsl:stylesheet[@first=current()/@first][@key=current()/@key]",
            Some("FXXP1001"),
        ),
        (
            "1.0",
            "document('')/xsl:stylesheet[@absent:key=current()/@key]",
            Some("XPST0081"),
        ),
        (
            "1.0",
            "document('')/xsl:stylesheet[@key=current()/@absent:key]",
            Some("XPST0081"),
        ),
        (
            "3.0",
            "document('')/xsl:stylesheet[@key=current()/@key]",
            None,
        ),
    ] {
        let style = format!(
            r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="{select}"/></xsl:template></xsl:stylesheet>"#
        );
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
        resources.admit(STYLE, style.into_bytes()).unwrap();
        let error = compile_resource(&resources.seal(), STYLE).unwrap_err();
        if let Some(code) = code {
            assert_eq!(error.code, code);
        }
        assert_eq!(error.location.unwrap().resource, STYLE);
    }
}

#[test]
fn current_document_lookup_rejects_temporary_focus_without_borrowing_principal_ids() {
    let style = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="t"><box key="x"/></xsl:variable><xsl:template match="/"><xsl:for-each select="$t/box"><xsl:value-of select="document('')/xsl:stylesheet[@key=current()/@key]"/></xsl:for-each></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit(STYLE, style.to_vec()).unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLE).unwrap();
    let (_, _, source) = fixture("");
    let error = execute_program_with_parameters_using(
        &program,
        &source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "temporary-lookup",
        WhitespaceRepresentation::VisibilityView,
        Some(&snapshot),
        None,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.code, "FXRT0007");
    assert_eq!(error.category, super::FailureCategory::Unsupported);
    assert_eq!(error.location.unwrap().resource, STYLE);
}
