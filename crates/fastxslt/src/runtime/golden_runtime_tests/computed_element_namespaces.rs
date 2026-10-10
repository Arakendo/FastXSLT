//! Computed-element explicit namespaces preserve local compatibility and control.

use super::{
    BTreeMap, CancellationToken, Document, InvocationControl, MultipleMatchPolicy, ParseLimits,
    ResourceLimits, ResourceSetBuilder, WhitespaceRepresentation, WorkDomain, WorkLimits,
    compile_resource, execute_program_with_parameters_using, parse_document, serialize_xml,
};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

const SOURCE: &[u8] =
    br#"<doc xmlns:p="urn:source"> <p:item name="unbound:chosen" uri=""/> </doc>"#;
const EMPTY_BODY: &str = r#"<out><xsl:element name="unbound:chosen" namespace=""/><xsl:element name="xml:chosen" namespace=""/><xsl:element name="{doc/p:item/@name}" namespace=""/><xsl:element name="unbound:chosen" namespace="{doc/p:item/@uri}"/><xsl:for-each select="doc/p:item"><xsl:element name="{name()}" namespace=""/></xsl:for-each><xsl:variable name="tree"><p:item/></xsl:variable><xsl:for-each select="$tree/p:item"><xsl:element name="{name()}" namespace=""/></xsl:for-each></out>"#;

fn document(identity: &str, bytes: &[u8]) -> Document {
    Document::from_parsed(
        parse_document(
            identity,
            bytes,
            ParseLimits {
                max_events: 128,
                max_depth: 16,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

fn compile(body: &str, version: &str) -> StylesheetProgram {
    let bytes = format!(r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:source" exclude-result-prefixes="p"><xsl:strip-space elements="*"/><xsl:output omit-xml-declaration="yes"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#).into_bytes();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit("urn:element-namespaces:style", bytes)
        .unwrap();
    compile_resource(&resources.seal(), "urn:element-namespaces:style").unwrap()
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
        "element-namespaces",
        representation,
        None,
        None,
        control,
    )?;
    serialize_xml(
        &result,
        &program.output,
        "element-namespaces",
        8192,
        control,
    )
}

#[test]
fn xslt10_computed_element_empty_namespace_overrides_every_admitted_name_source() {
    let program = compile(EMPTY_BODY, "1.0");
    let source = document("urn:element-namespaces:source", SOURCE);
    let expected = "<out><chosen></chosen><chosen></chosen><chosen></chosen><chosen></chosen><item></item><item></item></out>";
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
        assert_eq!(actual, expected);
        let parsed = document("urn:element-namespaces:result", actual.as_bytes());
        let root = parsed.children(parsed.document_node())[0];
        for child in parsed.children(root) {
            assert_eq!(parsed.name(*child).unwrap().namespace, None);
        }
    }
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
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
            .collect();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}

#[test]
fn xslt10_computed_element_explicit_namespaces_discard_only_unsafe_prefixes() {
    let body = r#"<xsl:variable name="reserved" select="'xml:chosen'"/><xsl:variable name="ordinary" select="'p:chosen'"/><out><xsl:element name="xml:chosen" namespace="urn:new"/><xsl:element name="xmlns:chosen" namespace="urn:new"/><xsl:element name="{$reserved}" namespace="urn:new"/><xsl:element name="p:chosen" namespace="http://www.w3.org/XML/1998/namespace"/><xsl:element name="{$ordinary}" namespace="http://www.w3.org/XML/1998/namespace"/><xsl:element name="chosen" namespace="http://www.w3.org/XML/1998/namespace"/><xsl:element name="p:chosen" namespace="urn:new"/></out>"#;
    let program = compile(body, "1.0");
    let source = document("urn:element-namespaces:source", SOURCE);
    let actual = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let parsed = document("urn:element-namespaces:result", actual.as_bytes());
    let root = parsed.children(parsed.document_node())[0];
    let children = parsed.children(root);
    assert_eq!(children.len(), 7);
    for (index, child) in children.iter().enumerate() {
        let name = parsed.name(*child).unwrap();
        assert_eq!(name.local, "chosen");
        assert_eq!(
            name.namespace.as_deref(),
            Some(if (3..6).contains(&index) {
                "http://www.w3.org/XML/1998/namespace"
            } else {
                "urn:new"
            })
        );
    }
    assert!(!actual.contains("xmlns:xml=\"urn:new\""));
    assert!(!actual.contains("xmlns:xmlns"));
    assert!(actual.contains("<p:chosen xmlns:p=\"urn:new\""));
}

#[test]
fn computed_element_namespace_compatibility_obeys_local_static_version() {
    let source = document("urn:element-namespaces:source", SOURCE);
    for name in ["unbound:chosen", "{name()}"] {
        let body = format!(
            r#"<out xsl:version="1.0"><xsl:for-each select="doc/p:item"><xsl:element name="{name}" namespace=""/></xsl:for-each></out>"#
        );
        let program = compile(&body, "3.0");
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
    }
    // Local modern context remains on the existing rejection path.
    let stylesheet = document("urn:element-namespaces:modern", br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out xsl:version="3.0"><xsl:element name="unbound:chosen" namespace=""/></out></xsl:template></xsl:stylesheet>"#);
    assert_eq!(
        crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
            .unwrap_err()
            .code,
        "XTDE0835"
    );
    let program = compile(
        r#"<out xsl:version="3.0"><xsl:for-each select="doc/p:item"><xsl:element name="{name()}" namespace=""/></xsl:for-each></out>"#,
        "1.0",
    );
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "XTDE0830"
    );
}

#[test]
fn computed_element_explicit_namespace_retains_budgets_cancellation_and_provenance() {
    let program = compile(EMPTY_BODY, "1.0");
    let source = document("urn:element-namespaces:source", SOURCE);
    let mut measured = InvocationControl::unbounded();
    let expected = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut measured,
    )
    .unwrap();
    for domain in [WorkDomain::XPathNodeVisit, WorkDomain::ResultNode] {
        let charge = measured.consumed(domain);
        assert!(charge > 0);
        for allowance in [charge - 1, charge] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XPathNodeVisit => limits.xpath_node_visits = allowance,
                WorkDomain::ResultNode => limits.result_nodes = allowance,
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
                assert_eq!(actual.unwrap_err().code, "FXCT0002");
            }
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 0);
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
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
    let invalid = compile(
        r#"<out><xsl:element name="{doc/p:item/@name}" namespace=""/></out>"#,
        "1.0",
    );
    let source = document(
        "urn:element-namespaces:invalid-source",
        br#"<doc xmlns:p="urn:source"><p:item name="not a QName"/></doc>"#,
    );
    let error = run(
        &invalid,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.code, "XTDE0820");
    assert_eq!(
        error.location.unwrap().resource,
        "urn:element-namespaces:style"
    );
    let reserved = compile(
        r#"<out><xsl:element name="{doc/p:item/@name}" namespace="http://www.w3.org/2000/xmlns/"/></out>"#,
        "1.0",
    );
    assert_eq!(
        run(
            &reserved,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "XTDE0835"
    );
}
