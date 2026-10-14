//! Private graph-profile integration controls; no public selector is selected.

use super::*;
use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document;

const PRINCIPAL: &str = "https://example.invalid/main.xsl";

fn compile(modules: &[(&str, &str)], strict: bool) -> Result<StylesheetProgram, ExecutionFailure> {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(8, 8192, 32768));
    for (identity, xml) in modules {
        builder.admit(*identity, xml.as_bytes().to_vec()).unwrap();
    }
    let limits = StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS);
    compile_resource_with_denied_and_limits(
        &builder.seal(),
        PRINCIPAL,
        std::iter::empty(),
        if strict {
            limits.with_strict_preprocessing()
        } else {
            limits
        },
    )
}

fn execute(modules: &[(&str, &str)], strict: bool) -> String {
    let program = compile(modules, strict).unwrap();
    let source = Document::from_parsed(
        parse_document("urn:source", b"<r/>", super::super::XML_LIMITS).unwrap(),
    )
    .unwrap();
    execute_compiled_stylesheet_for_test(&program, &source, "preprocessing").unwrap()
}

#[test]
fn principal_profile_is_not_principal_or_local_version() {
    for version in ["1.0", "3.0"] {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out xsl:version="1.0">A<!--ignored--> <z/></out></xsl:template></xsl:stylesheet>"#
        );
        assert_eq!(execute(&[(PRINCIPAL, &xml)], false), "<out>A <z></z></out>");
        assert_eq!(execute(&[(PRINCIPAL, &xml)], true), "<out>A<z></z></out>");
    }
}

#[test]
fn transitive_include_and_import_inherit_one_profile() {
    let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:import href="import.xsl"/><xsl:include href="include.xsl"/><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:call-template name="imported"/><xsl:call-template name="included"/></out></xsl:template></xsl:stylesheet>"#;
    let imported = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template name="imported">I<!--ignored--> <z/></xsl:template></xsl:stylesheet>"#;
    let included = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="nested.xsl"/></xsl:stylesheet>"#;
    let nested = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template name="included">N<?ignored yes?> <z/></xsl:template></xsl:stylesheet>"#;
    let modules = [
        (PRINCIPAL, principal),
        ("https://example.invalid/import.xsl", imported),
        ("https://example.invalid/include.xsl", included),
        ("https://example.invalid/nested.xsl", nested),
    ];
    assert_eq!(execute(&modules, false), "<out>I <z></z>N <z></z></out>");
    assert_eq!(execute(&modules, true), "<out>I<z></z>N<z></z></out>");
}

#[test]
fn global_and_local_temporary_constructors_share_text_selection() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:variable name="g"><g>A<!--ignored--> <z/></g></xsl:variable><xsl:template match="/"><xsl:variable name="l"><l>B<?ignored yes?> <z/></l></xsl:variable><out><xsl:copy-of select="$g"/><xsl:copy-of select="$l"/></out></xsl:template></xsl:stylesheet>"#;
    assert_eq!(
        execute(&[(PRINCIPAL, xml)], false),
        "<out><g>A <z></z></g><l>B <z></z></l></out>"
    );
    assert_eq!(
        execute(&[(PRINCIPAL, xml)], true),
        "<out><g>A<z></z></g><l>B<z></z></l></out>"
    );
}

#[test]
fn text_only_globals_use_the_same_edition_as_sequence_constructors() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:variable name="g"> <!--ignored-->A<!--ignored--> </xsl:variable><xsl:variable name="p" xml:space="preserve"> <!--ignored-->P<!--ignored--> </xsl:variable><xsl:template match="/"><out><xsl:value-of select="$g"/>|<xsl:value-of select="$p"/></out></xsl:template></xsl:stylesheet>"#;
    assert_eq!(execute(&[(PRINCIPAL, xml)], false), "<out> A | P </out>");
    assert_eq!(execute(&[(PRINCIPAL, xml)], true), "<out>A| P </out>");
}

#[test]
fn comment_pi_and_local_text_specializations_share_preprocessing() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="l"> <!--ignored-->L<!--ignored--> </xsl:variable><out><xsl:comment>C<!--ignored--> </xsl:comment><xsl:processing-instruction name="p">P<!--ignored--> </xsl:processing-instruction><xsl:value-of select="$l"/></out></xsl:template></xsl:stylesheet>"#;
    assert_eq!(
        execute(&[(PRINCIPAL, xml)], false),
        "<out><!--C --><?p P ?> L </out>"
    );
    assert_eq!(
        execute(&[(PRINCIPAL, xml)], true),
        "<out><!--C--><?p P?>L</out>"
    );
}

#[test]
fn space_preservation_reset_and_explicit_text_survive_strict_graph() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:variable name="g"><g xml:space="preserve"> <!--ignored--> <z xml:space="default"> </z></g></xsl:variable><xsl:template match="/"><out><xsl:copy-of select="$g"/><xsl:text> </xsl:text></out></xsl:template></xsl:stylesheet>"#;
    for strict in [false, true] {
        assert_eq!(
            execute(&[(PRINCIPAL, xml)], strict),
            r#"<out><g xml:space="preserve">  <z xml:space="default"></z></g> </out>"#
        );
    }
}

#[test]
fn literal_result_stylesheet_uses_the_compilation_profile() {
    let xml = r#"<out xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xsl:version="1.0">A<!--ignored--> <z/></out>"#;
    assert!(execute(&[(PRINCIPAL, xml)], false).ends_with("<out>A <z></z></out>"));
    assert!(execute(&[(PRINCIPAL, xml)], true).ends_with("<out>A<z></z></out>"));
}

#[test]
fn embedded_module_preserves_ancestry_but_does_not_choose_an_edition() {
    let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:include href="embedded.xml#sheet"/><xsl:output omit-xml-declaration="yes"/></xsl:stylesheet>"#;
    for inherited_space in ["preserve", "default"] {
        let embedded = format!(
            r#"<container xml:space="{inherited_space}"><xsl:stylesheet xml:id="sheet" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><out>A<!--ignored--> <z/></out></xsl:template></xsl:stylesheet></container>"#
        );
        let modules = [
            (PRINCIPAL, principal),
            ("https://example.invalid/embedded.xml", &embedded),
        ];
        assert_eq!(execute(&modules, false), "<out>A <z></z></out>");
        let expected = if inherited_space == "preserve" {
            "<out>A <z></z></out>"
        } else {
            "<out>A<z></z></out>"
        };
        assert_eq!(execute(&modules, true), expected);
    }
}

fn control_graph() -> ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    builder.admit(PRINCIPAL, br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:include href="child.xsl"/><xsl:output omit-xml-declaration="yes"/></xsl:stylesheet>"#.to_vec()).unwrap();
    builder.admit("https://example.invalid/child.xsl", br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><out>A<!--ignored--> <z/></out></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    builder.seal()
}

fn controlled_compile(
    snapshot: &ResourceSnapshot,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, ExecutionFailure> {
    compile_resource_with_preprocessing_control(
        snapshot,
        PRINCIPAL,
        std::iter::empty(),
        StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS)
            .with_strict_preprocessing(),
        control,
    )
}

#[test]
fn graph_preprocessing_budget_is_aggregate_and_exact() {
    let snapshot = control_graph();
    let mut measured = InvocationControl::unbounded();
    controlled_compile(&snapshot, &mut measured).unwrap();
    let total = measured.consumed(WorkDomain::XdmNode);
    assert!(total > 1);
    for (limit, succeeds) in [(total, true), (total - 1, false)] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xdm_nodes: limit,
                ..WorkLimits::unbounded()
            },
        );
        let result = controlled_compile(&snapshot, &mut control);
        if succeeds {
            result.unwrap();
            assert_eq!(control.consumed(WorkDomain::XdmNode), total);
        } else {
            let failure = result.unwrap_err();
            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(failure.category, FailureCategory::Limit);
            assert_eq!(failure.work_domain, Some(WorkDomain::XdmNode));
            assert_eq!(failure.request_id, None);
            assert_eq!(
                failure.location.unwrap().resource,
                "https://example.invalid/child.xsl"
            );
        }
    }
    // A failed partial graph publishes no program and does not poison resources.
    controlled_compile(&snapshot, &mut InvocationControl::unbounded()).unwrap();
}

#[test]
fn graph_preprocessing_cancellation_preserves_phase_provenance() {
    let snapshot = control_graph();
    let token = CancellationToken::new();
    token.cancel();
    let mut control = InvocationControl::new(token, WorkLimits::unbounded());
    let failure = controlled_compile(&snapshot, &mut control).unwrap_err();
    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.category, FailureCategory::Cancelled);
    assert_eq!(failure.work_domain, Some(WorkDomain::XdmNode));
    assert_eq!(failure.request_id, None);
    assert_eq!(failure.location.unwrap().resource, PRINCIPAL);
    assert_eq!(control.consumed(WorkDomain::XdmNode), 0);

    // The same control crosses module boundaries rather than resetting per module.
    let mut measured = InvocationControl::unbounded();
    controlled_compile(&snapshot, &mut measured).unwrap();
    // Sweep charge calls, not work units: empty child slices still poll control.
    // Each zero-unit poll is paired with a positive node charge in this view.
    let mut saw_dependency = false;
    let mut reached_completion = false;
    for accepted in 0..=measured.consumed(WorkDomain::XdmNode) * 2 {
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, accepted);
        match controlled_compile(&snapshot, &mut control) {
            Ok(_) => {
                reached_completion = true;
                break;
            }
            Err(failure) => {
                assert_eq!(failure.code, "FXCT0001");
                assert_eq!(failure.category, FailureCategory::Cancelled);
                assert_eq!(failure.request_id, None);
                saw_dependency |=
                    failure.location.unwrap().resource == "https://example.invalid/child.xsl";
            }
        }
    }
    assert!(saw_dependency);
    assert!(reached_completion);
    controlled_compile(&snapshot, &mut InvocationControl::unbounded()).unwrap();
}

#[test]
fn preserved_text_disqualifies_lossy_local_constructor_shortcuts() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="v" xml:space="preserve"> <xsl:value-of select="'V'"/> </xsl:variable><xsl:variable name="f" xml:space="preserve"> <xsl:for-each select="/"><xsl:value-of select="."/> </xsl:for-each> </xsl:variable><out><xsl:value-of select="$v"/>|<xsl:value-of select="$f"/></out></xsl:template></xsl:stylesheet>"#;
    for strict in [false, true] {
        assert_eq!(execute(&[(PRINCIPAL, xml)], strict), "<out> V |   </out>");
    }
}

#[cfg(feature = "workbench")]
#[test]
fn strict_workbench_probe_uses_its_supplied_preprocessing_budget() {
    use crate::runtime::workbench_experiment::{
        ExperimentalEngine, PreprocessingProbeOptions, WorkbenchExternalSubsetLimits,
        WorkbenchLimits, WorkbenchStylesheetResources,
    };
    let stylesheet = br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:template match="/"><out>A<!--ignored--> <z/></out></xsl:template></xsl:stylesheet>"#;
    let limits = WorkbenchLimits {
        max_xdm_nodes: 8,
        ..WorkbenchLimits::default()
    };
    // Modern production construction is not charged by the test-only strict pass.
    ExperimentalEngine::new(
        "urn:source",
        b"<r/>".to_vec(),
        PRINCIPAL,
        stylesheet.to_vec(),
        limits,
    )
    .unwrap();
    let failure = ExperimentalEngine::new_with_preprocessing_probe(
        "urn:source",
        b"<r/>".to_vec(),
        PRINCIPAL,
        stylesheet.to_vec(),
        WorkbenchStylesheetResources::default(),
        limits,
        PreprocessingProbeOptions {
            strict: true,
            subsets: WorkbenchExternalSubsetLimits {
                declarations: InternalSubsetLimits {
                    declarations: 1,
                    nesting_depth: 1,
                    references: 1,
                    replacement_bytes: 1,
                },
                source: None,
                stylesheet: None,
            },
        },
    )
    .err()
    .expect("the supplied strict preprocessing ceiling must reject creation");
    assert_eq!(failure.code, "FXCT0002");
    assert_eq!(failure.category, "limit");
    assert_eq!(failure.request_id, None);
    assert_eq!(failure.location.unwrap().resource, PRINCIPAL);
}

#[test]
fn global_dynamic_text_parts_preserve_constructor_whitespace() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:variable name="g" xml:space="preserve"> <xsl:value-of select="'G'"/> </xsl:variable><xsl:template match="/"><out><xsl:value-of select="$g"/></out></xsl:template></xsl:stylesheet>"#;
    for strict in [false, true] {
        assert_eq!(execute(&[(PRINCIPAL, xml)], strict), "<out> G </out>");
    }
}

#[test]
fn named_and_matched_parameter_defaults_consume_selected_text_only() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:param name="m"> <!--ignored-->M<!--ignored--> </xsl:param><out><xsl:value-of select="$m"/>|<xsl:call-template name="n"/></out></xsl:template><xsl:template name="n"><xsl:param name="p"> <?ignored yes?>P<!--ignored--> </xsl:param><xsl:param name="e"> </xsl:param><xsl:param name="k" xml:space="preserve"> </xsl:param><xsl:value-of select="$p"/>|<xsl:value-of select="$e"/>|<xsl:value-of select="$k"/></xsl:template></xsl:stylesheet>"#;
    assert_eq!(execute(&[(PRINCIPAL, xml)], false), "<out> M | P || </out>");
    assert_eq!(execute(&[(PRINCIPAL, xml)], true), "<out>M|P|| </out>");
}

#[test]
fn typed_range_shortcut_cannot_discard_preserved_constructor_text() {
    for body in [
        r#" <xsl:for-each select="1 to 3"><v><xsl:value-of select="."/></v></xsl:for-each>"#,
        r#"<xsl:for-each select="1 to 3"> <v><xsl:value-of select="."/></v></xsl:for-each>"#,
        r#"<xsl:for-each select="1 to 3"><v> <xsl:value-of select="."/></v></xsl:for-each>"#,
    ] {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:xs="http://www.w3.org/2001/XMLSchema" version="3.0"><xsl:template match="/"><xsl:variable name="v" as="xs:integer *" xml:space="preserve">{body}</xsl:variable><out/></xsl:template></xsl:stylesheet>"#
        );
        for strict in [false, true] {
            let failure = compile(&[(PRINCIPAL, &xml)], strict).unwrap_err();
            assert_eq!(failure.code, "FXST1017");
            assert_eq!(failure.category, FailureCategory::Unsupported);
            assert_eq!(failure.request_id, None);
            let location = failure.location.unwrap();
            assert_eq!(location.resource, PRINCIPAL);
            assert!(location.span.end <= xml.len());
        }
    }
}

#[test]
fn typed_range_shortcut_still_accepts_strip_eligible_formatting() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:xs="http://www.w3.org/2001/XMLSchema" version="3.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="v" as="xs:integer *"> <!--ignored--><xsl:for-each select="1 to 3"> <?ignored yes?><v> <xsl:value-of select="."/> </v> </xsl:for-each> </xsl:variable><out>ok</out></xsl:template></xsl:stylesheet>"#;
    for strict in [false, true] {
        assert_eq!(
            execute(&[(PRINCIPAL, xml)], strict),
            r#"<out xmlns:xs="http://www.w3.org/2001/XMLSchema">ok</out>"#
        );
    }
}

#[test]
fn modern_reference_preprocesses_included_parameter_boundaries_before_lowering() {
    let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:include href="child.xsl"/><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:call-template name="n"/></xsl:template></xsl:stylesheet>"#;
    let included = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0" xml:space="preserve"><xsl:template name="n"> <!--ignored--> <xsl:param name="p"> P</xsl:param><out><xsl:value-of select="$p"/></out></xsl:template></xsl:stylesheet>"#;
    let modules = [
        (PRINCIPAL, principal),
        ("https://example.invalid/child.xsl", included),
    ];
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    for (identity, xml) in modules {
        builder.admit(identity, xml.as_bytes().to_vec()).unwrap();
    }
    let snapshot = builder.seal();
    let limits = StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS)
        .with_modern_preprocessing_reference();
    let mut measured = InvocationControl::unbounded();
    let program = compile_resource_with_preprocessing_control(
        &snapshot,
        PRINCIPAL,
        std::iter::empty(),
        limits,
        &mut measured,
    )
    .unwrap();
    let source = Document::from_parsed(
        parse_document("urn:source", b"<r/>", super::super::XML_LIMITS).unwrap(),
    )
    .unwrap();
    assert_eq!(
        execute_compiled_stylesheet_for_test(&program, &source, "modern-reference").unwrap(),
        "<out> P</out>"
    );
    assert_eq!(execute(&modules, false), "  <out> P</out>");
    let required = measured.consumed(WorkDomain::XdmNode);
    for limit in [required, required - 1] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xdm_nodes: limit,
                ..WorkLimits::unbounded()
            },
        );
        let result = compile_resource_with_preprocessing_control(
            &snapshot,
            PRINCIPAL,
            std::iter::empty(),
            limits,
            &mut control,
        );
        if limit == required {
            result.unwrap();
        } else {
            let failure = result.unwrap_err();
            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(
                failure.location.unwrap().resource,
                "https://example.invalid/child.xsl"
            );
        }
    }
}

#[test]
fn modern_reference_empty_content_failure_is_module_owned_and_controlled() {
    let principal = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:include href="child.xsl"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#;
    let child = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0" xml:space="preserve"><xsl:output omit-xml-declaration="yes"> <!--ignored--> </xsl:output></xsl:stylesheet>"#;
    let child_id = "https://example.invalid/child.xsl";
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    builder
        .admit(PRINCIPAL, principal.as_bytes().to_vec())
        .unwrap();
    builder.admit(child_id, child.as_bytes().to_vec()).unwrap();
    let snapshot = builder.seal();
    let limits = StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS)
        .with_modern_preprocessing_reference();
    let run = |control: &mut InvocationControl| {
        compile_resource_with_preprocessing_control(
            &snapshot,
            PRINCIPAL,
            std::iter::empty(),
            limits,
            control,
        )
    };
    let mut measured = InvocationControl::unbounded();
    let failure = run(&mut measured).unwrap_err();
    assert_eq!(failure.code, "XTSE0260");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id, None);
    let original = Document::from_parsed(
        parse_document(child_id, child.as_bytes(), super::super::XML_LIMITS).unwrap(),
    )
    .unwrap();
    let root = original.children(original.document_node())[0];
    let output = original.children(root)[0];
    assert_eq!(failure.location.as_ref(), Some(original.location(output)));
    let required = measured.consumed(WorkDomain::XdmNode);
    for (limit, code) in [(required, "XTSE0260"), (required - 1, "FXCT0002")] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xdm_nodes: limit,
                ..WorkLimits::unbounded()
            },
        );
        assert_eq!(run(&mut control).unwrap_err().code, code);
    }
    // Fault injection counts charge calls, not the aggregate charged units.
    let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 3);
    assert_eq!(run(&mut cancelled).unwrap_err().code, "FXCT0001");
    // Failed preprocessing publishes no mutated graph or program. The original
    // snapshot remains reusable under the separate unchanged reference lane.
    let program = compile_resource_with_denied_and_limits(
        &snapshot,
        PRINCIPAL,
        std::iter::empty(),
        StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS),
    )
    .unwrap();
    let source = Document::from_parsed(
        parse_document("urn:source", b"<r/>", super::super::XML_LIMITS).unwrap(),
    )
    .unwrap();
    assert_eq!(
        execute_compiled_stylesheet_for_test(&program, &source, "recovery").unwrap(),
        "<out></out>"
    );
    assert_eq!(
        run(&mut InvocationControl::unbounded())
            .unwrap_err()
            .location,
        failure.location
    );
}

#[test]
fn template_argument_content_consumes_only_profile_selected_fragments() {
    let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:call-template name="n"><xsl:with-param name="e"> <!--ignored--> </xsl:with-param><xsl:with-param name="p" xml:space="preserve"> <?ignored yes?> </xsl:with-param><xsl:with-param name="m"> <!--ignored-->M<!--ignored--> </xsl:with-param></xsl:call-template></out></xsl:template><xsl:template name="n"><xsl:param name="e"/><xsl:param name="p"/><xsl:param name="m"/><xsl:value-of select="$e"/>|<xsl:value-of select="$p"/>|<xsl:value-of select="$m"/></xsl:template></xsl:stylesheet>"#;
    assert_eq!(execute(&[(PRINCIPAL, xml)], false), "<out>|  | M </out>");
    assert_eq!(execute(&[(PRINCIPAL, xml)], true), "<out>|  |M</out>");
    let dynamic = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:call-template name="n"><xsl:with-param name="p" xml:space="preserve"> <xsl:value-of select="'X'"/> </xsl:with-param></xsl:call-template></out></xsl:template><xsl:template name="n"><xsl:param name="p"/><xsl:value-of select="$p"/></xsl:template></xsl:stylesheet>"#;
    for strict in [false, true] {
        assert_eq!(execute(&[(PRINCIPAL, dynamic)], strict), "<out> X </out>");
    }
}

#[test]
fn completed_preprocessing_stamps_survive_clone_and_separate_reference_stages() {
    use crate::xslt::golden_semantics_experiment::StylesheetPreprocessingStamp;
    let snapshot = control_graph();
    let limits = StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS);
    let run = |limits| {
        compile_resource_with_denied_and_limits(&snapshot, PRINCIPAL, std::iter::empty(), limits)
            .unwrap()
    };
    let baseline = run(limits);
    let strict = run(limits.with_strict_preprocessing());
    let modern = run(limits.with_modern_preprocessing_reference());
    assert_eq!(
        baseline.preprocessing,
        StylesheetPreprocessingStamp::ModernBaseline
    );
    assert_eq!(
        strict.preprocessing,
        StylesheetPreprocessingStamp::StrictXslt10Reference
    );
    assert_eq!(
        modern.preprocessing,
        StylesheetPreprocessingStamp::ModernReference
    );
    assert_eq!(strict.clone().preprocessing, strict.preprocessing);
    assert_eq!(modern.clone().preprocessing, modern.preprocessing);
    assert_eq!(baseline.declared_version, strict.declared_version);
    assert_eq!(baseline.root_template, modern.root_template);
    assert_ne!(baseline, modern);
    // Selection is graph-owned: choosing strict clears the modern candidate.
    let selected = run(limits
        .with_modern_preprocessing_reference()
        .with_strict_preprocessing());
    assert_eq!(
        selected.preprocessing,
        StylesheetPreprocessingStamp::StrictXslt10Reference
    );
}

#[test]
fn retained_compiled_module_reuse_rejects_mismatched_preprocessing_stamps() {
    use crate::compile::golden_stylesheet_experiment::{
        compile_stylesheet_with_imported_and_included_programs_at,
        compile_stylesheet_with_imported_programs_at, compile_stylesheet_with_included_programs_at,
        compile_stylesheet_with_single_imported_program_at,
        compile_stylesheet_with_single_include_program_at,
    };
    let dependency =
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"/>"#;
    let baseline = compile(&[(PRINCIPAL, dependency)], false).unwrap();
    let strict = compile(&[(PRINCIPAL, dependency)], true).unwrap();
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    builder
        .admit(PRINCIPAL, dependency.as_bytes().to_vec())
        .unwrap();
    let modern = compile_resource_with_denied_and_limits(
        &builder.seal(),
        PRINCIPAL,
        std::iter::empty(),
        StylesheetCompileLimits::new(5, 8, 32768, 8, super::super::XML_LIMITS)
            .with_modern_preprocessing_reference(),
    )
    .unwrap();
    for (body, mixed) in [
        (r#"<xsl:include href="a.xsl"/>"#, false),
        (r#"<xsl:import href="a.xsl"/>"#, false),
        (
            r#"<xsl:include href="a.xsl"/><xsl:include href="b.xsl"/>"#,
            false,
        ),
        (
            r#"<xsl:import href="a.xsl"/><xsl:import href="b.xsl"/>"#,
            false,
        ),
        (
            r#"<xsl:import href="a.xsl"/><xsl:include href="b.xsl"/>"#,
            true,
        ),
    ] {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">{body}</xsl:stylesheet>"#
        );
        let document = Document::from_parsed(
            parse_document(PRINCIPAL, xml.as_bytes(), super::super::XML_LIMITS).unwrap(),
        )
        .unwrap();
        let root = document.children(document.document_node())[0];
        let reuse = |program: StylesheetProgram, position: usize| {
            let (first, second) = if position == 0 {
                (program, baseline.clone())
            } else {
                (baseline.clone(), program)
            };
            if mixed {
                compile_stylesheet_with_imported_and_included_programs_at(
                    &document, root, first, second,
                )
            } else if body.matches("href").count() == 2 {
                if body.starts_with("<xsl:include") {
                    compile_stylesheet_with_included_programs_at(
                        &document,
                        root,
                        vec![first, second],
                    )
                } else {
                    compile_stylesheet_with_imported_programs_at(
                        &document,
                        root,
                        vec![first, second],
                    )
                }
            } else if body.starts_with("<xsl:include") {
                compile_stylesheet_with_single_include_program_at(&document, root, first)
            } else {
                compile_stylesheet_with_single_imported_program_at(&document, root, first)
            }
        };
        reuse(baseline.clone(), 0).unwrap();
        for foreign in [&strict, &modern] {
            for position in 0..body.matches("href").count() {
                let failure = reuse(foreign.clone(), position).unwrap_err();
                assert_eq!(failure.code, "FXST1058");
                assert_eq!(failure.category, CompileCategory::Unsupported);
                assert_eq!(
                    failure.location,
                    *document.location(document.children(root)[position])
                );
            }
        }
    }
}
