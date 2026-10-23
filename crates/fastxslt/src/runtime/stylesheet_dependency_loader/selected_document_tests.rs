use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::resources::ResourceSnapshot;
use crate::runtime::golden_runtime_experiment::{execute_program_with_resources, serialize_xml};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use std::collections::HashSet;

const DATA: &str = "https://example.invalid/shared/data.xml";

fn resources(version: &str, payload: Option<&str>) -> ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(5, 16_384, 65_536));
    builder.admit(PRINCIPAL, br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="../shared/child.xsl"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="child"/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    builder.admit(CHILD, format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:variable name="data" select="document('data.xml')"/><xsl:template name="child"><xsl:value-of select="$data/r/value"/></xsl:template></xsl:stylesheet>"#).into_bytes()).unwrap();
    builder
        .admit(
            "https://example.invalid/styles/data.xml",
            b"<r><value>wrong-base</value></r>".to_vec(),
        )
        .unwrap();
    if let Some(payload) = payload {
        builder.admit(DATA, payload.as_bytes().to_vec()).unwrap();
    }
    builder.seal()
}

fn run(
    program: &StylesheetProgram,
    snapshot: &ResourceSnapshot,
    denied: Option<&HashSet<String>>,
    control: &mut InvocationControl,
) -> Result<String, String> {
    let source = Document::from_parsed(
        parse_document(
            "urn:principal-source",
            b"<r><value>wrong-source</value></r>",
            ParseLimits {
                max_events: 100,
                max_depth: 10,
            },
        )
        .unwrap(),
    )
    .unwrap();
    let result = execute_program_with_resources(
        program,
        &source,
        snapshot,
        denied,
        "selected-document",
        control,
    )
    .map_err(|error| format!("{error:?}"))?;
    serialize_xml(
        &result,
        &program.output,
        "selected-document",
        65_536,
        control,
    )
    .map_err(|error| format!("{error:?}"))
}

#[test]
fn selected_policy_document_globals_preserve_module_base_and_snapshot_isolation() {
    let reference_snapshot = resources(
        "1.0",
        Some("<r><value>first</value><value>ignored</value></r>"),
    );
    let graph = load(&reference_snapshot, PRINCIPAL, vec![], 2).unwrap();
    let child = &graph.dependencies[0];
    let reference =
        crate::compile::golden_stylesheet_experiment::compile_stylesheet_with_single_include(
            &graph.document,
            &child.document,
            child.root,
        )
        .unwrap();
    let mut reference_control = InvocationControl::unbounded();
    assert_eq!(
        run(
            &reference,
            &reference_snapshot,
            None,
            &mut reference_control
        )
        .unwrap(),
        "first"
    );
    drop(graph);
    for version in ["1.0", "2.0", "3.0"] {
        let first = resources(
            version,
            Some("<r><value>first</value><value>ignored</value></r>"),
        );
        let graph = load(&first, PRINCIPAL, vec![], 2).unwrap();
        let program = compile(&graph);
        drop(graph);
        let mut actual = InvocationControl::unbounded();
        assert_eq!(run(&program, &first, None, &mut actual).unwrap(), "first");
        for domain in [
            WorkDomain::XmlEvent,
            WorkDomain::XPathNodeVisit,
            WorkDomain::XPathOperation,
            WorkDomain::XsltInstruction,
            WorkDomain::ResultTextByte,
        ] {
            assert_eq!(actual.consumed(domain), reference_control.consumed(domain));
        }
        drop(first);
        let second = resources(version, Some("<r><value>second</value></r>"));
        assert_eq!(
            run(&program, &second, None, &mut InvocationControl::unbounded()).unwrap(),
            "second"
        );
        let denied = HashSet::from([DATA.to_owned()]);
        let failure = run(
            &program,
            &second,
            Some(&denied),
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert!(failure.contains(DATA), "{failure}");
        assert!(
            failure.contains("Denied") || failure.contains("denied"),
            "{failure}"
        );
        let absent = resources(version, None);
        let failure =
            run(&program, &absent, None, &mut InvocationControl::unbounded()).unwrap_err();
        assert!(failure.contains(DATA), "{failure}");
        assert!(
            failure.contains("Missing") || failure.contains("missing"),
            "{failure}"
        );
        assert_eq!(
            run(&program, &second, None, &mut InvocationControl::unbounded()).unwrap(),
            "second"
        );
    }
}

#[test]
fn selected_policy_document_globals_preserve_parse_controls_and_recovery() {
    let snapshot = resources("2.0", Some("<r><value>result</value></r>"));
    let program = compile(&load(&snapshot, PRINCIPAL, vec![], 2).unwrap());
    let mut observed = InvocationControl::unbounded();
    assert_eq!(
        run(&program, &snapshot, None, &mut observed).unwrap(),
        "result"
    );
    let required = observed.consumed(WorkDomain::XmlEvent);
    assert!(required > 0);
    let mut limits = WorkLimits::unbounded();
    limits.xml_events = required;
    assert_eq!(
        run(
            &program,
            &snapshot,
            None,
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap(),
        "result"
    );
    limits.xml_events -= 1;
    assert!(
        run(
            &program,
            &snapshot,
            None,
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap_err()
        .contains("FXCT0002")
    );
    let mut cancel = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XmlEvent, 1);
    assert!(
        run(&program, &snapshot, None, &mut cancel)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        run(
            &program,
            &snapshot,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "result"
    );
}
