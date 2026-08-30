//! Private AR-0005 experiment for bounded compiled-semantic inspection.

use std::collections::BTreeMap;

use super::golden_semantics_experiment::{GlobalBindingKind, Instruction, StylesheetProgram};

#[derive(Debug, Clone, Copy)]
struct InspectionLimits {
    max_text_bytes: usize,
    max_feature_kinds: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InspectionFailure {
    TextLimit { maximum: usize, attempted: usize },
    FeatureKindLimit { maximum: usize, observed: usize },
    CountOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SemanticFeature {
    LiteralElement,
    ComputedAttribute,
    Text,
    ValueOf,
    LocalVariable,
    SequenceNodes,
    SequenceItems,
    ApplyTemplates,
    ForEach,
    NextMatch,
    ApplyImports,
    If,
    Choose,
    CallTemplate,
    Copy,
    CopyOf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FeatureObservation {
    feature: SemanticFeature,
    occurrences: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OutputInspection {
    method: Option<String>,
    encoding: Option<String>,
    media_type: Option<String>,
    include_content_type: Option<bool>,
    byte_order_mark: Option<bool>,
    omit_xml_declaration: bool,
    indent: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledInspection {
    stylesheet_identity: String,
    declared_version: String,
    output: OutputInspection,
    root_template_count: usize,
    root_template_modes: Vec<String>,
    exact_element_template_count: usize,
    named_template_count: usize,
    global_variable_count: usize,
    global_parameter_count: usize,
    instruction_count: usize,
    features: Vec<FeatureObservation>,
}

fn inspect_compiled(
    stylesheet_identity: &str,
    program: &StylesheetProgram,
    limits: InspectionLimits,
) -> Result<CompiledInspection, InspectionFailure> {
    let text_bytes = stylesheet_identity
        .len()
        .checked_add(program.declared_version.len())
        .and_then(|bytes| bytes.checked_add(program.output.method.as_ref().map_or(0, String::len)))
        .and_then(|bytes| {
            bytes.checked_add(program.output.encoding.as_ref().map_or(0, String::len))
        })
        .and_then(|bytes| {
            bytes.checked_add(program.output.media_type.as_ref().map_or(0, String::len))
        })
        .and_then(|bytes| {
            bytes.checked_add(program.root_template_modes.iter().map(String::len).sum())
        })
        .ok_or(InspectionFailure::CountOverflow)?;
    if text_bytes > limits.max_text_bytes {
        return Err(InspectionFailure::TextLimit {
            maximum: limits.max_text_bytes,
            attempted: text_bytes,
        });
    }

    let mut feature_counts = BTreeMap::new();
    let mut instruction_count = 0_usize;
    if let Some(root_template) = &program.root_template {
        observe_instructions(
            &root_template.body,
            &mut instruction_count,
            &mut feature_counts,
        )?;
    }
    for template in &program.matched_templates {
        observe_instructions(
            &template.template.body,
            &mut instruction_count,
            &mut feature_counts,
        )?;
    }
    for template in &program.named_templates {
        observe_instructions(
            &template.template.body,
            &mut instruction_count,
            &mut feature_counts,
        )?;
    }
    if feature_counts.len() > limits.max_feature_kinds {
        return Err(InspectionFailure::FeatureKindLimit {
            maximum: limits.max_feature_kinds,
            observed: feature_counts.len(),
        });
    }

    Ok(CompiledInspection {
        stylesheet_identity: stylesheet_identity.to_owned(),
        declared_version: program.declared_version.clone(),
        output: OutputInspection {
            method: program.output.method.clone(),
            encoding: program.output.encoding.clone(),
            media_type: program.output.media_type.clone(),
            include_content_type: program.output.include_content_type,
            byte_order_mark: program.output.byte_order_mark,
            omit_xml_declaration: program.output.omit_xml_declaration,
            indent: program.output.indent,
        },
        root_template_count: usize::from(program.root_template.is_some()),
        root_template_modes: program.root_template_modes.clone(),
        exact_element_template_count: program
            .matched_templates
            .iter()
            .filter(|template| {
                matches!(
                    template.pattern,
                    super::golden_semantics_experiment::MatchPattern::Element(_)
                )
            })
            .count(),
        named_template_count: program.named_templates.len(),
        global_variable_count: program
            .global_bindings
            .iter()
            .filter(|binding| binding.kind == GlobalBindingKind::Variable)
            .count(),
        global_parameter_count: program
            .global_bindings
            .iter()
            .filter(|binding| binding.kind == GlobalBindingKind::Parameter)
            .count(),
        instruction_count,
        features: feature_counts
            .into_iter()
            .map(|(feature, occurrences)| FeatureObservation {
                feature,
                occurrences,
            })
            .collect(),
    })
}

fn observe_instructions(
    instructions: &[Instruction],
    instruction_count: &mut usize,
    feature_counts: &mut BTreeMap<SemanticFeature, usize>,
) -> Result<(), InspectionFailure> {
    for instruction in instructions {
        *instruction_count = instruction_count
            .checked_add(1)
            .ok_or(InspectionFailure::CountOverflow)?;
        let (feature, body) = match instruction {
            Instruction::LiteralElement {
                computed_attributes,
                body,
                ..
            } => {
                if !computed_attributes.is_empty() {
                    *instruction_count = instruction_count
                        .checked_add(computed_attributes.len())
                        .ok_or(InspectionFailure::CountOverflow)?;
                    let occurrences = feature_counts
                        .entry(SemanticFeature::ComputedAttribute)
                        .or_default();
                    *occurrences = occurrences
                        .checked_add(computed_attributes.len())
                        .ok_or(InspectionFailure::CountOverflow)?;
                }
                (SemanticFeature::LiteralElement, Some(body.as_slice()))
            }
            Instruction::Text { .. } => (SemanticFeature::Text, None),
            Instruction::ValueOf { .. } => (SemanticFeature::ValueOf, None),
            Instruction::Variable { .. }
            | Instruction::IntegerRangeVariable { .. }
            | Instruction::TemporaryTreeVariable { .. } => (SemanticFeature::LocalVariable, None),
            Instruction::SequenceNodes { .. } => (SemanticFeature::SequenceNodes, None),
            Instruction::SequenceItems { .. } => (SemanticFeature::SequenceItems, None),
            Instruction::ApplyTemplates { .. } => (SemanticFeature::ApplyTemplates, None),
            Instruction::ForEachTemporaryRoot { body, .. } => {
                (SemanticFeature::ForEach, Some(body.as_slice()))
            }
            Instruction::NextMatch { .. } => (SemanticFeature::NextMatch, None),
            Instruction::ApplyImports { .. } => (SemanticFeature::ApplyImports, None),
            Instruction::If { body, .. } => (SemanticFeature::If, Some(body.as_slice())),
            Instruction::Choose { .. } => (SemanticFeature::Choose, None),
            Instruction::CallTemplate { .. } => (SemanticFeature::CallTemplate, None),
            Instruction::Copy { .. } => (SemanticFeature::Copy, None),
            Instruction::CopyOfCurrent { .. } => (SemanticFeature::CopyOf, None),
        };
        let occurrences = feature_counts.entry(feature).or_default();
        *occurrences = occurrences
            .checked_add(1)
            .ok_or(InspectionFailure::CountOverflow)?;
        if let Some(body) = body {
            observe_instructions(body, instruction_count, feature_counts)?;
        }
        if let Instruction::Choose {
            branches,
            otherwise,
            ..
        } = instruction
        {
            for branch in branches {
                observe_instructions(&branch.body, instruction_count, feature_counts)?;
            }
            observe_instructions(otherwise, instruction_count, feature_counts)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    use super::{
        CompiledInspection, FeatureObservation, InspectionFailure, InspectionLimits,
        OutputInspection, SemanticFeature, inspect_compiled,
    };

    const IDENTITY: &str = "urn:fastxslt:inspection:hello-stylesheet";

    fn compile_hello() -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
        let parsed = parse_document(
            IDENTITY,
            include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl"),
            ParseLimits {
                max_events: 1_024,
                max_depth: 64,
            },
        )
        .expect("parse inspection stylesheet");
        let document = Document::from_parsed(parsed).expect("build inspection stylesheet XDM");
        compile_stylesheet(&document).expect("compile inspection stylesheet")
    }

    #[test]
    fn projects_owned_bounded_semantics_without_exposing_private_representation() {
        let program = compile_hello();
        let unchanged = program.clone();
        let inspection = inspect_compiled(
            IDENTITY,
            &program,
            InspectionLimits {
                max_text_bytes: 256,
                max_feature_kinds: 4,
            },
        )
        .expect("inspect compiled semantics");
        assert_eq!(program, unchanged, "inspection must be semantically inert");
        drop(program);

        assert_eq!(
            inspection,
            CompiledInspection {
                stylesheet_identity: IDENTITY.to_owned(),
                declared_version: "1.0".to_owned(),
                output: OutputInspection {
                    method: Some("xml".to_owned()),
                    encoding: None,
                    media_type: Some("application/xml".to_owned()),
                    include_content_type: None,
                    byte_order_mark: None,
                    omit_xml_declaration: true,
                    indent: None,
                },
                root_template_count: 1,
                root_template_modes: Vec::new(),
                exact_element_template_count: 0,
                named_template_count: 0,
                global_variable_count: 0,
                global_parameter_count: 0,
                instruction_count: 4,
                features: vec![
                    FeatureObservation {
                        feature: SemanticFeature::LiteralElement,
                        occurrences: 1,
                    },
                    FeatureObservation {
                        feature: SemanticFeature::Text,
                        occurrences: 2,
                    },
                    FeatureObservation {
                        feature: SemanticFeature::ValueOf,
                        occurrences: 1,
                    },
                ],
            }
        );
    }

    #[test]
    fn rejects_reports_that_exceed_text_or_feature_kind_limits() {
        let program = compile_hello();
        assert_eq!(
            inspect_compiled(
                IDENTITY,
                &program,
                InspectionLimits {
                    max_text_bytes: 1,
                    max_feature_kinds: 4,
                }
            ),
            Err(InspectionFailure::TextLimit {
                maximum: 1,
                attempted: IDENTITY.len() + "1.0".len() + "xml".len() + "application/xml".len(),
            })
        );
        assert_eq!(
            inspect_compiled(
                IDENTITY,
                &program,
                InspectionLimits {
                    max_text_bytes: 256,
                    max_feature_kinds: 2,
                }
            ),
            Err(InspectionFailure::FeatureKindLimit {
                maximum: 2,
                observed: 3,
            })
        );
    }

    #[test]
    fn reports_specialized_computed_attributes_without_exposing_plan_layout() {
        let parsed = parse_document(
            IDENTITY,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc"><out><xsl:attribute name="magic"><xsl:value-of select="$magic"/></xsl:attribute></out></xsl:template></xsl:stylesheet>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 16,
            },
        )
        .expect("parse computed-attribute inspection stylesheet");
        let document = Document::from_parsed(parsed).expect("build inspection stylesheet XDM");
        let program = compile_stylesheet(&document).expect("compile computed-attribute stylesheet");

        let inspection = inspect_compiled(
            IDENTITY,
            &program,
            InspectionLimits {
                max_text_bytes: 256,
                max_feature_kinds: 4,
            },
        )
        .expect("inspect computed-attribute semantics");

        assert_eq!(inspection.instruction_count, 2);
        assert!(inspection.features.contains(&FeatureObservation {
            feature: SemanticFeature::ComputedAttribute,
            occurrences: 1,
        }));
        assert!(inspection.features.contains(&FeatureObservation {
            feature: SemanticFeature::LiteralElement,
            occurrences: 1,
        }));
    }
}
