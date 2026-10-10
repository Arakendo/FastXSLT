//! Whole-union positional identity uses normalized source nodes and atomic frames.

use super::{
    BTreeMap, CancellationToken, Document, FailureCategory, InvocationControl, MultipleMatchPolicy,
    ParseLimits, ResourceLimits, ResourceSetBuilder, WhitespaceRepresentation, WorkDomain,
    WorkLimits, compile_resource, execute_program_with_parameters_using, parse_document,
    serialize_xml,
};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

fn compile(body: &str, version: &str) -> Result<StylesheetProgram, super::super::ExecutionFailure> {
    let bytes = format!(r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:strip-space elements="*"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#).into_bytes();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit("urn:union-id:style", bytes).unwrap();
    compile_resource(&resources.seal(), "urn:union-id:style")
}

fn source() -> Document {
    Document::from_parsed(
        parse_document(
            "urn:union-id:source",
            br#"<r> <a key="x"/> <b key="y"/> </r>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

fn run(
    program: &StylesheetProgram,
    source: &Document,
    representation: WhitespaceRepresentation,
    control: &mut InvocationControl,
) -> Result<String, super::super::ExecutionFailure> {
    let result = execute_program_with_parameters_using(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "union-id",
        representation,
        None,
        None,
        control,
    )?;
    serialize_xml(&result, &program.output, "union-id", 8192, control)
}

const BODY: &str = r#"<xsl:variable name="pos" select="3"/><xsl:variable name="identity" select="generate-id((r/b|r/*/@*|r/a|r/*)[position() = $pos])"/><xsl:value-of select="$identity"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(r/b)"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id((r/*|r/*/@*)[$pos = position()])"/>"#;

#[test]
fn filtered_union_identity_normalizes_before_position_and_binds_a_string() {
    let program = compile(BODY, "1.0").unwrap();
    let source = source();
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        let output = run(
            &program,
            &source,
            representation,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let fields = output.split('|').collect::<Vec<_>>();
        assert_eq!(fields.len(), 3);
        assert!(!fields[0].is_empty());
        assert_eq!(fields[0], fields[1]);
        assert_eq!(fields[1], fields[2]);
    }
    for position in ["0", "-1", "1.5", "99", "'NaN'", "'Infinity'", "'1e0'"] {
        let body = format!(
            r#"<xsl:variable name="pos" select="{position}"/><xsl:variable name="id" select="generate-id((r/a|r/b)[position()=$pos])"/><xsl:value-of select="$id"/><xsl:if test="$id"><xsl:text>WRONG</xsl:text></xsl:if>"#
        );
        assert_eq!(
            run(
                &compile(&body, "1.0").unwrap(),
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "",
            "position {position}: an empty string is false, unlike an RTF root"
        );
    }
    for position in ["1", "'1'"] {
        let body = format!(
            r#"<xsl:variable name="pos" select="{position}"/><xsl:value-of select="generate-id((r/b|r/a)[position()=$pos])"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(r/a)"/>"#
        );
        let output = run(
            &compile(&body, "1.0").unwrap(),
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let fields = output.split('|').collect::<Vec<_>>();
        assert_eq!(fields[0], fields[1], "position {position}");
    }
    assert_eq!(
        compile(BODY, "3.0").unwrap_err().category,
        FailureCategory::Unsupported
    );
}

#[test]
fn filtered_union_identity_preserves_limits_cancellation_and_reuse() {
    let program = compile(BODY, "1.0").unwrap();
    let source = source();
    let mut measured = InvocationControl::unbounded();
    let expected = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut measured,
    )
    .unwrap();
    for domain in [
        WorkDomain::XPathOperation,
        WorkDomain::XPathNodeVisit,
        WorkDomain::ResultTextByte,
    ] {
        let charge = measured.consumed(domain);
        assert!(charge > 0);
        for allowance in [charge - 1, charge] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XPathOperation => limits.xpath_operations = allowance,
                WorkDomain::XPathNodeVisit => limits.xpath_node_visits = allowance,
                WorkDomain::ResultTextByte => limits.result_text_bytes = allowance,
                _ => unreachable!(),
            }
            let result = run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
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
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded().cancelling_on_charge(domain, 1)
            )
            .unwrap_err()
            .code,
            "FXCT0001"
        );
    }
    let non_atomic = compile(r#"<xsl:variable name="pos" select="r/a/@key"/><xsl:value-of select="generate-id((r/a|r/b)[position()=$pos])"/>"#, "1.0").unwrap();
    let failure = run(
        &non_atomic,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.location.unwrap().resource, "urn:union-id:style");
    let unbound = compile(
        r#"<xsl:value-of select="generate-id((r/a|r/b)[position()=$missing])"/>"#,
        "1.0",
    )
    .unwrap();
    let failure = run(
        &unbound,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT0002");
    assert_eq!(failure.location.unwrap().resource, "urn:union-id:style");
    std::thread::scope(|scope| {
        let workers = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    run(
                        &program,
                        &source,
                        WhitespaceRepresentation::VisibilityView,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), expected);
        }
    });
}

#[test]
fn filtered_union_identity_applies_atomic_comparison_rules_and_lexical_shadowing() {
    let bytes = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:param name="pos" select="1"/><xsl:template match="/"><xsl:variable name="id" select="generate-id((r/b|r/a)[position()=$pos])"/><xsl:value-of select="$id"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(r/a)"/></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit("urn:union-id:parameters", bytes.to_vec())
        .unwrap();
    let program = compile_resource(&resources.seal(), "urn:union-id:parameters").unwrap();
    let source = source();
    for (kind, lexical, matches) in [
        (BuiltinAtomicType::Boolean, "true", true),
        (BuiltinAtomicType::Boolean, "1", true),
        (BuiltinAtomicType::Boolean, "false", false),
        (BuiltinAtomicType::Boolean, "0", false),
        (BuiltinAtomicType::Double, "1e0", true),
        (BuiltinAtomicType::Double, "NaN", false),
        (BuiltinAtomicType::Double, "INF", false),
        (BuiltinAtomicType::Double, "-INF", false),
        (BuiltinAtomicType::String, "1e0", false),
        (BuiltinAtomicType::String, " 1 ", true),
    ] {
        let parameters = BTreeMap::from([(
            "pos".to_owned(),
            super::InvocationParameter {
                value: AtomicValue::from_validated_lexical(kind, lexical).into(),
                tunnel: false,
            },
        )]);
        let mut control = InvocationControl::unbounded();
        let result = execute_program_with_parameters_using(
            &program,
            &source,
            &parameters,
            MultipleMatchPolicy::UseLast,
            "union-id",
            WhitespaceRepresentation::VisibilityView,
            None,
            None,
            &mut control,
        )
        .unwrap();
        let output =
            serialize_xml(&result, &program.output, "union-id", 8192, &mut control).unwrap();
        let fields = output.split('|').collect::<Vec<_>>();
        if matches {
            assert_eq!(fields[0], fields[1], "{kind:?} {lexical}");
        } else {
            assert!(fields[0].is_empty(), "{kind:?} {lexical}");
        }
    }
    let nested = compile(r#"<xsl:variable name="pos" select="1"/><xsl:variable name="id" select="generate-id((r/a|r/b)[position()=$pos])"/><xsl:value-of select="$id"/><xsl:text>|</xsl:text><xsl:for-each select="r"><xsl:variable name="pos" select="99"/><xsl:variable name="id" select="generate-id((a|b)[position()=$pos])"/><xsl:value-of select="$id"/></xsl:for-each><xsl:text>|</xsl:text><xsl:value-of select="$id"/>"#, "1.0").unwrap();
    let output = run(
        &nested,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let fields = output.split('|').collect::<Vec<_>>();
    assert!(!fields[0].is_empty());
    assert!(fields[1].is_empty());
    assert_eq!(fields[0], fields[2]);
    let mut reference_control = InvocationControl::unbounded()
        .with_complete_atomic_frame_clones()
        .with_complete_sequence_frame_clones();
    assert_eq!(
        run(
            &nested,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut reference_control
        )
        .unwrap(),
        output
    );
}
