//! Typed-ID name consumers preserve effective source semantics and control.

use super::{
    BTreeMap, CancellationToken, Document, FailureCategory, InvocationControl, MultipleMatchPolicy,
    ParseLimits, ResourceLimits, ResourceSetBuilder, WhitespaceRepresentation, WorkDomain,
    WorkLimits, compile_resource, execute_program_with_parameters_using, parse_document,
    serialize_xml,
};
use crate::xml::internal_subset::InternalSubsetLimits;
use crate::xml::quick_xml_experiment::parse_document_with_internal_subset;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

const SOURCE: &[u8] = br#"<!DOCTYPE r [<!ATTLIST p:item key ID #IMPLIED>]><r xmlns:p="urn:p"> <p:item key="alpha"><leaf/></p:item> <p:item key="beta"/> <ids keys="beta alpha alpha"/> </r>"#;
const BODY: &str = r#"<xsl:variable name="ids" select="'beta alpha alpha'"/><xsl:value-of select="name(id($ids))"/><xsl:text>|</xsl:text><xsl:value-of select="local-name(id('beta alpha'))"/><xsl:text>|</xsl:text><xsl:value-of select="namespace-uri(id(r/ids/@keys))"/><xsl:text>|</xsl:text><xsl:value-of select="name(id('alpha')/@key)"/><xsl:text>|</xsl:text><xsl:value-of select="name(id('alpha')/*)"/><xsl:text>|</xsl:text><xsl:value-of select="name(id('absent'))"/><xsl:text>|</xsl:text><xsl:value-of select="local-name(id('absent'))"/><xsl:text>|</xsl:text><xsl:value-of select="namespace-uri(id('absent'))"/>"#;

fn compile(body: &str, version: &str) -> Result<StylesheetProgram, super::super::ExecutionFailure> {
    let bytes = format!(r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:strip-space elements="*"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#).into_bytes();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources.admit("urn:id-name:style", bytes).unwrap();
    compile_resource(&resources.seal(), "urn:id-name:style")
}

fn source() -> Document {
    Document::from_parsed(
        parse_document_with_internal_subset(
            "urn:id-name:source",
            SOURCE,
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
            InternalSubsetLimits {
                declarations: 8,
                nesting_depth: 4,
                references: 32,
                replacement_bytes: 4096,
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
        "id-name",
        representation,
        None,
        None,
        control,
    )?;
    serialize_xml(&result, &program.output, "id-name", 8192, control)
}

#[test]
fn xslt10_identity_function_availability_matches_executable_capability() {
    let style = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:p" xmlns:msxsl="urn:schemas-microsoft-com:xslt"><xsl:output method="text"/><xsl:key name="k" match="p:item" use="@key"/><xsl:template match="/"><xsl:value-of select="function-available('id')"/>|<xsl:value-of select="function-available('key')"/>|<xsl:value-of select="function-available('generate-id')"/>|<xsl:if test="function-available('id')"><xsl:value-of select="name(id('alpha'))"/></xsl:if>|<xsl:if test="function-available('key')"><xsl:value-of select="count(key('k','alpha'))"/></xsl:if>|<xsl:if test="function-available('generate-id')"><xsl:value-of select="generate-id(r)"/></xsl:if>|<xsl:value-of xmlns="urn:default" select="function-available('id')"/>|<xsl:value-of select="function-available('p:id')"/>|<xsl:value-of select="function-available('xsl:key')"/>|<xsl:value-of select="function-available('msxsl:node-set')"/>|<xsl:value-of select="function-available('unparsed-entity-uri')"/></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit("urn:availability:style", style.to_vec())
        .unwrap();
    let program = compile_resource(&resources.seal(), "urn:availability:style").unwrap();
    let source = source();
    let mut first = None;
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        let actual = run(
            &program,
            &source,
            representation,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let fields = actual.split('|').collect::<Vec<_>>();
        assert_eq!(&fields[..5], &["true", "true", "true", "p:item", "1"]);
        assert!(!fields[5].is_empty(), "advertised generate-id must execute");
        assert_eq!(&fields[6..], &["true", "false", "false", "false", "false"]);
        if let Some(expected) = &first {
            assert_eq!(&actual, expected);
        } else {
            first = Some(actual);
        }
    }
}

#[test]
fn xslt10_id_names_use_first_typed_id_and_effective_source_navigation() {
    let program = compile(BODY, "1.0").unwrap();
    let source = source();
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        assert_eq!(
            run(
                &program,
                &source,
                representation,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "p:item|item|urn:p|key|leaf|||"
        );
    }
    assert_eq!(source.elements_with_id("alpha").len(), 1);
    assert_eq!(source.elements_with_id("beta").len(), 1);
    assert!(
        parse_document(
            "urn:id-name:source",
            SOURCE,
            ParseLimits {
                max_events: 64,
                max_depth: 8
            }
        )
        .is_err(),
        "ordinary XML parsing still denies DTDs"
    );
    assert_eq!(
        compile(BODY, "3.0").unwrap_err().category,
        FailureCategory::Unsupported
    );
    let empty_source = Document::from_parsed(
        parse_document(
            "urn:id-name:untyped",
            br#"<r><item key="alpha"/></r>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        run(
            &program,
            &empty_source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "|||||||"
    );
}

#[test]
fn xslt10_id_names_preserve_control_provenance_recovery_and_concurrent_reuse() {
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
            let actual = run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance == charge {
                assert_eq!(actual.unwrap(), expected);
            } else {
                let failure = actual.unwrap_err();
                assert_eq!(failure.code, "FXCT0002");
                assert_eq!(failure.request_id.as_deref(), Some("id-name"));
            }
        }
        let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(domain, 1);
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
    }
    let bad = compile(r#"<xsl:value-of select="name(id())"/>"#, "1.0").unwrap_err();
    assert_eq!(bad.code, "XPST0017");
    assert_eq!(bad.location.unwrap().resource, "urn:id-name:style");
    let unbound = compile(r#"<xsl:value-of select="name(id($missing))"/>"#, "1.0").unwrap();
    let failure = run(
        &unbound,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT0002");
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
