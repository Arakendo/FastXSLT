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
fn xslt10_path_valued_attribute_namespace_uses_first_node_string_value() {
    let stylesheet = document(
        "memory:dynamic-attribute-namespace.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="{docs/a}" namespace="{docs/a/@href}">one</xsl:attribute><xsl:attribute name="static" namespace="{docs/b/@href}">two</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document(
        "memory:source.xml",
        br#"<docs><a href="urn:dynamic">chosen</a><b href=""/></docs>"#,
    );
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "dynamic-attribute-namespace-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "dynamic-attribute-namespace-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("dynamic attribute should serialize");

    assert_eq!(
        result,
        r#"<out xmlns:ns0="urn:dynamic" ns0:chosen="one" static="two"></out>"#
    );
}

#[test]
fn xslt10_context_name_attribute_uses_source_lexical_name() {
    let stylesheet = document(
        "memory:context-name-attribute.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc/item"/></xsl:template><xsl:template match="item"><out><xsl:attribute name="{name(.)}">kept</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", br"<doc><item/></doc>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "context-name-attribute-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "context-name-attribute-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("context-name attribute should serialize");

    assert_eq!(result, "<out item=\"kept\"></out>");
}

#[test]
fn xslt10_xmlns_prefix_discarding_is_explicit_and_profile_local() {
    let source = document("memory:source.xml", b"<doc><name>xmlns:chosen</name></doc>");
    for name in [
        "xmlns:chosen",
        "{'xmlns:chosen'}",
        "{doc/name}",
        "xml:chosen",
        "{'xml:chosen'}",
        "unbound:chosen",
        "{'unbound:chosen'}",
    ] {
        for namespace in ["urn:ordinary", "", "http://www.w3.org/XML/1998/namespace"] {
            let stylesheet = document(
                "memory:xmlns-discard.xsl",
                format!(r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="{name}" namespace="{namespace}">kept</xsl:attribute></out></xsl:template></xsl:stylesheet>"#).as_bytes(),
            );
            let program = compile_stylesheet(&stylesheet).unwrap();
            let result = execute_program(
                &program,
                &source,
                "xmlns-discard",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let serialized = serialize_xml(
                &result,
                &program.output,
                "xmlns-discard",
                4096,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert!(!serialized.contains("xmlns:chosen"));
            let parsed = document("memory:result.xml", serialized.as_bytes());
            let out = parsed.children(parsed.document_node())[0];
            let attributes = parsed.attributes(out);
            assert_eq!(attributes.len(), 1);
            let attribute = attributes[0];
            assert_eq!(parsed.name(attribute).unwrap().local, "chosen");
            assert_eq!(
                parsed.name(attribute).unwrap().namespace.as_deref(),
                (!namespace.is_empty()).then_some(namespace)
            );
            assert_eq!(parsed.value(attribute), Some("kept"));
        }
    }
    for (version, name, namespace) in [
        ("3.0", "xmlns:chosen", "urn:ordinary"),
        ("1.0", "xmlns:chosen", "http://www.w3.org/2000/xmlns/"),
        ("1.0", "xmlns", "urn:ordinary"),
    ] {
        let stylesheet = document(
            "memory:xmlns-invalid.xsl",
            format!(r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out><xsl:attribute name="{name}" namespace="{namespace}">bad</xsl:attribute></out></xsl:template></xsl:stylesheet>"#).as_bytes(),
        );
        assert_eq!(
            compile_stylesheet(&stylesheet).unwrap_err().code,
            "XTDE0855"
        );
    }
}

#[test]
fn explicit_attribute_namespace_uses_local_not_principal_version() {
    let source = document("memory:source.xml", b"<doc/>");
    for name in ["xmlns:chosen", "{'xmlns:chosen'}"] {
        for (principal, local, succeeds) in [("3.0", "1.0", true), ("1.0", "3.0", false)] {
            let stylesheet = document(
                "memory:local-attribute-version.xsl",
                format!(r#"<xsl:stylesheet version="{principal}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out xsl:version="{local}"><xsl:attribute name="{name}" namespace="urn:ordinary">kept</xsl:attribute></out></xsl:template></xsl:stylesheet>"#).as_bytes(),
            );
            match compile_stylesheet(&stylesheet) {
                Err(error) => {
                    assert!(!succeeds);
                    assert_eq!(
                        error.code,
                        if name.starts_with('{') {
                            "FXST1062"
                        } else {
                            "XTDE0855"
                        }
                    );
                }
                Ok(program) => {
                    let result = execute_program(
                        &program,
                        &source,
                        "local-attribute-version",
                        &mut InvocationControl::unbounded(),
                    );
                    if succeeds {
                        result.expect("local 1.0 context permits discarding the lexical prefix");
                    } else {
                        assert_eq!(result.unwrap_err().code, "XTDE0855");
                    }
                }
            }
        }
    }
}

#[test]
fn xslt10_literal_name_avt_discards_xmlns_prefix_with_explicit_ordinary_namespace() {
    let stylesheet = document(
        "memory:literal-dynamic-attribute-name.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out><xsl:attribute name="{'xmlns:bad'}" namespace="urn:not-xmlns">bad</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", br"<doc/>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "literal-dynamic-attribute-name-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("an explicit ordinary namespace permits discarding the reserved prefix");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "literal-dynamic-attribute-name-request",
        4096,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert!(serialized.contains("xmlns:ns0=\"urn:not-xmlns\""));
    assert!(serialized.contains("ns0:bad=\"bad\""));
    assert!(!serialized.contains("xmlns:bad"));
}

#[test]
fn xslt10_variable_parts_compose_one_computed_attribute_name() {
    let stylesheet = document(
        "memory:variable-dynamic-attribute-name.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:param name="left">joined</xsl:param><xsl:variable name="right">-name</xsl:variable><xsl:template match="/"><out><xsl:attribute name="{$left}{$right}">kept</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document("memory:source.xml", br"<doc/>");
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "variable-dynamic-attribute-name-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "variable-dynamic-attribute-name-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("variable-composed attribute should serialize");

    assert_eq!(result, "<out joined-name=\"kept\"></out>");
}

#[test]
fn xslt10_focus_position_composes_one_computed_attribute_name() {
    let stylesheet = document(
        "memory:position-dynamic-attribute-name.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:attribute name="same-{position()}">wrong</xsl:attribute><xsl:attribute name="same-{position()}">right</xsl:attribute><xsl:for-each select="doc/item"><xsl:attribute name="item-{position()}"><xsl:value-of select="."/></xsl:attribute></xsl:for-each></out></xsl:template></xsl:stylesheet>"#,
    );
    let source = document(
        "memory:source.xml",
        br"<doc><item>one</item><item>two</item></doc>",
    );
    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");
    let result = execute_program(
        &program,
        &source,
        "position-dynamic-attribute-name-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("stylesheet should execute");
    let result = serialize_xml(
        &result,
        &program.output,
        "position-dynamic-attribute-name-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("position-composed attribute should serialize");

    assert_eq!(
        result,
        "<out same-1=\"right\" item-1=\"one\" item-2=\"two\"></out>"
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
