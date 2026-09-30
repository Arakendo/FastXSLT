use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xslt::golden_semantics_experiment::{Instruction, StylesheetProgram, TemplateArgument};

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
            | Instruction::ContextNameElement { body, .. }
            | Instruction::DynamicNameElement { body, .. }
            | Instruction::ForEachVariable { body, .. }
            | Instruction::ForEachStaticIntegerRange { body, .. }
            | Instruction::ForEachNodes { body, .. }
            | Instruction::SequenceTreeVariable { body, .. }
            | Instruction::Xslt10Message { body, .. }
            | Instruction::If { body, .. } => {
                validate_named_calls(program, body)?;
            }
            Instruction::Xslt10ProcessingInstructionNode { body, .. }
            | Instruction::Xslt10CommentNode { body, .. } => {
                validate_named_calls(program, body.as_ref())?;
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
            } => validate_named_call(program, name, arguments, location)?,
            Instruction::Text { .. }
            | Instruction::Xslt10DeferredFailure { .. }
            | Instruction::Number { .. }
            | Instruction::ProcessingInstructionNode { .. }
            | Instruction::CommentNode { .. }
            | Instruction::Attribute { .. }
            | Instruction::ValueOf { .. }
            | Instruction::Variable { .. }
            | Instruction::StaticAtomicVariable { .. }
            | Instruction::VariableAlias { .. }
            | Instruction::ContextPositionVariable { .. }
            | Instruction::ContextNodeNameVariable { .. }
            | Instruction::ContextCountPathVariable { .. }
            | Instruction::Xslt10BinaryNumericVariable { .. }
            | Instruction::Xslt10ConcatVariable { .. }
            | Instruction::Xslt10KeyVariable { .. }
            | Instruction::Xslt10LiteralDocumentVariable { .. }
            | Instruction::SourceNodeVariable { .. }
            | Instruction::Xslt10SourceNodesAttributeEqualsCurrentName { .. }
            | Instruction::SourceVariablePathVariable { .. }
            | Instruction::SourceNodeUnionVariable { .. }
            | Instruction::IntegerRangeVariable { .. }
            | Instruction::TemporaryTreeVariable { .. }
            | Instruction::Xslt10TextTreeVariable { .. }
            | Instruction::Xslt10ValueOfTreeVariable { .. }
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
            | Instruction::CopyOfDocument { .. }
            | Instruction::CopyOfVariableDocument { .. }
            | Instruction::CopyOfNestedDocuments { .. }
            | Instruction::CopyOfSourceDocuments { .. }
            | Instruction::CopyOfXslt10KeyLookup { .. }
            | Instruction::CopyOfPathUnion { .. }
            | Instruction::CopyOfStaticAtomicText { .. }
            | Instruction::CopyOfVariable { .. }
            | Instruction::CopyOfAtomicValue { .. }
            | Instruction::Copy { .. } => {}
        }
    }
    Ok(())
}

fn validate_named_call(
    program: &StylesheetProgram,
    name: &str,
    arguments: &[TemplateArgument],
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    let called = program
        .named_templates
        .iter()
        .find(|template| template.name == name)
        .ok_or_else(|| {
            invalid(
                "FXST0014",
                format!("unknown named template: {name}"),
                location,
            )
        })?;
    if let Some(parameter) = called.template.parameters.iter().find(|parameter| {
        parameter.required
            && !parameter.tunnel
            && !arguments
                .iter()
                .any(|argument| argument.name == parameter.name)
    }) {
        return Err(invalid(
            "XTSE0690",
            format!(
                "call to named template {name} omits required parameter {}",
                parameter.name
            ),
            location,
        ));
    }
    Ok(())
}
