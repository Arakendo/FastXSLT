//! Private vertical canonical-content lowering reference. It uses the existing
//! instruction compiler and retains fragment-wise literal emission/provenance.
//! Literal constructors recurse canonically; XSLT instruction bodies still use
//! baseline traversal of the selected view;
//! this is not canonical graph promotion or a second instruction compiler.

use super::*;
use crate::compile::golden_stylesheet_experiment::stylesheet_preprocessing::{
    StrictPreprocessingFailure,
    canonical_content::{CanonicalChild, TextFragment, coalesce_visible_children},
    stylesheet_visibility_view,
};
use crate::execution_control_experiment::InvocationControl;

fn compile_canonical_sequence(
    document: &Document,
    parent: NodeId,
    profile: StylesheetPreprocessingProfile,
    excluded: &[NodeId],
    initial_local_bindings: &[String],
    control: &mut InvocationControl,
) -> Result<Vec<Instruction>, StrictPreprocessingFailure> {
    let view = stylesheet_visibility_view(document, parent, profile, control)?;
    compile_canonical_sequence_in_view(
        document,
        &view,
        parent,
        excluded,
        initial_local_bindings,
        control,
    )
}

fn compile_canonical_sequence_in_view(
    document: &Document,
    view: &Document,
    parent: NodeId,
    excluded: &[NodeId],
    initial_local_bindings: &[String],
    control: &mut InvocationControl,
) -> Result<Vec<Instruction>, StrictPreprocessingFailure> {
    let content = coalesce_visible_children(document, view, parent, control)?;
    debug_assert!(content.source.same_origin(view));
    let mut instructions = Vec::new();
    let mut local_variables = initial_local_bindings.to_vec();
    for child in content.children {
        match child {
            CanonicalChild::Text(fragments) => {
                if fragments.len() > 1
                    && !fragments
                        .iter()
                        .any(|fragment| excluded.contains(&fragment.original_node))
                    && effective_expand_text(view, fragments[0].original_node)?
                {
                    instructions.push(compile_expanded_run(view, parent, &fragments, control)?);
                    continue;
                }
                // Exclusion is a lowering decision, not permission to join
                // text across an excluded element or reinterpret a split TVT.
                for fragment in &fragments {
                    if excluded.contains(&fragment.original_node) {
                        continue;
                    }
                    if fragments.len() > 1 && effective_expand_text(view, fragment.original_node)? {
                        return Err(CompileFailure {
                            code: "FXST1066",
                            category: CompileCategory::Unsupported,
                            detail: "canonical split expanded-text lowering is not yet integrated"
                                .to_owned(),
                            location: fragment.location.clone(),
                        }
                        .into());
                    }
                    if let Some(instruction) =
                        compile_literal_text_node(view, fragment.original_node, true)?
                    {
                        instructions.push(instruction);
                    }
                }
            }
            CanonicalChild::Node(node) => {
                if excluded.contains(&node) {
                    continue;
                }
                if view.kind(node) != NodeKind::Element {
                    return Err(invalid(
                        "FXST0006",
                        "unexpected node kind in stylesheet sequence",
                        view.location(node),
                    )
                    .into());
                }
                compile_sequence_element_with_literal_compiler(
                    view,
                    node,
                    &mut local_variables,
                    &mut instructions,
                    &mut |view, element| {
                        let (mut instruction, exclusions) =
                            compile_literal_element_parts(view, element)?;
                        let Instruction::LiteralElement { body, .. } = &mut instruction else {
                            unreachable!("literal element parts")
                        };
                        *body = compile_canonical_sequence_in_view(
                            document,
                            view,
                            element,
                            &exclusions,
                            &[],
                            control,
                        )?;
                        Ok::<_, StrictPreprocessingFailure>(instruction)
                    },
                )?;
            }
        }
    }
    Ok(instructions)
}

/// Only the existing admitted whole-expression form is lowered. Materializing
/// a logical value does not create a replacement XDM node or merged source span.
fn compile_expanded_run(
    view: &Document,
    parent: NodeId,
    fragments: &[TextFragment<'_>],
    control: &mut InvocationControl,
) -> Result<Instruction, StrictPreprocessingFailure> {
    let mut value = String::new();
    for fragment in fragments {
        // Conservative private preprocessing work: charge copied decoded bytes
        // before growth. This is not runtime result text or allocator accounting.
        control.charge(
            crate::execution_control_experiment::WorkDomain::XdmNode,
            fragment.value.len(),
        )?;
        value.push_str(fragment.value);
    }
    let location = fragments[0].location;
    if admitted_expanded_text_expression(&value).is_none() {
        return Err(CompileFailure {
            code: "FXST1066",
            category: CompileCategory::Unsupported,
            detail: "canonical expanded-text form is outside the existing whole-expression slice"
                .to_owned(),
            location: location.clone(),
        }
        .into());
    }
    // Anchor the logical expression to its first original fragment. Decoded
    // expression offsets must not be added to raw XML spans (CDATA/entities).
    Ok(compile_literal_text_value(
        view, parent, &value, true, location,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{
        CancellationToken, ControlFailure, WorkDomain, WorkLimits,
    };
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn document(body: &str, attributes: &str) -> (Document, NodeId) {
        document_with_version(body, attributes, "1.0")
    }

    fn document_with_version(body: &str, attributes: &str, version: &str) -> (Document, NodeId) {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}" {attributes}><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
        );
        let document = Document::from_parsed(
            parse_document(
                "urn:canonical-lowering",
                xml.as_bytes(),
                ParseLimits {
                    max_events: 1000,
                    max_depth: 50,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let root = document.children(document.document_node())[0];
        let parent = document.children(root)[0];
        (document, parent)
    }

    fn assert_same_lowering(body: &str, attributes: &str, excluded: bool, bindings: &[String]) {
        let (document, parent) = document(body, attributes);
        let excluded = if excluded {
            vec![
                document
                    .children(parent)
                    .iter()
                    .copied()
                    .find(|node| document.kind(*node) == NodeKind::Element)
                    .unwrap(),
            ]
        } else {
            vec![]
        };
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let view = stylesheet_visibility_view(
                &document,
                parent,
                profile,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let expected =
                compile_sequence_excluding_with_bindings(&view, parent, &excluded, bindings)
                    .unwrap();
            let actual = compile_canonical_sequence(
                &document,
                parent,
                profile,
                &excluded,
                bindings,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn canonical_preprocessing_lowering_preserves_literal_fragment_instructions_and_locations() {
        let (document, parent) = document("é<!--removed--> B<?removed yes?>😀", "");
        let instructions = compile_canonical_sequence(
            &document,
            parent,
            StylesheetPreprocessingProfile::Modern,
            &[],
            &[],
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let text_nodes: Vec<_> = document
            .children(parent)
            .iter()
            .copied()
            .filter(|node| document.kind(*node) == NodeKind::Text)
            .collect();
        assert_eq!(instructions.len(), text_nodes.len());
        for (instruction, node) in instructions.iter().zip(text_nodes) {
            let Instruction::Text {
                value,
                location,
                disable_output_escaping,
            } = instruction
            else {
                panic!("literal text")
            };
            assert_eq!(value, document.value(node).unwrap());
            assert_eq!(location, document.location(node));
            assert!(!disable_output_escaping);
        }
        assert_same_lowering(" <!--removed-->A<?removed yes?> <z/>", "", false, &[]);
    }

    #[test]
    fn canonical_preprocessing_lowering_preserves_excluded_element_boundaries() {
        assert_same_lowering(
            "A<!--removed--> <xsl:param name='p'/> B<?removed yes?>C",
            "",
            true,
            &[],
        );
    }

    #[test]
    fn canonical_preprocessing_lowering_uses_shared_lexical_binding_dispatch() {
        assert_same_lowering(
            "<xsl:variable name='v' select='1'/><xsl:value-of select='$v'/>",
            "",
            false,
            &[],
        );
        assert_same_lowering("<xsl:value-of select='$p'/>", "", false, &["p".to_owned()]);
    }

    #[test]
    fn canonical_preprocessing_lowering_uses_shared_recursive_constructor_dispatch() {
        assert_same_lowering(
            "<out> A<!--removed-->B<xsl:text> </xsl:text><xsl:if test='true()'>C</xsl:if></out>",
            "",
            false,
            &[],
        );
    }

    #[test]
    fn canonical_preprocessing_recursive_literals_keep_static_attributes_and_exclusions() {
        assert_same_lowering(
            "<out literal='L'><xsl:attribute name='computed'>V</xsl:attribute>A<!--removed-->B<inner xml:space='preserve'> <!--removed-->C<?removed yes?> </inner></out>",
            "",
            false,
            &[],
        );
        assert_same_lowering(
            "<out xml:space='preserve'><inner xml:space='default'> <!--removed-->C<?removed yes?> </inner><xsl:text> </xsl:text></out>",
            "",
            false,
            &[],
        );
    }

    #[test]
    fn canonical_preprocessing_recursive_literal_failure_keeps_shared_validation_order_and_provenance()
     {
        for body in [
            "<out><inner xsl:exclude-result-prefixes='missing'>A<!--removed-->B<xsl:value-of/></inner></out>",
            "<out><inner>A<!--removed-->B<xsl:value-of/></inner></out>",
        ] {
            let (document, parent) = document(body, "");
            for profile in [
                StylesheetPreprocessingProfile::Modern,
                StylesheetPreprocessingProfile::StrictXslt10,
            ] {
                let view = stylesheet_visibility_view(
                    &document,
                    parent,
                    profile,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                let expected =
                    compile_sequence_excluding_with_bindings(&view, parent, &[], &[]).unwrap_err();
                let StrictPreprocessingFailure::Compile(actual) = compile_canonical_sequence(
                    &document,
                    parent,
                    profile,
                    &[],
                    &[],
                    &mut InvocationControl::unbounded(),
                )
                .unwrap_err() else {
                    panic!("compiler failure")
                };
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn canonical_preprocessing_recursive_literal_work_is_aggregate_and_failure_leaves_view_reusable()
     {
        let (document, parent) = document(
            "<out>A<!--removed-->B<inner>C<?removed yes?>D</inner></out>",
            "",
        );
        let original = document.children(parent).to_vec();
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let mut measured = InvocationControl::unbounded();
            let expected =
                compile_canonical_sequence(&document, parent, profile, &[], &[], &mut measured)
                    .unwrap();
            let required = measured.consumed(WorkDomain::XdmNode);
            for limit in [required, required - 1] {
                let mut control = InvocationControl::new(
                    CancellationToken::new(),
                    WorkLimits {
                        xdm_nodes: limit,
                        ..WorkLimits::unbounded()
                    },
                );
                let actual =
                    compile_canonical_sequence(&document, parent, profile, &[], &[], &mut control);
                if limit == required {
                    assert_eq!(actual.unwrap(), expected);
                } else {
                    assert!(matches!(
                        actual,
                        Err(StrictPreprocessingFailure::Control(
                            ControlFailure::BudgetExhausted {
                                domain: WorkDomain::XdmNode,
                                ..
                            }
                        ))
                    ));
                }
            }
            let view = stylesheet_visibility_view(
                &document,
                parent,
                profile,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let mut control =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 2);
            assert!(matches!(
                compile_canonical_sequence_in_view(
                    &document,
                    &view,
                    parent,
                    &[],
                    &[],
                    &mut control
                ),
                Err(StrictPreprocessingFailure::Control(
                    ControlFailure::Cancelled {
                        domain: WorkDomain::XdmNode
                    }
                ))
            ));
            assert_eq!(
                compile_canonical_sequence_in_view(
                    &document,
                    &view,
                    parent,
                    &[],
                    &[],
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
            assert_eq!(document.children(parent), original);
        }
    }

    #[test]
    fn canonical_preprocessing_recursive_literal_expanded_run_executes_after_owner_release() {
        use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
        use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
        let (source, _) = document("", "");
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let program = {
                let attributes = "expand-text='yes' xmlns:xs='http://www.w3.org/2001/XMLSchema'";
                let (unsplit, _) = document_with_version(
                    "<xsl:variable name='v'><r><a/></r></xsl:variable><out><inner>{$v//a instance of element(a, xs:untyped)}</inner></out>",
                    attributes,
                    "3.0",
                );
                let mut program = compile_stylesheet(&unsplit).unwrap();
                let (split, parent) = document_with_version(
                    "<xsl:variable name='v'><r><a/></r></xsl:variable><out><inner>{$v//a<!--removed--> instance of element(a, xs:untyped)}</inner></out>",
                    attributes,
                    "3.0",
                );
                program.root_template.as_mut().unwrap().body = compile_canonical_sequence(
                    &split,
                    parent,
                    profile,
                    &[],
                    &[],
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                program
            };
            let actual =
                execute_compiled_stylesheet_for_test(&program, &source, "reference").unwrap();
            assert!(actual.contains("<inner>true</inner>"), "{actual}");
        }
    }

    #[test]
    fn canonical_preprocessing_lowering_uses_shared_forward_compatible_fallback_dispatch() {
        assert_same_lowering(
            "<out xsl:version='1.1'><xsl:future><xsl:fallback><xsl:text>F</xsl:text></xsl:fallback></xsl:future></out>",
            "",
            false,
            &[],
        );
    }

    #[test]
    fn canonical_preprocessing_lowering_keeps_split_expanded_text_explicitly_unsupported() {
        let (document, parent) =
            document("before {1<!--removed--> + 2} after", "expand-text='yes'");
        let original = document.children(parent).to_vec();
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let failure = compile_canonical_sequence(
                &document,
                parent,
                profile,
                &[],
                &[],
                &mut InvocationControl::unbounded(),
            )
            .unwrap_err();
            let StrictPreprocessingFailure::Compile(failure) = failure else {
                panic!("unsupported compiler outcome")
            };
            assert_eq!(failure.code, "FXST1066");
            assert_eq!(failure.category, CompileCategory::Unsupported);
            assert_eq!(failure.location, document.location(original[0]).clone());
        }
        assert_eq!(document.children(parent), original);
        assert_same_lowering(
            "<out xsl:expand-text='no'>A<!--removed-->B</out>",
            "expand-text='yes'",
            false,
            &[],
        );
    }

    #[test]
    fn canonical_preprocessing_expanded_run_uses_shared_expression_compiler_and_original_anchor() {
        let (document, parent) = document_with_version(
            "{$v//a<!--removed--><![CDATA[ instance of ]]><?removed yes?>element(a, xs:untyped)}",
            "expand-text='yes' xmlns:xs='http://www.w3.org/2001/XMLSchema'",
            "3.0",
        );
        let first = document.children(parent)[0];
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let actual = compile_canonical_sequence(
                &document,
                parent,
                profile,
                &[],
                &[],
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let expected = compile_literal_text_value(
                &document,
                parent,
                "{$v//a instance of element(a, xs:untyped)}",
                true,
                document.location(first),
            )
            .unwrap();
            assert_eq!(actual, vec![expected]);
            let Instruction::ValueOf { location, .. } = &actual[0] else {
                panic!("whole expression")
            };
            assert_eq!(location, document.location(first));
        }
    }

    #[test]
    fn canonical_preprocessing_expanded_run_preserves_shared_compiler_failure() {
        let (document, parent) = document(
            "{.//a<!--removed--> instance of element(unbound:a)}",
            "expand-text='yes'",
        );
        let first = document.children(parent)[0];
        let expected = compile_literal_text_value(
            &document,
            parent,
            "{.//a instance of element(unbound:a)}",
            true,
            document.location(first),
        )
        .unwrap_err();
        let StrictPreprocessingFailure::Compile(actual) = compile_canonical_sequence(
            &document,
            parent,
            StylesheetPreprocessingProfile::Modern,
            &[],
            &[],
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err() else {
            panic!("compiler failure")
        };
        assert_eq!(actual, expected);
        assert_eq!(actual.location, document.location(first).clone());
    }

    #[test]
    fn canonical_preprocessing_expanded_run_charges_before_copy_and_recovers_after_failure() {
        let (document, parent) = document_with_version(
            "{$v//a<!--removed--> instance of element(a, xs:untyped)}",
            "expand-text='yes' xmlns:xs='http://www.w3.org/2001/XMLSchema'",
            "3.0",
        );
        let original = document.children(parent).to_vec();
        let mut measured = InvocationControl::unbounded();
        let expected = compile_canonical_sequence(
            &document,
            parent,
            StylesheetPreprocessingProfile::Modern,
            &[],
            &[],
            &mut measured,
        )
        .unwrap();
        let required = measured.consumed(WorkDomain::XdmNode);
        let mut view_control = InvocationControl::unbounded();
        let view = stylesheet_visibility_view(
            &document,
            parent,
            StylesheetPreprocessingProfile::Modern,
            &mut view_control,
        )
        .unwrap();
        let content =
            coalesce_visible_children(&document, &view, parent, &mut view_control).unwrap();
        let CanonicalChild::Text(fragments) = &content.children[0] else {
            panic!("text run")
        };
        assert_eq!(
            required - view_control.consumed(WorkDomain::XdmNode),
            fragments
                .iter()
                .map(|fragment| fragment.value.len())
                .sum::<usize>()
        );
        for limit in [required, required - 1] {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let actual = compile_canonical_sequence(
                &document,
                parent,
                StylesheetPreprocessingProfile::Modern,
                &[],
                &[],
                &mut control,
            );
            if limit == required {
                assert_eq!(actual.unwrap(), expected);
            } else {
                assert!(matches!(
                    actual,
                    Err(StrictPreprocessingFailure::Control(
                        ControlFailure::BudgetExhausted {
                            domain: WorkDomain::XdmNode,
                            ..
                        }
                    ))
                ));
            }
        }
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 1);
        assert!(matches!(
            compile_expanded_run(&view, parent, fragments, &mut control),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled {
                    domain: WorkDomain::XdmNode
                }
            ))
        ));
        assert_eq!(document.children(parent), original);
        assert_eq!(
            compile_canonical_sequence(
                &document,
                parent,
                StylesheetPreprocessingProfile::Modern,
                &[],
                &[],
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn canonical_preprocessing_expanded_run_executes_after_fragment_owner_release() {
        use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
        use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;

        let (source, _) = document("", "");
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let program = {
                let (unsplit, _) = document_with_version(
                    "<xsl:variable name='v'><r><a/></r></xsl:variable>{$v//a instance of element(a, xs:untyped)}",
                    "expand-text='yes' xmlns:xs='http://www.w3.org/2001/XMLSchema'",
                    "3.0",
                );
                let mut program = compile_stylesheet(&unsplit).unwrap();
                let (split, parent) = document_with_version(
                    "<xsl:variable name='v'><r><a/></r></xsl:variable>{$v//a<!--removed--> instance of element(a, xs:untyped)}",
                    "expand-text='yes' xmlns:xs='http://www.w3.org/2001/XMLSchema'",
                    "3.0",
                );
                program.root_template.as_mut().unwrap().body = compile_canonical_sequence(
                    &split,
                    parent,
                    profile,
                    &[],
                    &[],
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                program
            };
            let result =
                execute_compiled_stylesheet_for_test(&program, &source, "reference").unwrap();
            assert!(result.ends_with("true"), "{result}");
        }
    }

    #[test]
    fn canonical_preprocessing_lowering_keeps_original_compile_failure_provenance() {
        let (document, parent) = document("A<!--removed-->B<xsl:value-of/>", "");
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let view = stylesheet_visibility_view(
                &document,
                parent,
                profile,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let expected =
                compile_sequence_excluding_with_bindings(&view, parent, &[], &[]).unwrap_err();
            let StrictPreprocessingFailure::Compile(actual) = compile_canonical_sequence(
                &document,
                parent,
                profile,
                &[],
                &[],
                &mut InvocationControl::unbounded(),
            )
            .unwrap_err() else {
                panic!("compile failure")
            };
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn canonical_preprocessing_lowering_controls_precede_lowering_and_preserve_reuse() {
        let (document, parent) = document("A<!--removed-->B", "");
        let mut measured = InvocationControl::unbounded();
        let expected = compile_canonical_sequence(
            &document,
            parent,
            StylesheetPreprocessingProfile::Modern,
            &[],
            &[],
            &mut measured,
        )
        .unwrap();
        let required = measured.consumed(WorkDomain::XdmNode);
        for limit in [required, required - 1] {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let result = compile_canonical_sequence(
                &document,
                parent,
                StylesheetPreprocessingProfile::Modern,
                &[],
                &[],
                &mut control,
            );
            if limit == required {
                assert_eq!(result.unwrap(), expected);
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
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 2);
        assert!(matches!(
            compile_canonical_sequence(
                &document,
                parent,
                StylesheetPreprocessingProfile::Modern,
                &[],
                &[],
                &mut control
            ),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled {
                    domain: WorkDomain::XdmNode
                }
            ))
        ));
        assert_eq!(
            compile_canonical_sequence(
                &document,
                parent,
                StylesheetPreprocessingProfile::Modern,
                &[],
                &[],
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn canonical_preprocessing_lowered_artifact_executes_after_original_owner_is_dropped() {
        use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
        use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;

        let source = Document::from_parsed(
            parse_document(
                "urn:source",
                b"<r/>",
                ParseLimits {
                    max_events: 20,
                    max_depth: 5,
                },
            )
            .unwrap(),
        )
        .unwrap();
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let (expected_program, actual_program) = {
                let (document, parent) = document(
                    " <!--removed-->A<?removed yes?>B<out><xsl:value-of select=\"'C'\"/></out>",
                    "",
                );
                let view = stylesheet_visibility_view(
                    &document,
                    parent,
                    profile,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                let expected = compile_stylesheet(&view).unwrap();
                let mut actual = expected.clone();
                actual.root_template.as_mut().unwrap().body = compile_canonical_sequence(
                    &document,
                    parent,
                    profile,
                    &[],
                    &[],
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                (expected, actual)
            };
            assert_eq!(actual_program, expected_program);
            let expected =
                execute_compiled_stylesheet_for_test(&expected_program, &source, "reference")
                    .unwrap();
            let actual =
                execute_compiled_stylesheet_for_test(&actual_program, &source, "reference")
                    .unwrap();
            assert_eq!(actual, expected);
            assert!(actual.contains("AB<out>C</out>"));
        }
    }
}
