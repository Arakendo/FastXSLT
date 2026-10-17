//! Supplied include-chain experiment. No lookup, acquisition or public graph API.
use super::*;
use crate::execution_control_experiment::InvocationControl;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use compiler::stylesheet_preprocessing::{
    StrictPreprocessingFailure, StylesheetCompilationContext, stylesheet_visibility_view,
};

/// Caller supplies principal then dependencies, with exact logical hrefs. This
/// deliberately does not resolve relative identifiers or accept compiled modules
/// from another experiment. At most four documents, with one leaf root template.
fn lower_chain(
    principal: &Document,
    dependencies: &[&Document],
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(principal)?;
    if dependencies.len() > 3 {
        return Err(unsupported(
            "FXAR0031",
            "include-chain depth outside probe",
            principal.location(root),
        )
        .into());
    }
    let documents = std::iter::once(principal)
        .chain(dependencies.iter().copied())
        .collect::<Vec<_>>();
    for (index, document) in documents.iter().enumerate() {
        let identity = &document.location(document.document_node()).resource;
        if documents[..index]
            .iter()
            .any(|previous| &previous.location(previous.document_node()).resource == identity)
        {
            return Err(unsupported(
                "FXAR0031",
                "repeated module identity outside chain probe",
                document.location(document.document_node()),
            )
            .into());
        }
    }
    lower_chain_at(principal, dependencies, edition, control)
}

#[path = "artifact_isolation_tests.rs"]
mod artifact_isolation_tests;

#[path = "qualified_artifact.rs"]
mod qualified_artifact;

fn lower_chain_at(
    document: &Document,
    dependencies: &[&Document],
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let Some((next, remaining)) = dependencies.split_first() else {
        return lower_module(document, edition, control);
    };
    let (root, _) = include_edge(document, &next.location(next.document_node()).resource)?;
    let preprocessing = match edition {
        Edition::Legacy => StylesheetPreprocessingProfile::StrictXslt10,
        Edition::Modern => StylesheetPreprocessingProfile::Modern,
    };
    let view = stylesheet_visibility_view(document, root, preprocessing, control)?;
    let included = lower_chain_at(next, remaining, edition, control)?;
    // No principal instructions can enter the production lowerer: preflight
    // admits only the include declaration. Existing composition owns the merge.
    Ok(compiler::stylesheet_module_compiler::compile_stylesheet_with_included_programs_at_in_context(
        &view, root, vec![included], StylesheetCompilationContext::for_reference(preprocessing, true),
    )?)
}

fn include_edge(
    document: &Document,
    dependency_identity: &str,
) -> Result<(NodeId, NodeId), CompileFailure> {
    dependency_edge(document, dependency_identity, "include")
}

fn dependency_edge(
    document: &Document,
    dependency_identity: &str,
    kind: &str,
) -> Result<(NodeId, NodeId), CompileFailure> {
    let root = compiler::document_element(document)?;
    module_version(document, root)?;
    let children = meaningful_children(document, root);
    let [include] = children.as_slice() else {
        return Err(unsupported(
            "FXAR0031",
            "chain parent requires exactly one dependency declaration",
            document.location(root),
        ));
    };
    if !is_xslt_element(document, *include, kind) {
        return Err(unsupported(
            "FXAR0031",
            "unexpected dependency kind in chain probe",
            document.location(*include),
        ));
    }
    require_probe_attribute_set(document, *include, "href")?;
    compiler::ensure_no_meaningful_children(document, *include, kind)?;
    let href = compiler::required_attribute(document, *include, None, "href")?;
    if href != dependency_identity {
        return Err(unsupported(
            "FXAR0031",
            "supplied module does not match exact logical href",
            document.location(*include),
        ));
    }
    Ok((root, *include))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

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
    fn processor_edition_reference_include_chain_preserves_edition_and_leaf_version() {
        for principal_version in ["1.0", "2.0", "4.0"] {
            let principal = module(
                "urn:principal",
                principal_version,
                r#"<xsl:include href="urn:middle"/>"#,
            );
            let middle = module("urn:middle", "1.0", r#"<xsl:include href="urn:leaf"/>"#);
            for leaf_version in ["1.0", "2.0", "4.0"] {
                let leaf = module(
                    "urn:leaf",
                    leaf_version,
                    r#"<xsl:template match="/"><xsl:foo><xsl:fallback><xsl:text>included</xsl:text></xsl:fallback></xsl:foo></xsl:template>"#,
                );
                for edition in [Edition::Legacy, Edition::Modern] {
                    let expected = match edition {
                        Edition::Legacy => leaf_version != "1.0",
                        Edition::Modern => leaf_version == "4.0",
                    };
                    let result = lower_chain(
                        &principal,
                        &[&middle, &leaf],
                        edition,
                        &mut InvocationControl::unbounded(),
                    );
                    if expected {
                        let program = result.unwrap();
                        assert_eq!(program.declared_version, principal_version);
                        assert_eq!(
                            program.root_template.as_ref().unwrap().location.resource,
                            "urn:leaf"
                        );
                        assert_eq!(
                            execute_compiled_stylesheet_for_test(&program, &principal, "chain")
                                .unwrap(),
                            r#"<?xml version="1.0" encoding="UTF-8"?>included"#
                        );
                    } else {
                        let Err(StrictPreprocessingFailure::Compile(failure)) = result else {
                            panic!("expected leaf compile failure")
                        };
                        assert_eq!(failure.category, CompileCategory::Invalid);
                        assert_eq!(failure.location.resource, "urn:leaf");
                    }
                }
            }
        }
    }

    #[test]
    fn processor_edition_reference_include_chain_matches_leaf_preprocessing() {
        let principal = module("urn:principal", "4.0", r#"<xsl:include href="urn:leaf"/>"#);
        let leaf = module(
            "urn:leaf",
            "1.0",
            r#"<xsl:template match="/">A<!--separator--> </xsl:template>"#,
        );
        for (edition, text) in [(Edition::Legacy, "A"), (Edition::Modern, "A ")] {
            let program = lower_chain(
                &principal,
                &[&leaf],
                edition,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let direct = lower_module(&leaf, edition, &mut InvocationControl::unbounded()).unwrap();
            assert_eq!(program.preprocessing, direct.preprocessing);
            assert_eq!(program.root_template, direct.root_template);
            assert_eq!(
                execute_compiled_stylesheet_for_test(&program, &principal, "chain-text").unwrap(),
                format!(r#"<?xml version="1.0" encoding="UTF-8"?>{text}"#)
            );
        }
    }

    #[test]
    fn processor_edition_reference_include_chain_rejects_unproved_edges() {
        let leaf = module("urn:leaf", "2.0", r#"<xsl:template match="/"/>"#);
        for body in [
            r#"<xsl:import href="urn:leaf"/>"#,
            r#"<xsl:include href="missing.xsl"/>"#,
            r#"<xsl:include href="urn:leaf" extra="ignored"/>"#,
            r#"<xsl:include href="urn:leaf"/><xsl:template match="/"/>"#,
        ] {
            let principal = module("urn:principal", "4.0", body);
            assert!(
                lower_chain(
                    &principal,
                    &[&leaf],
                    Edition::Legacy,
                    &mut InvocationControl::unbounded()
                )
                .is_err()
            );
        }
        let principal = module("urn:principal", "2.0", r#"<xsl:include href="urn:leaf"/>"#);
        assert!(
            lower_chain(
                &principal,
                &[],
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
        assert!(
            lower_chain(
                &principal,
                &[&principal],
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
        assert!(
            lower_chain(
                &principal,
                &[&leaf; 4],
                Edition::Legacy,
                &mut InvocationControl::unbounded()
            )
            .is_err()
        );
    }

    #[test]
    fn processor_edition_reference_include_chain_cancellation_discards_graph() {
        use crate::execution_control_experiment::{
            CancellationToken, ControlFailure, WorkDomain, WorkLimits,
        };
        let principal = module("urn:principal", "2.0", r#"<xsl:include href="urn:leaf"/>"#);
        let leaf = module(
            "urn:leaf",
            "2.0",
            r#"<xsl:template match="/"><xsl:text>ok</xsl:text></xsl:template>"#,
        );
        let root = compiler::document_element(&principal).unwrap();
        let mut counter = InvocationControl::unbounded();
        stylesheet_visibility_view(
            &principal,
            root,
            StylesheetPreprocessingProfile::StrictXslt10,
            &mut counter,
        )
        .unwrap();
        let principal_work = counter.consumed(WorkDomain::XdmNode);
        let mut complete = InvocationControl::unbounded();
        lower_chain(&principal, &[&leaf], Edition::Legacy, &mut complete).unwrap();
        let total = complete.consumed(WorkDomain::XdmNode);
        assert!(total > principal_work);
        let mut cancelled_in_dependency = false;
        for charge in 0..total {
            let mut control =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, charge);
            match lower_chain(&principal, &[&leaf], Edition::Legacy, &mut control) {
                Err(StrictPreprocessingFailure::Control(ControlFailure::Cancelled { .. })) => {
                    cancelled_in_dependency |=
                        control.consumed(WorkDomain::XdmNode) >= principal_work;
                }
                Ok(_) => break,
                other => panic!("unexpected cancellation disposition: {other:?}"),
            }
        }
        assert!(cancelled_in_dependency);
        for limit in [principal_work, total - 1, total] {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let result = lower_chain(&principal, &[&leaf], Edition::Legacy, &mut control);
            if limit == total {
                assert!(result.is_ok());
            } else {
                assert!(matches!(
                    result,
                    Err(StrictPreprocessingFailure::Control(
                        ControlFailure::BudgetExhausted {
                            domain: WorkDomain::XdmNode,
                            ..
                        }
                    ))
                ));
            }
        }
    }
}
