use crate::xslt::golden_semantics_experiment::{Instruction, StylesheetProgram};

use super::{CompileFailure, invalid};

pub(super) fn validate_named_template_references(
    program: &StylesheetProgram,
) -> Result<(), CompileFailure> {
    if let Some(root) = &program.root_template {
        validate_named_calls(program, &root.body)?;
    }
    for template in &program.matched_templates {
        validate_named_calls(program, &template.template.body)?;
    }
    for template in &program.named_templates {
        validate_named_calls(program, &template.template.body)?;
    }
    Ok(())
}

fn validate_named_calls(
    program: &StylesheetProgram,
    instructions: &[Instruction],
) -> Result<(), CompileFailure> {
    for instruction in instructions {
        match instruction {
            Instruction::LiteralElement { body, .. }
            | Instruction::ForEachVariable { body, .. }
            | Instruction::ForEachStaticIntegerRange { body, .. }
            | Instruction::ForEachNodes { body, .. }
            | Instruction::If { body, .. } => {
                validate_named_calls(program, body)?;
            }
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                for branch in branches {
                    validate_named_calls(program, &branch.body)?;
                }
                validate_named_calls(program, otherwise)?;
            }
            Instruction::CallTemplate { name, location, .. } => {
                program
                    .named_templates
                    .iter()
                    .find(|template| template.name == *name)
                    .ok_or_else(|| {
                        invalid(
                            "FXST0014",
                            format!("unknown named template: {name}"),
                            location,
                        )
                    })?;
            }
            Instruction::Text { .. }
            | Instruction::Number { .. }
            | Instruction::ProcessingInstructionNode { .. }
            | Instruction::CommentNode { .. }
            | Instruction::Attribute { .. }
            | Instruction::ValueOf { .. }
            | Instruction::Variable { .. }
            | Instruction::StaticAtomicVariable { .. }
            | Instruction::AtomicVariableAlias { .. }
            | Instruction::ContextPositionVariable { .. }
            | Instruction::SourceNodeVariable { .. }
            | Instruction::SourceNodeUnionVariable { .. }
            | Instruction::IntegerRangeVariable { .. }
            | Instruction::TemporaryTreeVariable { .. }
            | Instruction::Xslt10TextTreeVariable { .. }
            | Instruction::Xslt10ForEachTextTreeVariable { .. }
            | Instruction::SequenceNodes { .. }
            | Instruction::SequenceItems { .. }
            | Instruction::ApplyTemplates { .. }
            | Instruction::NextMatch { .. }
            | Instruction::ApplyImports { .. }
            | Instruction::CopyOfCurrent { .. }
            | Instruction::CopyOfChildElements { .. }
            | Instruction::CopyOfAncestorOrSelfElements { .. }
            | Instruction::CopyOfLocationPath { .. }
            | Instruction::CopyOfPathUnion { .. }
            | Instruction::CopyOfStaticAtomicText { .. }
            | Instruction::CopyOfVariable { .. }
            | Instruction::Copy { .. } => {}
        }
    }
    Ok(())
}
