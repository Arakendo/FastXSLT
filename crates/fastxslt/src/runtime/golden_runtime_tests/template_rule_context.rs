//! Matched-root eligibility, named-call preservation and iteration rule clearing.

use super::*;
use crate::resources::ResourceSnapshot;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

type Fixture = (StylesheetProgram, ResourceSnapshot, Document);

fn fixture(version: &str, body: &str, extra: &str) -> Fixture {
    let style = format!(
        r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:template match="/">{body}</xsl:template>{extra}</xsl:stylesheet>"#
    );
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit(STYLESHEET_ID, style.into_bytes()).unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET_ID).unwrap();
    let source = Document::from_parsed(
        parse_document(
            SOURCE_ID,
            br#"<doc xmlns:p="urn:p"> <value>A</value><value>B</value> </doc>"#,
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

fn run(
    fixture: &Fixture,
    representation: WhitespaceRepresentation,
    control: &mut InvocationControl,
) -> Result<String, super::super::ExecutionFailure> {
    let (program, snapshot, source) = fixture;
    let result = execute_program_with_parameters_using(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "rule-context",
        representation,
        Some(snapshot),
        None,
        control,
    )?;
    serialize_xml(&result, &program.output, "rule-context", 8192, control)
}

#[test]
fn template_rule_context_root_and_named_calls_reach_builtins() {
    for version in ["1.0", "3.0"] {
        for (body, extra) in [
            ("<out><xsl:apply-imports/></out>", ""),
            (
                r#"<out><xsl:call-template name="delegate"/></out>"#,
                r#"<xsl:template name="delegate"><xsl:apply-imports/></xsl:template>"#,
            ),
        ] {
            let fixture = fixture(version, body, extra);
            assert!(fixture.0.root_template.is_none());
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
                        run(&fixture, representation, &mut control).unwrap(),
                        "<out>AB</out>"
                    );
                }
            }
        }
    }
    let next = fixture("3.0", "<out><xsl:next-match/></out>", "");
    assert_eq!(
        run(
            &next,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>AB</out>"
    );
    let simple = fixture("1.0", "<out/>", "");
    assert!(
        simple.0.root_template.is_some(),
        "ordinary fast path stays eligible"
    );
}

#[test]
fn template_rule_context_iteration_clears_but_does_not_erase_outer_rule() {
    for (selection, prefix) in [
        ("doc/value", ""),
        (
            "$nodes",
            r#"<xsl:variable name="nodes" select="doc/value"/>"#,
        ),
        (
            "$tree",
            r#"<xsl:variable name="tree"><box/></xsl:variable>"#,
        ),
        (
            "$tree/box",
            r#"<xsl:variable name="tree"><box/></xsl:variable>"#,
        ),
    ] {
        let body = format!(
            r#"{prefix}<out><xsl:for-each select="{selection}"><xsl:call-template name="delegate"/></xsl:for-each></out>"#
        );
        let fixture = fixture(
            "3.0",
            &body,
            r#"<xsl:template name="delegate"><xsl:apply-imports/></xsl:template>"#,
        );
        let failure = run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(failure.code, "XTDE0560", "{selection}");
        assert_eq!(failure.category, FailureCategory::Invalid);
        assert_eq!(failure.request_id.as_deref(), Some("rule-context"));
        assert_eq!(failure.location.unwrap().resource, STYLESHEET_ID);
    }
    let after = fixture(
        "1.0",
        r#"<out><xsl:for-each select="doc/value"><xsl:text>|</xsl:text></xsl:for-each><xsl:apply-imports/></out>"#,
        "",
    );
    assert_eq!(
        run(
            &after,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>||AB</out>"
    );
}

#[test]
fn template_rule_context_preserves_absent_initial_named_rule_and_controls() {
    let fixture = fixture(
        "1.0",
        r#"<xsl:call-template name="delegate"/>"#,
        r#"<xsl:template name="delegate"><xsl:apply-imports/></xsl:template>"#,
    );
    let failure = super::super::execute_initial_template_with_source(
        &fixture.0,
        "delegate",
        &fixture.2,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "named-entry",
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "XTDE0560");
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            &mut baseline
        )
        .unwrap(),
        "AB"
    );
    let consumed = baseline.consumed(WorkDomain::XsltInstruction);
    for limit in [consumed - 1, consumed] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xslt_instructions: limit,
                ..WorkLimits::unbounded()
            },
        );
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                &mut control
            )
            .is_ok(),
            limit == consumed
        );
    }
    let mut cancellation = InvocationControl::unbounded()
        .cancelling_on_charge(WorkDomain::XsltInstruction, consumed - 1);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            &mut cancellation
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
}

#[test]
fn template_rule_context_included_named_delegate_keeps_the_callers_rule() {
    const MAIN: &str = "https://example.test/rule/main.xsl";
    const INCLUDED: &str = "https://example.test/rule/helper.xsl";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    resources.admit(MAIN, br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="helper.xsl"/><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:call-template name="delegate"/></out></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    resources.admit(INCLUDED, br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="delegate"><xsl:apply-imports/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, MAIN).unwrap();
    assert!(program.root_template.is_none());
    let source = fixture("1.0", "<out/>", "").2;
    let fixture = (program, snapshot, source);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out> AB </out>"
    );
}
