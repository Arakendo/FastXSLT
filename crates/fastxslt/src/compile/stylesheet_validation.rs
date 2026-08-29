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
            Instruction::LiteralElement { body, .. } | Instruction::If { body, .. } => {
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
            Instruction::CallTemplate {
                name,
                arguments,
                location,
            } => {
                let target = program
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
                if let Some(argument) = arguments
                    .iter()
                    .find(|argument| !target.parameters.contains(&argument.name))
                {
                    return Err(invalid(
                        "FXST0015",
                        format!(
                            "unknown parameter {} for named template {name}",
                            argument.name
                        ),
                        location,
                    ));
                }
            }
            Instruction::Text { .. }
            | Instruction::ValueOf { .. }
            | Instruction::Variable { .. }
            | Instruction::IntegerRangeVariable { .. }
            | Instruction::SequenceNodes { .. }
            | Instruction::SequenceItems { .. }
            | Instruction::ApplyTemplates { .. }
            | Instruction::Copy { .. } => {}
        }
    }
    Ok(())
}
