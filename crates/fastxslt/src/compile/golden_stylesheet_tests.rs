use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use crate::xdm::atomic_value_experiment::BuiltinAtomicType;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::{
    BooleanExpression, GlobalBindingDefault, Instruction, MatchPattern,
    STANDARD_INITIAL_TEMPLATE_NAME, TemplatePriority, ValueExpression,
};

use super::{CompileCategory, compile_stylesheet, merge_character_map_entries};

const LIMITS: ParseLimits = ParseLimits {
    max_events: 256,
    max_depth: 32,
};

fn parse_stylesheet(resource: &str, bytes: &[u8]) -> Document {
    let parsed = parse_document(resource, bytes, LIMITS).expect("stylesheet XML should parse");
    Document::from_parsed(parsed).expect("stylesheet XDM should build")
}

#[test]
fn character_map_composition_sorts_keys_and_preserves_last_entry_precedence() {
    let mut resolved = BTreeMap::new();
    merge_character_map_entries(
        &mut resolved,
        &[
            ('z', "inherited".to_owned()),
            ('a', "first".to_owned()),
            ('z', "local".to_owned()),
        ],
    );

    assert_eq!(
        resolved.into_iter().collect::<Vec<_>>(),
        vec![('a', "first".to_owned()), ('z', "local".to_owned())]
    );
}

#[test]
#[ignore = "manual release-mode character-map composition scaling measurement"]
fn measure_character_map_composition_scaling() {
    for entry_count in [100_usize, 1_000, 5_000, 10_000] {
        let entries = (0..entry_count)
            .map(|offset| {
                let scalar = u32::try_from(offset).expect("measurement size fits u32") + 0x1000;
                (
                    char::from_u32(scalar).expect("measurement scalar should be valid"),
                    format!("replacement-{offset}"),
                )
            })
            .collect::<Vec<_>>();
        let started = Instant::now();
        let mut resolved = BTreeMap::new();
        merge_character_map_entries(&mut resolved, black_box(&entries));
        let elapsed = started.elapsed();
        assert_eq!(resolved.len(), entry_count);
        eprintln!(
            "character-map-compose entries={entry_count} elapsed_us={}",
            elapsed.as_micros()
        );
    }
}

#[test]
fn compiles_the_golden_stylesheet_into_owned_semantics() {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../corpus/golden/hello/stylesheet.xsl"
    ));
    let document = parse_stylesheet("golden:hello/stylesheet.xsl", bytes);

    let program = compile_stylesheet(&document).expect("golden stylesheet should compile");

    assert_eq!(program.declared_version, "1.0");
    assert_eq!(program.output.method.as_deref(), Some("xml"));
    assert!(program.output.omit_xml_declaration);
    let [Instruction::LiteralElement { name, body, .. }] = program
        .root_template
        .as_ref()
        .expect("root template")
        .body
        .as_slice()
    else {
        panic!("root template should contain one literal result element");
    };
    assert_eq!(name.namespace, None);
    assert_eq!(name.local, "message");
    assert!(matches!(
        body.as_slice(),
        [
            Instruction::Text { value: first, .. },
            Instruction::ValueOf { select, .. },
            Instruction::Text { value: last, .. }
        ] if first == "Hello, "
            && matches!(select, ValueExpression::LocationPath(path)
                if path.steps == ["greeting", "name"])
            && last == "!"
    ));
    assert_eq!(
        program
            .root_template
            .as_ref()
            .expect("root template")
            .location
            .resource,
        "golden:hello/stylesheet.xsl"
    );
}

#[test]
fn forward_and_cyclic_global_dependencies_are_explicitly_unsupported() {
    for (label, declarations) in [
        (
            "forward",
            r#"<xsl:variable name="first" select="$later"/><xsl:variable name="later" select="7"/>"#,
        ),
        (
            "cycle",
            r#"<xsl:variable name="first" select="$later"/><xsl:variable name="later" select="$first"/>"#,
        ),
    ] {
        let stylesheet = format!(
            r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{declarations}<xsl:template match="/"/></xsl:stylesheet>"#
        );
        let document = parse_stylesheet(
            &format!("memory:{label}-global-dependency.xsl"),
            stylesheet.as_bytes(),
        );

        let failure = compile_stylesheet(&document)
            .expect_err("unordered global dependency should remain explicit");

        assert_eq!(failure.code, "FXST1044");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert!(failure.detail.contains("$first -> $later"));
    }
}

#[test]
fn backward_global_dependencies_remain_in_the_admitted_slice() {
    let document = parse_stylesheet(
            "memory:backward-global-dependency.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="earlier" select="7"/><xsl:variable name="later" select="$earlier"/><xsl:template match="/"/></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&document).expect("backward dependency should compile");

    assert_eq!(program.global_bindings.len(), 2);
}

#[test]
fn typed_string_globals_resolve_the_schema_namespace_not_the_prefix_spelling() {
    let valid = parse_stylesheet(
            "memory:typed-string-global.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:s="http://www.w3.org/2001/XMLSchema"><xsl:variable name="value" as="s:string" select="'kept'"/><xsl:variable name="constructed" as="s:untypedAtomic" select="s:untypedAtomic('')"/><xsl:variable name="empty"><xsl:sequence select="()"/></xsl:variable><xsl:variable name="enabled" as="s:boolean" select="true()"/><xsl:template match="/"/></xsl:stylesheet>"#,
        );
    let invalid = parse_stylesheet(
            "memory:false-schema-prefix.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:xs="urn:not-schema"><xsl:variable name="value" as="xs:string" select="'wrong'"/><xsl:template match="/"/></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&valid).expect("schema-qualified string should compile");
    let GlobalBindingDefault::Atomic(value) = &program.global_bindings[0].default else {
        panic!("typed global should retain atomic identity");
    };
    assert_eq!(value.atomic_type(), BuiltinAtomicType::String);
    assert_eq!(value.lexical(), "kept");
    let GlobalBindingDefault::Atomic(constructed) = &program.global_bindings[1].default else {
        panic!("typed constructor should retain atomic identity");
    };
    assert_eq!(constructed.atomic_type(), BuiltinAtomicType::UntypedAtomic);
    assert_eq!(constructed.lexical(), "");
    assert_eq!(
        program.global_bindings[2].default,
        GlobalBindingDefault::EmptySequence
    );
    let GlobalBindingDefault::Atomic(enabled) = &program.global_bindings[3].default else {
        panic!("typed boolean should retain atomic identity");
    };
    assert_eq!(enabled.atomic_type(), BuiltinAtomicType::Boolean);
    assert_eq!(enabled.lexical(), "true");
    let failure = compile_stylesheet(&invalid).expect_err("a rebound xs prefix is not schema");
    assert_eq!(failure.code, "FXST1016");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn untyped_text_global_retains_temporary_document_semantics() {
    let document = parse_stylesheet(
            "memory:temporary-text-global.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="value">text</xsl:variable><xsl:template match="/"/></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&document).expect("temporary text global should compile");

    assert_eq!(
        program.global_bindings[0].default,
        GlobalBindingDefault::TemporaryText("text".to_owned())
    );
}

#[test]
fn conditional_integer_expression_is_shared_by_test_and_value_compilation() {
    let document = parse_stylesheet(
            "memory:conditional-integer.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:if test="if (contains(doc, 'yes')) then 1 else 0"><xsl:value-of select="if (1 lt 0) then 1 else 2"/></xsl:if></xsl:template></xsl:stylesheet>"#,
        );

    let failure = compile_stylesheet(&document)
        .expect_err("the unadmitted XPath lt spelling must not be approximated");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let document = parse_stylesheet(
            "memory:conditional-integer.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:if test="if (contains(doc, 'yes')) then 1 else 0"><xsl:value-of select="if (1 &lt; 0) then 1 else 2"/></xsl:if></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&document).expect("exact conditional forms should compile");

    assert!(matches!(
        program.root_template.expect("root template").body.as_slice(),
        [Instruction::If {
            test: BooleanExpression::ConditionalInteger(_),
            body,
            ..
        }] if matches!(body.as_slice(), [Instruction::ValueOf {
            select: ValueExpression::ConditionalInteger(_),
            ..
        }])
    ));
}

#[test]
fn conditional_path_casts_resolve_the_schema_namespace() {
    let valid = parse_stylesheet(
            "memory:conditional-path.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:s="http://www.w3.org/2001/XMLSchema"><xsl:template match="/"><xsl:value-of select="if (s:integer(a/@v) > s:integer(b/@v)) then a/@v else b/@v"/></xsl:template></xsl:stylesheet>"#,
        );
    let invalid = parse_stylesheet(
            "memory:conditional-path-invalid.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:xs="urn:not-schema"><xsl:template match="/"><xsl:value-of select="if (xs:integer(a/@v) > xs:integer(b/@v)) then a/@v else b/@v"/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&valid).expect("schema-bound cast should compile");
    assert!(matches!(
        program
            .root_template
            .expect("root template")
            .body
            .as_slice(),
        [Instruction::ValueOf {
            select: ValueExpression::ConditionalPath(_),
            ..
        }]
    ));
    let failure = compile_stylesheet(&invalid).expect_err("rebound xs prefix must not be trusted");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn retains_narrow_numeric_globals_without_erasing_atomic_types_or_path_operands() {
    let document = parse_stylesheet(
            "memory:numeric-globals.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:s="http://www.w3.org/2001/XMLSchema"><xsl:variable name="zero" select="s:integer('0')"/><xsl:variable name="tiny" select="s:double('0.0001')"/><xsl:variable name="quotient" select="s:double(/doc/a div /doc/b)"/><xsl:template match="/"/></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&document).expect("numeric globals should compile");

    let GlobalBindingDefault::Atomic(zero) = &program.global_bindings[0].default else {
        panic!("integer constructor should retain atomic identity");
    };
    assert_eq!(zero.atomic_type(), BuiltinAtomicType::Integer);
    assert_eq!(zero.lexical(), "0");
    let GlobalBindingDefault::Atomic(tiny) = &program.global_bindings[1].default else {
        panic!("double constructor should retain atomic identity");
    };
    assert_eq!(tiny.atomic_type(), BuiltinAtomicType::Double);
    assert_eq!(tiny.lexical(), "0.0001");
    assert!(matches!(
        program.global_bindings[2].default,
        GlobalBindingDefault::DoubleDivision { .. }
    ));
}

#[test]
fn rejects_invalid_xml_space_on_an_admitted_choose_instruction() {
    let document = parse_stylesheet(
            "memory:invalid-choose-xml-space.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:choose xml:space="sometimes"><xsl:when test="true()"/></xsl:choose></xsl:template></xsl:stylesheet>"#,
        );

    let failure = compile_stylesheet(&document).expect_err("invalid xml:space must fail");

    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn rejects_a_choose_default_collation_list_with_no_available_member() {
    let document = parse_stylesheet(
            "memory:unavailable-choose-collation.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:choose default-collation="urn:unavailable"><xsl:when test="name(.) = 'A'"/></xsl:choose></xsl:template></xsl:stylesheet>"#,
        );

    let failure = compile_stylesheet(&document).expect_err("collation must be available");

    assert_eq!(failure.code, "XTSE0125");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn preserves_absent_output_declaration_for_runtime_method_inference() {
    let stylesheet = parse_stylesheet(
            "memory:default-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");

    assert_eq!(program.output.method, None);
    assert_eq!(program.output.version, None);
    assert_eq!(program.output.encoding, None);
    assert_eq!(program.output.media_type, None);
    assert_eq!(program.output.doctype_system, None);
    assert_eq!(program.output.doctype_public, None);
    assert_eq!(program.output.include_content_type, None);
    assert_eq!(program.output.byte_order_mark, None);
    assert_eq!(program.output.normalization_form, None);
    assert_eq!(program.output.standalone, None);
    assert!(program.output.cdata_section_elements.is_empty());
    assert!(!program.output.omit_xml_declaration);
}

#[test]
fn html_method_is_retained_without_claiming_general_serializer_support() {
    let stylesheet = parse_stylesheet(
            "memory:unsupported-general-html.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="html"/><xsl:template match="/"><html/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&stylesheet)
        .expect("known output methods are retained for serialization-time selection");
    assert_eq!(program.output.method.as_deref(), Some("html"));
    assert!(program.output.character_map.is_empty());
}

#[test]
fn retains_requested_normalization_for_serializer_capability_selection() {
    let none = parse_stylesheet(
            "memory:no-normalization.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" normalization-form="none"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let nfc = parse_stylesheet(
            "memory:nfc-normalization.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" normalization-form="NFC"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&none).expect("none should preserve result characters");
    assert_eq!(program.output.normalization_form.as_deref(), Some("none"));
    let program = compile_stylesheet(&nfc)
        .expect("the compiler should retain rather than implement normalization");
    assert_eq!(program.output.normalization_form.as_deref(), Some("NFC"));
}

#[test]
fn retains_suppress_indentation_as_expanded_names() {
    let stylesheet = parse_stylesheet(
            "memory:suppress-indentation.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:z="http://example.com/z"><xsl:output method="xml" suppress-indentation="p z:p"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet).expect("compile suppress-indentation names");
    assert_eq!(
        program.output.suppress_indentation_elements,
        [
            crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "p".to_owned(),
            },
            crate::xml::quick_xml_experiment::ExpandedName {
                namespace: Some("http://example.com/z".to_owned()),
                local: "p".to_owned(),
            },
        ]
    );
}

#[test]
fn retains_xml_10_serialization_version_and_rejects_unadmitted_versions() {
    let xml_10 = parse_stylesheet(
            "memory:xml-10-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" version="1.0"/><xsl:template match="/"><html/></xsl:template></xsl:stylesheet>"#,
        );
    let xml_11 = parse_stylesheet(
            "memory:xml-11-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" version="1.1"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&xml_10).expect("XML 1.0 serialization should compile");
    assert_eq!(program.output.version.as_deref(), Some("1.0"));
    let failure = compile_stylesheet(&xml_11).expect_err("XML 1.1 remains unadmitted");
    assert_eq!(failure.code, "FXST1021");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn retains_doctype_identifiers_as_owned_serialization_metadata() {
    let stylesheet = parse_stylesheet(
            "memory:doctype-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" doctype-system="out.dtd" doctype-public="-//EXAMPLE//DTD Test//EN"/><xsl:template match="/"><html xmlns="http://www.w3.org/1999/xhtml"/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&stylesheet).expect("DOCTYPE metadata should compile");

    assert_eq!(program.output.doctype_system.as_deref(), Some("out.dtd"));
    assert_eq!(
        program.output.doctype_public.as_deref(),
        Some("-//EXAMPLE//DTD Test//EN")
    );
}

#[test]
fn output_ignores_only_the_admitted_xml_space_control_attribute() {
    let stylesheet = parse_stylesheet(
            "memory:foreign-output-attribute.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:example"><xsl:output e:unknown="value"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

    let failure = compile_stylesheet(&stylesheet)
        .expect_err("an arbitrary foreign output attribute remains unsupported");
    assert_eq!(failure.code, "FXST1009");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn rejects_overlapping_output_properties_during_bounded_merge() {
    let stylesheet = parse_stylesheet(
            "memory:overlapping-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml"/><xsl:output method="xhtml"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&stylesheet)
        .expect_err("repeated scalar properties remain outside bounded merging");
    assert_eq!(failure.code, "FXST1018");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn unused_named_output_does_not_change_principal_output_settings() {
    let stylesheet = parse_stylesheet(
            "memory:named-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output name="secondary" method="text"/><xsl:output method="xhtml" indent="no"/><xsl:template match="/"><html xmlns="http://www.w3.org/1999/xhtml"/></xsl:template></xsl:stylesheet>"#,
        );
    let program =
        compile_stylesheet(&stylesheet).expect("unused named output should remain separate");
    assert_eq!(program.output.method.as_deref(), Some("xhtml"));
    assert_eq!(program.output.indent, Some(false));
}

#[test]
fn retains_requested_encoding_for_serializer_capability_selection() {
    let iso_8859_1 = parse_stylesheet(
            "memory:iso-8859-1-output.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" encoding="ISO-8859-1"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&iso_8859_1)
        .expect("the bounded byte lane should retain ISO-8859-1 metadata");
    assert_eq!(program.output.encoding.as_deref(), Some("ISO-8859-1"));

    let utf_16 = parse_stylesheet(
            "memory:unsupported-encoding.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" encoding="UTF-16"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&utf_16)
        .expect("the compiler should retain rather than implement the requested encoding");
    assert_eq!(program.output.encoding.as_deref(), Some("UTF-16"));
}

#[test]
fn xslt30_boolean_output_lexicals_do_not_widen_xslt20_yes_no_values() {
    let xslt30 = parse_stylesheet(
            "memory:xslt30-output-boolean.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration=" 1 "/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let xslt20 = parse_stylesheet(
            "memory:xslt20-output-boolean.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="true"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&xslt30).expect("XSLT 3.0 boolean should compile");
    let failure = compile_stylesheet(&xslt20).expect_err("XSLT 2.0 requires yes or no");

    assert!(program.output.omit_xml_declaration);
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn validates_escape_uri_attributes_for_explicit_xml_and_xhtml_methods() {
    let xml = parse_stylesheet(
            "memory:xml-escape-uri.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" escape-uri-attributes="yes"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let xml_program = compile_stylesheet(&xml).expect("the explicit XML property should compile");
    assert_eq!(xml_program.output.escape_uri_attributes, Some(true));

    let xhtml = parse_stylesheet(
            "memory:xhtml-escape-uri.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" escape-uri-attributes="yes"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let xhtml_program = compile_stylesheet(&xhtml).expect("the XHTML property should compile");
    assert_eq!(xhtml_program.output.escape_uri_attributes, Some(true));

    let invalid = parse_stylesheet(
            "memory:invalid-escape-uri.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" escape-uri-attributes="true"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid).expect_err("XSLT 2.0 requires yes or no");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn validates_and_retains_only_xhtml_version_five() {
    let invalid = parse_stylesheet(
            "memory:invalid-html-version.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" html-version="five"/><xsl:template match="/"><html xmlns="http://www.w3.org/1999/xhtml"/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid).expect_err("invalid decimal must be rejected");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);

    for lexical in ["5", "5.0", " 5.00 ", "+005.000"] {
        let bytes = format!(
            r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" html-version="{lexical}"/><xsl:template match="/"><html xmlns="http://www.w3.org/1999/xhtml"/></xsl:template></xsl:stylesheet>"#
        );
        let valid = parse_stylesheet("memory:valid-html-version.xsl", bytes.as_bytes());
        let program = compile_stylesheet(&valid).expect("XHTML version 5 should compile");
        assert_eq!(program.output.html_version.as_deref(), Some("5"));
    }

    let other = parse_stylesheet(
            "memory:other-html-version.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" html-version="+4.1"/><xsl:template match="/"><html xmlns="http://www.w3.org/1999/xhtml"/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&other).expect_err("other versions remain unsupported");
    assert_eq!(failure.code, "FXST1049");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn preserves_output_media_type_as_owned_serialization_metadata() {
    let stylesheet = parse_stylesheet(
            "memory:media-type.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" media-type="application/x-fastxslt-test+xml"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );

    let program = compile_stylesheet(&stylesheet).expect("media type should compile");

    assert_eq!(program.output.method.as_deref(), Some("xml"));
    assert_eq!(
        program.output.media_type.as_deref(),
        Some("application/x-fastxslt-test+xml")
    );
}

#[test]
fn compiles_exact_element_template_dispatch_and_modes() {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../corpus/golden/template-dispatch/stylesheet.xsl"
    ));
    let document = parse_stylesheet("golden:template-dispatch/stylesheet.xsl", bytes);

    let program = compile_stylesheet(&document).expect("dispatch stylesheet should compile");

    assert_eq!(program.matched_templates.len(), 1);
    assert!(matches!(
        &program.matched_templates[0].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
            if name.local == "item"
    ));
    assert!(matches!(
        program
            .root_template
            .as_ref()
            .expect("root template")
            .body
            .as_slice(),
        [Instruction::LiteralElement { body, .. }]
            if matches!(body.as_slice(), [Instruction::ApplyTemplates { select: Some(select), .. }]
                if matches!(select,
                    crate::xslt::golden_semantics_experiment::ApplySelection::LocationPath(path)
                        if path.steps == ["catalog", "item"]))
    ));

    let duplicate = parse_stylesheet(
            "memory:duplicate-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out/></xsl:template><xsl:template match="item"><a/></xsl:template><xsl:template match="item"><b/></xsl:template></xsl:stylesheet>"#,
        );
    let duplicate_program =
        compile_stylesheet(&duplicate).expect("XSLT 3.0 use-last conflict should compile");
    assert_eq!(duplicate_program.matched_templates.len(), 2);
    assert_eq!(
        duplicate_program.matched_templates[0].priority,
        duplicate_program.matched_templates[1].priority
    );

    let mode = parse_stylesheet(
            "memory:mode.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:apply-templates select="root/item" mode="detail"/></xsl:template><xsl:template match="item" mode="detail"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&mode).expect("unprefixed modes should compile");
    assert_eq!(program.matched_templates[0].modes, ["detail"]);

    let current_mode = parse_stylesheet(
            "memory:current-mode.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xpath-default-namespace="http://example.test/"><xsl:template match="doc"><xsl:apply-templates select="item" mode="detail"/></xsl:template><xsl:template match="item" mode="detail"><xsl:call-template name="common"/></xsl:template><xsl:template name="common"><xsl:apply-templates select="/" mode="#current"/></xsl:template><xsl:template match="/" mode="detail"><out/></xsl:template></xsl:stylesheet>"##,
        );
    let program = compile_stylesheet(&current_mode)
        .expect("current mode and namespace-insensitive root path should compile");
    assert!(matches!(
        program.named_templates[0].template.body.as_slice(),
        [Instruction::ApplyTemplates {
            select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::LocationPath(path)),
            mode: Some(mode),
            ..
        }] if path.steps.is_empty() && mode == "#current"
    ));

    let default_mode = parse_stylesheet(
            "memory:default-mode.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xpath-default-namespace="http://example.test/"><xsl:template match="doc"><xsl:apply-templates select="item" mode="#default"/></xsl:template><xsl:template match="item" mode="a b #default"><xsl:call-template name="common"/></xsl:template><xsl:template name="common"><xsl:apply-templates select="//tail" mode="#current"/></xsl:template><xsl:template match="tail"><out/></xsl:template></xsl:stylesheet>"##,
        );
    let program =
        compile_stylesheet(&default_mode).expect("default and current mode forms should compile");
    assert_eq!(program.matched_templates[1].modes, ["a", "b", "#default"]);
    assert!(matches!(
        program.named_templates[0].template.body.as_slice(),
        [Instruction::ApplyTemplates {
            select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::DescendantElement(name)),
            mode: Some(mode),
            ..
        }] if name.namespace.as_deref() == Some("http://example.test/")
            && name.local == "tail"
            && mode == "#current"
    ));
}

#[test]
fn compiles_inherited_default_mode_without_overriding_explicit_mode() {
    let stylesheet = parse_stylesheet(
            "memory:inherited-default-mode.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" default-mode="a"><out xsl:default-mode="#unnamed"><xsl:apply-templates select="doc/a"/><xsl:apply-templates select="doc/a" mode="b"/></out></xsl:template><xsl:template match="a" mode="a b"/></xsl:stylesheet>"##,
        );
    let program = compile_stylesheet(&stylesheet).expect("default mode should compile");
    assert!(program.root_template.is_none());
    let document_rule = program
        .matched_templates
        .iter()
        .find(|template| template.pattern == MatchPattern::Document)
        .expect("default-mode applies to the template rule");
    assert_eq!(document_rule.modes, ["a"]);
    assert!(matches!(
        document_rule.template.body.as_slice(),
        [Instruction::LiteralElement { body, .. }]
            if matches!(body.as_slice(), [
                Instruction::ApplyTemplates { mode: None, .. },
                Instruction::ApplyTemplates { mode: Some(mode), .. }
            ] if mode == "b")
    ));

    let stylesheet = parse_stylesheet(
            "memory:default-initial-mode.xsl",
            br##"<xsl:stylesheet version="3.0" default-mode="a" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" mode="#unnamed a"><out/></xsl:template></xsl:stylesheet>"##,
        );
    let program = compile_stylesheet(&stylesheet).expect("default initial mode should compile");
    assert_eq!(program.default_initial_mode.as_deref(), Some("a"));
    assert_eq!(program.matched_templates[0].modes, ["#unnamed", "a"]);
}

#[test]
fn compiles_only_provably_disjoint_union_rules_with_individual_priorities() {
    let stylesheet = parse_stylesheet(
            "memory:disjoint-union.xsl",
            br##"<xsl:stylesheet version="3.0" default-mode=" a " xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="v | chapter/text()" mode="#unnamed"><xsl:apply-templates mode="#unnamed"/></xsl:template></xsl:stylesheet>"##,
        );
    let program = compile_stylesheet(&stylesheet).expect("disjoint union should compile");
    assert_eq!(program.default_initial_mode.as_deref(), Some("a"));
    assert_eq!(program.matched_templates.len(), 2);
    assert_eq!(
        program.matched_templates[0].priority,
        TemplatePriority::EXACT_NAME_DEFAULT
    );
    assert_eq!(
        program.matched_templates[1].priority,
        TemplatePriority::PATH_DEFAULT
    );
    assert!(
        program
            .matched_templates
            .iter()
            .all(|rule| rule.modes == ["#unnamed"])
    );
    assert!(program.matched_templates.iter().all(|rule| matches!(
        rule.template.body.as_slice(),
        [Instruction::ApplyTemplates { mode: None, .. }]
    )));

    let overlapping = parse_stylesheet(
            "memory:overlapping-union.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="text() | chapter/text()"/></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&overlapping)
        .expect_err("potentially overlapping alternatives must remain unsupported");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn retains_bounded_exact_template_priority_and_classifies_other_lexicals() {
    let stylesheet = parse_stylesheet(
            "memory:priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc" priority="10"><out/></xsl:template><xsl:template match="node()" priority="1"><fallback/></xsl:template><xsl:template match="*"><wildcard/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet).expect("integer priorities should compile");
    assert!(program.matched_templates[0].priority > program.matched_templates[1].priority);
    assert!(program.matched_templates[1].priority > program.matched_templates[2].priority);

    let fractional = parse_stylesheet(
            "memory:fractional-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item" priority=".5"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let fractional_program =
        compile_stylesheet(&fractional).expect("bounded fractional priority should compile");
    assert_eq!(
        fractional_program.matched_templates[0].priority,
        TemplatePriority::PATH_DEFAULT
    );

    let overprecision = parse_stylesheet(
            "memory:overprecision-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item" priority=".1234567"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&overprecision)
        .expect_err("priority beyond the fixed-point domain should remain unsupported");
    assert_eq!(failure.code, "FXST1025");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let invalid = parse_stylesheet(
            "memory:invalid-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item" priority="high"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid).expect_err("invalid priority should fail");
    assert_eq!(failure.code, "FXST0030");
    assert_eq!(failure.category, CompileCategory::Invalid);

    let root = parse_stylesheet(
            "memory:root-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" priority="1"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let root_program =
        compile_stylesheet(&root).expect("explicit root priority should use typed selection");
    assert!(root_program.root_template.is_none());
    assert_eq!(root_program.matched_templates.len(), 1);
    assert_eq!(
        root_program.matched_templates[0].priority,
        TemplatePriority::explicit_integer(1)
    );

    let default_mode_root = parse_stylesheet(
            "memory:default-mode-root.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><first/></xsl:template><xsl:template match="/" mode="#default"><second/></xsl:template></xsl:stylesheet>"##,
        );
    let default_mode_program = compile_stylesheet(&default_mode_root)
        .expect("#default root should compete through typed selection");
    assert!(default_mode_program.root_template.is_none());
    assert_eq!(default_mode_program.matched_templates.len(), 2);
}

#[test]
fn compiles_bounded_attribute_presence_match_predicate() {
    let stylesheet = parse_stylesheet(
            "memory:attribute-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc/foo"><path/></xsl:template><xsl:template match="foo[@test]"><predicate/></xsl:template></xsl:stylesheet>"#,
        );
    let program =
        compile_stylesheet(&stylesheet).expect("attribute presence pattern should compile");
    assert!(matches!(
        &program.matched_templates[1].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::ElementWithAttribute {
            element,
            attribute
        } if element.local == "foo" && attribute.local == "test"
    ));
    assert_eq!(
        program.matched_templates[0].priority,
        program.matched_templates[1].priority
    );

    let comparison = parse_stylesheet(
            "memory:attribute-comparison-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo[@test='true']"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let comparison_program = compile_stylesheet(&comparison)
        .expect("exact single-quoted attribute value predicate should compile");
    assert!(matches!(
        &comparison_program.matched_templates[0].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::ElementWithAttributeValue {
            element,
            attribute,
            value
        } if element.local == "foo" && attribute.local == "test" && value == "true"
    ));

    let general_comparison = parse_stylesheet(
            "memory:general-attribute-comparison-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo[@test!='true']"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&general_comparison)
        .expect_err("general attribute comparisons must remain unsupported");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn compiles_exact_descendant_wildcard_with_non_simple_priority() {
    let stylesheet = parse_stylesheet(
            "memory:descendant-wildcard-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo"><exact/></xsl:template><xsl:template match="//*"><descendant/></xsl:template></xsl:stylesheet>"#,
        );
    let program =
        compile_stylesheet(&stylesheet).expect("exact descendant wildcard should compile");
    assert!(matches!(
        program.matched_templates[1].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::DescendantAnyElement
    ));
    assert!(program.matched_templates[1].priority > program.matched_templates[0].priority);

    let document_rooted = parse_stylesheet(
            "memory:document-rooted-descendant-wildcard-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo"><exact/></xsl:template><xsl:template match="/root//*"><descendant/></xsl:template></xsl:stylesheet>"#,
        );
    let document_rooted_program = compile_stylesheet(&document_rooted)
        .expect("document-rooted descendant wildcard should compile");
    assert!(matches!(
        document_rooted_program.matched_templates[1].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::Path(_)
    ));
    assert!(
        document_rooted_program.matched_templates[1].priority
            > document_rooted_program.matched_templates[0].priority
    );

    let named_descendant = parse_stylesheet(
            "memory:named-descendant-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="//foo"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&named_descendant)
        .expect_err("general descendant patterns must remain unsupported");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn compiles_prefixed_element_and_explicit_namespace_wildcard_patterns() {
    let stylesheet = parse_stylesheet(
            "memory:namespace-wildcard-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:bar="http://bar.example/"><xsl:template match="bar:foo" priority="5"><exact/></xsl:template><xsl:template match="bar:*" priority="5"><wildcard/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet)
        .expect("prefixed element and explicit namespace wildcard should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
            if name.namespace.as_deref() == Some("http://bar.example/") && name.local == "foo"
    ));
    assert!(matches!(
        &program.matched_templates[1].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::ElementNamespace(namespace)
            if namespace == "http://bar.example/"
    ));
    assert_eq!(
        program.matched_templates[0].priority,
        program.matched_templates[1].priority
    );

    let implicit = parse_stylesheet(
            "memory:implicit-namespace-wildcard.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:bar="http://bar.example/"><xsl:template match="bar:*"><namespace/></xsl:template><xsl:template match="*:foo"><local/></xsl:template></xsl:stylesheet>"#,
        );
    let implicit_program = compile_stylesheet(&implicit)
        .expect("namespace and local-name wildcards should retain exact quarter priority");
    assert!(matches!(
        &implicit_program.matched_templates[1].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::ElementLocal(local)
            if local == "foo"
    ));
    assert_eq!(
        implicit_program.matched_templates[0].priority,
        implicit_program.matched_templates[1].priority
    );

    let unbound = parse_stylesheet(
            "memory:unbound-match-prefix.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="bar:foo"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&unbound).expect_err("unbound prefix should be invalid");
    assert_eq!(failure.code, "FXST0031");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn compiles_xpath_default_namespace_for_simple_pattern_and_selection() {
    let stylesheet = parse_stylesheet(
            "memory:xpath-default-namespace.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc" xpath-default-namespace="http://example.test/"><out><xsl:apply-templates select="item"/></out></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet)
        .expect("simple default-namespace pattern and selection should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
            if name.namespace.as_deref() == Some("http://example.test/") && name.local == "doc"
    ));
    assert!(matches!(
        program.matched_templates[0].template.body.as_slice(),
        [Instruction::LiteralElement { body, .. }]
            if matches!(body.as_slice(), [Instruction::ApplyTemplates {
                select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::ChildElement(name)),
                ..
            }] if name.namespace.as_deref() == Some("http://example.test/") && name.local == "item")
    ));

    let path_pattern = parse_stylesheet(
            "memory:xpath-default-namespace-pattern-path.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc/item" xpath-default-namespace="http://example.test/"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&path_pattern)
        .expect_err("multi-step default-namespace pattern must not lose expanded names");
    assert_eq!(failure.code, "FXST1027");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let selection_path = parse_stylesheet(
            "memory:xpath-default-namespace-selection-path.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc" xpath-default-namespace="http://example.test/"><xsl:apply-templates select="item/child"/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&selection_path)
        .expect_err("multi-step default-namespace selection must not lose expanded names");
    assert_eq!(failure.code, "FXST1027");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let literal_context = parse_stylesheet(
            "memory:literal-xpath-default-namespace.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc"><out xsl:xpath-default-namespace="http://example.test/"><xsl:apply-templates select="item"/></out></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&literal_context)
        .expect("literal result static-context attribute should compile");
    assert!(matches!(
        program.matched_templates[0].template.body.as_slice(),
        [Instruction::LiteralElement { body, .. }]
            if matches!(body.as_slice(), [Instruction::ApplyTemplates {
                select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::ChildElement(name)),
                ..
            }] if name.namespace.as_deref() == Some("http://example.test/") && name.local == "item")
    ));

    let stylesheet_context = parse_stylesheet(
            "memory:stylesheet-xpath-default-namespace.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xpath-default-namespace="http://example.test/"><xsl:template match="doc"><xsl:apply-templates select="item"/></xsl:template><xsl:template match="@code"><xsl:value-of select="."/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet_context)
        .expect("stylesheet-wide default element namespace should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
            if name.namespace.as_deref() == Some("http://example.test/") && name.local == "doc"
    ));
    assert!(matches!(
        program.matched_templates[0].template.body.as_slice(),
        [Instruction::ApplyTemplates {
            select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::ChildElement(name)),
            ..
        }] if name.namespace.as_deref() == Some("http://example.test/") && name.local == "item"
    ));
    assert!(matches!(
        &program.matched_templates[1].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::Attribute(name)
            if name.namespace.is_none() && name.local == "code"
    ));
}

#[test]
fn distinguishes_invalid_stylesheet_from_unsupported_instruction() {
    let invalid = parse_stylesheet(
            "memory:invalid.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:value-of/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid).expect_err("missing select should fail");
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert_eq!(failure.code, "FXST0008");
    assert_eq!(failure.location.resource, "memory:invalid.xsl");

    let unsupported = parse_stylesheet(
            "memory:unsupported.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:message>unsupported</xsl:message></xsl:template></xsl:stylesheet>"#,
        );
    let failure =
        compile_stylesheet(&unsupported).expect_err("unsupported instruction should fail");
    assert_eq!(failure.category, CompileCategory::Unsupported);
    assert_eq!(failure.code, "FXST1006");
    assert_eq!(failure.location.resource, "memory:unsupported.xsl");

    let named_template = parse_stylesheet(
            "memory:named-template.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="worker"><out/></xsl:template><xsl:template match="/"><xsl:call-template name="worker"/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&named_template).expect("named template should compile");
    assert_eq!(program.named_templates.len(), 1);
    assert_eq!(program.named_templates[0].name, "worker");

    let named_and_matched = parse_stylesheet(
            "memory:named-and-matched-template.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="scan" match="*" mode="a" priority="2"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let program =
        compile_stylesheet(&named_and_matched).expect("one template may be both named and matched");
    assert_eq!(program.named_templates.len(), 1);
    assert_eq!(program.named_templates[0].name, "scan");
    assert_eq!(program.matched_templates.len(), 1);
    assert_eq!(program.matched_templates[0].modes, ["a"]);
    assert_eq!(
        program.matched_templates[0].priority,
        TemplatePriority::explicit_integer(2)
    );

    let standard_initial_template = parse_stylesheet(
            "memory:standard-initial-template.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="xsl:initial-template"><out>ok</out></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&standard_initial_template)
        .expect("the reserved standard initial-template name should compile");
    assert_eq!(
        program.named_templates[0].name,
        STANDARD_INITIAL_TEMPLATE_NAME
    );

    let unknown_call = parse_stylesheet(
            "memory:unknown-template.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:call-template name="missing"/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&unknown_call)
        .expect_err("unknown named-template references are statically invalid");
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert_eq!(failure.code, "FXST0014");
}

#[test]
fn classifies_xpath_outside_the_private_location_path_slice_as_unsupported() {
    let stylesheet = parse_stylesheet(
            "memory:path.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><value><xsl:value-of select="greeting///name"/></value></xsl:template></xsl:stylesheet>"#,
        );

    let failure = compile_stylesheet(&stylesheet).expect_err("unsupported XPath should fail");

    assert_eq!(failure.category, CompileCategory::Unsupported);
    assert_eq!(failure.code, "FXXP1001");
    assert_eq!(failure.location.resource, "memory:path.xsl");
}

#[test]
fn compiles_only_the_exact_strip_all_whitespace_reference_policy() {
    let stylesheet = parse_stylesheet(
            "memory:strip-all.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:strip-space elements="*"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet).expect("exact strip-all policy should compile");
    assert_eq!(
        program.source_whitespace,
        crate::xslt::golden_semantics_experiment::SourceWhitespacePolicy::StripAllElementWhitespace
    );

    let unsupported = parse_stylesheet(
            "memory:selective-strip.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:strip-space elements="item"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&unsupported)
        .expect_err("selective whitespace rules remain outside the reference slice");
    assert_eq!(failure.code, "FXST1043");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn xsl_text_preserves_explicit_whitespace_and_rejects_element_content() {
    let stylesheet = parse_stylesheet(
            "memory:text.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text>  kept  </xsl:text></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet).expect("xsl:text should compile");
    let root_template = program.root_template.expect("root template");
    let [Instruction::Text { value, .. }] = root_template.body.as_slice() else {
        panic!("xsl:text should lower to one owned text instruction");
    };
    assert_eq!(value, "  kept  ");

    let invalid_text = parse_stylesheet(
            "memory:invalid-text.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text><bad/></xsl:text></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid_text).expect_err("element content must fail");
    assert_eq!(failure.code, "FXST0026");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn processing_instruction_compiles_static_target_and_literal_data() {
    let stylesheet = parse_stylesheet(
            "memory:processing-instruction.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:processing-instruction name="my-pi">href="book.css" type="text/css"</xsl:processing-instruction></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet).expect("static PI should compile");
    let root_template = program.root_template.expect("root template");
    let [Instruction::ProcessingInstructionNode { target, value, .. }] =
        root_template.body.as_slice()
    else {
        panic!("xsl:processing-instruction should lower to one PI instruction");
    };
    assert_eq!(target, "my-pi");
    assert_eq!(value, "href=\"book.css\" type=\"text/css\"");

    let invalid = parse_stylesheet(
            "memory:invalid-processing-instruction.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:processing-instruction name="xml">data</xsl:processing-instruction></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid).expect_err("reserved target should fail");
    assert_eq!(failure.code, "FXST0036");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn static_integer_range_requires_a_context_independent_body() {
    let stylesheet = parse_stylesheet(
            "memory:static-range.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:for-each select="2 to 4"><item>fixed</item></xsl:for-each></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&stylesheet).expect("bounded static range should compile");
    assert!(matches!(
        program
            .root_template
            .expect("root template")
            .body
            .as_slice(),
        [Instruction::ForEachStaticIntegerRange {
            start: 2,
            end: 4,
            ..
        }]
    ));

    let unsupported = parse_stylesheet(
            "memory:static-range-focus.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:for-each select="2 to 4"><xsl:value-of select="."/></xsl:for-each></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&unsupported)
        .expect_err("atomic-focus-dependent body should stay unsupported");
    assert_eq!(failure.code, "FXST1007");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn separates_invalid_deep_equal_arity_and_collation_semantics() {
    let invalid = parse_stylesheet(
            "memory:deep-equal-arity.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="deep-equal()"/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid).expect_err("invalid deep-equal arity should fail");
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert_eq!(failure.code, "XPST0017");
    assert_eq!(failure.location.resource, "memory:deep-equal-arity.xsl");
    assert!(!failure.location.span.is_empty());

    let invalid_collation_type = parse_stylesheet(
            "memory:deep-equal-collation.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="deep-equal(1, 1, ())"/></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid_collation_type)
        .expect_err("invalid deep-equal collation type should fail");
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.location.resource, "memory:deep-equal-collation.xsl");
    assert!(!failure.location.span.is_empty());

    let composed = parse_stylesheet(
            "memory:deep-equal-composed.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="not(deep-equal((1, 2), (2, 1)))"/></xsl:template></xsl:stylesheet>"#,
        );
    let program = compile_stylesheet(&composed)
        .expect("composed deep-equal expression should use the shared owner");
    assert!(matches!(
        program
            .root_template
            .expect("root template")
            .body
            .as_slice(),
        [Instruction::ValueOf {
            select: ValueExpression::DeepEqual(_),
            ..
        }]
    ));
}
