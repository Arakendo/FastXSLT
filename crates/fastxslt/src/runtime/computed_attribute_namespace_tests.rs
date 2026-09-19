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

#[test]
fn computed_attribute_reuses_static_substring_folding() {
    let stylesheet = document(
        "memory:attribute-substring.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="value"><xsl:value-of select="substring('abcd', 2, 2)"/></xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", b"<doc/>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "attribute-substring-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "attribute-substring-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("substring attribute should serialize");

    assert_eq!(result, "<out value=\"bc\"></out>");
}

#[test]
fn computed_attributes_reuse_sequence_position_and_size() {
    let stylesheet = document(
        "memory:attribute-focus.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc/item"/></xsl:template><xsl:template match="item"><out><xsl:attribute name="position"><xsl:value-of select="position()"/></xsl:attribute><xsl:attribute name="size"><xsl:value-of select="last()"/></xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", b"<doc><item/><item/></doc>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "attribute-focus-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "attribute-focus-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("focus attributes should serialize");

    assert_eq!(
        result,
        "<out position=\"1\" size=\"2\"></out><out position=\"2\" size=\"2\"></out>"
    );
}

#[test]
fn xslt10_path_valued_attribute_name_uses_first_node_string_value() {
    let stylesheet = document(
        "memory:dynamic-attribute-name.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="{doc/name/@value}">kept</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document(
        "memory:source.xml",
        br#"<doc><name value="chosen"/><name value="ignored"/></doc>"#,
    );
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "dynamic-attribute-name-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "dynamic-attribute-name-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("dynamic attribute should serialize");

    assert_eq!(result, "<out chosen=\"kept\"></out>");
}

#[test]
fn path_valued_attribute_name_retains_static_namespace_override() {
    let stylesheet = document(
        "memory:dynamic-namespaced-attribute.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="{doc/name}" namespace="urn:dynamic">kept</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", br"<doc><name>chosen</name></doc>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "dynamic-namespaced-attribute-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "dynamic-namespaced-attribute-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("dynamic namespaced attribute should serialize");

    assert_eq!(
        result,
        r#"<out xmlns:ns0="urn:dynamic" ns0:chosen="kept"></out>"#
    );
}

#[test]
fn xslt10_path_valued_attribute_name_reports_invalid_empty_qname() {
    let stylesheet = document(
        "memory:empty-dynamic-attribute-name.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out><xsl:attribute name="{doc/missing}">bad</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", br"<doc/>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let failure = execute_program(
        &program,
        &source,
        "empty-dynamic-attribute-name-request",
        &mut InvocationControl::unbounded(),
    )
    .expect_err("an empty dynamic name must fail");

    assert_eq!(failure.code, "XTDE0850");
    assert_eq!(failure.category, super::FailureCategory::Invalid);
}
