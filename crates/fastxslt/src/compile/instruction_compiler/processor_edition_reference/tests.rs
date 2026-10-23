use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::compile::golden_stylesheet_experiment::{document_element, stylesheet_preprocessing};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn module_document(xml: &str) -> Document {
    Document::from_parsed(
        parse_document(
            "urn:edition-module",
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
fn selected_migration_preserves_independent_root_reference_outcomes() {
    use crate::execution_control_experiment::InvocationControl;
    for body in [
        "<xsl:text>literal</xsl:text>",
        "<xsl:if test='false()'><xsl:foo/></xsl:if><xsl:text>survived</xsl:text>",
        "<xsl:if test='true()'><xsl:foo/></xsl:if>",
        "<xsl:sequence select='1'><xsl:fallback><xsl:text>fallback</xsl:text></xsl:fallback></xsl:sequence>",
    ] {
        let document = module_document(&format!(
            "<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform' version='2.0'><xsl:template match='/'>{body}</xsl:template></xsl:stylesheet>"
        ));
        // The original root-template reference retains its own admission and
        // traversal; it does not call the migrated matched-module lowerer.
        let reference = lower_module(
            &document,
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let candidate = compile_selected_legacy_import_module(
            &document,
            Vec::new(),
            &mut InvocationControl::unbounded(),
            true,
        )
        .unwrap();
        assert_eq!(candidate.processor, reference.processor);
        assert_eq!(candidate.preprocessing, reference.preprocessing);
        let source = module_document("<r/>");
        drop(document);
        assert_eq!(
            execute_compiled_stylesheet_for_test(&candidate, &source, "migration"),
            execute_compiled_stylesheet_for_test(&reference, &source, "migration"),
        );
    }
}

#[test]
fn processor_edition_reference_modern_missing_fallback_is_static_in_original_module() {
    use crate::execution_control_experiment::InvocationControl;
    for condition in ["true()", "false()"] {
        let document = module_document(&format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="4.0"><xsl:template match="/"><xsl:if test="{condition}"><xsl:foo/></xsl:if></xsl:template></xsl:stylesheet>"#
        ));
        let Err(stylesheet_preprocessing::StrictPreprocessingFailure::Compile(failure)) =
            lower_module(
                &document,
                Edition::Modern,
                &mut InvocationControl::unbounded(),
            )
        else {
            panic!("expected static modern failure")
        };
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "FXAR0031");
        let legacy = lower_module(
            &document,
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let result = execute_compiled_stylesheet_for_test(&legacy, &document, "timing");
        if condition == "true()" {
            assert!(result.unwrap_err().contains("XTDE1450"));
        } else {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn processor_edition_reference_local_version_forms_remain_guarded() {
    for attribute in [r#"version="4.0""#, r#"xsl:version="4.0""#] {
        let (document, element) = fixture(&format!(
            r#"<xsl:if test="true()" {attribute}><xsl:text>not admitted</xsl:text></xsl:if>"#
        ));
        for edition in [Edition::Legacy, Edition::Modern] {
            let failure = lower(&document, element, edition, DeclaredVersion::Two).unwrap_err();
            assert_eq!(failure.category, CompileCategory::Unsupported);
            assert_eq!(failure.code, "FXAR0031");
        }
    }
}

#[test]
fn processor_edition_reference_module_compiles_original_stylesheet_without_shell() {
    use crate::execution_control_experiment::InvocationControl;
    for version in ["1.0", "2.0", "3.0", "4.0"] {
        let document = module_document(&format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:template match="/"><xsl:foo><xsl:fallback><xsl:text>fallback</xsl:text></xsl:fallback></xsl:foo></xsl:template></xsl:stylesheet>"#
        ));
        for edition in [Edition::Legacy, Edition::Modern] {
            let eligible = match edition {
                Edition::Legacy => version != "1.0",
                Edition::Modern => version == "4.0",
            };
            let result = lower_module(&document, edition, &mut InvocationControl::unbounded());
            if eligible {
                let program = result.unwrap();
                assert_eq!(program.declared_version, version);
                assert_eq!(
                    execute_compiled_stylesheet_for_test(&program, &document, "original-module")
                        .unwrap(),
                    r#"<?xml version="1.0" encoding="UTF-8"?>fallback"#
                );
            } else {
                assert!(matches!(
                    result,
                    Err(
                        stylesheet_preprocessing::StrictPreprocessingFailure::Compile(
                            CompileFailure {
                                category: CompileCategory::Invalid,
                                ..
                            }
                        )
                    )
                ));
            }
        }
    }
}

#[test]
fn processor_edition_reference_module_matches_shared_program_and_preprocessing() {
    use crate::execution_control_experiment::InvocationControl;
    let (document, _) = fixture("A<!--separator--> ");
    let root = document_element(&document).unwrap();
    for (edition, preprocessing, expected_text) in [
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
        let view = stylesheet_preprocessing::stylesheet_visibility_view(
            &document,
            root,
            preprocessing,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let expected = compilation_context(edition, preprocessing)
            .qualify_module(compile_stylesheet(&view).unwrap());
        let actual = lower_module(&document, edition, &mut InvocationControl::unbounded()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            execute_compiled_stylesheet_for_test(&actual, &document, "preprocessed-module")
                .unwrap(),
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>{expected_text}"#)
        );
    }
}

#[test]
fn processor_edition_reference_module_refuses_unpropagated_declarations() {
    use crate::execution_control_experiment::InvocationControl;
    for declarations in [
        r#"<xsl:include href="other.xsl"/><xsl:template match="/"/>"#,
        r#"<xsl:variable name="v" select="1"/><xsl:template match="/"/>"#,
        r#"<xsl:template match="item"/>"#,
        r#"<xsl:template match="/" name="named"/>"#,
        r#"<xsl:template match="/"><xsl:param name="p"/></xsl:template>"#,
    ] {
        let document = module_document(&format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">{declarations}</xsl:stylesheet>"#
        ));
        assert!(
            lower_module(
                &document,
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
}

#[test]
fn processor_edition_reference_module_guards_ignore_compatibility_exceptions() {
    use crate::execution_control_experiment::InvocationControl;
    for version in ["1.0", "2.0", "4.0"] {
        for (root_attribute, template_attribute) in [
            ("", r#"mode="m""#),
            ("", r#"priority="1""#),
            ("", r#"name="named""#),
            ("", r#"p:extra="ignored""#),
            ("", r#"extra="ignored""#),
            (r#"default-mode="m""#, ""),
            (r#"p:extra="ignored""#, ""),
            (r#"xml:space="preserve""#, ""),
        ] {
            let document = module_document(&format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:probe" version="{version}" {root_attribute}><xsl:template match="/" {template_attribute}><xsl:text>body</xsl:text></xsl:template></xsl:stylesheet>"#
            ));
            for edition in [Edition::Legacy, Edition::Modern] {
                assert!(matches!(
                    lower_module(&document, edition, &mut InvocationControl::unbounded()),
                    Err(
                        stylesheet_preprocessing::StrictPreprocessingFailure::Compile(
                            CompileFailure {
                                code: "FXAR0031",
                                category: CompileCategory::Unsupported,
                                ..
                            }
                        )
                    )
                ));
            }
        }
    }
}

#[test]
fn processor_edition_reference_module_cancellation_returns_no_program() {
    use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
    let (document, _) = fixture("<xsl:text>body</xsl:text>");
    let mut control = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 3);
    assert!(matches!(
        lower_module(&document, Edition::Legacy, &mut control),
        Err(
            stylesheet_preprocessing::StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled { .. }
            )
        )
    ));
}

#[test]
fn processor_edition_reference_template_matches_shared_metadata_and_body() {
    let (document, instruction) = fixture(
        r#"<xsl:text>first</xsl:text><xsl:if test="true()"><xsl:text>second</xsl:text></xsl:if>"#,
    );
    let template = document.parent(instruction).unwrap();
    let expected =
        crate::compile::golden_stylesheet_experiment::compile_template(&document, template)
            .unwrap();
    for edition in [Edition::Legacy, Edition::Modern] {
        assert_eq!(
            lower_template(&document, template, edition, DeclaredVersion::Two).unwrap(),
            expected
        );
    }
}

#[test]
fn processor_edition_reference_complete_template_executes_without_body_replacement() {
    let (document, instruction) = fixture(
        "<xsl:text>before</xsl:text><xsl:foo><xsl:fallback><xsl:text>fallback</xsl:text></xsl:fallback></xsl:foo><xsl:text>after</xsl:text>",
    );
    let template = document.parent(instruction).unwrap();
    let compiled =
        lower_template(&document, template, Edition::Legacy, DeclaredVersion::Two).unwrap();
    assert_eq!(compiled.location, *document.location(template));
    let (shell, _) = fixture("<xsl:text>shell</xsl:text>");
    let mut program = compile_stylesheet(&shell).unwrap();
    program.root_template = Some(compiled);
    assert_eq!(
        execute_compiled_stylesheet_for_test(&program, &shell, "template-probe").unwrap(),
        r#"<?xml version="1.0" encoding="UTF-8"?>beforefallbackafter"#
    );
    let failure =
        lower_template(&document, template, Edition::Modern, DeclaredVersion::Two).unwrap_err();
    assert_eq!(failure.category, CompileCategory::Invalid);
    assert_eq!(
        failure.location,
        *document.location(document.children(template)[1])
    );
}

#[test]
fn processor_edition_reference_template_refuses_parameter_default_reentry() {
    for body in [
        r#"<xsl:param name="p" select="system-property('xsl:version')"/><xsl:text>body</xsl:text>"#,
        r#"<xsl:param name="p"><xsl:foo/></xsl:param>"#,
    ] {
        let (document, instruction) = fixture(body);
        let template = document.parent(instruction).unwrap();
        for edition in [Edition::Legacy, Edition::Modern] {
            let failure =
                lower_template(&document, template, edition, DeclaredVersion::Two).unwrap_err();
            assert_eq!(failure.code, "FXAR0031");
            assert_eq!(failure.category, CompileCategory::Unsupported);
            assert_eq!(failure.location, *document.location(template));
        }
    }
}

#[derive(Clone, Copy)]
struct CompilationWitness {
    edition: Edition,
    preprocessing: StylesheetPreprocessingProfile,
}

impl CompilationWitness {
    fn coherent(edition: Edition) -> Self {
        Self {
            edition,
            preprocessing: match edition {
                Edition::Legacy => StylesheetPreprocessingProfile::StrictXslt10,
                Edition::Modern => StylesheetPreprocessingProfile::Modern,
            },
        }
    }

    fn lower(
        self,
        document: &Document,
        element: NodeId,
        version: DeclaredVersion,
        control: &mut crate::execution_control_experiment::InvocationControl,
    ) -> Result<Vec<Instruction>, stylesheet_preprocessing::StrictPreprocessingFailure> {
        let root = document_element(document)?;
        let view = stylesheet_preprocessing::stylesheet_visibility_view(
            document,
            root,
            self.preprocessing,
            control,
        )?;
        Ok(lower(&view, element, self.edition, version)?)
    }
}

#[test]
fn processor_edition_reference_choose_matches_shared_structure_and_diagnostics() {
    let (document, element) = fixture(
        r#"<xsl:choose><xsl:when test="false()"><xsl:text>no</xsl:text></xsl:when><xsl:when test="true()"><xsl:text>yes</xsl:text></xsl:when><xsl:otherwise><xsl:text>other</xsl:text></xsl:otherwise></xsl:choose>"#,
    );
    for edition in [Edition::Legacy, Edition::Modern] {
        assert_eq!(
            lower(&document, element, edition, DeclaredVersion::Two).unwrap(),
            vec![compile_choose(&document, element).unwrap()]
        );
    }
    for body in [
        "<xsl:choose/>",
        "<xsl:choose><xsl:when/></xsl:choose>",
        r#"<xsl:choose><xsl:when test="true()"/><xsl:otherwise/><xsl:otherwise/></xsl:choose>"#,
        r#"<xsl:choose><xsl:otherwise/><xsl:when test="true()"/></xsl:choose>"#,
    ] {
        let (document, element) = fixture(body);
        assert_eq!(
            lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap_err(),
            compile_choose(&document, element).unwrap_err()
        );
    }
}

#[test]
fn processor_edition_reference_choose_keeps_branch_order_and_deferred_failure() {
    for (condition, success) in [("true()", true), ("false()", false)] {
        let (document, element) = fixture(&format!(
            r#"<xsl:choose><xsl:when test="{condition}"><xsl:text>first</xsl:text></xsl:when><xsl:when test="true()"><xsl:foo/></xsl:when><xsl:otherwise><xsl:text>never</xsl:text></xsl:otherwise></xsl:choose>"#
        ));
        for (edition, version) in [
            (Edition::Legacy, DeclaredVersion::Two),
            (Edition::Modern, DeclaredVersion::Four),
        ] {
            if matches!(edition, Edition::Modern) {
                assert_eq!(
                    lower(&document, element, edition, version)
                        .unwrap_err()
                        .category,
                    CompileCategory::Invalid
                );
                continue;
            }
            let lowered = lower(&document, element, edition, version).unwrap();
            let (shell, _) = fixture("<xsl:text>shell</xsl:text>");
            let mut program = compile_stylesheet(&shell).unwrap();
            program.root_template.as_mut().unwrap().body = lowered;
            let result = execute_compiled_stylesheet_for_test(&program, &shell, "choose-probe");
            if success {
                assert_eq!(
                    result.unwrap(),
                    r#"<?xml version="1.0" encoding="UTF-8"?>first"#
                );
            } else {
                assert!(result.unwrap_err().contains("XTDE1450"));
            }
        }
        assert_eq!(
            lower(&document, element, Edition::Modern, DeclaredVersion::Two)
                .unwrap_err()
                .category,
            CompileCategory::Invalid
        );
    }
}

#[test]
fn processor_edition_reference_choose_propagates_availability_and_otherwise_policy() {
    let (document, element) = fixture(
        r#"<xsl:choose><xsl:when test="element-available('xsl:sequence')"><xsl:text>modern</xsl:text></xsl:when><xsl:otherwise><xsl:foo><xsl:fallback><xsl:text>legacy</xsl:text></xsl:fallback></xsl:foo></xsl:otherwise></xsl:choose>"#,
    );
    let lowered = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap();
    let Instruction::Choose {
        branches,
        otherwise,
        ..
    } = &lowered[0]
    else {
        panic!("expected choose")
    };
    assert!(matches!(
        branches[0].test,
        BooleanExpression::Constant(false)
    ));
    assert_eq!(text(otherwise), "legacy");
    assert_eq!(
        lower(&document, element, Edition::Modern, DeclaredVersion::Four)
            .unwrap_err()
            .category,
        CompileCategory::Unsupported
    );
    for body in [
        r#"<xsl:choose><xsl:when test="true()" xsl:version="1.0"/></xsl:choose>"#,
        r#"<xsl:choose><xsl:when test="false()"/><xsl:otherwise xsl:version="1.0"/></xsl:choose>"#,
    ] {
        let (document, element) = fixture(body);
        assert!(lower(&document, element, Edition::Legacy, DeclaredVersion::Two).is_err());
    }
}

#[test]
fn processor_edition_reference_availability_drives_actual_conditional_execution() {
    for (expression, expected) in [
        ("element-available('xsl:text')", "visible"),
        ("function-available('true')", "visible"),
        ("element-available('xsl:sequence')", ""),
    ] {
        let (document, element) = fixture(&format!(
            r#"<xsl:if test="{expression}"><xsl:text>visible</xsl:text></xsl:if>"#
        ));
        let lowered = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap();
        let (shell, _) = fixture("<xsl:text>shell</xsl:text>");
        let mut program = compile_stylesheet(&shell).unwrap();
        program.root_template.as_mut().unwrap().body = lowered;
        assert_eq!(
            execute_compiled_stylesheet_for_test(&program, &shell, "availability-probe").unwrap(),
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>{expected}"#)
        );
    }
}

#[test]
fn processor_edition_reference_unproved_availability_never_becomes_false() {
    let (document, element) = fixture(
        r#"<xsl:if test="element-available('xsl:sequence')"><xsl:text>visible</xsl:text></xsl:if>"#,
    );
    // Production currently folds this to false. The experiment must refuse
    // the unproved modern capability rather than accidentally reusing it.
    assert!(matches!(
        compile_if(&document, element).unwrap(),
        Instruction::If {
            test: BooleanExpression::Constant(false),
            ..
        }
    ));
    let failure = lower(&document, element, Edition::Modern, DeclaredVersion::Two).unwrap_err();
    assert_eq!(failure.code, "FXAR0031");
    assert_eq!(failure.category, CompileCategory::Unsupported);
    assert_eq!(failure.location, *document.location(element));
}

#[test]
fn processor_edition_reference_introspection_in_nested_condition_keeps_policy() {
    let (document, element) = fixture(
        r#"<xsl:foo><xsl:fallback><xsl:if test="element-available('xsl:sequence')"><xsl:foo/></xsl:if></xsl:fallback></xsl:foo>"#,
    );
    let body = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap();
    assert!(matches!(
        &body[0],
        Instruction::If {
            test: BooleanExpression::Constant(false),
            ..
        }
    ));
    assert_eq!(
        lower(&document, element, Edition::Modern, DeclaredVersion::Four)
            .unwrap_err()
            .category,
        CompileCategory::Unsupported
    );
    let (document, element) = fixture(
        r#"<xsl:if test="system-property('xsl:version')"><xsl:text>not boolean probe</xsl:text></xsl:if>"#,
    );
    assert_eq!(
        lower(&document, element, Edition::Legacy, DeclaredVersion::Two)
            .unwrap_err()
            .category,
        CompileCategory::Unsupported
    );
}

#[test]
fn processor_edition_reference_conditional_lowering_matches_shared_compiler() {
    for condition in ["true()", "false()"] {
        let (document, element) = fixture(&format!(
            r#"<xsl:if test="{condition}"><xsl:text>body</xsl:text></xsl:if>"#
        ));
        for edition in [Edition::Legacy, Edition::Modern] {
            assert_eq!(
                lower(&document, element, edition, DeclaredVersion::Two).unwrap(),
                vec![compile_if(&document, element).unwrap()]
            );
        }
    }
    for body in ["<xsl:if/>", r#"<xsl:if test="true()" unexpected="yes"/>"#] {
        let (document, element) = fixture(body);
        let expected = compile_if(&document, element).unwrap_err();
        let actual = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap_err();
        assert_eq!(actual, expected);
    }
}

#[test]
fn processor_edition_reference_conditional_reachability_uses_real_lowered_body() {
    for (condition, reached) in [("false()", false), ("true()", true)] {
        let (document, element) = fixture(&format!(
            r#"<xsl:if test="{condition}"><xsl:foo/></xsl:if>"#
        ));
        for (edition, version) in [
            (Edition::Legacy, DeclaredVersion::Two),
            (Edition::Modern, DeclaredVersion::Four),
        ] {
            if matches!(edition, Edition::Modern) {
                assert_eq!(
                    lower(&document, element, edition, version)
                        .unwrap_err()
                        .category,
                    CompileCategory::Invalid
                );
                continue;
            }
            let lowered = lower(&document, element, edition, version).unwrap();
            let (shell, _) = fixture("<xsl:text>shell</xsl:text>");
            let mut program = compile_stylesheet(&shell).unwrap();
            program.root_template.as_mut().unwrap().body = lowered;
            let result =
                execute_compiled_stylesheet_for_test(&program, &shell, "conditional-edition-probe");
            if reached {
                assert!(result.unwrap_err().contains("XTDE1450"));
            } else {
                assert!(result.is_ok());
            }
        }
        // A false condition is not permission to erase a static modern error.
        assert_eq!(
            lower(&document, element, Edition::Modern, DeclaredVersion::Two)
                .unwrap_err()
                .category,
            CompileCategory::Invalid
        );
    }
}

#[test]
fn processor_edition_reference_conditional_nesting_keeps_edition_and_depth_guard() {
    let (document, element) = fixture(
        r#"<xsl:foo><xsl:fallback><xsl:if test="true()"><xsl:foo><xsl:fallback><xsl:text>yes</xsl:text></xsl:fallback></xsl:foo></xsl:if></xsl:fallback></xsl:foo>"#,
    );
    let lowered = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap();
    let Instruction::If { body, .. } = &lowered[0] else {
        panic!("expected shared conditional")
    };
    assert_eq!(text(body), "yes");
    let (deep, element) = fixture(&format!(
        "{}<xsl:text>deep</xsl:text>{}",
        r#"<xsl:if test="true()">"#.repeat(17),
        "</xsl:if>".repeat(17)
    ));
    assert_eq!(
        lower(&deep, element, Edition::Legacy, DeclaredVersion::Two)
            .unwrap_err()
            .category,
        CompileCategory::Unsupported
    );
}

#[test]
fn processor_edition_reference_context_preprocesses_before_nested_lowering() {
    use crate::execution_control_experiment::InvocationControl;
    let (document, element) = fixture(
        "<xsl:foo><xsl:fallback><xsl:foo><xsl:fallback>A<!--boundary--> </xsl:fallback></xsl:foo></xsl:fallback></xsl:foo>",
    );
    for (edition, expected) in [(Edition::Legacy, "A"), (Edition::Modern, "A ")] {
        let context = CompilationWitness::coherent(edition);
        let body = context
            .lower(
                &document,
                element,
                DeclaredVersion::Four,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
        assert_eq!(text(&body), expected);
        // The same context, not a local version helper, owns introspection.
        assert!(
            introspect(
                &document,
                element,
                context.edition,
                "system-property('xsl:version')"
            )
            .is_some()
        );
    }
    // Preprocessing is an immutable view, not a source rewrite.
    assert_eq!(
        text(&lower(&document, element, Edition::Legacy, DeclaredVersion::Four).unwrap()),
        "A "
    );
}

#[test]
fn processor_edition_reference_context_keeps_preprocessing_orthogonal() {
    use crate::execution_control_experiment::InvocationControl;
    let (document, element) =
        fixture("<xsl:foo><xsl:fallback>A<!--boundary--> </xsl:fallback></xsl:foo>");
    for preprocessing in [
        StylesheetPreprocessingProfile::StrictXslt10,
        StylesheetPreprocessingProfile::Modern,
    ] {
        let context = CompilationWitness {
            edition: Edition::Modern,
            preprocessing,
        };
        let failure = context
            .lower(
                &document,
                element,
                DeclaredVersion::Two,
                &mut InvocationControl::unbounded(),
            )
            .unwrap_err();
        assert!(matches!(
            failure,
            stylesheet_preprocessing::StrictPreprocessingFailure::Compile(CompileFailure {
                category: CompileCategory::Invalid,
                ..
            })
        ));
    }
}

#[test]
fn processor_edition_reference_context_preserves_preprocessing_cancellation() {
    use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
    let (document, element) = fixture(
        "<xsl:foo><xsl:fallback><xsl:text>never published</xsl:text></xsl:fallback></xsl:foo>",
    );
    for edition in [Edition::Legacy, Edition::Modern] {
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 3);
        let result = CompilationWitness::coherent(edition).lower(
            &document,
            element,
            DeclaredVersion::Four,
            &mut control,
        );
        assert!(matches!(
            result,
            Err(
                stylesheet_preprocessing::StrictPreprocessingFailure::Control(
                    ControlFailure::Cancelled { .. }
                )
            )
        ));
    }
}

fn fixture(body: &str) -> (Document, NodeId) {
    let xml = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
    );
    let document = Document::from_parsed(
        parse_document(
            "urn:edition-probe",
            xml.as_bytes(),
            ParseLimits {
                max_events: 1000,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap();
    let root = document.children(document.document_node())[0];
    let template = document.children(root)[0];
    let element = document.children(template)[0];
    (document, element)
}

fn text(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(|instruction| match instruction {
            Instruction::Text { value, .. } => value.as_str(),
            _ => panic!("expected ordinary text lowering"),
        })
        .collect()
}

#[test]
fn processor_edition_reference_introspection_is_explicit_not_local_version_selected() {
    use xslt10_static_introspection_compiler::StaticIntrospectionValue as Value;
    let (document, element) = fixture(r#"<xsl:text xsl:version="1.0">text</xsl:text>"#);
    for (edition, version) in [(Edition::Legacy, "1"), (Edition::Modern, "3.0")] {
        assert_eq!(
            introspect(
                &document,
                element,
                edition,
                "system-property('xsl:version')"
            ),
            Some(Value::String(version.to_owned()))
        );
        assert_eq!(
            introspect(&document, element, edition, "element-available('xsl:text')"),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            introspect(&document, element, edition, "function-available('true')"),
            Some(Value::Boolean(true))
        );
    }
}

#[test]
fn processor_edition_reference_recognition_does_not_assert_availability() {
    use xslt10_static_introspection_compiler::StaticIntrospectionValue as Value;
    let (document, element) = fixture(r#"<xsl:sequence select="'modern'"/>"#);
    assert_eq!(
        introspect(
            &document,
            element,
            Edition::Legacy,
            "element-available('xsl:sequence')"
        ),
        Some(Value::Boolean(false))
    );
    assert_eq!(
        introspect(
            &document,
            element,
            Edition::Modern,
            "element-available('xsl:sequence')"
        ),
        None
    );
    assert_eq!(
        lower(&document, element, Edition::Modern, DeclaredVersion::Two)
            .unwrap_err()
            .category,
        CompileCategory::Unsupported
    );
}

#[test]
fn processor_edition_reference_introspection_preserves_qname_and_refusal_boundaries() {
    use xslt10_static_introspection_compiler::StaticIntrospectionValue as Value;
    let (document, element) = fixture(
        r#"<xsl:text xmlns:p="http://www.w3.org/1999/XSL/Transform" xmlns="http://www.w3.org/1999/XSL/Transform">text</xsl:text>"#,
    );
    for expression in ["element-available('p:text')", "element-available('text')"] {
        assert_eq!(
            introspect(&document, element, Edition::Modern, expression),
            Some(Value::Boolean(true))
        );
    }
    for expression in [
        "element-available('missing:text')",
        "element-available('p:a:b')",
        "function-available('p:true')",
        "function-available('true', 0)",
        "system-property(concat('p:', 'version'))",
        "element-available('p:copy')",
    ] {
        assert_eq!(
            introspect(&document, element, Edition::Modern, expression),
            None,
            "{expression}"
        );
    }
}

#[test]
fn processor_edition_reference_shared_runtime_defers_only_reached_failure() {
    let (document, element) = fixture("<xsl:foo/>");
    let lowered = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap();
    let (shell, _) = fixture("<xsl:text>shell</xsl:text>");
    for taken in [false, true] {
        let mut program = compile_stylesheet(&shell).unwrap();
        program.root_template.as_mut().unwrap().body = vec![Instruction::If {
            test: BooleanExpression::Constant(taken),
            body: lowered.clone(),
            location: document.location(element).clone(),
        }];
        let result = execute_compiled_stylesheet_for_test(&program, &shell, "edition-probe");
        if taken {
            let failure = result.unwrap_err();
            assert!(failure.contains("XTDE1450"), "{failure}");
        } else {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn processor_edition_reference_known_but_unimplemented_is_not_unknown() {
    let (document, element) = fixture(r#"<xsl:sequence select="'modern'"/>"#);
    let failure = lower(&document, element, Edition::Modern, DeclaredVersion::Two).unwrap_err();
    assert_eq!(failure.code, "FXXP1003");
    assert_eq!(failure.category, CompileCategory::Unsupported);
    let legacy = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap();
    assert!(matches!(
        &legacy[0],
        Instruction::Xslt10DeferredFailure {
            code: "XTDE1450",
            ..
        }
    ));
}

#[test]
fn processor_edition_reference_same_document_different_recognition() {
    let (document, element) = fixture(
        "<xsl:foo>ignored<xsl:fallback><xsl:text>first</xsl:text></xsl:fallback>ignored<xsl:fallback><xsl:text>second</xsl:text></xsl:fallback></xsl:foo>",
    );
    let body = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap();
    assert_eq!(text(&body), "firstsecond");
    assert_eq!(
        lower(&document, element, Edition::Modern, DeclaredVersion::Two)
            .unwrap_err()
            .category,
        CompileCategory::Invalid
    );
    assert_eq!(
        body,
        compile_xslt10_forward_compatible_instruction(&document, element, "foo").unwrap()
    );
    // Experiment does not mutate the document or enable production fallback.
    assert!(compile_sequence(&document, document.parent(element).unwrap()).is_err());
}

#[test]
fn processor_edition_reference_version_matrix() {
    let (document, element) = fixture("<xsl:foo/>");
    for version in [
        DeclaredVersion::One,
        DeclaredVersion::Two,
        DeclaredVersion::Three,
        DeclaredVersion::Four,
    ] {
        for edition in [Edition::Legacy, Edition::Modern] {
            let expected = match edition {
                Edition::Legacy => !matches!(version, DeclaredVersion::One),
                Edition::Modern => false,
            };
            let result = lower(&document, element, edition, version);
            assert_eq!(result.is_ok(), expected);
            if let Ok(instructions) = result {
                assert!(
                    matches!(&instructions[0], Instruction::Xslt10DeferredFailure { location, .. } if location == document.location(element))
                );
            }
        }
    }
}

#[test]
fn processor_edition_reference_known_newer_instruction_is_not_legacy_known() {
    let (document, element) = fixture(
        r#"<xsl:sequence select="'modern'"><xsl:fallback><xsl:text>legacy</xsl:text></xsl:fallback></xsl:sequence>"#,
    );
    assert_eq!(
        text(&lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap()),
        "legacy"
    );
    // Delegate modern recognition to the real compiler, even if its current
    // capability rejects the fallback child. Never convert refusal to fallback.
    let expected = compile_xslt_instruction(&document, element, "sequence", &mut Vec::new());
    let actual = lower(&document, element, Edition::Modern, DeclaredVersion::Two);
    match (expected, actual) {
        (Ok(expected), Ok(actual)) => assert_eq!(actual, vec![expected]),
        (Err(expected), Err(actual)) => {
            assert_eq!(actual.code, expected.code);
            assert_eq!(actual.location, expected.location);
        }
        _ => panic!("modern lowering diverged"),
    }
}

#[test]
fn processor_edition_reference_propagates_nested_fallback_edition() {
    let (document, element) = fixture(
        "<xsl:foo><xsl:fallback><xsl:foo><xsl:fallback><xsl:text>nested</xsl:text></xsl:fallback></xsl:foo></xsl:fallback></xsl:foo>",
    );
    assert_eq!(
        text(&lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap()),
        "nested"
    );
    assert_eq!(
        text(&lower(&document, element, Edition::Modern, DeclaredVersion::Four).unwrap()),
        "nested"
    );
    assert!(lower(&document, element, Edition::Modern, DeclaredVersion::Two).is_err());
}

#[test]
fn processor_edition_reference_nested_known_instruction_preserves_compiler_failure() {
    let (document, element) = fixture(
        r#"<xsl:foo><xsl:fallback><xsl:sequence select="'modern'"><xsl:fallback><xsl:text>legacy</xsl:text></xsl:fallback></xsl:sequence></xsl:fallback></xsl:foo>"#,
    );
    assert_eq!(
        text(&lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap()),
        "legacy"
    );
    let fallback = document.children(element)[0];
    let sequence = document.children(fallback)[0];
    let expected =
        compile_xslt_instruction(&document, sequence, "sequence", &mut Vec::new()).unwrap_err();
    let actual = lower(&document, element, Edition::Modern, DeclaredVersion::Four).unwrap_err();
    assert_eq!(actual.code, expected.code);
    assert_eq!(actual.category, expected.category);
    assert_eq!(actual.location, expected.location);
}

#[test]
fn processor_edition_reference_nested_scope_and_depth_are_explicitly_bounded() {
    for body in [
        r#"<xsl:foo><xsl:fallback><xsl:if test="position() = 1"><xsl:text>hidden</xsl:text></xsl:if></xsl:fallback></xsl:foo>"#.to_owned(),
        r#"<xsl:foo><xsl:fallback><xsl:foo xsl:version="1.0"/></xsl:fallback></xsl:foo>"#.to_owned(),
        format!("{}<xsl:text>deep</xsl:text>{}", "<xsl:foo><xsl:fallback>".repeat(17), "</xsl:fallback></xsl:foo>".repeat(17)),
    ] {
        let (document, element) = fixture(&body);
        let error = lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap_err();
        assert_eq!(error.code, "FXAR0031");
        assert_eq!(error.category, CompileCategory::Unsupported);
    }
}

#[test]
fn processor_edition_reference_nested_calls_have_no_ambient_edition_state() {
    let (document, element) = fixture(
        "<xsl:foo><xsl:fallback><xsl:foo><xsl:fallback><xsl:text>yes</xsl:text></xsl:fallback></xsl:foo></xsl:fallback></xsl:foo>",
    );
    std::thread::scope(|scope| {
        let legacy = scope.spawn(|| {
            for _ in 0..16 {
                assert_eq!(
                    text(
                        &lower(&document, element, Edition::Legacy, DeclaredVersion::Two).unwrap()
                    ),
                    "yes"
                );
            }
        });
        let modern = scope.spawn(|| {
            for _ in 0..16 {
                assert!(lower(&document, element, Edition::Modern, DeclaredVersion::Two).is_err());
            }
        });
        legacy.join().unwrap();
        modern.join().unwrap();
    });
}
