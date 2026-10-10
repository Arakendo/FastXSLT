//! Source-copy construction and guarded single-source copy-union compilation.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xslt::golden_semantics_experiment::{ComputedAttribute, Instruction};

use super::{
    CompileFailure, compile_attribute_set_use_names, compile_sequence_excluding,
    ensure_only_attributes, invalid, is_ascii_ncname, is_xslt_element, meaningful_children,
    uses_xslt10_compatibility,
};

pub(super) fn compile_copy(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["use-attribute-sets"], "xsl:copy")?;
    let mut attribute_nodes = Vec::new();
    let attribute_set_names = compile_attribute_set_use_names(document, element, None)?;
    let mut attributes: Vec<ComputedAttribute> = Vec::new();
    let mut content_started = false;
    let recover_late_attributes = uses_xslt10_compatibility(document, element);
    for child in meaningful_children(document, element) {
        if !is_xslt_element(document, child, "attribute") {
            content_started |= !is_attribute_only_copy_of(document, child);
            continue;
        }
        if content_started {
            if recover_late_attributes {
                attribute_nodes.push(child);
                continue;
            }
            return Err(invalid(
                "XTDE0410",
                "xsl:attribute must precede child content in xsl:copy",
                document.location(child),
            ));
        }
        let attribute =
            super::computed_attribute_compiler::compile_computed_attribute(document, child)?;
        if let Some(index) = attributes.iter().position(|existing| {
            existing.dynamic_name.is_none()
                && attribute.dynamic_name.is_none()
                && existing.name == attribute.name
        }) {
            attributes.remove(index);
        }
        attributes.push(attribute);
        attribute_nodes.push(child);
    }
    Ok(Instruction::Copy {
        attributes,
        attribute_set_names,
        body: compile_sequence_excluding(document, element, &attribute_nodes)?,
        recover_unattached_attributes: recover_late_attributes,
        location: document.location(element).clone(),
    })
}

fn is_attribute_only_copy_of(document: &Document, element: NodeId) -> bool {
    is_xslt_element(document, element, "copy-of")
        && super::optional_attribute(document, element, None, "select").is_some_and(|select| {
            select
                .split('|')
                .map(str::trim)
                .all(|part| part == "@*" || part.strip_prefix('@').is_some_and(is_ascii_ncname))
        })
}

pub(super) fn compile_mixed_copy_union(
    document: &Document,
    element: NodeId,
    expression: &str,
) -> Result<Option<Instruction>, CompileFailure> {
    if !uses_xslt10_compatibility(document, element)
        || !super::split_top_level_union(expression).is_some_and(|parts| {
            parts
                .iter()
                .any(|part| part.trim_start().starts_with("document("))
        })
    {
        return Ok(None);
    }
    let select = super::template_invocation_compiler::parse_apply_selection(
        document,
        element,
        expression,
        document.location(element).clone(),
    )?;
    if !matches!(
        select,
        crate::xslt::golden_semantics_experiment::ApplySelection::Xslt10MixedUnion(_)
    ) {
        return Err(super::unsupported(
            "FXXP1001",
            "copy union requires admitted source/document path branches",
            document.location(element),
        ));
    }
    Ok(Some(Instruction::CopyOfXslt10MixedUnion {
        select,
        recover_unattached_attributes: true,
        location: document.location(element).clone(),
    }))
}
