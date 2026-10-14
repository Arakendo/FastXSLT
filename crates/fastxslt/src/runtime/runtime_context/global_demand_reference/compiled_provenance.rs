//! Actual compiler locations feeding the bounded diagnostic-context reference.
//! No production global representation or read-site callback is selected here.

use super::*;
use crate::compile::golden_stylesheet_experiment::{
    compile_stylesheet, compile_stylesheet_with_import_and_include,
};
use crate::runtime::golden_runtime_experiment::{FailureCategory, failure};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::Instruction;

fn document(identity: &str, body: &str) -> (Arc<Document>, String) {
    let xml = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:a="urn:a" xmlns:b="urn:b" version="1.0"><!-- é😀 -->{body}</xsl:stylesheet>"#
    );
    let doc = Document::from_parsed(
        parse_document(
            identity,
            xml.as_bytes(),
            ParseLimits {
                max_events: 1024,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap();
    (Arc::new(doc), xml)
}

#[test]
fn demand_provenance_declarations_keep_byte_spans_qualified_names_and_owned_storage() {
    let (source, xml) = document(
        "urn:declarations",
        r#"
      <xsl:variable name="a:value" select="'A'"/>
      <xsl:variable name="b:value" select="'B'"/>
      <xsl:template match="/"/>"#,
    );
    let weak = Arc::downgrade(&source);
    let program = compile_stylesheet(&source).unwrap();
    drop(source);
    assert!(weak.upgrade().is_none());
    for (name, lexical) in [("Q{urn:a}value", "a:value"), ("Q{urn:b}value", "b:value")] {
        let binding = program
            .global_bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap();
        let location = &binding.declaration_reference;
        assert_eq!(location.resource, "urn:declarations");
        assert_eq!(
            location.span.start,
            xml.find(&format!("<xsl:variable name=\"{lexical}\""))
                .unwrap()
        );
        assert!(xml[location.span.clone()].contains(lexical));
    }
}

#[test]
fn demand_provenance_module_merge_keeps_winning_declaration_not_import_site() {
    let (principal, principal_xml) = document(
        "urn:principal",
        r#"
      <xsl:import href="imported"/><xsl:include href="included"/>
      <xsl:variable name="first" select="'principal'"/>
      <xsl:template match="/"/>"#,
    );
    let (imported, _) = document(
        "urn:imported",
        r#"<xsl:variable name="first" select="'overridden'"/>"#,
    );
    let (included, included_xml) = document(
        "urn:included",
        r#"<xsl:variable name="second" select="$first"/>"#,
    );
    let program = compile_stylesheet_with_import_and_include(
        &principal,
        (&imported, imported.children(imported.document_node())[0]),
        (&included, included.children(included.document_node())[0]),
    )
    .unwrap();
    let first = program
        .global_bindings
        .iter()
        .position(|binding| binding.name == "first")
        .unwrap();
    let second = program
        .global_bindings
        .iter()
        .position(|binding| binding.name == "second")
        .unwrap();
    assert!(first < second);
    for (index, identity, xml) in [
        (first, "urn:principal", principal_xml),
        (second, "urn:included", included_xml),
    ] {
        let location = &program.global_bindings[index].declaration_reference;
        assert_eq!(location.resource, identity);
        assert!(xml[location.span.clone()].starts_with("<xsl:variable"));
    }
}

#[test]
fn demand_provenance_compiled_read_site_and_declaration_remain_distinct_in_context() {
    let (source, xml) = document(
        "urn:read",
        r#"
      <xsl:variable name="first" select="'placeholder'"/>
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>"#,
    );
    let program = compile_stylesheet(&source).unwrap();
    let Instruction::CopyOfVariable {
        variable, location, ..
    } = &program.root_template.as_ref().unwrap().body[0]
    else {
        panic!("typed variable copy");
    };
    let binding = program
        .global_bindings
        .iter()
        .position(|binding| &binding.name == variable)
        .unwrap();
    let edge = Dependency {
        binding,
        declaration: Arc::new(
            program.global_bindings[binding]
                .declaration_reference
                .clone(),
        ),
        read: Some(Arc::new(location.clone())),
    };
    assert!(xml[edge.declaration.span.clone()].starts_with("<xsl:variable"));
    assert!(xml[edge.read.as_ref().unwrap().span.clone()].starts_with("<xsl:copy-of"));
    let primary = Arc::new(failure(
        "FXAR0030",
        FailureCategory::Invalid,
        Some("request"),
        "experimental cycle",
    ));
    let context = FailureContext::capture(
        Arc::clone(&primary),
        &[edge],
        1,
        &mut InvocationControl::unbounded(),
    );
    drop(program);
    drop(source);
    assert!(Arc::ptr_eq(&primary, &context.primary));
    assert_eq!(context.end, ContextEnd::Complete);
    assert_ne!(
        context.dependencies[0].declaration.span,
        context.dependencies[0].read.as_ref().unwrap().span
    );
    assert_eq!(context.primary.location, None);
}
