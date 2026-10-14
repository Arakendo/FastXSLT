//! Explicit module context for already-preprocessed admitted graph shapes.
//! Unsupported nested mixed shapes defer to the existing classification path.

use super::{
    CompileFailure, LoadedStylesheetModule, StylesheetDependencyKind, StylesheetProgram,
    compile_stylesheet, compile_stylesheet_module_at_unlinked, validate_loaded_import_order,
    validate_sibling_dependency_shape,
};
use crate::compile::golden_stylesheet_experiment::{
    compile_stylesheet_with_imported_and_included_programs_at_in_context,
    compile_stylesheet_with_imported_programs_at_in_context,
    compile_stylesheet_with_imports_in_context,
    compile_stylesheet_with_included_programs_at_in_context,
    compile_stylesheet_with_single_imported_program_at_in_context,
    stylesheet_preprocessing::StylesheetCompilationContext,
};

/// No module may choose a different context. An unsupported shape causes the
/// whole shape to defer to the original assembler, not a partial composition.
/// Principal leaves keep full principal validation; dependency leaves retain
/// the existing unlinked contract until their parent assembles them.
pub(super) fn compile_context_tree(
    module: &LoadedStylesheetModule,
    context: StylesheetCompilationContext,
    principal: bool,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    if module.dependencies.is_empty() {
        return Some(
            if principal {
                compile_stylesheet(&module.document)
            } else {
                compile_stylesheet_module_at_unlinked(&module.document, module.root)
            }
            .map(|program| context.qualify_module(program)),
        );
    }
    let include_only = module
        .dependencies
        .iter()
        .all(|dependency| dependency.dependency_kind == Some(StylesheetDependencyKind::Include));
    if principal {
        if let Err(error) = validate_loaded_import_order(module)
            .and_then(|()| validate_sibling_dependency_shape(module, include_only))
        {
            return Some(Err(error));
        }
    }
    let import_only = module
        .dependencies
        .iter()
        .all(|dependency| dependency.dependency_kind == Some(StylesheetDependencyKind::Import));
    let principal_mixed = principal
        && module.dependencies.len() == 2
        && module.dependencies[0].dependency_kind == Some(StylesheetDependencyKind::Import)
        && module.dependencies[1].dependency_kind == Some(StylesheetDependencyKind::Include);
    if !include_only && !import_only && !principal_mixed {
        return None;
    }
    // Flat principal imports have a document-aware compilation path: higher
    // precedence aliases and visibility declarations affect child compilation.
    // Retained-program assembly cannot reconstruct already-lowered aliases.
    if principal
        && import_only
        && module
            .dependencies
            .iter()
            .all(|child| child.dependencies.is_empty())
    {
        let imported = module
            .dependencies
            .iter()
            .map(|child| (&child.document, child.root))
            .collect::<Vec<_>>();
        return Some(compile_stylesheet_with_imports_in_context(
            &module.document,
            &imported,
            context,
        ));
    }
    let programs = module
        .dependencies
        .iter()
        .map(|dependency| compile_context_tree(dependency, context, false))
        .collect::<Option<Vec<_>>>()?;
    Some(
        programs
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .and_then(|mut programs| {
                if include_only {
                    compile_stylesheet_with_included_programs_at_in_context(
                        &module.document,
                        module.root,
                        programs,
                        context,
                    )
                } else if import_only && programs.len() == 1 {
                    compile_stylesheet_with_single_imported_program_at_in_context(
                        &module.document,
                        module.root,
                        programs.pop().expect("one import"),
                        context,
                    )
                } else if import_only {
                    compile_stylesheet_with_imported_programs_at_in_context(
                        &module.document,
                        module.root,
                        programs,
                        context,
                    )
                } else {
                    let included = programs.pop().expect("mixed include");
                    let imported = programs.pop().expect("mixed import");
                    compile_stylesheet_with_imported_and_included_programs_at_in_context(
                        &module.document,
                        module.root,
                        imported,
                        included,
                        context,
                    )
                }
            }),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{
        DependencyLimits, compile_loaded_graph, load_stylesheet_dependency_graph,
        preprocess_stylesheet_graph,
    };
    use super::*;
    use crate::compile::golden_stylesheet_experiment::stylesheet_preprocessing::StylesheetPreprocessingProfile;
    use crate::execution_control_experiment::InvocationControl;
    use crate::resources::{
        ResolutionLimits, ResourceLimits, ResourceSetBuilder, SnapshotResolver,
    };
    use crate::xml::quick_xml_experiment::ParseLimits;

    const PRINCIPAL: &str = "https://example.invalid/main.xsl";
    const CHILD: &str = "https://example.invalid/child.xsl";
    const NESTED: &str = "https://example.invalid/nested.xsl";

    fn stylesheet(body: &str) -> String {
        format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">{body}</xsl:stylesheet>"#
        )
    }

    fn graph(modules: &[(&str, &str)]) -> LoadedStylesheetModule {
        let mut builder = ResourceSetBuilder::new(ResourceLimits::new(8, 16384, 65536));
        for (identity, xml) in modules {
            builder.admit(*identity, xml.as_bytes().to_vec()).unwrap();
        }
        let snapshot = builder.seal();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(8));
        load_stylesheet_dependency_graph(
            &mut resolver,
            PRINCIPAL,
            DependencyLimits::new(4, 8, 65536),
            ParseLimits {
                max_events: 1000,
                max_depth: 50,
            },
            None,
            None,
        )
        .unwrap()
    }

    fn contexts() -> [(StylesheetPreprocessingProfile, bool); 3] {
        [
            (StylesheetPreprocessingProfile::Modern, false),
            (StylesheetPreprocessingProfile::Modern, true),
            (StylesheetPreprocessingProfile::StrictXslt10, false),
        ]
    }

    fn selected_graph(
        modules: &[(&str, &str)],
        profile: StylesheetPreprocessingProfile,
        reference: bool,
    ) -> LoadedStylesheetModule {
        let mut graph = graph(modules);
        if profile == StylesheetPreprocessingProfile::StrictXslt10 || reference {
            preprocess_stylesheet_graph(&mut graph, profile, &mut InvocationControl::unbounded())
                .unwrap();
        }
        graph
    }

    #[test]
    fn preprocessing_context_reaches_every_include_module_before_composition() {
        let principal = stylesheet(
            "<xsl:include href='child.xsl'/><xsl:template match='/'>P<!--removed--> </xsl:template>",
        );
        let child = stylesheet(
            "<xsl:include href='nested.xsl'/><xsl:template name='child'>C<!--removed--> </xsl:template>",
        );
        let nested = stylesheet("<xsl:template name='nested'>N<?removed yes?> </xsl:template>");
        let modules = [
            (PRINCIPAL, principal.as_str()),
            (CHILD, child.as_str()),
            (NESTED, nested.as_str()),
        ];
        for (profile, reference) in contexts() {
            let graph = selected_graph(&modules, profile, reference);
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            let expected = context.qualify_module(compile_loaded_graph(&graph).unwrap());
            let actual = compile_context_tree(&graph, context, true)
                .unwrap()
                .unwrap();
            assert_eq!(actual, expected);
            for module in [
                &graph.dependencies[0],
                &graph.dependencies[0].dependencies[0],
            ] {
                let intermediate = compile_context_tree(module, context, false)
                    .unwrap()
                    .unwrap();
                assert_eq!(intermediate.preprocessing, actual.preprocessing);
                assert_eq!(intermediate.clone().preprocessing, actual.preprocessing);
            }
        }
    }

    #[test]
    fn preprocessing_context_branching_includes_preserve_assembly_and_module_stamps() {
        let principal = stylesheet(
            "<xsl:include href='child.xsl'/><xsl:include href='second.xsl'/><xsl:include href='third.xsl'/><xsl:template match='/'><out/></xsl:template>",
        );
        let child = stylesheet(
            "<xsl:include href='nested.xsl'/><xsl:template name='child'>C</xsl:template>",
        );
        let nested = stylesheet("<xsl:template name='nested'>N</xsl:template>");
        let second = stylesheet("<xsl:template name='second'>S</xsl:template>");
        let third = stylesheet("<xsl:template name='third'>T</xsl:template>");
        let modules = [
            (PRINCIPAL, principal.as_str()),
            (CHILD, child.as_str()),
            (NESTED, nested.as_str()),
            ("https://example.invalid/second.xsl", second.as_str()),
            ("https://example.invalid/third.xsl", third.as_str()),
        ];
        for (profile, reference) in contexts() {
            let graph = selected_graph(&modules, profile, reference);
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            let actual = compile_context_tree(&graph, context, true)
                .unwrap()
                .unwrap();
            assert_eq!(
                actual,
                context.qualify_module(compile_loaded_graph(&graph).unwrap())
            );
            for module in &graph.dependencies {
                let intermediate = compile_context_tree(module, context, false)
                    .unwrap()
                    .unwrap();
                assert_eq!(intermediate.preprocessing, actual.preprocessing);
            }
        }
    }

    #[test]
    fn preprocessing_context_include_assembly_rejects_foreign_retained_modules() {
        let principal = stylesheet(
            "<xsl:include href='child.xsl'/><xsl:include href='nested.xsl'/><xsl:template match='/'><out/></xsl:template>",
        );
        let empty = stylesheet("");
        let modules = [
            (PRINCIPAL, principal.as_str()),
            (CHILD, empty.as_str()),
            (NESTED, empty.as_str()),
        ];
        for (profile, reference) in contexts() {
            let graph = selected_graph(&modules, profile, reference);
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            let included: Vec<_> = graph
                .dependencies
                .iter()
                .map(|module| {
                    compile_context_tree(module, context, false)
                        .unwrap()
                        .unwrap()
                })
                .collect();
            let expected = compile_context_tree(&graph, context, true)
                .unwrap()
                .unwrap();
            assert_eq!(
                compile_stylesheet_with_included_programs_at_in_context(
                    &graph.document,
                    graph.root,
                    included.clone(),
                    context
                )
                .unwrap(),
                expected
            );
            for (foreign_profile, foreign_reference) in contexts() {
                let foreign =
                    StylesheetCompilationContext::for_reference(foreign_profile, foreign_reference);
                if foreign == context {
                    continue;
                }
                for index in 0..2 {
                    let mut supplied = included.clone();
                    // This simulates a completed artifact from a different
                    // compilation. Assembly must not relabel it to fit.
                    supplied[index] = foreign.qualify_module(supplied[index].clone());
                    let error = compile_stylesheet_with_included_programs_at_in_context(
                        &graph.document,
                        graph.root,
                        supplied,
                        context,
                    )
                    .unwrap_err();
                    assert_eq!(error.code, "FXST1058");
                    assert_eq!(error.category, super::super::CompileCategory::Unsupported);
                    let declaration = graph.document.children(graph.root)[index];
                    assert_eq!(error.location, graph.document.location(declaration).clone());
                }
            }
        }
    }

    #[test]
    fn preprocessing_context_includes_imports_without_mutating_effective_graph() {
        let principal = stylesheet(
            "<xsl:include href='child.xsl'/><xsl:template match='/'><out/></xsl:template>",
        );
        let child = stylesheet(
            "<xsl:import href='nested.xsl'/><xsl:template name='child'>C<!--removed--> </xsl:template>",
        );
        let nested = stylesheet("<xsl:template name='nested'>N</xsl:template>");
        let modules = [
            (PRINCIPAL, principal.as_str()),
            (CHILD, child.as_str()),
            (NESTED, nested.as_str()),
        ];
        for (profile, reference) in contexts() {
            let graph = selected_graph(&modules, profile, reference);
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            let expected = compile_loaded_graph(&graph).unwrap();
            let children = graph.document.children(graph.root).to_vec();
            assert_eq!(
                compile_context_tree(&graph, context, true)
                    .unwrap()
                    .unwrap(),
                context.qualify_module(expected.clone())
            );
            assert_eq!(graph.document.children(graph.root), children);
            assert_eq!(compile_loaded_graph(&graph).unwrap(), expected);
        }
    }

    #[test]
    fn preprocessing_context_preserves_principal_validation_and_unlinked_dependency_contract() {
        let bad = stylesheet(
            "<xsl:template match='/'><xsl:call-template name='missing'/></xsl:template>",
        );
        let principal = stylesheet(
            "<xsl:include href='child.xsl'/><xsl:template name='available'><out/></xsl:template>",
        );
        let child = stylesheet(
            "<xsl:template match='/'><xsl:call-template name='available'/></xsl:template>",
        );
        for (profile, reference) in contexts() {
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            let graph = selected_graph(&[(PRINCIPAL, &bad)], profile, reference);
            assert_eq!(
                compile_context_tree(&graph, context, true)
                    .unwrap()
                    .unwrap_err(),
                compile_loaded_graph(&graph).unwrap_err()
            );
            let graph = selected_graph(
                &[(PRINCIPAL, &principal), (CHILD, &child)],
                profile,
                reference,
            );
            assert!(
                compile_context_tree(&graph.dependencies[0], context, false)
                    .unwrap()
                    .is_ok()
            );
            assert_eq!(
                compile_context_tree(&graph, context, true)
                    .unwrap()
                    .unwrap(),
                context.qualify_module(compile_loaded_graph(&graph).unwrap())
            );
        }
    }

    #[test]
    fn preprocessing_context_preserves_original_child_compile_failure_and_reuse() {
        let principal = stylesheet(
            "<xsl:include href='child.xsl'/><xsl:template match='/'><out/></xsl:template>",
        );
        let child =
            stylesheet("<xsl:template name='child'>A<!--removed-->B<xsl:value-of/></xsl:template>");
        for (profile, reference) in contexts() {
            let graph = selected_graph(
                &[(PRINCIPAL, &principal), (CHILD, &child)],
                profile,
                reference,
            );
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            let expected = compile_loaded_graph(&graph).unwrap_err();
            assert_eq!(expected.location.resource, CHILD);
            for _ in 0..2 {
                assert_eq!(
                    compile_context_tree(&graph, context, true)
                        .unwrap()
                        .unwrap_err(),
                    expected
                );
            }
        }
    }

    #[test]
    fn preprocessing_context_import_and_mixed_trees_preserve_precedence_and_intermediate_stamps() {
        let child = stylesheet(
            "<xsl:include href='nested.xsl'/><xsl:template match='r'>I</xsl:template><xsl:template name='shared'>I</xsl:template>",
        );
        let nested = stylesheet("<xsl:template name='nested'>N<!--removed--> </xsl:template>");
        let other = stylesheet(
            "<xsl:template match='r'>S</xsl:template><xsl:template name='shared'>S</xsl:template>",
        );
        for declarations in [
            "<xsl:import href='child.xsl'/>",
            "<xsl:import href='child.xsl'/><xsl:import href='other.xsl'/>",
            "<xsl:import href='child.xsl'/><xsl:include href='other.xsl'/>",
        ] {
            let principal = stylesheet(&format!(
                "{declarations}<xsl:template match='/'><out><xsl:apply-templates/><xsl:call-template name='shared'/></out></xsl:template><xsl:template match='r'>P<xsl:apply-imports/></xsl:template>"
            ));
            let modules = [
                (PRINCIPAL, principal.as_str()),
                (CHILD, child.as_str()),
                (NESTED, nested.as_str()),
                ("https://example.invalid/other.xsl", other.as_str()),
            ];
            for (profile, reference) in contexts() {
                let graph = selected_graph(&modules, profile, reference);
                let context = StylesheetCompilationContext::for_reference(profile, reference);
                let expected = context.qualify_module(compile_loaded_graph(&graph).unwrap());
                let actual = compile_context_tree(&graph, context, true)
                    .unwrap()
                    .unwrap();
                assert_eq!(actual, expected);
                for dependency in &graph.dependencies {
                    assert_eq!(
                        compile_context_tree(dependency, context, false)
                            .unwrap()
                            .unwrap()
                            .preprocessing,
                        actual.preprocessing
                    );
                }
            }
        }
    }

    #[test]
    fn preprocessing_context_flat_imports_keep_document_aware_alias_precedence() {
        let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:s="urn:source" xmlns:r="urn:principal" version="1.0"><xsl:import href="child.xsl"/><xsl:namespace-alias stylesheet-prefix="s" result-prefix="r"/></xsl:stylesheet>"#;
        let child = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:s="urn:source" xmlns:r="urn:child" version="1.0"><xsl:namespace-alias stylesheet-prefix="s" result-prefix="r"/><xsl:template match="/"><s:out/></xsl:template></xsl:stylesheet>"#;
        for (profile, reference) in contexts() {
            let graph = selected_graph(
                &[(PRINCIPAL, principal), (CHILD, child)],
                profile,
                reference,
            );
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            assert_eq!(
                compile_context_tree(&graph, context, true)
                    .unwrap()
                    .unwrap(),
                context.qualify_module(compile_loaded_graph(&graph).unwrap())
            );
        }
    }

    #[derive(Clone, Copy)]
    enum RetainedShape {
        SingleImport,
        Imports,
        Mixed,
    }

    fn compose_retained(
        graph: &LoadedStylesheetModule,
        context: StylesheetCompilationContext,
        shape: RetainedShape,
        mut programs: Vec<StylesheetProgram>,
    ) -> Result<StylesheetProgram, CompileFailure> {
        match shape {
            RetainedShape::SingleImport => {
                compile_stylesheet_with_single_imported_program_at_in_context(
                    &graph.document,
                    graph.root,
                    programs.pop().unwrap(),
                    context,
                )
            }
            RetainedShape::Imports => compile_stylesheet_with_imported_programs_at_in_context(
                &graph.document,
                graph.root,
                programs,
                context,
            ),
            RetainedShape::Mixed => {
                let included = programs.pop().unwrap();
                compile_stylesheet_with_imported_and_included_programs_at_in_context(
                    &graph.document,
                    graph.root,
                    programs.pop().unwrap(),
                    included,
                    context,
                )
            }
        }
    }

    #[test]
    fn preprocessing_context_import_and_mixed_assembly_reject_foreign_retained_artifacts() {
        let empty = stylesheet("");
        for (shape, declarations) in [
            (
                RetainedShape::SingleImport,
                "<xsl:import href='child.xsl'/>",
            ),
            (
                RetainedShape::Imports,
                "<xsl:import href='child.xsl'/><xsl:import href='nested.xsl'/>",
            ),
            (
                RetainedShape::Mixed,
                "<xsl:import href='child.xsl'/><xsl:include href='nested.xsl'/>",
            ),
        ] {
            let principal = stylesheet(&format!(
                "{declarations}<xsl:template match='/'><out/></xsl:template>"
            ));
            for (profile, reference) in contexts() {
                let graph = selected_graph(
                    &[(PRINCIPAL, &principal), (CHILD, &empty), (NESTED, &empty)],
                    profile,
                    reference,
                );
                let context = StylesheetCompilationContext::for_reference(profile, reference);
                let programs: Vec<_> = graph
                    .dependencies
                    .iter()
                    .map(|module| {
                        compile_context_tree(module, context, false)
                            .unwrap()
                            .unwrap()
                    })
                    .collect();
                assert_eq!(
                    compose_retained(&graph, context, shape, programs.clone()).unwrap(),
                    compile_context_tree(&graph, context, true)
                        .unwrap()
                        .unwrap()
                );
                for (foreign_profile, foreign_reference) in contexts() {
                    let foreign = StylesheetCompilationContext::for_reference(
                        foreign_profile,
                        foreign_reference,
                    );
                    if foreign == context {
                        continue;
                    }
                    for index in 0..programs.len() {
                        let mut supplied = programs.clone();
                        supplied[index] = foreign.qualify_module(supplied[index].clone());
                        let error = compose_retained(&graph, context, shape, supplied).unwrap_err();
                        assert_eq!(error.code, "FXST1058");
                        assert_eq!(error.category, super::super::CompileCategory::Unsupported);
                        assert_eq!(
                            error.location,
                            graph
                                .document
                                .location(graph.document.children(graph.root)[index])
                                .clone()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn preprocessing_context_keeps_import_order_and_root_dependency_count_gates() {
        let empty = stylesheet("");
        for declarations in [
            "<xsl:include href='child.xsl'/><xsl:import href='nested.xsl'/>",
            "<xsl:import href='child.xsl'/><xsl:import href='nested.xsl'/><xsl:import href='other.xsl'/>",
        ] {
            let principal = stylesheet(&format!(
                "{declarations}<xsl:template match='/'><out/></xsl:template>"
            ));
            for (profile, reference) in contexts() {
                let graph = selected_graph(
                    &[
                        (PRINCIPAL, &principal),
                        (CHILD, &empty),
                        (NESTED, &empty),
                        ("https://example.invalid/other.xsl", &empty),
                    ],
                    profile,
                    reference,
                );
                let context = StylesheetCompilationContext::for_reference(profile, reference);
                assert_eq!(
                    compile_context_tree(&graph, context, true)
                        .unwrap()
                        .unwrap_err(),
                    compile_loaded_graph(&graph).unwrap_err()
                );
            }
        }
    }

    #[test]
    fn preprocessing_context_does_not_admit_nested_mixed_shapes() {
        let principal = stylesheet(
            "<xsl:include href='child.xsl'/><xsl:template match='/'><out/></xsl:template>",
        );
        let child = stylesheet("<xsl:import href='nested.xsl'/><xsl:include href='other.xsl'/>");
        let empty = stylesheet("");
        for (profile, reference) in contexts() {
            let graph = selected_graph(
                &[
                    (PRINCIPAL, &principal),
                    (CHILD, &child),
                    (NESTED, &empty),
                    ("https://example.invalid/other.xsl", &empty),
                ],
                profile,
                reference,
            );
            let context = StylesheetCompilationContext::for_reference(profile, reference);
            let expected = compile_loaded_graph(&graph).unwrap_err();
            assert_eq!(
                expected.category,
                super::super::CompileCategory::Unsupported
            );
            assert!(compile_context_tree(&graph, context, true).is_none());
            assert_eq!(compile_loaded_graph(&graph).unwrap_err(), expected);
        }
    }
}
