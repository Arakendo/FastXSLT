//! Typed ordinary-source identity operands; resolution remains invocation-owned.

use super::{CompileFailure, Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::{
    DocumentRootReference, GeneratedSourceIdentity, GeneratedSourceSelection,
};

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    argument: &str,
    location: &SourceLocation,
) -> Result<GeneratedSourceIdentity, CompileFailure> {
    let argument = argument.trim();
    let selection = if let Some(variable) = argument.strip_prefix('$')
        && super::is_ascii_ncname(variable)
    {
        GeneratedSourceSelection::Variable(variable.to_owned())
    } else if let Some((reference, base, tail)) = super::parse_document_call(argument) {
        if base.is_some() {
            return Err(super::unsupported(
                "FXXP1003",
                "generated identity with an explicit document base is not yet admitted",
                location,
            ));
        }
        let path = super::parse_copy_of_literal_document_path(
            document,
            element,
            if tail.is_empty() { "." } else { tail },
        )?;
        if let Some(reference) = super::xpath_string_literal(reference.trim()) {
            GeneratedSourceSelection::LiteralDocument {
                reference: DocumentRootReference {
                    base: location.resource.clone(),
                    reference: reference.to_owned(),
                    descendant_name: None,
                },
                path,
            }
        } else {
            GeneratedSourceSelection::ReferencedDocuments {
                references: super::parse_copy_of_literal_document_path(
                    document, element, reference,
                )?,
                path,
            }
        }
    } else {
        GeneratedSourceSelection::Path(super::parse_copy_of_literal_document_path(
            document, element, argument,
        )?)
    };
    Ok(GeneratedSourceIdentity {
        selection,
        location: location.clone(),
    })
}

pub(super) fn is_owned_operand(argument: &str) -> bool {
    argument.trim().starts_with("document(") || argument.trim().starts_with('$')
}

pub(super) fn compile_comparison(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<crate::xslt::golden_semantics_experiment::BooleanExpression>, CompileFailure> {
    let Some((left, right, equal)) =
        super::namespace_focus_compiler::generated_identity_operands(expression)
    else {
        return Ok(None);
    };
    if !is_owned_operand(left) && !is_owned_operand(right) {
        return Ok(None);
    }
    Ok(Some(crate::xslt::golden_semantics_experiment::BooleanExpression::GeneratedSourceIdentityComparison {
        left: Box::new(compile(document, element, left, location)?),
        right: Box::new(compile(document, element, right, location)?), equal,
    }))
}
