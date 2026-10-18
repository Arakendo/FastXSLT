//! Single-edge import qualification; no competing rules or apply-imports slice.
use super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

#[test]
fn processor_edition_reference_shared_linker_preserves_or_rejects_processor_stamp() {
    use crate::xslt::golden_semantics_experiment::ProcessorPolicyStamp;
    // Deliberately bypass CompiledModule: the real program/linker must retain
    // qualification after the experiment's outer wrapper has been removed.
    let leaf = module(
        "urn:leaf",
        "2.0",
        r#"<xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template>"#,
    );
    let preprocessing = StylesheetPreprocessingProfile::Modern;
    for edge in ["include", "import"] {
        let principal = module(
            "urn:principal",
            "2.0",
            &format!(r#"<xsl:{edge} href="urn:leaf"/>"#),
        );
        let root = compiler::document_element(&principal).unwrap();
        let location = principal.location(meaningful_children(&principal, root)[0]);
        for caller in [None, Some(Edition::Legacy), Some(Edition::Modern)] {
            for owner in [None, Some(Edition::Legacy), Some(Edition::Modern)] {
                let context = caller.map_or_else(
                    || StylesheetCompilationContext::for_reference(preprocessing, true),
                    |edition| compilation_context(edition, preprocessing),
                );
                let dependency = match owner {
                    Some(edition) => lower_module_with_preprocessing(
                        &leaf,
                        edition,
                        preprocessing,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap(),
                    None => StylesheetCompilationContext::for_reference(preprocessing, true)
                        .qualify_module(compiler::compile_stylesheet(&leaf).unwrap()),
                };
                let retained_stamp = dependency.processor;
                let linked = if edge == "include" {
                    compiler::stylesheet_module_compiler::compile_stylesheet_with_included_programs_at_in_context(
                        &principal, root, vec![dependency], context,
                    )
                } else {
                    compiler::stylesheet_module_compiler::compile_stylesheet_with_single_imported_program_at_in_context(
                        &principal, root, dependency, context,
                    )
                };
                if caller == owner {
                    let program = linked.unwrap();
                    assert_eq!(program.processor, retained_stamp);
                    if caller.is_none() {
                        assert_eq!(
                            program.processor,
                            ProcessorPolicyStamp::DeclarationDrivenBaseline
                        );
                    }
                    assert!(
                        execute_compiled_stylesheet_for_test(&program, &principal, "qualified")
                            .unwrap()
                            .ends_with("ok")
                    );
                } else {
                    let failure = linked.unwrap_err();
                    assert_eq!(failure.code, "FXST1058");
                    assert_eq!(failure.category, CompileCategory::Unsupported);
                    assert_eq!(failure.location, *location);
                    assert!(failure.detail.contains("processor policy"));
                }
            }
        }
    }
}

fn module(identity: &str, version: &str, body: &str) -> Document {
    let xml = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{body}</xsl:stylesheet>"#
    );
    Document::from_parsed(
        parse_document(
            identity,
            xml.as_bytes(),
            ParseLimits {
                max_events: 1000,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn processor_edition_reference_import_chain_preserves_profile_precedence_and_provenance() {
    let leaf = module(
        "urn:leaf",
        "4.0",
        r#"<xsl:template match="/"><xsl:foo><xsl:fallback><xsl:text>leaf</xsl:text></xsl:fallback></xsl:foo></xsl:template>"#,
    );
    let inner = module("urn:inner", "1.0", r#"<xsl:import href="urn:leaf"/>"#);
    let outer = module("urn:outer", "2.0", r#"<xsl:import href="urn:inner"/>"#);
    let principal = module("urn:principal", "1.0", r#"<xsl:include href="urn:outer"/>"#);
    for (edition, preprocessing) in [
        (
            Edition::Legacy,
            StylesheetPreprocessingProfile::StrictXslt10,
        ),
        (Edition::Modern, StylesheetPreprocessingProfile::Modern),
    ] {
        let profile = Profile {
            edition,
            preprocessing,
        };
        let mut control = InvocationControl::unbounded();
        let artifact = CompiledModule::compile(&leaf, profile, &mut control)
            .unwrap()
            .import(&inner, profile, &mut control)
            .unwrap()
            .import(&outer, profile, &mut control)
            .unwrap();
        assert_eq!(artifact.profile, profile);
        assert_eq!(artifact.program.matched_templates.len(), 1);
        assert!(artifact.program.root_template.is_none());
        assert_eq!(artifact.program.matched_templates[0].import_precedence, -2);
        assert_eq!(
            artifact.program.matched_templates[0]
                .template
                .location
                .resource,
            "urn:leaf"
        );
        let program = artifact.link(&principal, profile, &mut control).unwrap();
        assert_eq!(program.matched_templates[0].import_precedence, -2);
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &principal, "imports").unwrap(),
            r#"<?xml version="1.0" encoding="UTF-8"?>leaf"#
        );
    }
}

#[test]
fn processor_edition_reference_single_import_matches_existing_composition() {
    let leaf = module(
        "urn:leaf",
        "2.0",
        r#"<xsl:template match="/">A<!--comment--> </xsl:template>"#,
    );
    let principal = module("urn:principal", "4.0", r#"<xsl:import href="urn:leaf"/>"#);
    let root = compiler::document_element(&principal).unwrap();
    for (edition, preprocessing, expected) in [
        (
            Edition::Legacy,
            StylesheetPreprocessingProfile::StrictXslt10,
            "A",
        ),
        (
            Edition::Modern,
            StylesheetPreprocessingProfile::Modern,
            "A ",
        ),
    ] {
        let profile = Profile {
            edition,
            preprocessing,
        };
        let mut control = InvocationControl::unbounded();
        let actual = CompiledModule::compile(&leaf, profile, &mut control)
            .unwrap()
            .import(&principal, profile, &mut control)
            .unwrap();
        let mut reference_control = InvocationControl::unbounded();
        let dependency = lower_module(&leaf, edition, &mut reference_control).unwrap();
        let view =
            stylesheet_visibility_view(&principal, root, preprocessing, &mut reference_control)
                .unwrap();
        let reference = compiler::stylesheet_module_compiler::compile_stylesheet_with_single_imported_program_at_in_context(
            &view, root, dependency, compilation_context(edition, preprocessing),
        ).unwrap();
        assert_eq!(actual.program, reference);
        assert_eq!(
            control.consumed(crate::execution_control_experiment::WorkDomain::XdmNode),
            reference_control.consumed(crate::execution_control_experiment::WorkDomain::XdmNode)
        );
        assert_eq!(
            execute_compiled_stylesheet_for_test(&actual.program, &principal, "import-text")
                .unwrap(),
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>{expected}"#)
        );
    }
}

#[test]
fn processor_edition_reference_import_guards_and_cancellation_remain_explicit() {
    use crate::execution_control_experiment::{ControlFailure, WorkDomain};
    let leaf = module("urn:leaf", "2.0", r#"<xsl:template match="/"/>"#);
    let principal = module("urn:principal", "2.0", r#"<xsl:import href="urn:leaf"/>"#);
    let profile = Profile {
        edition: Edition::Legacy,
        preprocessing: StylesheetPreprocessingProfile::Modern,
    };
    let compile =
        || CompiledModule::compile(&leaf, profile, &mut InvocationControl::unbounded()).unwrap();
    let other = Profile {
        edition: Edition::Modern,
        ..profile
    };
    let Err(StrictPreprocessingFailure::Compile(failure)) =
        compile().import(&principal, other, &mut InvocationControl::unbounded())
    else {
        panic!("expected profile mismatch")
    };
    assert_eq!(failure.code, "FXAR0031");
    assert_eq!(failure.category, CompileCategory::Unsupported);
    let root = compiler::document_element(&principal).unwrap();
    assert_eq!(
        failure.location,
        *principal.location(meaningful_children(&principal, root)[0])
    );
    for body in [
        r#"<xsl:include href="urn:leaf"/>"#,
        r#"<xsl:import href="urn:leaf"/><xsl:template match="/"/>"#,
        r#"<xsl:import href="urn:leaf" extra="ignored"/>"#,
    ] {
        let invalid = module("urn:principal", "4.0", body);
        assert!(
            compile()
                .import(&invalid, profile, &mut InvocationControl::unbounded())
                .is_err()
        );
    }
    let mut control = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
    assert!(matches!(
        compile().import(&principal, profile, &mut control),
        Err(StrictPreprocessingFailure::Control(
            ControlFailure::Cancelled { .. }
        ))
    ));
    assert!(
        compile()
            .import(&principal, profile, &mut InvocationControl::unbounded())
            .is_ok()
    );
}
