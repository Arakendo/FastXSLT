use super::super::named_tests::{document, sheet};
use super::*;
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;

#[test]
fn selected_constant_parameters_preserve_shared_import_and_include_precedence() {
    use crate::xslt::golden_semantics_experiment::GlobalBindingDefault;
    let dependency = sheet(
        "2.0",
        r#"<xsl:param name="p" select="string('imported')"/>"#,
    );
    let lower = || {
        compile_selected_legacy_import_module_for_test(
            &dependency,
            vec![],
            &mut InvocationControl::unbounded(),
            false,
        )
        .unwrap()
    };
    let principal = document("principal", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="dependency"/><xsl:param name="p" select="'principal'"/></xsl:stylesheet>"#);
    let program = compile_selected_legacy_import_module_for_test(
        &principal,
        vec![lower()],
        &mut InvocationControl::unbounded(),
        true,
    )
    .unwrap();
    assert_eq!(program.global_bindings.len(), 1);
    assert!(
        matches!(&program.global_bindings[0].default, GlobalBindingDefault::Text(value) if value == "principal")
    );
    let principal = document("principal", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="dependency"/><xsl:param name="p" select="'principal'"/></xsl:stylesheet>"#);
    let Err(StrictPreprocessingFailure::Compile(error)) =
        compile_selected_legacy_include_module_for_test(
            &principal,
            vec![lower()],
            &mut InvocationControl::unbounded(),
        )
    else {
        panic!("same-precedence duplicate must fail")
    };
    assert_eq!(error.category, compiler::CompileCategory::Invalid);
    assert_eq!(error.code, "FXST0029");
    assert!(error.detail.contains("duplicate global"), "{error:?}");
    let principal = document("principal", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="dependency"/></xsl:stylesheet>"#);
    let program = compile_selected_legacy_include_module_for_test(
        &principal,
        vec![lower()],
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    drop(dependency);
    assert!(
        matches!(&program.global_bindings[0].default, GlobalBindingDefault::Text(value) if value == "imported")
    );
}

#[test]
fn selected_mixed_composition_preserves_included_import_precedence_and_qualification() {
    let principal = document("principal", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="low"/><xsl:include href="included"/><xsl:output method="text"/></xsl:stylesheet>"#);
    let included = document("included", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:import href="high"/></xsl:stylesheet>"#);
    let low = sheet("1.0", r#"<xsl:template match="r">low</xsl:template>"#);
    let high = sheet(
        "2.0",
        r#"<xsl:template match="r"><xsl:sequence><xsl:fallback>high</xsl:fallback></xsl:sequence></xsl:template>"#,
    );
    let make_included = || {
        let mut control = InvocationControl::unbounded();
        let high =
            compile_selected_legacy_import_module_for_test(&high, vec![], &mut control, false)
                .unwrap();
        compile_selected_legacy_import_module_for_test(&included, vec![high], &mut control, false)
            .unwrap()
    };
    let low_selected = compile_selected_legacy_import_module_for_test(
        &low,
        vec![],
        &mut InvocationControl::unbounded(),
        false,
    )
    .unwrap();
    let program = compile_selected_legacy_mixed_module_for_test(
        &principal,
        low_selected,
        make_included(),
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(
        execute(
            &program,
            &document("source", b"<r/>"),
            "mixed",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "high"
    );
    assert!(
        matches!(compile_selected_legacy_mixed_module_for_test(&principal, compiler::compile_stylesheet(&low).unwrap(), make_included(), &mut InvocationControl::unbounded()), Err(StrictPreprocessingFailure::Compile(error)) if error.code == "FXST1058")
    );
}

#[test]
fn selected_branching_imports_delay_name_validation_until_complete_graph() {
    let leaf = sheet(
        "1.0",
        r#"<xsl:template match="r"><xsl:call-template name="from-principal"/></xsl:template>"#,
    );
    let empty = sheet("1.0", "");
    let branch = document("branch", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="a"/><xsl:import href="b"/></xsl:stylesheet>"#);
    let principal = document("principal", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="branch"/><xsl:import href="empty"/><xsl:output method="text"/><xsl:template name="from-principal">resolved</xsl:template></xsl:stylesheet>"#);
    let build = |complete| {
        let mut control = InvocationControl::unbounded();
        let a = compile_selected_legacy_import_module_for_test(&leaf, vec![], &mut control, false)
            .unwrap();
        let b = compile_selected_legacy_import_module_for_test(&empty, vec![], &mut control, false)
            .unwrap();
        compile_selected_legacy_import_module_for_test(&branch, vec![a, b], &mut control, complete)
    };
    assert!(
        matches!(build(true), Err(StrictPreprocessingFailure::Compile(error)) if error.code == "FXST0014"),
        "unresolved name must fail at the graph boundary"
    );
    let program = compile_selected_legacy_import_module_for_test(
        &principal,
        vec![
            build(false).unwrap(),
            compile_selected_legacy_import_module_for_test(
                &empty,
                vec![],
                &mut InvocationControl::unbounded(),
                false,
            )
            .unwrap(),
        ],
        &mut InvocationControl::unbounded(),
        true,
    )
    .unwrap();
    assert_eq!(
        execute(
            &program,
            &document("source", b"<r/>"),
            "graph",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "resolved"
    );
    // The baseline wrapper still finalizes, rather than accepting unresolved names.
    assert!(
        compiler::stylesheet_module_compiler::merge_imported_programs_at(
            &branch,
            compiler::document_element(&branch).unwrap(),
            lower_module(
                &branch,
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
                &meaningful_children(&branch, compiler::document_element(&branch).unwrap())
            )
            .unwrap(),
            vec![
                lower_module(
                    &leaf,
                    Edition::Legacy,
                    &mut InvocationControl::unbounded(),
                    &[]
                )
                .unwrap(),
                lower_module(
                    &empty,
                    Edition::Legacy,
                    &mut InvocationControl::unbounded(),
                    &[]
                )
                .unwrap()
            ],
        )
        .is_err()
    );
}

#[test]
fn selected_positioned_includes_preserve_order_and_reject_incompatible_artifacts() {
    let imported = sheet("1.0", r#"<xsl:template match="r">included</xsl:template>"#);
    let source = document("source", b"<r/>");
    for (body, expected) in [
        (
            r#"<xsl:template match="r">principal</xsl:template><xsl:include href="i"/>"#,
            "included",
        ),
        (
            r#"<xsl:include href="i"/><xsl:template match="r">principal</xsl:template>"#,
            "principal",
        ),
    ] {
        let principal = sheet("1.0", body);
        let included = lower_module(
            &imported,
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
            &[],
        )
        .unwrap();
        let program = compile_selected_legacy_include_module_for_test(
            &principal,
            vec![included],
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(
            execute(
                &program,
                &source,
                "include",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
        let incompatible = compiler::compile_stylesheet(&imported).unwrap();
        assert!(matches!(
            compile_selected_legacy_include_module_for_test(
                &principal,
                vec![incompatible],
                &mut InvocationControl::unbounded()
            ), Err(StrictPreprocessingFailure::Compile(error)) if error.code == "FXST1058"
        ));
    }
    let principal = document("principal", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="a"/><xsl:import href="b"/></xsl:stylesheet>"#);
    assert!(matches!(
        compile_selected_legacy_import_module_for_test(
            &principal,
            vec![
                compiler::compile_stylesheet(&imported).unwrap(),
                compiler::compile_stylesheet(&imported).unwrap()
            ],
            &mut InvocationControl::unbounded(),
            true
        ), Err(StrictPreprocessingFailure::Compile(error)) if error.code == "FXST1058"
    ));
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_apply_imports_without_imports_replays_impincl18() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let folder = root.join("Xalan_Conformance_Tests/impincl");
    let style = document(
        "impincl18.xsl",
        &std::fs::read(folder.join("impincl18.xsl")).unwrap(),
    );
    let source = document(
        "source",
        &std::fs::read(folder.join("impincl18.xml")).unwrap(),
    );
    let program = compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
    let expected =
        std::fs::read(root.join("Xalan_Conformance_Tests/REF_OUT/impincl/impincl18.out")).unwrap();
    let actual = execute(
        &program,
        &source,
        "impincl18",
        &mut InvocationControl::unbounded(),
    )
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
        "{actual}"
    );
}

#[test]
fn selected_apply_imports_preserves_named_call_rule_and_work_controls() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let principal = document("p", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="i"/><xsl:output method="text"/><xsl:template match="r"><xsl:text>high:</xsl:text><xsl:call-template name="delegate"/></xsl:template><xsl:template name="delegate"><xsl:apply-imports/></xsl:template></xsl:stylesheet>"#);
    let imported = document("i", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="r">low</xsl:template></xsl:stylesheet>"#);
    let selected = compile_supplied(
        &principal,
        &imported,
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let baseline =
        compiler::stylesheet_module_compiler::compile_stylesheet_with_single_imported_program_at(
            &principal,
            compiler::document_element(&principal).unwrap(),
            compiler::compile_stylesheet(&imported).unwrap(),
        )
        .unwrap();
    let source = document("source", b"<r/>");
    let mut actual = InvocationControl::unbounded();
    let mut reference = InvocationControl::unbounded();
    assert_eq!(
        execute(&selected, &source, "selected", &mut actual).unwrap(),
        "high:low"
    );
    assert_eq!(
        execute(&baseline, &source, "baseline", &mut reference).unwrap(),
        "high:low"
    );
    for domain in [
        WorkDomain::XPathNodeVisit,
        WorkDomain::XPathOperation,
        WorkDomain::XsltInstruction,
        WorkDomain::XsltTemplateCandidate,
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::SerializedByte,
    ] {
        assert_eq!(
            actual.consumed(domain),
            reference.consumed(domain),
            "{domain:?}"
        );
    }
    let mut limits = WorkLimits::unbounded();
    limits.xslt_instructions = actual.consumed(WorkDomain::XsltInstruction);
    assert_eq!(
        execute(
            &selected,
            &source,
            "exact-budget",
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap(),
        "high:low"
    );
    limits.xslt_instructions -= 1;
    assert!(
        execute(
            &selected,
            &source,
            "short-budget",
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap_err()
        .contains("FXCT0002")
    );
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XsltInstruction, 0);
    assert!(
        execute(&selected, &source, "cancel", &mut cancelled)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        execute(
            &selected,
            &source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "high:low"
    );
    for instruction in [
        "<xsl:apply-imports mode=\"m\"/>",
        "<xsl:apply-imports><xsl:with-param name=\"p\" select=\"1\"/></xsl:apply-imports>",
        "<xsl:apply-imports>text</xsl:apply-imports>",
    ] {
        assert!(
            compile(
                &sheet(
                    "1.0",
                    &format!("<xsl:template match=\"/\">{instruction}</xsl:template>")
                ),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_sibling_import_include_replays_impincl01() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let folder = root.join("Xalan_Conformance_Tests/impincl");
    let documents = ["impincl01.xsl", "i.xsl", "j.xsl"]
        .map(|name| document(name, &std::fs::read(folder.join(name)).unwrap()));
    let program = compile_supplied_siblings(
        &documents[0],
        &documents[1],
        &documents[2],
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document(
        "source",
        &std::fs::read(folder.join("impincl01.xml")).unwrap(),
    );
    let expected =
        std::fs::read(root.join("Xalan_Conformance_Tests/REF_OUT/impincl/impincl01.out")).unwrap();
    let actual = execute(
        &program,
        &source,
        "impincl01",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(
        crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
            &actual,
            &expected,
            false,
            None,
            Some("html")
        ),
        Ok(true),
        "{actual}"
    );
}

#[test]
fn selected_self_node_value_uses_shared_focus_conversion() {
    for (pattern, expression) in ["/", "r", "text()"]
        .into_iter()
        .flat_map(|pattern| ["self::node()", "."].map(|expression| (pattern, expression)))
    {
        let declarations = format!(
            "<xsl:template match=\"{pattern}\"><xsl:value-of select=\"{expression}\"/></xsl:template>"
        );
        let style = sheet("1.0", &declarations);
        let selected =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        let reference = compiler::compile_stylesheet(&style).unwrap();
        for xml in ["<r/>", "<r>A<c>B</c>C</r>"] {
            let source = document("source", xml.as_bytes());
            assert_eq!(
                execute(
                    &selected,
                    &source,
                    "selected",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                execute(
                    &reference,
                    &source,
                    "reference",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "{pattern}: {expression}: {xml}"
            );
        }
    }
}

#[test]
fn selected_siblings_resolve_names_and_preserve_import_precedence() {
    let principal = document("p", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="i"/><xsl:include href="j"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="entry"/></xsl:template></xsl:stylesheet>"#);
    let imported = document("i", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template name="entry"><xsl:call-template name="show"/></xsl:template><xsl:template name="show">wrong</xsl:template></xsl:stylesheet>"#);
    let included = document("j", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template name="show"><xsl:sequence><xsl:fallback>included</xsl:fallback></xsl:sequence></xsl:template></xsl:stylesheet>"#);
    let program = compile_supplied_siblings(
        &principal,
        &imported,
        &included,
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert!(
        compile_supplied_siblings(
            &principal,
            &included,
            &imported,
            Edition::Legacy,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
    drop(principal);
    drop(imported);
    drop(included);
    assert_eq!(
        execute(
            &program,
            &document("source", b"<doc/>"),
            "siblings",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "included"
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_include_import_replays_impincl02() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let folder = root.join("Xalan_Conformance_Tests/impincl");
    let documents = ["impincl02.xsl", "f.xsl", "g.xsl"]
        .map(|name| document(name, &std::fs::read(folder.join(name)).unwrap()));
    let program = compile_supplied_chain(
        &documents.iter().collect::<Vec<_>>(),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document(
        "source",
        &std::fs::read(folder.join("impincl02.xml")).unwrap(),
    );
    let expected =
        std::fs::read(root.join("Xalan_Conformance_Tests/REF_OUT/impincl/impincl02.out")).unwrap();
    let actual = execute(
        &program,
        &source,
        "impincl02",
        &mut InvocationControl::unbounded(),
    )
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
        "{actual}"
    );
}

#[test]
fn selected_constant_attributes_do_not_admit_avts_or_xslt_controls() {
    for attribute in [
        "a=\"{1+1}\"",
        "a=\"{{literal}}\"",
        "xsl:use-attribute-sets=\"n\"",
        "xmlns:p=\"urn:p\" p:a=\"constant\"",
    ] {
        assert!(
            compile(
                &sheet(
                    "1.0",
                    &format!("<xsl:template match=\"/\"><out {attribute}/></xsl:template>")
                ),
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err(),
            "{attribute}"
        );
    }
    let style = sheet(
        "1.0",
        "<xsl:template match=\"/\"><out a=\"Ω&amp;&quot;\" empty=\"\"/></xsl:template>",
    );
    assert!(compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).is_ok());
}

#[test]
fn selected_include_retains_equal_precedence_and_duplicate_name_errors() {
    let included = document("i", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="foo"><xsl:sequence><xsl:fallback>included</xsl:fallback></xsl:sequence></xsl:template><xsl:template name="n">named</xsl:template></xsl:stylesheet>"#);
    let principal = |extra: &str| {
        document("p", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="i"/><xsl:output method="text"/><xsl:template match="foo">principal</xsl:template>{extra}</xsl:stylesheet>"#).as_bytes())
    };
    let program = compile_supplied_chain(
        &[&principal(""), &included],
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(
        execute(
            &program,
            &document("source", b"<foo/>"),
            "include",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "principal"
    );
    assert!(
        compile_supplied_chain(
            &[
                &principal("<xsl:template name=\"n\">duplicate</xsl:template>"),
                &included
            ],
            Edition::Legacy,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}

#[test]
fn selected_nested_import_bounds_cycles_and_cancellation() {
    let p = document("p", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="m"/></xsl:stylesheet>"#);
    let m = document("m", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="p"/></xsl:stylesheet>"#);
    for chain in [
        vec![&p, &m, &p],
        vec![&p, &m],
        vec![&p],
        vec![&p, &m, &p, &m, &p],
    ] {
        assert!(
            compile_supplied_chain(&chain, Edition::Legacy, &mut InvocationControl::unbounded())
                .is_err()
        );
    }
    let leaf = document("m", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#);
    let mut cancelled = InvocationControl::unbounded()
        .cancelling_on_charge(crate::execution_control_experiment::WorkDomain::XdmNode, 0);
    assert!(matches!(
        compile_supplied_chain(&[&p, &leaf], Edition::Legacy, &mut cancelled),
        Err(StrictPreprocessingFailure::Control(_))
    ));
    assert!(
        compile_supplied_chain(
            &[&p, &leaf],
            Edition::Legacy,
            &mut InvocationControl::unbounded()
        )
        .is_ok()
    );
}

#[test]
fn selected_nested_import_defers_leaf_reads_until_complete_graph() {
    let parent = document("p", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="m"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="entry"/></xsl:template><xsl:template name="answer">parent</xsl:template></xsl:stylesheet>"#);
    let middle = document("m", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:import href="l"/><xsl:template name="show"><xsl:sequence><xsl:fallback>middle</xsl:fallback></xsl:sequence></xsl:template></xsl:stylesheet>"#);
    let leaf = document("l", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template name="entry"><xsl:call-template name="show"/><xsl:call-template name="answer"/></xsl:template><xsl:template name="show">wrong</xsl:template></xsl:stylesheet>"#);
    assert!(
        compile_supplied_chain(
            &[&middle, &leaf],
            Edition::Legacy,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
    let program = compile_supplied_chain(
        &[&parent, &middle, &leaf],
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    drop(parent);
    drop(middle);
    drop(leaf);
    assert_eq!(
        execute(
            &program,
            &document("source", b"<doc/>"),
            "nested",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "middleparent"
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_nested_import_replays_namedtemplate19() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let folder = root.join("Xalan_Conformance_Tests/namedtemplate");
    let documents = ["namedtemplate19.xsl", "ntimpc.xsl", "ntimpa.xsl"]
        .map(|name| document(name, &std::fs::read(folder.join(name)).unwrap()));
    let program = compile_supplied_chain(
        &documents.iter().collect::<Vec<_>>(),
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document(
        "source",
        &std::fs::read(folder.join("namedtemplate19.xml")).unwrap(),
    );
    let expected = std::fs::read(
        root.join("Xalan_Conformance_Tests/REF_OUT/namedtemplate/namedtemplate19.out"),
    )
    .unwrap();
    let actual = execute(
        &program,
        &source,
        "namedtemplate19",
        &mut InvocationControl::unbounded(),
    )
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
        "{actual}"
    );
}

#[test]
fn selected_single_import_links_before_resolving_names_and_keeps_precedence() {
    for override_body in ["", "<xsl:template name=\"show\">principal</xsl:template>"] {
        let principal = document("principal", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="dependency"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="show"/></xsl:template>{override_body}</xsl:stylesheet>"#).as_bytes());
        let imported = document("dependency", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template name="show"><xsl:sequence><xsl:fallback>imported</xsl:fallback></xsl:sequence></xsl:template></xsl:stylesheet>"#);
        let program = compile_supplied(
            &principal,
            &imported,
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        drop(principal);
        drop(imported);
        assert_eq!(
            execute(
                &program,
                &document("source", b"<doc/>"),
                "import",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            if override_body.is_empty() {
                "imported"
            } else {
                "principal"
            }
        );
    }
}

#[test]
fn selected_single_import_resolves_imported_reads_only_after_linking() {
    let imported = document("dependency", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template name="show"><xsl:call-template name="principal"/></xsl:template></xsl:stylesheet>"#);
    for definition in [
        "",
        "<xsl:template name=\"principal\">resolved</xsl:template>",
    ] {
        let principal = document("principal", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="dependency"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="show"/></xsl:template>{definition}</xsl:stylesheet>"#).as_bytes());
        let result = compile_supplied(
            &principal,
            &imported,
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        );
        if definition.is_empty() {
            assert!(
                result.is_err(),
                "unresolved target must fail the linked program"
            );
        } else {
            assert_eq!(
                execute(
                    &result.unwrap(),
                    &document("source", b"<doc/>"),
                    "linked",
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "resolved"
            );
        }
    }
}

#[test]
fn selected_single_import_rejects_unprovided_and_nested_edges() {
    let leaf = sheet("1.0", "<xsl:template name=\"show\">leaf</xsl:template>");
    for declarations in [
        "<xsl:import href=\"missing\"/>",
        "<xsl:output method=\"text\"/><xsl:import href=\"urn:selected-named\"/>",
        "<xsl:import href=\"urn:selected-named\"/><xsl:import href=\"other\"/>",
    ] {
        let principal = document("principal", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">{declarations}</xsl:stylesheet>"#).as_bytes());
        assert!(
            compile_supplied(
                &principal,
                &leaf,
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }
}

#[test]
fn selected_multiple_imports_resolve_sibling_reads_and_later_precedence() {
    let principal = document("principal", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:import href="a"/><xsl:import href="b"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="entry"/></xsl:template></xsl:stylesheet>"#);
    let a = document("a", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template name="entry"><xsl:call-template name="show"/><xsl:call-template name="only-b"/></xsl:template><xsl:template name="show">wrong</xsl:template></xsl:stylesheet>"#);
    let b = document("b", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template name="show"><xsl:sequence><xsl:fallback>later</xsl:fallback></xsl:sequence></xsl:template><xsl:template name="only-b">!</xsl:template></xsl:stylesheet>"#);
    let mut cancelled = InvocationControl::unbounded()
        .cancelling_on_charge(crate::execution_control_experiment::WorkDomain::XdmNode, 0);
    assert!(matches!(
        compile_supplied_leaves(&principal, &[&a, &b], Edition::Legacy, &mut cancelled),
        Err(StrictPreprocessingFailure::Control(_))
    ));
    let program = compile_supplied_leaves(
        &principal,
        &[&a, &b],
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    drop(principal);
    drop(a);
    drop(b);
    assert_eq!(
        execute(
            &program,
            &document("source", b"<doc/>"),
            "siblings",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "later!"
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_multiple_imports_replay_namedtemplate17() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let folder = root.join("Xalan_Conformance_Tests/namedtemplate");
    let principal = document(
        "namedtemplate17.xsl",
        &std::fs::read(folder.join("namedtemplate17.xsl")).unwrap(),
    );
    let a = document(
        "ntimpa.xsl",
        &std::fs::read(folder.join("ntimpa.xsl")).unwrap(),
    );
    let b = document(
        "ntimpb.xsl",
        &std::fs::read(folder.join("ntimpb.xsl")).unwrap(),
    );
    let program = compile_supplied_leaves(
        &principal,
        &[&a, &b],
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document(
        "source",
        &std::fs::read(folder.join("namedtemplate17.xml")).unwrap(),
    );
    let expected = std::fs::read(
        root.join("Xalan_Conformance_Tests/REF_OUT/namedtemplate/namedtemplate17.out"),
    )
    .unwrap();
    let actual = execute(
        &program,
        &source,
        "namedtemplate17",
        &mut InvocationControl::unbounded(),
    )
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
        "{actual}"
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_single_import_replays_namedtemplate18() {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let folder = root.join("Xalan_Conformance_Tests/namedtemplate");
    let principal = document(
        "namedtemplate18.xsl",
        &std::fs::read(folder.join("namedtemplate18.xsl")).unwrap(),
    );
    let imported = document(
        "ntimpa.xsl",
        &std::fs::read(folder.join("ntimpa.xsl")).unwrap(),
    );
    let program = compile_supplied(
        &principal,
        &imported,
        Edition::Legacy,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let source = document(
        "source",
        &std::fs::read(folder.join("namedtemplate18.xml")).unwrap(),
    );
    let expected = std::fs::read(
        root.join("Xalan_Conformance_Tests/REF_OUT/namedtemplate/namedtemplate18.out"),
    )
    .unwrap();
    let actual = execute(
        &program,
        &source,
        "namedtemplate18",
        &mut InvocationControl::unbounded(),
    )
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
        "{actual}"
    );
}
