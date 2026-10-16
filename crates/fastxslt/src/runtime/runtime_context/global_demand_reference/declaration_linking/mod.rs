//! Declaration and module-link evidence through the shared compiler and executor.

use super::*;

mod production_cost;
mod schedule_cost;
mod static_admission;

fn declaration_program(
    body: &str,
) -> Result<
    crate::compile::golden_stylesheet_experiment::global_constructor_reference::CompiledReference,
    crate::compile::golden_stylesheet_experiment::CompileFailure,
> {
    let xml = format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:a="urn:binding" xmlns:b="urn:binding" version="1.0">{body}</xsl:stylesheet>"#
    );
    crate::compile::golden_stylesheet_experiment::global_constructor_reference::compile(&document(
        "urn:declaration:stylesheet",
        &xml,
    ))
}

fn include_program(
    principal: &str,
    included: &str,
    strict: bool,
) -> Result<
    crate::compile::golden_stylesheet_experiment::global_constructor_reference::CompiledReference,
    crate::compile::golden_stylesheet_experiment::CompileFailure,
> {
    dependency_program(principal, included, strict, false)
}

fn dependency_program(
    principal: &str,
    included: &str,
    strict: bool,
    import: bool,
) -> Result<
    crate::compile::golden_stylesheet_experiment::global_constructor_reference::CompiledReference,
    crate::compile::golden_stylesheet_experiment::CompileFailure,
> {
    use crate::compile::golden_stylesheet_experiment::{
        global_constructor_reference,
        stylesheet_preprocessing::{
            StylesheetCompilationContext, StylesheetPreprocessingProfile, strict_stylesheet_view,
        },
    };
    use crate::xdm::owned_tree_experiment::NodeKind;
    let module = |identity, body| {
        let parsed = document(
            identity,
            &format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:a="urn:binding" xmlns:b="urn:binding" version="1.0">{body}</xsl:stylesheet>"#
            ),
        );
        if strict {
            let root = parsed
                .children(parsed.document_node())
                .iter()
                .copied()
                .find(|node| parsed.kind(*node) == NodeKind::Element)
                .unwrap();
            Arc::new(
                strict_stylesheet_view(&parsed, root, &mut InvocationControl::unbounded()).unwrap(),
            )
        } else {
            parsed
        }
    };
    let principal = module("urn:include:principal", principal);
    let included = module("urn:include:dependency", included);
    let profile = if strict {
        StylesheetPreprocessingProfile::StrictXslt10
    } else {
        StylesheetPreprocessingProfile::Modern
    };
    let compile = if import {
        global_constructor_reference::compile_with_single_import
    } else {
        global_constructor_reference::compile_with_single_include
    };
    compile(
        &principal,
        &included,
        StylesheetCompilationContext::for_reference(profile, false),
    )
}

#[test]
fn demand_import_reference_keeps_winning_constructor_and_template() {
    let principal = r#"<xsl:import href="sealed.xsl"/>
      <xsl:variable name="b:first"><xsl:call-template name="helper"/></xsl:variable>
      <xsl:template name="helper">A<!-- split -->   <!-- split -->B</xsl:template>
      <xsl:template match="/"><xsl:value-of select="$a:first"/><xsl:value-of select="$second"/></xsl:template>"#;
    let imported = r#"<xsl:variable name="a:first"><xsl:message terminate="yes">loser</xsl:message></xsl:variable>
      <xsl:variable name="second"><xsl:value-of select="$b:first"/></xsl:variable>
      <xsl:template name="helper"><xsl:message terminate="yes">old helper</xsl:message></xsl:template>"#;
    for strict in [false, true] {
        for clones in [false, true] {
            let compiled = dependency_program(principal, imported, strict, true).unwrap();
            assert_eq!(compiled.bodies.len(), 2);
            for binding in &compiled.program.global_bindings {
                assert_eq!(
                    binding.import_precedence,
                    if binding.name == "second" { -1 } else { 0 }
                );
                assert_eq!(
                    binding.declaration_reference.resource,
                    if binding.name == "second" {
                        "urn:include:dependency"
                    } else {
                        "urn:include:principal"
                    }
                );
            }
            let mut control = InvocationControl::unbounded();
            let driver = Driver::with_constructor_bodies(
                Arc::new(compiled.program),
                document("urn:source", "<r/>"),
                &BTreeMap::new(),
                clones,
                compiled.bodies,
                &mut control,
            )
            .unwrap();
            assert_eq!(
                driver.run(&mut control).unwrap(),
                if strict { "ABAB" } else { "A   BA   B" }
            );
            assert!(control.messages().is_empty());
            assert!(driver.evaluations.borrow().iter().all(|count| *count == 1));
        }
    }
}

#[test]
fn demand_import_reference_preserves_imported_failure_and_cycle_cleanup() {
    let principal = r#"<xsl:import href="sealed.xsl"/>
      <xsl:variable name="first"><xsl:copy-of select="$second"/></xsl:variable>
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>"#;
    for (body, code) in [
        (r#"<xsl:copy-of select="$first"/>"#, "FXAR0030"),
        (
            r#"<xsl:message terminate="yes">imported failure</xsl:message>"#,
            "XTMM9000",
        ),
    ] {
        let imported = format!(r#"<xsl:variable name="second">{body}</xsl:variable>"#);
        let compiled = dependency_program(principal, &imported, false, true).unwrap();
        let mut control = InvocationControl::unbounded();
        let driver = Driver::with_constructor_bodies(
            Arc::new(compiled.program),
            document("urn:source", "<r/>"),
            &BTreeMap::new(),
            false,
            compiled.bodies,
            &mut control,
        )
        .unwrap();
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, code);
        if code == "XTMM9000" {
            assert_eq!(
                error.location.as_ref().unwrap().resource,
                "urn:include:dependency"
            );
        }
        assert_eq!(
            driver.read_string("first", &mut control).unwrap_err(),
            error
        );
        assert!(driver.resolver.stack.borrow().is_empty());
        assert!(driver.evaluations.borrow().iter().all(|count| *count == 1));
    }
}

#[test]
fn demand_import_reference_retains_order_and_graph_guards() {
    let late = dependency_program(
        r#"<xsl:variable name="x">value</xsl:variable><xsl:import href="sealed.xsl"/>"#,
        "",
        false,
        true,
    )
    .err()
    .expect("late import must fail");
    assert_eq!(late.code, "XTSE0200");
    assert_eq!(late.location.resource, "urn:include:principal");
    for principal in [
        r#"<xsl:include href="sealed.xsl"/>"#,
        r#"<xsl:import href="a"/><xsl:import href="b"/>"#,
    ] {
        assert_eq!(
            dependency_program(principal, "", false, true)
                .err()
                .unwrap()
                .code,
            "FXAR0030"
        );
    }
    assert_eq!(
        dependency_program(
            r#"<xsl:import href="sealed.xsl"/>"#,
            r#"<xsl:import href="nested.xsl"/>"#,
            false,
            true
        )
        .err()
        .unwrap()
        .code,
        "FXAR0030"
    );
}

#[test]
fn demand_include_reference_links_constructor_calls_after_module_merge() {
    let principal = r#"<xsl:include href="sealed.xsl"/>
      <xsl:variable name="second">A<!-- split -->   <!-- split -->B</xsl:variable>
      <xsl:template name="helper"><xsl:value-of select="$second"/></xsl:template>
      <xsl:template match="/"><xsl:value-of select="$b:first"/><xsl:value-of select="$a:first"/></xsl:template>"#;
    let included =
        r#"<xsl:variable name="a:first"><xsl:call-template name="helper"/></xsl:variable>"#;
    for strict in [false, true] {
        for clones in [false, true] {
            let compiled = include_program(principal, included, strict).unwrap();
            let first = compiled
                .program
                .global_bindings
                .iter()
                .find(|binding| binding.name != "second")
                .unwrap();
            assert_eq!(
                first.declaration_reference.resource,
                "urn:include:dependency"
            );
            let first_name = first.name.clone();
            let mut control = InvocationControl::unbounded();
            let driver = Driver::with_constructor_bodies(
                Arc::new(compiled.program),
                document("urn:source", "<r/>"),
                &BTreeMap::new(),
                clones,
                compiled.bodies,
                &mut control,
            )
            .unwrap();
            assert_eq!(
                driver.run(&mut control).unwrap(),
                if strict { "ABAB" } else { "A   BA   B" }
            );
            assert_eq!(driver.evaluations(&first_name), 1);
            assert_eq!(driver.evaluations("second"), 1);
        }
    }
}

#[test]
fn demand_include_reference_preserves_duplicate_and_missing_call_diagnostics() {
    let duplicate = include_program(
        r#"<xsl:include href="sealed.xsl"/><xsl:variable name="a:x">principal</xsl:variable>"#,
        r#"<xsl:variable name="b:x">included</xsl:variable>"#,
        false,
    )
    .err()
    .expect("same-precedence duplicates must fail");
    assert_eq!(duplicate.code, "FXST0029");
    assert_eq!(duplicate.location.resource, "urn:include:principal");
    let missing = include_program(
        r#"<xsl:include href="sealed.xsl"/>"#,
        r#"<xsl:variable name="x"><xsl:call-template name="absent"/></xsl:variable>"#,
        false,
    )
    .err()
    .expect("included constructor must be linked");
    assert_eq!(missing.location.resource, "urn:include:dependency");
    assert!(missing.detail.contains("absent"));
    for principal in [
        r#"<xsl:import href="sealed.xsl"/>"#,
        r#"<xsl:include href="a"/><xsl:include href="b"/>"#,
    ] {
        assert_eq!(
            include_program(principal, "", false).err().unwrap().code,
            "FXAR0030"
        );
    }
    assert_eq!(
        include_program(
            r#"<xsl:include href="sealed.xsl"/>"#,
            r#"<xsl:include href="nested.xsl"/>"#,
            false
        )
        .err()
        .unwrap()
        .code,
        "FXAR0030"
    );
}

#[test]
fn demand_include_reference_cycle_and_failure_keep_module_ownership() {
    let principal = r#"<xsl:include href="sealed.xsl"/>
      <xsl:variable name="first"><xsl:copy-of select="$second"/></xsl:variable>
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>"#;
    for (included, code) in [
        (
            r#"<xsl:variable name="second"><xsl:copy-of select="$first"/></xsl:variable>"#,
            "FXAR0030",
        ),
        (
            r#"<xsl:variable name="second"><xsl:message terminate="yes">included failure</xsl:message></xsl:variable>"#,
            "XTMM9000",
        ),
    ] {
        let compiled = include_program(principal, included, false).unwrap();
        let mut control = InvocationControl::unbounded();
        let driver = Driver::with_constructor_bodies(
            Arc::new(compiled.program),
            document("urn:source", "<r/>"),
            &BTreeMap::new(),
            false,
            compiled.bodies,
            &mut control,
        )
        .unwrap();
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, code);
        if code == "XTMM9000" {
            assert_eq!(
                error.location.as_ref().unwrap().resource,
                "urn:include:dependency"
            );
            assert_eq!(control.messages(), &["included failure"]);
        }
        assert_eq!(
            driver.read_string("first", &mut control).unwrap_err(),
            error
        );
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
#[cfg(feature = "workbench")]
#[ignore = "manual unchanged OASIS variable40 declaration-reference replay"]
fn demand_declaration_reference_replays_local_variable40() {
    use crate::compile::golden_stylesheet_experiment::{
        global_constructor_reference, stylesheet_preprocessing::strict_stylesheet_view,
    };
    use crate::runtime::golden_runtime_experiment::{SemanticResult, serialize_xml};
    use crate::xdm::owned_tree_experiment::NodeKind;
    let root = std::path::PathBuf::from(
        std::env::var_os("FASTXSLT_OASIS_XSLT10_ROOT").expect("explicit local corpus root"),
    );
    let suite = root.join("Xalan_Conformance_Tests");
    let stylesheet_xml = std::fs::read_to_string(suite.join("variable/variable40.xsl")).unwrap();
    let source_xml = std::fs::read_to_string(suite.join("variable/variable40.xml")).unwrap();
    let expected = std::fs::read_to_string(suite.join("REF_OUT/variable/variable40.out")).unwrap();
    let stylesheet = document("urn:oasis:variable40:stylesheet", &stylesheet_xml);
    let source = document("urn:oasis:variable40:source", &source_xml);
    for strict in [false, true] {
        let mut control = InvocationControl::unbounded();
        let element = stylesheet
            .children(stylesheet.document_node())
            .iter()
            .copied()
            .find(|node| stylesheet.kind(*node) == NodeKind::Element)
            .unwrap();
        let effective = if strict {
            Arc::new(strict_stylesheet_view(&stylesheet, element, &mut control).unwrap())
        } else {
            Arc::clone(&stylesheet)
        };
        let production = compile_stylesheet(&effective).unwrap();
        let production_actual =
            crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test(
                &production,
                &source,
                "variable40-production",
            )
            .unwrap();
        assert_eq!(
            crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                &production_actual,
                expected.as_bytes(),
                false,
                None,
                None,
            ),
            Ok(true),
        );
        let compiled = global_constructor_reference::compile(&effective).unwrap();
        static_admission::assert_outside_subset(&compiled);
        let driver = Driver::with_constructor_bodies(
            Arc::new(compiled.program),
            Arc::clone(&source),
            &BTreeMap::new(),
            false,
            compiled.bodies,
            &mut control,
        )
        .unwrap();
        // Initial built-in document rule: dispatch its children in the default mode.
        // This bootstraps the unchanged program, not an injected stylesheet rule.
        let initial = [Instruction::ApplyTemplates {
            select: None,
            sorts: Vec::new(),
            mode: None,
            arguments: Vec::new(),
            location: source.location(source.document_node()).clone(),
        }];
        let children = driver
            .execute_nodes_with_frame(
                &initial,
                &RuntimeVariables::default(),
                &source,
                &mut control,
            )
            .unwrap();
        let actual = serialize_xml(
            &SemanticResult { children },
            &driver.program.output,
            "demand-shared-reference",
            64 * 1024,
            &mut control,
        )
        .unwrap();
        assert_eq!(
            crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
                &actual,
                expected.as_bytes(),
                false,
                None,
                None,
            ),
            Ok(true)
        );
        assert_eq!(driver.evaluations("TreeFrag"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
        println!(
            "AR-0030 variable40 strict={strict}: production and reference agree with unchanged OASIS XML comparator"
        );
    }
}

#[test]
fn demand_declaration_reference_links_forward_calls_and_qualified_bindings() {
    let compiled = declaration_program(r#"
      <xsl:variable name="a:first"><xsl:apply-templates select="//b"/></xsl:variable>
      <xsl:variable name="second"><xsl:call-template name="later"/></xsl:variable>
      <xsl:template match="/"><xsl:value-of select="$b:first"/><xsl:value-of select="$a:first"/></xsl:template>
      <xsl:template match="b"><xsl:value-of select="$second"/></xsl:template>
      <xsl:template name="later">linked</xsl:template>"#).unwrap();
    assert_eq!(compiled.program.global_bindings.len(), 2);
    for binding in &compiled.program.global_bindings {
        assert_eq!(
            binding.declaration_reference.resource,
            "urn:declaration:stylesheet"
        );
        assert!(compiled.bodies.contains_key(&binding.name));
    }
    let mut control = InvocationControl::unbounded();
    let driver = Driver::with_constructor_bodies(
        Arc::new(compiled.program),
        document("urn:declaration:source", "<r><b/></r>"),
        &BTreeMap::new(),
        false,
        compiled.bodies,
        &mut control,
    )
    .unwrap();
    assert_eq!(driver.run(&mut control).unwrap(), "linkedlinked");
    assert!(driver.evaluations.borrow().iter().all(|count| *count == 1));
}

#[test]
fn demand_declaration_reference_supplied_parameter_skips_its_body() {
    let body = r#"
      <xsl:param name="first"><xsl:message terminate="yes">default</xsl:message></xsl:param>
      <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>"#;
    for supplied in [false, true] {
        let compiled = declaration_program(body).unwrap();
        let mut control = InvocationControl::unbounded();
        let params = if supplied {
            BTreeMap::from([("first".to_owned(), "host".to_owned())])
        } else {
            BTreeMap::new()
        };
        let driver = Driver::with_constructor_bodies(
            Arc::new(compiled.program),
            document("urn:declaration:source", "<r/>"),
            &params,
            false,
            compiled.bodies,
            &mut control,
        )
        .unwrap();
        let result = driver.run(&mut control);
        if supplied {
            assert_eq!(result.unwrap(), "host");
            assert_eq!(driver.evaluations("first"), 0);
            assert!(control.messages().is_empty());
        } else {
            assert_eq!(result.unwrap_err().code, "XTMM9000");
            assert_eq!(driver.evaluations("first"), 1);
            assert_eq!(control.messages(), &["default"]);
        }
    }
}

#[test]
fn demand_declaration_reference_preserves_static_errors_and_scope_guards() {
    let invalid = [
        (
            r#"<xsl:variable name="a:x">A</xsl:variable><xsl:variable name="b:x">B</xsl:variable>"#,
            "FXST0022",
        ),
        (
            r#"<xsl:variable name="missing:x">A</xsl:variable>"#,
            "FXST0023",
        ),
        (r#"<xsl:param name="x" required="maybe"/>"#, "FXST0025"),
    ];
    for (body, code) in invalid {
        let error = declaration_program(body).err().expect("must fail");
        assert_eq!(error.code, code);
        assert_eq!(error.location.resource, "urn:declaration:stylesheet");
    }
    for body in [
        r#"<xsl:variable name="x" select="'x'"/>"#,
        r#"<xsl:variable name="x"/>"#,
        r#"<xsl:param name="x">   </xsl:param>"#,
        r#"<xsl:variable name="x" as="element()"><box/></xsl:variable>"#,
        r#"<xsl:include href="absent.xsl"/>"#,
        r#"<xsl:namespace-alias stylesheet-prefix="a" result-prefix="b"/>"#,
        r#"<xsl:attribute-set name="x"/>"#,
    ] {
        assert_eq!(
            declaration_program(body)
                .err()
                .expect("explicit guard")
                .code,
            "FXAR0030"
        );
    }
    let missing = declaration_program(r#"<xsl:variable name="x"><xsl:if test="true()"><xsl:call-template name="absent"/></xsl:if></xsl:variable>"#).err().expect("body calls must be linked");
    assert_eq!(
        missing.category,
        crate::compile::golden_stylesheet_experiment::CompileCategory::Invalid
    );
    assert!(missing.detail.contains("absent"));
    assert_eq!(missing.location.resource, "urn:declaration:stylesheet");
}
