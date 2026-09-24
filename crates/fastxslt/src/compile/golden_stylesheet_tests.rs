use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use crate::xdm::atomic_value_experiment::BuiltinAtomicType;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::{
    BooleanExpression, ElementConstructorOrigin, GlobalBindingDefault, Instruction,
    LiteralAttributeValue, MatchPattern, STANDARD_INITIAL_TEMPLATE_NAME, TemplatePriority,
    ValueExpression,
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
fn xslt10_key_declarations_retain_static_name_match_and_use_state() {
    let document = parse_stylesheet(
        "test:key-declaration.xsl",
        br#"<xsl:stylesheet version="1.0"
              xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
              xmlns:k="urn:key">
              <xsl:key name="k:codes" match="item" use="@code"/>
              <xsl:key name="k:codes" match="entry" use="."/>
              <xsl:template match="/"/>
            </xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("bounded key declarations should compile");
    assert_eq!(program.key_definitions.len(), 2);
    assert!(program.key_definitions.iter().all(|definition| {
        definition.name.namespace.as_deref() == Some("urn:key") && definition.name.local == "codes"
    }));
    assert!(matches!(
        program.key_definitions[0].match_pattern,
        MatchPattern::Element(_)
    ));
}

#[test]
fn xslt10_key_use_rejects_variable_and_recursive_key_dependencies() {
    for use_expression in ["$value", "key('other', @code)"] {
        let bytes = format!(
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="{}">
                  <xsl:key name="codes" match="item" use="{}"/>
                  <xsl:template match="/"/>
                </xsl:stylesheet>"#,
            super::XSLT_NAMESPACE,
            use_expression
        );
        let document = parse_stylesheet("test:invalid-key-use.xsl", bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("key dependency must be rejected");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "XTSE1205");
    }
}

#[test]
fn xslt10_key_lookup_with_variable_name_compiles_as_a_typed_plan() {
    let document = parse_stylesheet(
        "test:dynamic-key-name.xsl",
        br#"<xsl:stylesheet version="1.0"
              xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:param name="keysp" select="'sections'"/>
              <xsl:key name="sections" match="section" use="@name"/>
              <xsl:template match="/">
                <xsl:value-of select="key($keysp, 'Introduction')/subdiv/p"/>
              </xsl:template>
            </xsl:stylesheet>"#,
    );

    compile_stylesheet(&document).expect("variable key name should compile");
}

#[test]
fn xslt10_static_key_match_patterns_compile_as_typed_plans() {
    let document = parse_stylesheet(
        "test:key-match-pattern.xsl",
        br#"<xsl:stylesheet version="1.0"
              xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:key name="sections" match="section" use="title"/>
              <xsl:template match="key('sections', 'Introduction')/p"/>
            </xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("static key match pattern should compile");
    assert!(matches!(
        program.matched_templates[0].pattern,
        MatchPattern::Xslt10KeyLookup(_)
    ));
}

#[test]
fn xslt10_namespace_alias_rewrites_literal_result_names_and_bindings() {
    let document = parse_stylesheet(
        "test:namespace-alias.xsl",
        br#"<xsl:stylesheet version="1.0"
              xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
              xmlns:axsl="urn:literal-xsl"
              exclude-result-prefixes="axsl">
              <xsl:namespace-alias stylesheet-prefix="axsl" result-prefix="xsl"/>
              <xsl:template match="/">
                <axsl:stylesheet axsl:version="1.0" plain="kept"><axsl:template/></axsl:stylesheet>
              </xsl:template>
            </xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("static namespace alias should compile");
    let template = program.root_template.expect("root template");
    let [
        Instruction::LiteralElement {
            name,
            namespaces,
            attributes,
            body,
            ..
        },
    ] = template.body.as_slice()
    else {
        panic!("namespace alias test should retain one literal result element");
    };
    assert_eq!(name.namespace.as_deref(), Some(super::XSLT_NAMESPACE));
    assert!(namespaces.iter().any(|binding| {
        binding.prefix.as_deref() == Some("xsl") && binding.namespace == super::XSLT_NAMESPACE
    }));
    assert_eq!(
        attributes[0].name.namespace.as_deref(),
        Some(super::XSLT_NAMESPACE)
    );
    assert_eq!(attributes[1].name.namespace, None);
    let [Instruction::LiteralElement { name, .. }] = body.as_slice() else {
        panic!("nested aliased literal result element should be preserved");
    };
    assert_eq!(name.namespace.as_deref(), Some(super::XSLT_NAMESPACE));
}

#[test]
fn xslt10_namespace_alias_validates_required_bound_prefixes() {
    for (declaration, code) in [
        (r#"<xsl:namespace-alias result-prefix="xsl"/>"#, "XTSE0010"),
        (
            r#"<xsl:namespace-alias stylesheet-prefix="missing" result-prefix="xsl"/>"#,
            "XTSE0812",
        ),
    ] {
        let bytes = format!(
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="{}">{}<xsl:template match="/"/></xsl:stylesheet>"#,
            super::XSLT_NAMESPACE,
            declaration
        );
        let document = parse_stylesheet("test:invalid-namespace-alias.xsl", bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("invalid alias must fail");
        assert_eq!(failure.code, code);
        assert_eq!(failure.category, CompileCategory::Invalid);
    }
}

#[test]
fn xslt10_namespace_alias_treats_an_undeclared_default_as_no_namespace() {
    let document = parse_stylesheet(
        "test:default-namespace-alias.xsl",
        br##"<xsl:stylesheet version="1.0"
              xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:namespace-alias stylesheet-prefix="#default" result-prefix="xsl"/>
              <xsl:template match="/"><generated/></xsl:template>
            </xsl:stylesheet>"##,
    );

    let program = compile_stylesheet(&document).expect("the null namespace can be aliased");
    let template = program.root_template.expect("root template");
    let [Instruction::LiteralElement { name, .. }] = template.body.as_slice() else {
        panic!("default alias should retain one literal element");
    };
    assert_eq!(name.namespace.as_deref(), Some(super::XSLT_NAMESPACE));
}

#[test]
fn xslt10_decimal_formats_are_statically_resolved() {
    let unnamed = parse_stylesheet(
        "test:decimal-format.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:decimal-format decimal-separator="|" grouping-separator="."/>
              <xsl:template match="/"><xsl:value-of select="format-number(931.4857, '000.000|###')"/></xsl:template>
            </xsl:stylesheet>"#,
    );
    compile_stylesheet(&unnamed).expect("unnamed decimal format should compile statically");

    let named = parse_stylesheet(
        "test:named-decimal-format.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:decimal-format name="european" decimal-separator="," grouping-separator="."/>
              <xsl:template match="/"><xsl:value-of select="format-number(1234.5, '#.##0,0', 'european')"/></xsl:template>
            </xsl:stylesheet>"#,
    );
    compile_stylesheet(&named).expect("static named format lookup should compile");

    let missing = parse_stylesheet(
        "test:missing-decimal-format.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:template match="/"><xsl:value-of select="format-number(1, '0', 'missing')"/></xsl:template>
            </xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&missing).expect_err("missing format must remain explicit");
    assert_eq!(failure.code, "FXST1092");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn format_number_invalid_arity_is_a_static_error() {
    for select in [
        "format-number()",
        "format-number(1)",
        "format-number(1, '0', 'named', 'extra')",
    ] {
        let bytes = format!(
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="{}"><xsl:template match="/"><xsl:value-of select="{select}"/></xsl:template></xsl:stylesheet>"#,
            super::XSLT_NAMESPACE
        );
        let document = parse_stylesheet("test:format-number-arity.xsl", bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("invalid arity must fail");
        assert_eq!(failure.code, "XPST0017", "{select}");
        assert_eq!(failure.category, CompileCategory::Invalid, "{select}");
    }
}

#[test]
fn format_number_missing_first_argument_is_invalid_xpath_syntax() {
    let document = parse_stylesheet(
        "test:format-number-missing-first-argument.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="format-number(,'#')"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&document).expect_err("missing expression must fail");
    assert_eq!(failure.code, "XPST0003");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn sort_controls_fold_exact_literal_avts() {
    let document = parse_stylesheet(
        "test:sort-static-avt.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:template match="/"><xsl:for-each select="doc/item">
                <xsl:sort select="@rank" data-type="{'number'}" order="{'descending'}"/>
              </xsl:for-each></xsl:template>
            </xsl:stylesheet>"#,
    );
    compile_stylesheet(&document).expect("literal sort-control AVTs should fold statically");

    let dynamic = parse_stylesheet(
        "test:sort-dynamic-avt.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:param name="kind" select="'number'"/>
              <xsl:template match="/"><xsl:for-each select="doc/item">
                <xsl:sort select="@rank" data-type="{$kind}"/>
              </xsl:for-each></xsl:template>
            </xsl:stylesheet>"#,
    );
    compile_stylesheet(&dynamic).expect("one variable sort control should compile");

    let wider_dynamic = parse_stylesheet(
        "test:sort-dynamic-path-avt.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
              <xsl:template match="/"><xsl:for-each select="doc/item">
                <xsl:sort select="@rank" data-type="{../kind}"/>
              </xsl:for-each></xsl:template>
            </xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&wider_dynamic).expect_err("path sort control stays explicit");
    assert_eq!(failure.code, "FXST1044");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    for value in ["", "unknown", "number;text"] {
        let invalid = parse_stylesheet(
            "test:sort-invalid-static-data-type.xsl",
            format!(
                r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:for-each select="doc/item"><xsl:sort data-type="{value}"/></xsl:for-each></xsl:template></xsl:stylesheet>"#
            )
            .as_bytes(),
        );
        let failure = compile_stylesheet(&invalid).expect_err("invalid static data-type must fail");
        assert_eq!(failure.code, "XTDE0030");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    let extension = parse_stylesheet(
        "test:sort-extension-data-type.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:ext="urn:example"><xsl:template match="/"><xsl:for-each select="doc/item"><xsl:sort data-type="ext:custom"/></xsl:for-each></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&extension)
        .expect_err("valid extension sort data-type remains unsupported");
    assert_eq!(failure.code, "FXST1044");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn xslt10_decimal_format_rejects_conflicts_and_non_distinct_symbols() {
    for (declarations, code) in [
        (
            r#"<xsl:decimal-format decimal-separator="."/><xsl:decimal-format decimal-separator=","/>"#,
            "XTSE1290",
        ),
        (
            r#"<xsl:decimal-format decimal-separator="!" grouping-separator="!"/>"#,
            "XTSE0020",
        ),
    ] {
        let bytes = format!(
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="{}">{declarations}<xsl:template match="/"/></xsl:stylesheet>"#,
            super::XSLT_NAMESPACE
        );
        let document = parse_stylesheet("test:invalid-decimal-format.xsl", bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("invalid decimal format must fail");
        assert_eq!(failure.code, code);
        assert_eq!(failure.category, CompileCategory::Invalid);
    }
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
            && matches!(select, ValueExpression::Xslt10FirstNodeLocationPath(path)
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
fn keeps_key_number_patterns_inside_xslt10_compatibility() {
    let modern = parse_stylesheet(
        "memory:modern-key-number-pattern.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:key name="selected" match="note" use="@flag"/><xsl:template match="note"><xsl:number count="key('selected','yes')"/></xsl:template></xsl:stylesheet>"#,
    );

    let failure = compile_stylesheet(&modern)
        .expect_err("the compatibility-only key number pattern must not widen modern semantics");

    assert_eq!(failure.code, "FXST1050");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn compiles_a_simplified_stylesheet_through_the_literal_result_element_path() {
    let document = parse_stylesheet(
        "test:simplified.xsl",
        br#"<out xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                  xsl:version="1.0"
                  marker="yes">
                <xsl:value-of select="/*/value"/>
              </out>"#,
    );

    let program = compile_stylesheet(&document).expect("simplified stylesheet should compile");

    assert_eq!(program.declared_version, "1.0");
    let [
        Instruction::LiteralElement {
            name,
            attributes,
            body,
            ..
        },
    ] = program
        .root_template
        .as_ref()
        .expect("simplified stylesheet has an implicit root template")
        .body
        .as_slice()
    else {
        panic!("simplified stylesheet should compile as one literal result element");
    };
    assert_eq!(name.local, "out");
    assert_eq!(attributes.len(), 1);
    assert_eq!(attributes[0].name.local, "marker");
    assert_eq!(
        attributes[0].value,
        LiteralAttributeValue::Text("yes".to_owned())
    );
    assert!(matches!(
        body.as_slice(),
        [Instruction::ValueOf {
            select: ValueExpression::Xslt10FirstNodeLocationPath(_),
            ..
        }]
    ));
}

#[test]
fn rejects_an_xslt_instruction_as_a_simplified_stylesheet_root() {
    let document = parse_stylesheet(
        "test:invalid-simplified-root.xsl",
        br#"<xsl:template xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0" match="/"/>"#,
    );
    let failure = compile_stylesheet(&document)
        .expect_err("an XSLT instruction is not a literal-result simplified stylesheet root");

    assert_eq!(failure.code, "XTSE0010");
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert!(failure.detail.contains("literal result element"));
}

#[test]
fn ignores_foreign_top_level_data_and_rejects_unqualified_top_level_elements() {
    let foreign = parse_stylesheet(
        "memory:foreign-top-level.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:help="urn:help" version="1.0"><help:metadata expression="{not-an-avt}"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&foreign).expect("foreign top-level data should be ignored");
    assert!(program.root_template.is_some());

    let unqualified = parse_stylesheet(
        "memory:unqualified-top-level.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><metadata/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&unqualified).expect_err("unqualified top-level is invalid");
    assert_eq!(failure.code, "FXST1003");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn compiles_static_unprefixed_xsl_element_without_calling_it_literal() {
    let document = parse_stylesheet(
        "memory:static-computed-element.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:template match="/"><xsl:element name="out">value</xsl:element></xsl:template>
        </xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("static xsl:element should compile");
    let [
        Instruction::LiteralElement {
            origin, name, body, ..
        },
    ] = program
        .root_template
        .as_ref()
        .expect("root template")
        .body
        .as_slice()
    else {
        panic!("root template should contain one element constructor");
    };
    assert_eq!(*origin, ElementConstructorOrigin::ComputedStatic);
    assert_eq!(name.namespace, None);
    assert_eq!(name.local, "out");
    assert!(matches!(body.as_slice(), [Instruction::Text { value, .. }] if value == "value"));
}

#[test]
fn rejects_an_unknown_xsl_number_level_as_invalid() {
    let document = parse_stylesheet(
        "memory:invalid-number-level.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><xsl:number level="unknown"/></xsl:template></xsl:stylesheet>"#,
    );

    let failure = compile_stylesheet(&document).expect_err("unknown level must be invalid");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn rejects_non_decimal_stylesheet_versions_and_mode_on_named_only_templates() {
    for (label, stylesheet, code) in [
        (
            "version",
            br#"<xsl:stylesheet version="Hello" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"/>"#
                .as_slice(),
            "XTSE0110",
        ),
        (
            "named-only-mode",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="worker" mode="named"/></xsl:stylesheet>"#
                .as_slice(),
            "XTSE0500",
        ),
    ] {
        let document = parse_stylesheet(&format!("memory:{label}.xsl"), stylesheet);
        let failure = compile_stylesheet(&document).expect_err("invalid stylesheet must fail");
        assert_eq!(failure.code, code, "{label}");
        assert_eq!(failure.category, CompileCategory::Invalid, "{label}");
    }
}

#[test]
fn rejects_forbidden_mode_and_malformed_extension_prefixes_on_stylesheet_root() {
    for (label, attribute, code) in [
        ("root-mode", "mode=\"named\"", "XTSE0090"),
        (
            "xslt-namespace-attribute",
            "xsl:use-attribute-sets=\"common\"",
            "XTSE0090",
        ),
        (
            "extension-prefix",
            "extension-element-prefixes=\"foo:bar\"",
            "XTSE1430",
        ),
    ] {
        let bytes = format!(
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" {attribute}/>"#
        );
        let document = parse_stylesheet(&format!("memory:{label}.xsl"), bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("invalid root control must fail");
        assert_eq!(failure.code, code, "{label}");
        assert_eq!(failure.category, CompileCategory::Invalid, "{label}");
    }
}

#[test]
fn declared_extension_elements_compile_only_standard_fallback_content() {
    let prefixed = parse_stylesheet(
        "memory:extension-prefixed.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:extension" extension-element-prefixes="e"><xsl:template match="/"><e:invoke xmlns:n="urn:fallback-scope" xsl:exclude-result-prefixes="n"><ignored/><xsl:fallback xsl:exclude-result-prefixes="n"><out/></xsl:fallback></e:invoke></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&prefixed).expect("standard fallback should compile");
    let root_template = program.root_template.expect("root template");
    let [
        Instruction::LiteralElement {
            name, namespaces, ..
        },
    ] = root_template.body.as_slice()
    else {
        panic!("only the fallback result element should be retained");
    };
    assert_eq!(name.local, "out");
    assert!(namespaces.iter().any(|binding| {
        binding.prefix.as_deref() == Some("n") && binding.namespace == "urn:fallback-scope"
    }));

    let without_fallback = parse_stylesheet(
        "memory:extension-default.xsl",
        br##"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns="urn:extension" extension-element-prefixes="#default"><xsl:template match="/"><invoke/></xsl:template></xsl:stylesheet>"##,
    );
    let failure = compile_stylesheet(&without_fallback)
        .expect_err("extension execution without fallback is unsupported");
    assert_eq!(failure.code, "FXST1059");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let declaration_only = parse_stylesheet(
        "memory:extension-declaration-only.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:extension" extension-element-prefixes="e"><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    compile_stylesheet(&declaration_only)
        .expect("an unused extension declaration does not require execution support");
}

#[test]
fn computed_element_keeps_dynamic_namespaces_and_unknown_attribute_sets_explicit() {
    for (attribute, code) in [
        ("name=\"out\" namespace=\"{namespace-uri()}\"", "FXST1045"),
        ("name=\"out\" use-attribute-sets=\"common\"", "FXST1065"),
    ] {
        let bytes = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:test" version="1.0">
              <xsl:template match="/"><xsl:element {attribute}/></xsl:template>
            </xsl:stylesheet>"#
        );
        let document = parse_stylesheet("memory:bounded-computed-element.xsl", bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("unadmitted form should fail");
        assert_eq!(failure.code, code);
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }
}

#[test]
fn computed_element_distinguishes_invalid_static_qnames_from_path_names() {
    for name in ["", ":foo", "foo:", "a:b:c", "not a name"] {
        let bytes = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><xsl:element name="{name}"/></xsl:template></xsl:stylesheet>"#
        );
        let document = parse_stylesheet("memory:invalid-computed-name.xsl", bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("invalid QName must fail");
        assert_eq!(failure.code, "XTDE0820", "{name}");
        assert_eq!(failure.category, CompileCategory::Invalid, "{name}");
    }

    let document = parse_stylesheet(
        "memory:dynamic-computed-name.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><xsl:element name="{@name}"/></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&document).expect("path name should compile");
    assert!(matches!(
        program.root_template.as_ref().unwrap().body.as_slice(),
        [Instruction::DynamicNameElement { .. }]
    ));

    let modern = parse_stylesheet(
        "memory:modern-dynamic-computed-name.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:element name="{@name}"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&modern).expect_err("modern semantics remain independent");
    assert_eq!(failure.code, "FXST1047");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn local_attribute_set_graph_rejects_undefined_and_circular_references() {
    for (label, declaration, code) in [
        (
            "undefined",
            r#"<xsl:attribute-set name="outer" use-attribute-sets="missing"/>"#,
            "XTSE0710",
        ),
        (
            "self-cycle",
            r#"<xsl:attribute-set name="outer" use-attribute-sets="outer"/>"#,
            "XTSE0720",
        ),
        (
            "indirect-cycle",
            r#"<xsl:attribute-set name="outer" use-attribute-sets="inner"/><xsl:attribute-set name="inner" use-attribute-sets="outer"/>"#,
            "XTSE0720",
        ),
    ] {
        let stylesheet = format!(
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{declaration}<xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#
        );
        let document = parse_stylesheet(
            &format!("memory:attribute-set-{label}.xsl"),
            stylesheet.as_bytes(),
        );
        let failure = compile_stylesheet(&document).expect_err("invalid graph should fail");
        assert_eq!(failure.code, code, "{label}");
        assert_eq!(failure.category, CompileCategory::Invalid, "{label}");
    }
}

#[test]
fn compiles_static_xsl_element_namespace_without_runtime_qname_work() {
    let document = parse_stylesheet(
        "memory:static-computed-element-namespace.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:template match="/"><xsl:element name="out" namespace="urn:result"/></xsl:template>
        </xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("static namespace should compile");
    let root_template = program.root_template.as_ref().expect("root template");
    let [
        Instruction::LiteralElement {
            origin,
            name,
            namespaces,
            ..
        },
    ] = root_template.body.as_slice()
    else {
        panic!("root template should contain one computed element");
    };
    assert_eq!(*origin, ElementConstructorOrigin::ComputedStatic);
    assert_eq!(name.namespace.as_deref(), Some("urn:result"));
    assert_eq!(name.local, "out");
    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces[0].prefix, None);
    assert_eq!(namespaces[0].namespace, "urn:result");
}

#[test]
fn static_xsl_element_rejects_the_reserved_xmlns_namespace() {
    let document = parse_stylesheet(
        "memory:reserved-computed-element-namespace.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:template match="/"><xsl:element name="out" namespace="http://www.w3.org/2000/xmlns/"/></xsl:template>
        </xsl:stylesheet>"#,
    );

    let failure = compile_stylesheet(&document).expect_err("reserved namespace must fail");
    assert_eq!(failure.code, "XTDE0835");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn unprefixed_static_xsl_element_name_uses_its_in_scope_default_namespace() {
    let document = parse_stylesheet(
        "memory:inherited-computed-element-namespace.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:template match="/"><outer xmlns="urn:outer"><xsl:element name="inner"/></outer></xsl:template>
        </xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("in-scope default namespace should compile");
    let root = program.root_template.as_ref().expect("root template");
    let [Instruction::LiteralElement { body, .. }] = root.body.as_slice() else {
        panic!("root template should contain the outer literal element");
    };
    let [
        Instruction::LiteralElement {
            origin,
            name,
            namespaces,
            ..
        },
    ] = body.as_slice()
    else {
        panic!("outer element should contain the computed element");
    };
    assert_eq!(*origin, ElementConstructorOrigin::ComputedStatic);
    assert_eq!(name.namespace.as_deref(), Some("urn:outer"));
    assert_eq!(namespaces[0].prefix, None);
    assert_eq!(namespaces[0].namespace, "urn:outer");
}

#[test]
fn compiles_static_prefixed_xsl_element_with_its_required_binding() {
    let document = parse_stylesheet(
        "memory:static-prefixed-computed-element.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:test" version="1.0">
          <xsl:template match="/"><xsl:element name="p:out"/></xsl:template>
        </xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("prefixed xsl:element should compile");
    let [
        Instruction::LiteralElement {
            origin,
            name,
            namespaces,
            ..
        },
    ] = program
        .root_template
        .as_ref()
        .expect("root template")
        .body
        .as_slice()
    else {
        panic!("root template should contain one element constructor");
    };
    assert_eq!(*origin, ElementConstructorOrigin::ComputedStatic);
    assert_eq!(name.namespace.as_deref(), Some("urn:test"));
    assert_eq!(name.local, "out");
    assert_eq!(namespaces.len(), 1);
    assert_eq!(namespaces[0].prefix.as_deref(), Some("p"));
    assert_eq!(namespaces[0].namespace, "urn:test");
}

#[test]
fn forward_global_dependencies_are_ordered_before_materialization() {
    let document = parse_stylesheet(
        "memory:forward-global-dependency.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="first" select="$later"/><xsl:variable name="later" select="7"/><xsl:template match="/"/></xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&document).expect("forward dependency should compile");

    assert_eq!(program.global_bindings.len(), 2);
    assert_eq!(program.global_bindings[0].name, "later");
    assert_eq!(program.global_bindings[1].name, "first");
}

#[test]
fn xslt10_content_global_dependencies_are_ordered_and_cycles_rejected() {
    let forward = parse_stylesheet(
        "memory:xslt10-content-forward-global-dependency.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="first"><xsl:value-of select="$later"/></xsl:variable><xsl:variable name="later"><xsl:value-of select="'value'"/></xsl:variable><xsl:template match="/"/></xsl:stylesheet>"#,
    );
    let cycle = parse_stylesheet(
        "memory:xslt10-content-global-cycle.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="first"><xsl:value-of select="$later"/></xsl:variable><xsl:variable name="later"><xsl:value-of select="$first"/></xsl:variable><xsl:template match="/"/></xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&forward).expect("forward content dependency should compile");
    assert_eq!(program.global_bindings[0].name, "later");
    assert_eq!(program.global_bindings[1].name, "first");

    let failure = compile_stylesheet(&cycle).expect_err("content dependency cycle should fail");
    assert_eq!(failure.code, "XTDE0640");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn cyclic_global_dependencies_are_rejected() {
    let document = parse_stylesheet(
        "memory:cyclic-global-dependency.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="first" select="$later"/><xsl:variable name="later" select="$first"/><xsl:template match="/"/></xsl:stylesheet>"#,
    );

    let failure = compile_stylesheet(&document).expect_err("cycle should fail statically");

    assert_eq!(failure.code, "XTDE0640");
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert!(failure.detail.contains("circular global dependency"));
}

#[test]
fn text_sort_collation_and_dynamic_numeric_metadata_remain_explicit() {
    let text = parse_stylesheet(
        "memory:text-sort-collation.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:for-each select="doc/n"><xsl:sort lang="en"/></xsl:for-each></xsl:template></xsl:stylesheet>"#,
    );
    let dynamic_numeric = parse_stylesheet(
        "memory:dynamic-numeric-sort-metadata.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:for-each select="doc/n"><xsl:sort data-type="number" lang="{$lang}"/></xsl:for-each></xsl:template></xsl:stylesheet>"#,
    );

    assert_eq!(
        compile_stylesheet(&text)
            .expect_err("text collation should remain unsupported")
            .code,
        "FXST1063"
    );
    assert_eq!(
        compile_stylesheet(&dynamic_numeric)
            .expect_err("dynamic ignored metadata should remain unsupported")
            .code,
        "FXST1064"
    );
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
fn html_method_is_retained_for_shared_serializer_selection() {
    let stylesheet = parse_stylesheet(
            "memory:general-html.xsl",
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

    let xslt10 = parse_stylesheet(
        "memory:xslt10-overlapping-output.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" encoding="US-ASCII" omit-xml-declaration="yes"/><xsl:output method="text" encoding="UTF-8" omit-xml-declaration="no"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&xslt10)
        .expect("XSLT 1.0 may recover conflicting output properties by choosing last");
    assert_eq!(program.output.method.as_deref(), Some("text"));
    assert_eq!(program.output.encoding.as_deref(), Some("UTF-8"));
    assert!(!program.output.omit_xml_declaration);
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
fn rejects_invalid_template_mode_lists_without_treating_them_as_unsupported() {
    for (label, version, mode) in [
        ("empty", "1.0", ""),
        ("invalid-qname", "1.0", "::"),
        ("xslt10-list", "1.0", "one two"),
        ("xslt10-reserved", "1.0", "#all"),
        ("duplicate", "3.0", "one one"),
        ("all-plus-name", "3.0", "#all one"),
    ] {
        let bytes = format!(
            r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" mode="{mode}"/></xsl:stylesheet>"#
        );
        let document = parse_stylesheet(&format!("memory:{label}.xsl"), bytes.as_bytes());
        let failure = compile_stylesheet(&document).expect_err("invalid mode list must fail");
        assert_eq!(failure.code, "XTSE0550", "{label}");
        assert_eq!(failure.category, CompileCategory::Invalid, "{label}");
    }
}

#[test]
fn rejects_invalid_template_invocation_children_as_static_errors() {
    for (label, instruction) in [
        (
            "apply-templates",
            "<xsl:apply-templates><xsl:text>bad</xsl:text></xsl:apply-templates>",
        ),
        (
            "apply-imports",
            "<xsl:apply-imports><xsl:text>bad</xsl:text></xsl:apply-imports>",
        ),
        (
            "call-template",
            "<xsl:call-template name=\"worker\"><xsl:sort/></xsl:call-template>",
        ),
    ] {
        let bytes = format!(
            r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/">{instruction}</xsl:template><xsl:template name="worker"/></xsl:stylesheet>"#
        );
        let document = parse_stylesheet(&format!("memory:{label}.xsl"), bytes.as_bytes());
        let failure =
            compile_stylesheet(&document).expect_err("invalid invocation child must fail");
        assert_eq!(failure.code, "XTSE0010", "{label}");
        assert_eq!(failure.category, CompileCategory::Invalid, "{label}");
    }
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
fn compiles_union_rules_with_individual_default_priorities() {
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
    let overlapping_program = compile_stylesheet(&overlapping)
        .expect("overlapping alternatives retain their individual default priorities");
    assert_eq!(overlapping_program.matched_templates.len(), 2);
    assert_eq!(
        overlapping_program.matched_templates[0].priority,
        TemplatePriority::NODE_TEST_DEFAULT
    );
    assert_eq!(
        overlapping_program.matched_templates[1].priority,
        TemplatePriority::PATH_DEFAULT
    );
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
fn canonicalizes_expanded_axis_wildcard_patterns() {
    let stylesheet = parse_stylesheet(
        "memory:expanded-axis-patterns.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="attribute::*"/><xsl:template match="child::*"/></xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&stylesheet).expect("expanded axis wildcard patterns");

    assert_eq!(program.matched_templates.len(), 2);
    assert!(matches!(
        program.matched_templates[0].pattern,
        MatchPattern::AnyAttribute
    ));
    assert!(matches!(
        program.matched_templates[1].pattern,
        MatchPattern::AnyElement
    ));
    assert!(
        program
            .matched_templates
            .iter()
            .all(|rule| rule.priority == TemplatePriority::NODE_TEST_DEFAULT)
    );

    let node_test = parse_stylesheet(
        "memory:expanded-attribute-node-pattern.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="attribute::node()"/></xsl:stylesheet>"#,
    );
    let node_test_program =
        compile_stylesheet(&node_test).expect("expanded attribute node-test pattern");
    assert!(matches!(
        node_test_program.matched_templates[0].pattern,
        MatchPattern::AnyAttribute
    ));
    assert_eq!(
        node_test_program.matched_templates[0].priority,
        TemplatePriority::NODE_TEST_DEFAULT
    );
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

    let wildcard = parse_stylesheet(
        "memory:wildcard-attribute-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="*[@test]"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let wildcard_program = compile_stylesheet(&wildcard)
        .expect("wildcard element attribute-presence pattern should compile");
    assert!(matches!(
        &wildcard_program.matched_templates[0].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::AnyElementWithAttribute(attribute)
            if attribute.namespace.is_none() && attribute.local == "test"
    ));
    assert_eq!(
        wildcard_program.matched_templates[0].priority,
        TemplatePriority::PATH_DEFAULT
    );

    let generalized = parse_stylesheet(
        "memory:generalized-attribute-patterns.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="*[@test='true']"/><xsl:template match="node()[@test]"/><xsl:template match="*[.=117]"/><xsl:template match="*[@value=4]"/></xsl:stylesheet>"#,
    );
    let generalized_program = compile_stylesheet(&generalized)
        .expect("generalized bounded attribute patterns should compile");
    assert!(matches!(
        &generalized_program.matched_templates[0].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::AnyElementWithAttributeValue {
            attribute,
            value,
        } if attribute.local == "test" && value == "true"
    ));
    assert!(matches!(
        &generalized_program.matched_templates[1].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::AnyElementWithAttribute(attribute)
            if attribute.local == "test"
    ));
    assert!(matches!(
        generalized_program.matched_templates[2].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::AnyElementNumberEquals(117)
    ));
    assert!(matches!(
        &generalized_program.matched_templates[3].pattern,
        crate::xslt::golden_semantics_experiment::MatchPattern::AnyElementWithAttributeNumberEquals {
            attribute,
            value: 4,
        } if attribute.local == "value"
    ));

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
fn compiles_exact_node_string_value_match_predicates() {
    use crate::xslt::golden_semantics_experiment::{MatchNodeTest, MatchStringPredicate};

    let stylesheet = parse_stylesheet(
        "memory:node-string-value-patterns.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="letter[.='b']"/><xsl:template match="text()[.='text']"/><xsl:template match="comment()[.='comment']"/><xsl:template match="processing-instruction('target')[.='data']"/><xsl:template match="letter[.='b' or .='h']"/><xsl:template match="letter[not(.='b')]"/><xsl:template match="letter[.!='b']"/><xsl:template match="text()[contains(., 'needle')]"/></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet)
        .expect("exact node string-value match predicates should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        MatchPattern::NodeStringPredicate {
            node_test: MatchNodeTest::Element(name),
            predicate: MatchStringPredicate::Equals(value)
        } if name.local == "letter" && value == "b"
    ));
    assert!(matches!(
        &program.matched_templates[1].pattern,
        MatchPattern::NodeStringPredicate {
            node_test: MatchNodeTest::Text,
            predicate: MatchStringPredicate::Equals(value)
        } if value == "text"
    ));
    assert!(matches!(
        &program.matched_templates[2].pattern,
        MatchPattern::NodeStringPredicate {
            node_test: MatchNodeTest::Comment,
            predicate: MatchStringPredicate::Equals(value)
        } if value == "comment"
    ));
    assert!(matches!(
        &program.matched_templates[3].pattern,
        MatchPattern::NodeStringPredicate {
            node_test: MatchNodeTest::ProcessingInstruction(Some(target)),
            predicate: MatchStringPredicate::Equals(value)
        } if target == "target" && value == "data"
    ));
    assert!(matches!(
        &program.matched_templates[4].pattern,
        MatchPattern::NodeStringPredicate {
            predicate: MatchStringPredicate::EqualsEither(left, right),
            ..
        } if left == "b" && right == "h"
    ));
    assert!(matches!(
        &program.matched_templates[5].pattern,
        MatchPattern::NodeStringPredicate {
            predicate: MatchStringPredicate::NotEquals(value),
            ..
        } if value == "b"
    ));
    assert!(matches!(
        &program.matched_templates[6].pattern,
        MatchPattern::NodeStringPredicate {
            predicate: MatchStringPredicate::NotEquals(value),
            ..
        } if value == "b"
    ));
    assert!(matches!(
        &program.matched_templates[7].pattern,
        MatchPattern::NodeStringPredicate {
            node_test: MatchNodeTest::Text,
            predicate: MatchStringPredicate::Contains(value)
        } if value == "needle"
    ));
    assert!(
        program
            .matched_templates
            .iter()
            .all(|template| template.priority == TemplatePriority::PATH_DEFAULT)
    );
}

#[test]
fn normalizes_two_exact_attribute_value_match_predicates() {
    let stylesheet = parse_stylesheet(
        "memory:two-attribute-value-patterns.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo[@a='x'][@b='y']"/><xsl:template match="foo[@a='x' and @b='y']"/></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet)
        .expect("both two-attribute predicate spellings should compile");
    assert_eq!(
        program.matched_templates[0].pattern,
        program.matched_templates[1].pattern
    );
    assert!(matches!(
        &program.matched_templates[0].pattern,
        MatchPattern::ElementWithTwoAttributeValues {
            element,
            first_attribute,
            first_value,
            second_attribute,
            second_value,
        } if element.local == "foo"
            && first_attribute.local == "a"
            && first_value == "x"
            && second_attribute.local == "b"
            && second_value == "y"
    ));
}

#[test]
fn compiles_namespace_aware_attribute_match_patterns() {
    let stylesheet = parse_stylesheet(
        "memory:namespace-attribute-patterns.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:test"><xsl:template match="@p:*"/><xsl:template match="@p:code"/><xsl:template match="p:book[@style='leather']"/></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet)
        .expect("namespace-aware attribute match patterns should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        MatchPattern::AttributeNamespace(namespace) if namespace == "urn:test"
    ));
    assert!(matches!(
        &program.matched_templates[1].pattern,
        MatchPattern::Attribute(name)
            if name.namespace.as_deref() == Some("urn:test") && name.local == "code"
    ));
    assert!(matches!(
        &program.matched_templates[2].pattern,
        MatchPattern::ElementWithAttributeValue { element, attribute, value }
            if element.namespace.as_deref() == Some("urn:test")
                && element.local == "book"
                && attribute.namespace.is_none()
                && attribute.local == "style"
                && value == "leather"
    ));

    let unbound = parse_stylesheet(
        "memory:unbound-attribute-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="*[@missing:value]"/></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&unbound).expect_err("unbound attribute prefix must fail");
    assert_eq!(failure.code, "FXST0031");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn compiles_attribute_name_predicate_with_path_priority() {
    let stylesheet = parse_stylesheet(
        "memory:attribute-name-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="@*[name()='x1']"/></xsl:stylesheet>"#,
    );
    let program =
        compile_stylesheet(&stylesheet).expect("bounded attribute name predicate should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        MatchPattern::AttributeNameEquals(name) if name == "x1"
    ));
    assert_eq!(
        program.matched_templates[0].priority,
        TemplatePriority::PATH_DEFAULT
    );
}

#[test]
fn distinguishes_invalid_match_grammar_from_unimplemented_id_semantics() {
    for lexical in ["$var", "key(bookstore, bookstore)"] {
        let stylesheet = parse_stylesheet(
            "memory:invalid-match-grammar.xsl",
            format!(
                r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="{lexical}"/></xsl:stylesheet>"#
            )
            .as_bytes(),
        );
        let failure = compile_stylesheet(&stylesheet).expect_err("invalid match grammar must fail");
        assert_eq!(failure.code, "FXST1005");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    let stylesheet = parse_stylesheet(
        "memory:unsupported-match-capability.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="id('b')"/></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&stylesheet)
        .expect_err("valid but unimplemented id() semantics must fail");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn xslt10_rejects_match_variables_and_current_without_narrowing_modern_patterns() {
    for lexical in ["foo[. &gt; $screen]", "foo[current()]"] {
        let stylesheet = parse_stylesheet(
            "memory:xslt10-forbidden-match-context.xsl",
            format!(
                r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="screen" select="7"/><xsl:template match="{lexical}"/></xsl:stylesheet>"#
            )
            .as_bytes(),
        );
        let failure = compile_stylesheet(&stylesheet).expect_err("XSLT 1.0 pattern must fail");
        assert_eq!(failure.code, "FXST1005");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    let modern = parse_stylesheet(
        "memory:modern-match-variable.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="expected" select="'yes'"/><xsl:template match="*[@mark=$expected]"/></xsl:stylesheet>"#,
    );
    compile_stylesheet(&modern).expect("modern variable match pattern remains admitted");
}

#[test]
fn compiles_ordered_position_and_attribute_match_predicates() {
    let stylesheet = parse_stylesheet(
        "memory:ordered-position-attribute-patterns.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo[2][@att2='ok']"/><xsl:template match="foo[position()=2 and @att2='ok']"/><xsl:template match="foo[@att1='c'][2]"/></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet)
        .expect("ordered position/attribute patterns should compile");
    let expected = [
        ("att2", "ok", false),
        ("att2", "ok", false),
        ("att1", "c", true),
    ];
    for (template, (attribute_local, value, attribute_filters_position)) in
        program.matched_templates.iter().zip(expected)
    {
        assert!(matches!(
            &template.pattern,
            MatchPattern::ElementAtNamedSiblingWithAttributeValue {
                element,
                position: 2,
                attribute,
                value: actual_value,
                attribute_filters_position: actual_order,
            } if element.namespace.is_none()
                && element.local == "foo"
                && attribute.namespace.is_none()
                && attribute.local == attribute_local
                && actual_value == value
                && *actual_order == attribute_filters_position
        ));
        assert_eq!(template.priority, TemplatePriority::PATH_DEFAULT);
    }
}

#[test]
fn compiles_namespace_aware_descendant_path_with_positioned_middle_step() {
    let stylesheet = parse_stylesheet(
        "memory:qualified-descendant-position-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:n="urn:foo"><xsl:template match="//n:book/n:chapter[2]/foo"/></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet)
        .expect("qualified descendant-position pattern should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        MatchPattern::DescendantElementPathAtPosition {
            ancestor,
            positioned,
            position: 2,
            leaf,
        } if ancestor.namespace.as_deref() == Some("urn:foo")
            && ancestor.local == "book"
            && positioned.namespace.as_deref() == Some("urn:foo")
            && positioned.local == "chapter"
            && leaf.namespace.is_none()
            && leaf.local == "foo"
    ));
    assert_eq!(
        program.matched_templates[0].priority,
        TemplatePriority::PATH_DEFAULT
    );

    let unbound = parse_stylesheet(
        "memory:unbound-descendant-position-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="//n:book/n:chapter[2]/foo"/></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&unbound).expect_err("unbound prefix must fail");
    assert_eq!(failure.code, "FXST0031");
    assert_eq!(failure.category, CompileCategory::Invalid);

    let namespace_axis = parse_stylesheet(
        "memory:namespace-axis-selection.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:for-each select="namespace::*"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure =
        compile_stylesheet(&namespace_axis).expect_err("namespace axis remains unsupported");
    assert_eq!(failure.code, "FXXP1001");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn compiles_descendant_match_position_relative_to_the_child_axis() {
    let stylesheet = parse_stylesheet(
        "memory:descendant-child-axis-position.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="chapter//footnote[position() != 1]"/></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet)
        .expect("descendant child-axis position pattern should compile");
    assert!(matches!(
        &program.matched_templates[0].pattern,
        MatchPattern::DescendantElementAtNamedSiblingBoundary {
            ancestor,
            element,
            boundary: crate::xslt::golden_semantics_experiment::NamedSiblingBoundary::NotExact(1),
        } if ancestor.namespace.is_none()
            && ancestor.local == "chapter"
            && element.namespace.is_none()
            && element.local == "footnote"
    ));
    assert_eq!(
        program.matched_templates[0].priority,
        TemplatePriority::PATH_DEFAULT
    );
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

    let last_minus_position = parse_stylesheet(
        "memory:last-minus-position-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="chapter/footnote[last()-1]"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&last_minus_position)
        .expect_err("last-minus position needs match-pattern-specific semantics");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let chained_position = parse_stylesheet(
        "memory:chained-position-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="chapter/footnote[1][last()]"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&chained_position)
        .expect_err("chained position needs match-pattern-specific semantics");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let position_then_name = parse_stylesheet(
        "memory:position-name-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="chapter/footnote[last()][name()='footnote']"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&position_then_name)
        .expect_err("position/name chains need match-pattern-specific semantics");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let named_descendant = parse_stylesheet(
            "memory:named-descendant-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="//foo"><one/></xsl:template><xsl:template match="//foo/bar"><two/></xsl:template><xsl:template match="//foo//bar"><three/></xsl:template></xsl:stylesheet>"#,
        );
    let named_descendant_program = compile_stylesheet(&named_descendant)
        .expect("bounded descendant named patterns should compile");
    assert!(
        named_descendant_program
            .matched_templates
            .iter()
            .all(|template| matches!(template.pattern, MatchPattern::Path(_)))
    );

    let predicate_descendant = parse_stylesheet(
        "memory:predicate-descendant-pattern.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="//foo[@id='1']"><one/></xsl:template><xsl:template match="//foo[2]"><two/></xsl:template></xsl:stylesheet>"#,
    );
    let predicate_descendant_program = compile_stylesheet(&predicate_descendant)
        .expect("bounded descendant predicate patterns should compile");
    assert!(
        predicate_descendant_program
            .matched_templates
            .iter()
            .all(|template| matches!(template.pattern, MatchPattern::Path(_)))
    );

    let static_true_descendant = parse_stylesheet(
        "memory:static-true-descendant-pattern.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="//foo[true()]"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let static_true_program = compile_stylesheet(&static_true_descendant)
        .expect("static-true descendant predicate should normalize");
    assert!(matches!(
        static_true_program.matched_templates[0].pattern,
        MatchPattern::Path(_)
    ));

    let child_value_descendant = parse_stylesheet(
        "memory:child-value-descendant-pattern.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="//foo[bar='1']"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&child_value_descendant)
        .expect_err("child-value descendant predicates remain outside the bounded slice");
    assert_eq!(failure.code, "FXST1005");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn compiles_single_step_typed_predicate_match_patterns_as_paths() {
    let stylesheet = parse_stylesheet(
        "memory:predicate-pattern.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo[(bar[2])='this']"><one/></xsl:template><xsl:template match="foo[(bar[2][(baz[2])='goodbye'])]"><two/></xsl:template></xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&stylesheet).expect("typed predicate patterns should compile");

    assert!(
        program
            .matched_templates
            .iter()
            .all(|template| matches!(template.pattern, MatchPattern::Path(_)))
    );
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

    let top_level_text = parse_stylesheet(
        "memory:invalid-top-level-text.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"/>text is not allowed</xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&top_level_text)
        .expect_err("non-whitespace stylesheet top-level text must fail");
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert_eq!(failure.code, "FXST0008");
    assert!(failure.detail.contains("top level"));

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
fn validates_xslt10_message_before_reporting_unsupported_execution() {
    for terminate in [None, Some("no"), Some("yes")] {
        let attribute = terminate
            .map(|value| format!(r#" terminate="{value}""#))
            .unwrap_or_default();
        let stylesheet = parse_stylesheet(
            "memory:valid-message.xsl",
            format!(
                r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:message{attribute}>message</xsl:message></xsl:template></xsl:stylesheet>"#
            )
            .as_bytes(),
        );
        let failure = compile_stylesheet(&stylesheet)
            .expect_err("valid XSLT 1.0 message should remain explicitly unsupported");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXST1006");
    }

    for terminate in ["", "true", "foobar", " yes "] {
        let stylesheet = parse_stylesheet(
            "memory:invalid-message-terminate.xsl",
            format!(
                r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:message terminate="{terminate}">message</xsl:message></xsl:template></xsl:stylesheet>"#
            )
            .as_bytes(),
        );
        let failure = compile_stylesheet(&stylesheet)
            .expect_err("invalid XSLT 1.0 terminate value should fail validation");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "XTSE0020");
        assert!(failure.detail.contains("terminate"));
    }

    let stylesheet = parse_stylesheet(
        "memory:unsupported-message-attribute.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:message extra="value">message</xsl:message></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&stylesheet)
        .expect_err("unknown XSLT 1.0 message attribute should be classified");
    assert_eq!(failure.category, CompileCategory::Unsupported);
    assert_eq!(failure.code, "FXST1009");
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
fn xslt10_variable_position_predicates_do_not_approximate_multi_step_focus() {
    let stylesheet = parse_stylesheet(
        "memory:variable-position-focus.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="third" select="3"/><xsl:template match="/"><xsl:value-of select="a/b[$third]"/></xsl:template></xsl:stylesheet>"#,
    );

    let failure = compile_stylesheet(&stylesheet)
        .expect_err("general multi-step variable position requires per-step predicate focus");

    assert_eq!(failure.category, CompileCategory::Unsupported);
    assert_eq!(failure.code, "FXXP1001");
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
fn compiles_exact_preserve_all_as_the_default_whitespace_policy() {
    let stylesheet = parse_stylesheet(
        "memory:preserve-all.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:preserve-space elements="*"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let program =
        compile_stylesheet(&stylesheet).expect("exact preserve-all policy should compile");
    assert_eq!(
        program.source_whitespace,
        crate::xslt::golden_semantics_experiment::SourceWhitespacePolicy::Preserve
    );

    let selective = parse_stylesheet(
        "memory:selective-preserve.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:preserve-space elements="item"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&selective)
        .expect_err("selective whitespace rules remain outside the private slice");
    assert_eq!(failure.code, "FXST1043");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let composed = parse_stylesheet(
        "memory:composed-whitespace.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:preserve-space elements="*"/><xsl:strip-space elements="*"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&composed)
        .expect_err("mixed whitespace declarations remain outside the private slice");
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

    let xml_space = parse_stylesheet(
        "memory:text-xml-space.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text xml:space="preserve">   </xsl:text></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&xml_space).expect("xml:space should be valid on xsl:text");
    let root_template = program.root_template.expect("root template");
    assert!(matches!(
        root_template.body.as_slice(),
        [Instruction::Text { value, .. }] if value == "   "
    ));

    let invalid_space = parse_stylesheet(
        "memory:text-invalid-xml-space.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text xml:space="sometimes">text</xsl:text></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&invalid_space).expect_err("invalid xml:space must fail");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);

    let invalid_text = parse_stylesheet(
            "memory:invalid-text.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text><bad/></xsl:text></xsl:template></xsl:stylesheet>"#,
        );
    let failure = compile_stylesheet(&invalid_text).expect_err("element content must fail");
    assert_eq!(failure.code, "FXST0026");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn xsl_text_accepts_only_the_semantically_inert_disable_output_escaping_value() {
    let no = parse_stylesheet(
        "memory:text-doe-no.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text disable-output-escaping="no">&lt;safe&gt;</xsl:text></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&no).expect("the default escaping request should compile");
    let root_template = program.root_template.expect("root template");
    let [Instruction::Text { value, .. }] = root_template.body.as_slice() else {
        panic!("xsl:text should retain one text instruction");
    };
    assert_eq!(value, "<safe>");

    let yes = parse_stylesheet(
        "memory:text-doe-yes.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text disable-output-escaping="yes">&lt;unsafe&gt;</xsl:text></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&yes).expect_err("disabled escaping remains unsupported");
    assert_eq!(failure.code, "FXST1060");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let invalid = parse_stylesheet(
        "memory:text-doe-invalid.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text disable-output-escaping="maybe">text</xsl:text></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&invalid).expect_err("invalid lexical value must fail");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn computed_attribute_retains_static_namespace_and_literal_text() {
    let stylesheet = parse_stylesheet(
        "memory:computed-attribute-namespace.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out><xsl:attribute name="answer" namespace="urn:example:answer">forty-two</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let program =
        compile_stylesheet(&stylesheet).expect("static namespaced attribute should compile");
    let root_template = program.root_template.expect("root template");
    let [
        Instruction::LiteralElement {
            computed_attributes,
            ..
        },
    ] = root_template.body.as_slice()
    else {
        panic!("template should retain one result element");
    };
    let [attribute] = computed_attributes.as_slice() else {
        panic!("result element should retain one computed attribute");
    };
    assert_eq!(
        attribute.name.namespace.as_deref(),
        Some("urn:example:answer")
    );
    assert_eq!(attribute.name.local, "answer");
    assert_eq!(
        attribute.value,
        LiteralAttributeValue::Text("forty-two".to_owned())
    );

    let prefixed = parse_stylesheet(
        "memory:computed-attribute-prefixed-name.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:original"><xsl:template match="/"><out><xsl:attribute name="p:answer" namespace="urn:override">value</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&prefixed).expect("prefixed static QName should compile");
    let root_template = program.root_template.expect("root template");
    let [
        Instruction::LiteralElement {
            computed_attributes,
            ..
        },
    ] = root_template.body.as_slice()
    else {
        panic!("template should retain one result element");
    };
    assert_eq!(
        computed_attributes[0].name.namespace.as_deref(),
        Some("urn:override")
    );
    assert_eq!(computed_attributes[0].name.local, "answer");

    let dynamic = parse_stylesheet(
        "memory:computed-attribute-dynamic-namespace.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out><xsl:attribute name="answer" namespace="{namespace-uri()}">value</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&dynamic)
        .expect_err("namespace AVTs outside the admitted path form remain unsupported");
    assert_eq!(failure.code, "FXST1061");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let modern = parse_stylesheet(
        "memory:computed-attribute-modern-dynamic-namespace.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out><xsl:attribute name="answer" namespace="{@namespace}">value</xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&modern)
        .expect_err("the path namespace compatibility slice must not widen modern semantics");
    assert_eq!(failure.code, "FXST1061");
    assert_eq!(failure.category, CompileCategory::Unsupported);
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
fn comment_and_processing_instruction_fold_explicit_static_text() {
    let stylesheet = parse_stylesheet(
        "memory:static-node-content.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:comment><xsl:text disable-output-escaping="yes">&lt;&gt;&amp;'</xsl:text></xsl:comment><xsl:processing-instruction name="doe"><xsl:text disable-output-escaping="yes">&lt;&quot;&gt;</xsl:text></xsl:processing-instruction></xsl:template></xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&stylesheet).expect("static node content should compile");
    let root_template = program.root_template.expect("root template");
    assert!(matches!(
        root_template.body.as_slice(),
        [
            Instruction::CommentNode { value: comment, .. },
            Instruction::ProcessingInstructionNode { value: pi, .. }
        ] if comment == "<>&'" && pi == "<\">"
    ));
}

#[test]
fn comment_and_processing_instruction_fold_static_value_expressions() {
    let stylesheet = parse_stylesheet(
        "memory:static-node-value.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:comment><xsl:value-of select="substring('abcdef', 2, 3)"/></xsl:comment><xsl:processing-instruction name="doe"><xsl:value-of disable-output-escaping="yes" select="concat('a', 'b')"/></xsl:processing-instruction></xsl:template></xsl:stylesheet>"#,
    );

    let program = compile_stylesheet(&stylesheet).expect("static node values should compile");
    let root_template = program.root_template.expect("root template");
    assert!(matches!(
        root_template.body.as_slice(),
        [
            Instruction::CommentNode { value: comment, .. },
            Instruction::ProcessingInstructionNode { value: pi, .. }
        ] if comment == "bcd" && pi == "ab"
    ));
}

#[test]
fn xslt10_comment_recovers_xml_comment_delimiters_without_widening_modern_semantics() {
    let stylesheet = parse_stylesheet(
        "memory:xslt10-comment-recovery.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:comment>one--two---</xsl:comment></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet).expect("XSLT 1.0 recovery should compile");
    let root_template = program.root_template.expect("root template");
    assert!(matches!(
        root_template.body.as_slice(),
        [Instruction::CommentNode { value, .. }] if value == "one- -two- - - "
    ));

    let modern = parse_stylesheet(
        "memory:modern-comment-recovery.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:comment>one--two-</xsl:comment></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&modern).expect_err("modern recovery remains unsupported");
    assert_eq!(failure.code, "FXST1037");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn literal_result_element_applies_namespaced_exclude_result_prefixes() {
    let stylesheet = parse_stylesheet(
        "memory:lre-excluded-prefixes.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:keep="urn:keep" xmlns:drop="urn:drop"><xsl:template match="/"><out xsl:exclude-result-prefixes="drop"/></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet).expect("LRE prefix exclusion should compile");
    let root_template = program.root_template.expect("root template");
    let [
        Instruction::LiteralElement {
            namespaces,
            attributes,
            ..
        },
    ] = root_template.body.as_slice()
    else {
        panic!("template should retain one literal result element");
    };
    assert!(
        attributes.is_empty(),
        "the XSLT control must not become a result attribute"
    );
    assert!(
        namespaces
            .iter()
            .any(|binding| binding.prefix.as_deref() == Some("keep"))
    );
    assert!(
        !namespaces
            .iter()
            .any(|binding| binding.prefix.as_deref() == Some("drop"))
    );
}

#[test]
fn exclude_result_prefixes_rejects_unbound_prefixes_at_each_declaration_site() {
    for (label, bytes) in [
        (
            "stylesheet root",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" exclude-result-prefixes="missing"><xsl:template match="/"/></xsl:stylesheet>"#.as_slice(),
        ),
        (
            "literal result element",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out xsl:exclude-result-prefixes="missing"/></xsl:template></xsl:stylesheet>"#.as_slice(),
        ),
    ] {
        let stylesheet = parse_stylesheet("memory:unbound-excluded-prefix.xsl", bytes);
        let failure = compile_stylesheet(&stylesheet).expect_err(label);
        assert_eq!(failure.category, CompileCategory::Invalid, "{label}");
        assert_eq!(failure.code, "XTSE0808", "{label}");
    }
}

#[test]
fn xslt10_literal_result_ignores_unknown_xslt_namespace_attributes() {
    let stylesheet = parse_stylesheet(
        "memory:xslt10-unknown-lre-control.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out xsl:unknown="not-a-result-attribute" keep="yes"/></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet).expect("XSLT 1.0 stylesheet should compile");
    let root_template = program.root_template.expect("root template");
    let [Instruction::LiteralElement { attributes, .. }] = root_template.body.as_slice() else {
        panic!("template should retain one literal result element");
    };
    assert_eq!(attributes.len(), 1);
    assert_eq!(attributes[0].name.local, "keep");

    let modern = parse_stylesheet(
        "memory:modern-unknown-lre-control.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out xsl:unknown="unsupported"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&modern).expect_err("modern behavior remains explicit");
    assert_eq!(failure.code, "FXST1007");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let extension_control = parse_stylesheet(
        "memory:xslt10-extension-lre-control.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out xsl:extension-element-prefixes="missing"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&extension_control)
        .expect_err("unbound extension-element prefix must be invalid");
    assert_eq!(failure.code, "XTSE1430");
    assert_eq!(failure.category, CompileCategory::Invalid);

    let unbound_default = parse_stylesheet(
        "memory:xslt10-unbound-default-extension.xsl",
        br##"<out xsl:version="1.0" xsl:extension-element-prefixes="#default" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"/>"##,
    );
    let failure =
        compile_stylesheet(&unbound_default).expect_err("unbound #default must be invalid");
    assert_eq!(failure.code, "XTSE1430");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn literal_result_extension_prefixes_are_scoped_validated_and_excluded() {
    let stylesheet = parse_stylesheet(
        "memory:lre-extension-prefix.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:keep="urn:keep" xmlns:ext="urn:extension"><xsl:template match="/"><out xsl:extension-element-prefixes="ext"/></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&stylesheet).expect("literal result should compile");
    let root_template = program.root_template.expect("root template");
    let [Instruction::LiteralElement { namespaces, .. }] = root_template.body.as_slice() else {
        panic!("template should retain one literal result element");
    };
    assert!(
        namespaces
            .iter()
            .any(|binding| binding.prefix.as_deref() == Some("keep"))
    );
    assert!(
        !namespaces
            .iter()
            .any(|binding| binding.prefix.as_deref() == Some("ext"))
    );

    let extension = parse_stylesheet(
        "memory:self-declared-extension.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:ext="urn:extension"><xsl:template match="/"><ext:run xsl:extension-element-prefixes="ext"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&extension).expect_err("extension execution stays explicit");
    assert_eq!(failure.code, "FXST1059");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let required_attribute_binding = parse_stylesheet(
        "memory:required-extension-attribute-prefix.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:ext="urn:extension" extension-element-prefixes="ext"><xsl:template match="/"><out ext:kept="yes"/></xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&required_attribute_binding)
        .expect("a result attribute must keep its required namespace binding");
    let root_template = program.root_template.expect("root template");
    let [Instruction::LiteralElement { namespaces, .. }] = root_template.body.as_slice() else {
        panic!("template should retain one literal result element");
    };
    assert!(
        namespaces
            .iter()
            .any(|binding| binding.prefix.as_deref() == Some("ext"))
    );
}

#[test]
fn xslt10_instructions_ignore_foreign_namespaced_attributes() {
    let stylesheet = parse_stylesheet(
        "memory:xslt10-extension-attributes.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:example"><xsl:template match="/" e:template="ignored"><out><xsl:copy-of select="doc" e:copy="ignored"/></out></xsl:template></xsl:stylesheet>"#,
    );
    compile_stylesheet(&stylesheet).expect("XSLT 1.0 extension attributes should be ignored");

    let modern = parse_stylesheet(
        "memory:modern-extension-attributes.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:example"><xsl:template match="/" e:template="unsupported"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&modern).expect_err("modern behavior remains explicit");
    assert_eq!(failure.code, "FXST1009");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let output = parse_stylesheet(
        "memory:xslt10-output-extension-attribute.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:example"><xsl:output e:control="ignored"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    compile_stylesheet(&output).expect("XSLT 1.0 output extension attribute should be ignored");
}

#[test]
fn xml_space_is_validated_and_controls_stylesheet_text_preservation() {
    let preserve = parse_stylesheet(
        "memory:xml-space-preserve.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" xml:space="preserve"> <out/> </xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&preserve).expect("xml:space preserve should compile");
    let body = &program.root_template.expect("root template").body;
    assert!(matches!(
        body.as_slice(),
        [
            Instruction::Text { .. },
            Instruction::LiteralElement { .. },
            Instruction::Text { .. }
        ]
    ));

    let defaulted = parse_stylesheet(
        "memory:xml-space-default.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"><xsl:template match="/" xml:space="default"> <out/> </xsl:template></xsl:stylesheet>"#,
    );
    let program = compile_stylesheet(&defaulted).expect("xml:space default should compile");
    let body = &program.root_template.expect("root template").body;
    assert!(matches!(
        body.as_slice(),
        [Instruction::LiteralElement { .. }]
    ));

    let invalid_space = parse_stylesheet(
        "memory:xml-space-invalid.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" xml:space="sometimes"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&invalid_space).expect_err("xml:space lexical must be valid");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn value_of_admits_only_semantically_inert_disable_output_escaping() {
    let disabled = parse_stylesheet(
        "memory:value-of-doe-no.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out><xsl:value-of select="doc/value" disable-output-escaping="no"/></out></xsl:template></xsl:stylesheet>"#,
    );
    compile_stylesheet(&disabled).expect("disable-output-escaping no should compile");

    let enabled = parse_stylesheet(
        "memory:value-of-doe-yes.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="doc/value" disable-output-escaping="yes"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&enabled).expect_err("DOE yes remains unsupported");
    assert_eq!(failure.code, "FXST1060");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let invalid_lexical = parse_stylesheet(
        "memory:value-of-doe-invalid.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="doc/value" disable-output-escaping="sometimes"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&invalid_lexical).expect_err("DOE lexical must be valid");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);
}

#[test]
fn number_admits_static_letter_values_only_when_existing_tokens_are_equivalent() {
    for (format, letter_value) in [
        ("i.I.a.A", "traditional"),
        ("a.A", "alphabetic"),
        ("1", "traditional"),
    ] {
        let stylesheet = parse_stylesheet(
            "memory:number-letter-value.xsl",
            format!(
                r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:number value="1" format="{format}" letter-value="{letter_value}"/></xsl:template></xsl:stylesheet>"#
            )
            .as_bytes(),
        );
        compile_stylesheet(&stylesheet).expect("equivalent static letter value should compile");
    }

    let unsupported = parse_stylesheet(
        "memory:number-alphabetic-roman.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:number value="1" format="i" letter-value="alphabetic"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&unsupported)
        .expect_err("alphabetic Roman-token reinterpretation remains unsupported");
    assert_eq!(failure.code, "FXST1049");
    assert_eq!(failure.category, CompileCategory::Unsupported);

    let invalid = parse_stylesheet(
        "memory:number-invalid-letter-value.xsl",
        br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:number value="1" letter-value="unknown"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure =
        compile_stylesheet(&invalid).expect_err("unknown letter-value must be rejected as invalid");
    assert_eq!(failure.code, "XTSE0020");
    assert_eq!(failure.category, CompileCategory::Invalid);

    let modern_dynamic = parse_stylesheet(
        "memory:modern-dynamic-number-format.xsl",
        br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:param name="format" select="'1'"/><xsl:template match="/"><xsl:number value="1" format="{$format}"/></xsl:template></xsl:stylesheet>"#,
    );
    let failure = compile_stylesheet(&modern_dynamic)
        .expect_err("the XSLT 1.0 compatibility slice must not widen modern formats");
    assert_eq!(failure.code, "FXST1049");
    assert_eq!(failure.category, CompileCategory::Unsupported);
}

#[test]
fn number_rejects_invalid_static_grouping_values_without_calling_them_unsupported() {
    for (separator, size) in [("too-long", "3"), (",", "0"), (",", "bad")] {
        let stylesheet = parse_stylesheet(
            "memory:number-invalid-grouping.xsl",
            format!(
                r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:number value="1000" grouping-separator="{separator}" grouping-size="{size}"/></xsl:template></xsl:stylesheet>"#
            )
            .as_bytes(),
        );
        let failure = compile_stylesheet(&stylesheet)
            .expect_err("invalid effective grouping value must be rejected");
        assert_eq!(failure.code, "XTDE0030");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }
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

#[test]
fn top_level_union_splitter_ignores_nested_and_quoted_separators() {
    use super::instruction_compiler::split_top_level_union;

    assert_eq!(
        split_top_level_union("a|b | c"),
        Some(vec!["a", "b ", " c"])
    );
    assert_eq!(split_top_level_union("(a|b)/c"), None);
    assert_eq!(split_top_level_union("a[b='x|y']"), None);
    assert_eq!(
        split_top_level_union("(a|b)/c | d[e='x|y']"),
        Some(vec!["(a|b)/c ", " d[e='x|y']"])
    );
}
