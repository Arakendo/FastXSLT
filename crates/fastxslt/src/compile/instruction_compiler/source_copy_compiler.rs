//! Compilation of the private source-element `xsl:copy` construction seam.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{
    Instruction, LiteralAttribute, LiteralAttributeValue,
};

use super::{
    CompileFailure, compile_sequence_excluding, ensure_only_attributes, invalid, is_ascii_ncname,
    is_xslt_element, meaningful_children, required_attribute, unsupported,
};

pub(super) fn compile_copy(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &[], "xsl:copy")?;
    let mut attribute_nodes = Vec::new();
    let mut attributes = Vec::new();
    let mut content_started = false;
    for child in meaningful_children(document, element) {
        if !is_xslt_element(document, child, "attribute") {
            content_started = true;
            continue;
        }
        if content_started {
            return Err(invalid(
                "XTDE0410",
                "xsl:attribute must precede child content in xsl:copy",
                document.location(child),
            ));
        }
        attributes.push(compile_static_attribute(document, child)?);
        attribute_nodes.push(child);
    }
    Ok(Instruction::Copy {
        attributes,
        body: compile_sequence_excluding(document, element, &attribute_nodes)?,
        location: document.location(element).clone(),
    })
}

fn compile_static_attribute(
    document: &Document,
    element: NodeId,
) -> Result<LiteralAttribute, CompileFailure> {
    ensure_only_attributes(document, element, &["name"], "xsl:attribute")?;
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(unsupported(
            "FXST1031",
            "computed or namespace-qualified xsl:attribute names are outside the private copy slice",
            document.location(element),
        ));
    }
    Ok(LiteralAttribute {
        name: ExpandedName {
            namespace: None,
            local: name.to_owned(),
        },
        value: LiteralAttributeValue::Text(document.string_value(element)),
        location: document.location(element).clone(),
    })
}
