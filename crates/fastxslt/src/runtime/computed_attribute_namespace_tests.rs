//! Static computed-attribute namespace construction and serialization tests.

use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

use super::{execute_program, serialize_xml};

fn document(identity: &str, bytes: &[u8]) -> Document {
    let parsed = parse_document(
        identity,
        bytes,
        ParseLimits {
            max_events: 64,
            max_depth: 16,
        },
    )
    .expect("XML should parse");
    Document::from_parsed(parsed).expect("XDM should build")
}

#[test]
fn static_namespaced_attribute_retains_a_serializable_prefix_binding() {
    let stylesheet = document(
        "memory:attribute-namespace.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="answer" namespace="urn:example:answer">forty-two</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", b"<source/>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "attribute-namespace-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "attribute-namespace-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("namespaced attribute should serialize");

    assert_eq!(
        serialized,
        r#"<out xmlns:ns0="urn:example:answer" ns0:answer="forty-two"></out>"#
    );
}

#[test]
fn computed_attribute_accepts_explicit_text_content() {
    let stylesheet = document(
        "memory:attribute-text.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="answer"><xsl:text>forty-two &amp; safe</xsl:text></xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", b"<source/>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "attribute-text-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "attribute-text-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("attribute should serialize");

    assert_eq!(serialized, r#"<out answer="forty-two &amp; safe"></out>"#);
}

#[test]
fn computed_attribute_reuses_number_evaluation_with_source_focus() {
    let stylesheet = document(
        "memory:attribute-number.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc/item"/></xsl:template><xsl:template match="item"><out><xsl:attribute name="n"><xsl:number/></xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", b"<doc><item/><item/></doc>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "attribute-number-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "attribute-number-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("numbered attributes should serialize");

    assert_eq!(serialized, r#"<out n="1"></out><out n="2"></out>"#);
}
