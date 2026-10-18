//! Artifact lifetime/isolation evidence, not an edition-qualified reuse API.
use super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn document(identity: &str, xml: &str) -> Document {
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

fn principal() -> Document {
    document(
        "urn:principal",
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="4.0"><xsl:include href="urn:leaf"/></xsl:stylesheet>"#,
    )
}

fn leaf(body: &str) -> Document {
    document(
        "urn:leaf",
        &format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
        ),
    )
}

#[test]
fn processor_edition_reference_artifacts_outlive_concurrent_compilation_documents() {
    let (legacy, modern) = {
        let principal = principal();
        let leaf = leaf("A<!--separator--> ");
        std::thread::scope(|scope| {
            let compile = |edition| {
                lower_chain(
                    &principal,
                    &[&leaf],
                    edition,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            };
            let legacy = scope.spawn(move || compile(Edition::Legacy));
            let modern = scope.spawn(move || compile(Edition::Modern));
            (legacy.join().unwrap(), modern.join().unwrap())
        })
    }; // Both parsed modules and their preprocessing views have been dropped.
    let source = document("urn:source", "<source/>");
    assert_ne!(legacy.preprocessing, modern.preprocessing);
    std::thread::scope(|scope| {
        for (program, expected) in [
            (&legacy, "A"),
            (&modern, "A "),
            (&legacy, "A"),
            (&modern, "A "),
        ] {
            let source = &source;
            scope.spawn(move || {
                for _ in 0..32 {
                    assert_eq!(
                        execute_compiled_stylesheet_for_test(program, source, "reuse").unwrap(),
                        format!(r#"<?xml version="1.0" encoding="UTF-8"?>{expected}"#)
                    );
                }
            });
        }
    });
}

#[test]
fn processor_edition_reference_failed_recompile_and_same_uri_replacement_preserve_old_artifact() {
    let principal = principal();
    let original =
        leaf("<xsl:foo><xsl:fallback><xsl:text>original</xsl:text></xsl:fallback></xsl:foo>");
    let old = lower_chain(
        &principal,
        &[&original],
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let before = old.clone();
    let Err(StrictPreprocessingFailure::Compile(failure)) = lower_chain(
        &principal,
        &[&original],
        Edition::Modern,
        &mut InvocationControl::unbounded(),
    ) else {
        panic!("modern declaration 2.0 must refuse unknown instruction");
    };
    assert_eq!(failure.location.resource, "urn:leaf");
    assert_eq!(failure.category, CompileCategory::Invalid);
    let replacement = leaf("<xsl:text>replacement</xsl:text>");
    let new = lower_chain(
        &principal,
        &[&replacement],
        Edition::Modern,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    drop(original);
    drop(replacement);
    drop(principal);
    assert_eq!(old, before);
    let source = document("urn:source", "<source/>");
    for (program, expected) in [
        (&old, "original"),
        (&new, "replacement"),
        (&old, "original"),
    ] {
        assert_eq!(
            execute_compiled_stylesheet_for_test(program, &source, "generation").unwrap(),
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>{expected}"#)
        );
    }
}

#[test]
fn processor_edition_reference_existing_linker_refuses_mixed_preprocessing_artifacts() {
    let principal = principal();
    let leaf = leaf("<xsl:text>leaf</xsl:text>");
    let root = compiler::document_element(&principal).unwrap();
    for (caller, dependency) in [
        (Edition::Legacy, Edition::Modern),
        (Edition::Modern, Edition::Legacy),
    ] {
        let profile = match caller {
            Edition::Legacy => StylesheetPreprocessingProfile::StrictXslt10,
            Edition::Modern => StylesheetPreprocessingProfile::Modern,
        };
        let view = stylesheet_visibility_view(
            &principal,
            root,
            profile,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let foreign_profile = match dependency {
            Edition::Legacy => StylesheetPreprocessingProfile::StrictXslt10,
            Edition::Modern => StylesheetPreprocessingProfile::Modern,
        };
        let foreign = lower_module_with_preprocessing(
            &leaf,
            caller,
            foreign_profile,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let failure = compiler::stylesheet_module_compiler::compile_stylesheet_with_included_programs_at_in_context(
            &view, root, vec![foreign], compilation_context(caller, profile),
        ).unwrap_err();
        assert_eq!(failure.code, "FXST1058");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(
            failure.location,
            *principal.location(meaningful_children(&principal, root)[0])
        );
    }
    // Hold processor identity constant so preprocessing rejection is independent.
}
