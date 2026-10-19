//! Selected-edition top-level recognition must precede shared lowering.
use super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn document(identity: &str, bytes: &[u8]) -> Document {
    Document::from_parsed(
        parse_document(
            identity,
            bytes,
            ParseLimits {
                max_events: 10000,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

fn sheet(version: &str, declaration: &str) -> Document {
    document("urn:top-level-policy", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{declaration}<xsl:output method="text"/><xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet>"#).as_bytes())
}

#[test]
fn selected_legacy_ignores_unknown_top_level_content_before_lowering() {
    for version in ["1.5", "2.0", "3.0"] {
        for name in [
            "foo",
            "mode",
            "function",
            "character-map",
            "accumulator",
            "fallback",
        ] {
            let style = sheet(
                version,
                &format!(
                    r#"<xsl:{name} unknown="{{"><xsl:include href="file:///not-admitted.xsl"/><xsl:variable name="leaked" select="$unbound"/><xsl:message terminate="yes">not executed</xsl:message><xsl:fallback>not executed either</xsl:fallback></xsl:{name}>"#
                ),
            );
            let program =
                compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
            let source = document("urn:source", b"<r/>");
            assert_eq!(
                execute_compiled_stylesheet_for_test(&program, &source, "ignored").unwrap(),
                "ok"
            );
        }
    }
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_legacy_replays_original_unknown_top_level_declarations() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let source = document(
        "urn:oasis:source",
        &std::fs::read(root.join("MSFT_Conformance_Tests/ForwardComp/books.xml")).unwrap(),
    );
    for case in ["91837", "91838", "91839", "91840", "91847", "91848"] {
        let style = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("MSFT_Conformance_Tests/ForwardComp/{case}.xsl")))
                .unwrap(),
        );
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let actual = execute_compiled_stylesheet_for_test(&program, &source, case).unwrap();
        let expected = std::fs::read(root.join(format!(
            "MSFT_Conformance_Tests/REF_OUT/ForwardComp/{case}.txt"
        )))
        .unwrap();
        assert_eq!(
            crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                &actual,
                &expected,
                false,
                None,
                Some("xml")
            ),
            Ok(true),
            "{case}"
        );
    }
}

#[test]
fn selected_output_boolean_policy_does_not_inherit_modern_aliases() {
    for version in ["1.0", "1.5", "2.0", "3.0"] {
        for value in ["sure", "true", "1", "false", "0"] {
            let style = document("urn:output-policy", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="xml" indent="{value}" omit-xml-declaration="{value}"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#).as_bytes());
            let result = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
            if version == "1.0" {
                let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                    panic!("normal legacy must reject invalid Boolean");
                };
                assert_eq!(failure.code, "XTSE0020");
                assert_eq!(failure.location.resource, "urn:output-policy");
            } else {
                let program = result.unwrap();
                assert_eq!(program.output.indent, None);
                assert!(!program.output.omit_xml_declaration);
                for property in ["indent", "omit-xml-declaration"] {
                    assert!(
                        !program
                            .output_specified_properties
                            .iter()
                            .any(|p| p == property)
                    );
                }
            }
            if version == "3.0" && value != "sure" {
                let program =
                    compile(&style, Edition::Modern, &mut InvocationControl::unbounded()).unwrap();
                assert_eq!(program.output.indent, Some(matches!(value, "true" | "1")));
                assert_eq!(
                    program.output.omit_xml_declaration,
                    matches!(value, "true" | "1")
                );
            }
        }
    }
}

#[test]
fn selected_output_ignored_boolean_preserves_valid_declarations_in_both_orders() {
    let source = document("urn:output-source", b"<r/>");
    for version in ["1.5", "2.0", "3.0"] {
        for reverse in [false, true] {
            let valid = r#"<xsl:output method="xml" indent="yes" omit-xml-declaration="yes"/>"#;
            let ignored = r#"<xsl:output indent="sure" omit-xml-declaration="sure"/>"#;
            let outputs = if reverse {
                format!("{ignored}{valid}")
            } else {
                format!("{valid}{ignored}")
            };
            let style = document("urn:output-merge-policy", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{outputs}<xsl:template match="/"><out><child/></out></xsl:template></xsl:stylesheet>"#).as_bytes());
            let program =
                compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
            assert_eq!(program.output.indent, Some(true));
            assert!(program.output.omit_xml_declaration);
            let actual = execute_compiled_stylesheet_for_test(&program, &source, "output").unwrap();
            let reference_style = document("urn:output-valid-only", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{valid}<xsl:template match="/"><out><child/></out></xsl:template></xsl:stylesheet>"#).as_bytes());
            let reference = compile(
                &reference_style,
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(
                actual,
                execute_compiled_stylesheet_for_test(&reference, &source, "reference").unwrap()
            );
        }
    }
}

#[test]
fn selected_template_unknown_attribute_uses_processor_not_declared_version() {
    for version in ["1.0", "1.5", "2.0", "3.0"] {
        let style = document("urn:template-attributes", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:template match="item" future="{{ $unbound"><xsl:value-of select="text()"/></xsl:template><xsl:template match="text()"/></xsl:stylesheet>"#).as_bytes());
        let result = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
        if version == "1.0" {
            let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                panic!("normal legacy unknown attribute");
            };
            assert_eq!(failure.code, "XTSE0090");
            assert_eq!(failure.category, CompileCategory::Invalid);
            assert_eq!(failure.location.resource, "urn:template-attributes");
            assert!(!failure.location.span.is_empty());
        } else {
            let program = result.unwrap();
            let source = document("urn:source", b"<root><item>A</item><item>B</item></root>");
            assert_eq!(
                execute_compiled_stylesheet_for_test(&program, &source, "attributes").unwrap(),
                "AB"
            );
        }
        assert!(compile(&style, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
    }
}

#[test]
fn selected_declaration_attributes_keep_unpropagated_readers_guarded() {
    for attribute in [
        r#"name="n""#,
        r#"mode="m""#,
        r#"priority="1""#,
        r#"xpath-default-namespace="urn:source""#,
        r#"default-mode="m""#,
        r#"default-collation="urn:unproved""#,
        r##"exclude-result-prefixes="#all""##,
        r#"version="1.0""#,
        r#"xml:space="preserve""#,
        r#"xsl:version="1.0""#,
    ] {
        let style = document("urn:template-reader-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="item" {attribute}/></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("unpropagated reader: {attribute}");
        };
        assert_eq!(failure.code, "FXAR0031");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }
    for attribute in [
        r#"method="xhtml""#,
        r#"method="future""#,
        r#"encoding="UTF-16""#,
        r#"name="named""#,
        r#"standalone="yes""#,
        r#"html-version="5""#,
        r#"byte-order-mark="yes""#,
        r#"version="1.0""#,
        r#"cdata-section-elements="out""#,
    ] {
        let style = sheet("2.0", &format!("<xsl:output {attribute}/>"));
        let Err(StrictPreprocessingFailure::Compile(failure)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("unpropagated output: {attribute}");
        };
        assert_eq!(failure.code, "FXAR0031");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }
}

#[test]
fn selected_output_merge_metadata_survives_include_and_import_composition() {
    // Exercise existing artifact composition, not a new resource graph loader.
    let source = document("urn:composition-source", b"<r/>");
    for import in [false, true] {
        for valid_principal in [false, true] {
            let valid =
                r#"<xsl:output method="xml" indent="yes" omit-xml-declaration="yes" future="A"/>"#;
            let ignored = r#"<xsl:output indent="sure" omit-xml-declaration="sure" future="B"/>"#;
            let (principal_output, dependency_output) = if valid_principal {
                (valid, ignored)
            } else {
                (ignored, valid)
            };
            let (program, reference) = {
                let principal = document("urn:principal-output", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">{principal_output}<xsl:template match="text()" future="ignored"/></xsl:stylesheet>"#).as_bytes());
                let dependency = document("urn:dependency-output", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">{dependency_output}<xsl:template match="/"><out><child/></out></xsl:template></xsl:stylesheet>"#).as_bytes());
                let mut program = compile(
                    &principal,
                    Edition::Legacy,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                let included = compile(
                    &dependency,
                    Edition::Legacy,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                let location = principal.location(principal.document_node());
                if import {
                    program = compiler::stylesheet_module_compiler::merge_single_imported_program(
                        program, included, location, location,
                    )
                    .unwrap();
                } else {
                    compiler::stylesheet_module_compiler::merge_included_program(
                        &mut program,
                        included,
                        location,
                    )
                    .unwrap();
                }
                let reference_sheet = document("urn:reference-output", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">{valid}<xsl:template match="/"><out><child/></out></xsl:template></xsl:stylesheet>"#).as_bytes());
                let reference = compile(
                    &reference_sheet,
                    Edition::Legacy,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                (program, reference)
            };
            assert_eq!(program.processor, reference.processor);
            assert_eq!(program.preprocessing, reference.preprocessing);
            assert_eq!(program.output.indent, Some(true));
            assert!(program.output.omit_xml_declaration);
            assert_eq!(
                execute_compiled_stylesheet_for_test(&program, &source, "composed").unwrap(),
                execute_compiled_stylesheet_for_test(&reference, &source, "reference").unwrap()
            );
        }
    }
}

#[test]
fn selected_output_unknown_attributes_are_not_values_or_merge_properties() {
    for version in ["1.0", "1.5", "2.0", "3.0"] {
        let style = document("urn:unknown-output", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text" future="{{ $unbound"/><xsl:output future="different"/><xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet>"#).as_bytes());
        let result = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded());
        if version == "1.0" {
            let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                panic!("normal legacy unknown output attribute");
            };
            assert_eq!(failure.code, "XTSE0090");
            assert_eq!(failure.category, CompileCategory::Invalid);
            assert_eq!(failure.location.resource, "urn:unknown-output");
        } else {
            let program = result.unwrap();
            assert!(
                !program
                    .output_specified_properties
                    .iter()
                    .any(|p| p == "future")
            );
            assert!(
                !program
                    .output_same_precedence_properties
                    .iter()
                    .any(|p| p == "future")
            );
            let source = document("urn:source", b"<r/>");
            assert_eq!(
                execute_compiled_stylesheet_for_test(&program, &source, "unknown-output").unwrap(),
                "ok"
            );
        }
        assert!(compile(&style, Edition::Modern, &mut InvocationControl::unbounded()).is_err());
    }
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_legacy_replays_complete_forward_comp_catalog() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let catalog = document(
        "urn:oasis:forward-catalog",
        &std::fs::read(root.join("MSFT_Conformance_Tests/ForwardComp/ForwardComp.xml")).unwrap(),
    );
    let catalog_root = compiler::document_element(&catalog).unwrap();
    let cases = compiler::meaningful_children(&catalog, catalog_root);
    assert_eq!(cases.len(), 18, "review the acquired catalog delta");
    let source = document(
        "urn:oasis:forward-source",
        &std::fs::read(root.join("MSFT_Conformance_Tests/ForwardComp/books.xml")).unwrap(),
    );
    let (mut exact, mut errors) = (0, 0);
    let mut seen = std::collections::BTreeSet::new();
    for variation in cases {
        let case = required_attribute(&catalog, variation, None, "name")
            .unwrap()
            .strip_prefix('_')
            .unwrap();
        assert!(seen.insert(case), "duplicate catalog identity: {case}");
        let data = catalog_child(&catalog, variation, "data");
        assert_eq!(
            catalog
                .string_value(catalog_child(&catalog, data, "xml"))
                .trim(),
            "books.xml"
        );
        assert_eq!(
            catalog
                .string_value(catalog_child(&catalog, data, "xsl"))
                .trim(),
            format!("{case}.xsl")
        );
        let result_node = catalog
            .children(variation)
            .iter()
            .copied()
            .find(|child| {
                catalog
                    .name(*child)
                    .is_some_and(|name| name.local == "result")
            })
            .unwrap();
        let expected = required_attribute(&catalog, result_node, None, "expected").unwrap();
        let style = document(
            &format!("urn:oasis:{case}"),
            &std::fs::read(root.join(format!("MSFT_Conformance_Tests/ForwardComp/{case}.xsl")))
                .unwrap(),
        );
        let program =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let actual = execute_compiled_stylesheet_for_test(&program, &source, case);
        match expected {
            "valid" => {
                assert_eq!(
                    catalog.string_value(result_node).trim(),
                    format!("{case}.txt")
                );
                let reference = std::fs::read(root.join(format!(
                    "MSFT_Conformance_Tests/REF_OUT/ForwardComp/{case}.txt"
                )))
                .unwrap();
                assert_eq!(
                    crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                        &actual.unwrap(),
                        &reference,
                        false,
                        None,
                        Some("xml")
                    ),
                    Ok(true),
                    "{case}"
                );
                exact += 1;
            }
            "invalid" => {
                let code = match case {
                    "91841" | "91844" => "XTDE1450",
                    "91850" => "XPST0003",
                    "91854" => "XPST0017",
                    "91856" => "XTDE1390",
                    _ => panic!("unreviewed expected error: {case}"),
                };
                let failure = actual.unwrap_err();
                assert!(
                    failure.contains(code) && failure.contains(&format!("urn:oasis:{case}")),
                    "{failure}"
                );
                errors += 1;
            }
            _ => panic!("unreviewed catalog expectation: {expected}"),
        }
    }
    assert_eq!((exact, errors), (13, 5));
    println!(
        "selected-legacy/strict ForwardComp: 18 total, 13 exact, 5 expected-error outcomes; not baseline corpus credit"
    );
}

fn catalog_child(document: &Document, parent: NodeId, local: &str) -> NodeId {
    let matches = document
        .children(parent)
        .iter()
        .copied()
        .filter(|child| {
            document
                .name(*child)
                .is_some_and(|name| name.namespace.is_none() && name.local == local)
        })
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "catalog requires one {local}");
    matches[0]
}

#[test]
fn selected_top_level_recognition_does_not_hide_unintegrated_legacy_declarations() {
    for name in [
        "import",
        "include",
        "strip-space",
        "preserve-space",
        "key",
        "decimal-format",
        "namespace-alias",
        "attribute-set",
        "variable",
        "param",
    ] {
        for version in ["1.0", "1.5", "2.0"] {
            let style = sheet(version, &format!("<xsl:{name}/>"));
            let Err(StrictPreprocessingFailure::Compile(failure)) =
                compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
            else {
                panic!("recognized declaration must not be ignored: {name}");
            };
            assert_eq!(failure.category, CompileCategory::Unsupported);
            assert_eq!(failure.code, "FXAR0031");
            assert_eq!(failure.location.resource, "urn:top-level-policy");
            assert!(!failure.location.span.is_empty());
        }
    }
    // The two integrated declaration families still validate their attributes.
    for declaration in [r#"<xsl:output method="invalid"/>"#, "<xsl:template/>"] {
        assert!(
            compile(
                &sheet("2.0", declaration),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
}

#[test]
fn selected_top_level_ignoring_is_edition_and_context_scoped() {
    for name in [
        "foo",
        "mode",
        "function",
        "character-map",
        "accumulator",
        "fallback",
    ] {
        let declaration = format!("<xsl:{name}/>");
        let Err(StrictPreprocessingFailure::Compile(failure)) = compile(
            &sheet("1.0", &declaration),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        ) else {
            panic!("normal legacy rejection");
        };
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "FXST1002");
        for version in ["1.0", "2.0", "3.0"] {
            // No new modern declaration route is admitted by a legacy change.
            assert!(
                compile(
                    &sheet(version, &declaration),
                    Edition::Modern,
                    &mut InvocationControl::unbounded()
                )
                .is_err()
            );
        }
    }
    // Unqualified and foreign elements retain the existing bounded guard.
    for declaration in ["<foo/>", r#"<foo xmlns="urn:foreign"/>"#] {
        assert!(
            compile(
                &sheet("2.0", declaration),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
    let style = document("urn:no-leak", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:future><xsl:variable name="leaked" select="'secret'"/></xsl:future><xsl:template match="/"><xsl:value-of select="$leaked"/></xsl:template></xsl:stylesheet>"#);
    let Err(StrictPreprocessingFailure::Compile(failure)) =
        compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
    else {
        panic!("ignored declaration must not bind variables");
    };
    assert_eq!(failure.code, "XPST0008");
}

#[test]
fn selected_top_level_ignoring_preserves_preprocessing_cancellation_and_reuse() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let style = sheet(
        "2.0",
        "<xsl:future><xsl:text>ignored</xsl:text></xsl:future>",
    );
    let mut control = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
    assert!(matches!(
        compile(&style, Edition::Legacy, &mut control),
        Err(StrictPreprocessingFailure::Control(_))
    ));
    let mut measured = InvocationControl::unbounded();
    let program = compile(&style, Edition::Legacy, &mut measured).unwrap();
    let charges = measured.consumed(WorkDomain::XdmNode);
    let mut plain = InvocationControl::unbounded();
    compile(&sheet("2.0", ""), Edition::Legacy, &mut plain).unwrap();
    assert!(charges > plain.consumed(WorkDomain::XdmNode));
    // Ignoring language content does not exempt its XML/XDM preparation work.
    for allowance in [charges - 1, charges] {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = allowance;
        let run = || {
            let result = compile(
                &style,
                Edition::Legacy,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance == charges {
                assert!(result.is_ok());
            } else {
                assert!(matches!(
                    result,
                    Err(StrictPreprocessingFailure::Control(_))
                ));
            }
        };
        run();
        #[cfg(feature = "allocation-observation")]
        {
            let observed = allocation_counter::measure(run);
            assert_eq!(observed.bytes_current, 0);
            assert_eq!(observed.count_current, 0);
        }
    }
    drop(style);
    let source = document("urn:source", b"<r/>");
    for request in ["first", "second"] {
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &source, request).unwrap(),
            "ok"
        );
    }
}
