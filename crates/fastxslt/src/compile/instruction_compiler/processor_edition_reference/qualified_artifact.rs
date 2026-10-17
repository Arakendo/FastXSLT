//! Test-only bounded-chain artifact qualification candidate; not a cache or API.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Profile {
    edition: Edition,
    preprocessing: StylesheetPreprocessingProfile,
}

/// No raw-program wrapping or relabel operation. Qualification is created only
/// by compilation under the supplied policy. Cross-combinations are controls.
struct CompiledModule {
    profile: Profile,
    identity: String,
    module_identities: Vec<String>,
    program: StylesheetProgram,
}

impl CompiledModule {
    fn compile(
        document: &Document,
        profile: Profile,
        control: &mut InvocationControl,
    ) -> Result<Self, StrictPreprocessingFailure> {
        let program = lower_module_with_preprocessing(
            document,
            profile.edition,
            profile.preprocessing,
            control,
        )?;
        Ok(Self {
            profile,
            identity: document.location(document.document_node()).resource.clone(),
            module_identities: vec![document.location(document.document_node()).resource.clone()],
            program,
        })
    }

    // Finalization does not permit wrapping the ordinary program back into a
    // qualified module. Intermediate inclusion retains the original profile.
    fn link(
        self,
        principal: &Document,
        expected: Profile,
        control: &mut InvocationControl,
    ) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
        Ok(self.include(principal, expected, control)?.program)
    }

    fn include(
        self,
        principal: &Document,
        expected: Profile,
        control: &mut InvocationControl,
    ) -> Result<Self, StrictPreprocessingFailure> {
        self.compose(principal, expected, false, control)
    }

    fn import(
        self,
        principal: &Document,
        expected: Profile,
        control: &mut InvocationControl,
    ) -> Result<Self, StrictPreprocessingFailure> {
        self.compose(principal, expected, true, control)
    }

    fn compose(
        self,
        principal: &Document,
        expected: Profile,
        is_import: bool,
        control: &mut InvocationControl,
    ) -> Result<Self, StrictPreprocessingFailure> {
        let kind = if is_import { "import" } else { "include" };
        let (root, include) = dependency_edge(principal, &self.identity, kind)?;
        let identity = &principal.location(root).resource;
        if self.profile != expected
            || self.module_identities.len() >= 4
            || self.module_identities.contains(identity)
        {
            return Err(unsupported(
                "FXAR0031",
                "incompatible qualification, repeated identity or chain depth outside probe",
                principal.location(include),
            )
            .into());
        }
        let view = stylesheet_visibility_view(principal, root, expected.preprocessing, control)?;
        let context = StylesheetCompilationContext::for_reference(expected.preprocessing, true);
        let program = if is_import {
            compiler::stylesheet_module_compiler::compile_stylesheet_with_single_imported_program_at_in_context(
                &view, root, self.program, context,
            )?
        } else {
            compiler::stylesheet_module_compiler::compile_stylesheet_with_included_programs_at_in_context(
            &view, root, vec![self.program], StylesheetCompilationContext::for_reference(expected.preprocessing, true),
            )?
        };
        let mut module_identities = self.module_identities;
        module_identities.push(identity.clone());
        Ok(Self {
            profile: self.profile,
            identity: identity.clone(),
            module_identities,
            program,
        })
    }
}

#[cfg(test)]
#[path = "qualified_import_tests.rs"]
mod import_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn module(identity: &str, body: &str) -> Document {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">{body}</xsl:stylesheet>"#
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
    fn processor_edition_reference_qualified_chain_matches_document_chain() {
        let leaf = module(
            "urn:leaf",
            r#"<xsl:template match="/">A<!--separator--> </xsl:template>"#,
        );
        let inner = module("urn:inner", r#"<xsl:include href="urn:leaf"/>"#);
        let outer = module("urn:outer", r#"<xsl:include href="urn:inner"/>"#);
        let principal = module("urn:principal", r#"<xsl:include href="urn:outer"/>"#);
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
                .include(&inner, profile, &mut control)
                .unwrap()
                .include(&outer, profile, &mut control)
                .unwrap();
            assert_eq!(artifact.profile, profile);
            assert_eq!(
                artifact.module_identities,
                ["urn:leaf", "urn:inner", "urn:outer"]
            );
            let actual = artifact.link(&principal, profile, &mut control).unwrap();
            let mut reference_control = InvocationControl::unbounded();
            let reference = lower_chain(
                &principal,
                &[&outer, &inner, &leaf],
                edition,
                &mut reference_control,
            )
            .unwrap();
            assert_eq!(actual, reference);
            assert_eq!(
                control.consumed(crate::execution_control_experiment::WorkDomain::XdmNode),
                reference_control
                    .consumed(crate::execution_control_experiment::WorkDomain::XdmNode)
            );
        }
    }

    #[test]
    fn processor_edition_reference_qualified_chain_preserves_transitive_guards() {
        let leaf = module("urn:leaf", r#"<xsl:template match="/"/>"#);
        let inner = module("urn:inner", r#"<xsl:include href="urn:leaf"/>"#);
        let outer = module("urn:outer", r#"<xsl:include href="urn:inner"/>"#);
        let principal = module("urn:principal", r#"<xsl:include href="urn:outer"/>"#);
        let profile = Profile {
            edition: Edition::Legacy,
            preprocessing: StylesheetPreprocessingProfile::Modern,
        };
        let prepare = || {
            CompiledModule::compile(&leaf, profile, &mut InvocationControl::unbounded())
                .unwrap()
                .include(&inner, profile, &mut InvocationControl::unbounded())
                .unwrap()
                .include(&outer, profile, &mut InvocationControl::unbounded())
                .unwrap()
        };
        // Same preprocessing must not erase the original edition at any hop.
        let wrong = Profile {
            edition: Edition::Modern,
            ..profile
        };
        assert!(
            prepare()
                .link(&principal, wrong, &mut InvocationControl::unbounded())
                .is_err()
        );
        let repeated = module("urn:leaf", r#"<xsl:include href="urn:outer"/>"#);
        assert!(
            prepare()
                .link(&repeated, profile, &mut InvocationControl::unbounded())
                .is_err()
        );
        let fifth = module("urn:fifth", r#"<xsl:include href="urn:principal"/>"#);
        let four = prepare()
            .include(&principal, profile, &mut InvocationControl::unbounded())
            .unwrap();
        assert!(
            four.link(&fifth, profile, &mut InvocationControl::unbounded())
                .is_err()
        );
    }

    #[test]
    fn processor_edition_reference_qualified_chain_final_link_cancellation_is_not_publication() {
        use crate::execution_control_experiment::{ControlFailure, WorkDomain};
        let leaf = module(
            "urn:leaf",
            r#"<xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template>"#,
        );
        let inner = module("urn:inner", r#"<xsl:include href="urn:leaf"/>"#);
        let principal = module("urn:principal", r#"<xsl:include href="urn:inner"/>"#);
        let profile = Profile {
            edition: Edition::Legacy,
            preprocessing: StylesheetPreprocessingProfile::StrictXslt10,
        };
        let artifact = CompiledModule::compile(&leaf, profile, &mut InvocationControl::unbounded())
            .unwrap()
            .include(&inner, profile, &mut InvocationControl::unbounded())
            .unwrap();
        drop(leaf);
        drop(inner);
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
        assert!(matches!(
            artifact.link(&principal, profile, &mut control),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled { .. }
            ))
        ));
    }

    #[test]
    fn processor_edition_reference_qualified_artifact_checks_independent_dimensions() {
        let principal = module("urn:principal", r#"<xsl:include href="urn:leaf"/>"#);
        let leaf = module(
            "urn:leaf",
            r#"<xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template>"#,
        );
        let profiles = [
            Profile {
                edition: Edition::Legacy,
                preprocessing: StylesheetPreprocessingProfile::StrictXslt10,
            },
            Profile {
                edition: Edition::Modern,
                preprocessing: StylesheetPreprocessingProfile::StrictXslt10,
            },
            Profile {
                edition: Edition::Legacy,
                preprocessing: StylesheetPreprocessingProfile::Modern,
            },
            Profile {
                edition: Edition::Modern,
                preprocessing: StylesheetPreprocessingProfile::Modern,
            },
        ];
        for actual in profiles {
            for expected in profiles {
                let artifact =
                    CompiledModule::compile(&leaf, actual, &mut InvocationControl::unbounded())
                        .unwrap();
                let mut control = InvocationControl::unbounded();
                let result = artifact.link(&principal, expected, &mut control);
                if actual == expected {
                    let program = result.unwrap();
                    assert_eq!(
                        execute_compiled_stylesheet_for_test(&program, &principal, "qualified")
                            .unwrap(),
                        r#"<?xml version="1.0" encoding="UTF-8"?>ok"#
                    );
                } else {
                    let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                        panic!("expected profile rejection")
                    };
                    assert_eq!(failure.code, "FXAR0031");
                    assert_eq!(failure.category, CompileCategory::Unsupported);
                    let root = compiler::document_element(&principal).unwrap();
                    assert_eq!(
                        failure.location,
                        *principal.location(meaningful_children(&principal, root)[0])
                    );
                    assert_eq!(
                        control.consumed(crate::execution_control_experiment::WorkDomain::XdmNode),
                        0
                    );
                }
            }
        }
    }

    #[test]
    fn processor_edition_reference_qualified_artifact_cannot_relabel_legacy_fallback() {
        let principal = module("urn:principal", r#"<xsl:include href="urn:leaf"/>"#);
        let leaf = module(
            "urn:leaf",
            r#"<xsl:template match="/"><xsl:foo><xsl:fallback><xsl:text>legacy</xsl:text></xsl:fallback></xsl:foo></xsl:template>"#,
        );
        let legacy = Profile {
            edition: Edition::Legacy,
            preprocessing: StylesheetPreprocessingProfile::Modern,
        };
        let modern = Profile {
            edition: Edition::Modern,
            ..legacy
        };
        assert!(
            CompiledModule::compile(&leaf, modern, &mut InvocationControl::unbounded()).is_err()
        );
        let artifact =
            CompiledModule::compile(&leaf, legacy, &mut InvocationControl::unbounded()).unwrap();
        assert!(
            artifact
                .link(&principal, modern, &mut InvocationControl::unbounded())
                .is_err()
        );
        let artifact =
            CompiledModule::compile(&leaf, legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(leaf);
        let program = artifact
            .link(&principal, legacy, &mut InvocationControl::unbounded())
            .unwrap();
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &principal, "legacy-control").unwrap(),
            r#"<?xml version="1.0" encoding="UTF-8"?>legacy"#
        );
    }

    #[test]
    fn processor_edition_reference_qualified_artifact_preserves_bounded_link_controls() {
        use crate::execution_control_experiment::{ControlFailure, WorkDomain};
        let profile = Profile {
            edition: Edition::Legacy,
            preprocessing: StylesheetPreprocessingProfile::StrictXslt10,
        };
        let leaf = module("urn:leaf", r#"<xsl:template match="/"/>"#);
        for (identity, body) in [
            ("urn:principal", r#"<xsl:include href="urn:wrong"/>"#),
            ("urn:leaf", r#"<xsl:include href="urn:leaf"/>"#),
            (
                "urn:principal",
                r#"<xsl:include href="urn:leaf"/><xsl:template match="/"/>"#,
            ),
        ] {
            let principal = module(identity, body);
            let artifact =
                CompiledModule::compile(&leaf, profile, &mut InvocationControl::unbounded())
                    .unwrap();
            assert!(
                artifact
                    .link(&principal, profile, &mut InvocationControl::unbounded())
                    .is_err()
            );
        }
        let principal = module("urn:principal", r#"<xsl:include href="urn:leaf"/>"#);
        let artifact =
            CompiledModule::compile(&leaf, profile, &mut InvocationControl::unbounded()).unwrap();
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
        assert!(matches!(
            artifact.link(&principal, profile, &mut control),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled { .. }
            ))
        ));
        let artifact =
            CompiledModule::compile(&leaf, profile, &mut InvocationControl::unbounded()).unwrap();
        assert!(
            artifact
                .link(&principal, profile, &mut InvocationControl::unbounded())
                .is_ok()
        );
    }
}
