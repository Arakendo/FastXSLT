//! Compilation of the private source-element `xsl:copy` construction seam.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{
    Instruction, LiteralAttribute, LiteralAttributeValue,
};

use super::{
    CompileFailure, compile_local_attribute_sets, compile_sequence_excluding,
    ensure_no_meaningful_children, ensure_only_attributes, invalid, is_ascii_ncname,
    is_xslt_element, meaningful_children, required_attribute, unsupported,
    uses_xslt10_compatibility,
};

pub(super) fn compile_copy(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["use-attribute-sets"], "xsl:copy")?;
    let mut attribute_nodes = Vec::new();
    let mut attributes = compile_local_attribute_sets(document, element, None)?
        .into_iter()
        .map(|attribute| LiteralAttribute {
            name: attribute.name,
            value: attribute.value,
            location: attribute.location,
        })
        .collect::<Vec<_>>();
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
        let attribute = compile_static_attribute(document, child, recover_late_attributes)?;
        if let Some(index) = attributes
            .iter()
            .position(|existing| existing.name == attribute.name)
        {
            attributes.remove(index);
        }
        attributes.push(attribute);
        attribute_nodes.push(child);
    }
    Ok(Instruction::Copy {
        attributes,
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

fn compile_static_attribute(
    document: &Document,
    element: NodeId,
    xslt10_compatibility: bool,
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
    let children = meaningful_children(document, element);
    let value = match children.as_slice() {
        [value_of] if is_xslt_element(document, *value_of, "value-of") => {
            ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
            ensure_no_meaningful_children(document, *value_of, "xsl:value-of")?;
            let select = required_attribute(document, *value_of, None, "select")?;
            if select.trim() == "." {
                LiteralAttributeValue::ContextStringValue
            } else if let Some(local) = select
                .strip_prefix('@')
                .filter(|local| is_ascii_ncname(local))
            {
                LiteralAttributeValue::SourceAttribute(ExpandedName {
                    namespace: None,
                    local: local.to_owned(),
                })
            } else if xslt10_compatibility
                && let Ok(path) = crate::xpath::path_experiment::parse_location_path(
                    select.trim(),
                    document.location(*value_of).clone(),
                )
            {
                LiteralAttributeValue::Xslt10TextAndPath {
                    prefix: String::new(),
                    path,
                    suffix: String::new(),
                }
            } else {
                return Err(unsupported(
                    "FXXP1012",
                    format!("unsupported source-copy attribute value expression: {select}"),
                    document.location(*value_of),
                ));
            }
        }
        _ if children.iter().all(|child| {
            document.kind(*child) == crate::xdm::owned_tree_experiment::NodeKind::Text
        }) =>
        {
            LiteralAttributeValue::Text(document.string_value(element))
        }
        _ => {
            return Err(unsupported(
                "FXST1031",
                "the private source-copy attribute slice requires text or one unqualified source-attribute value",
                document.location(element),
            ));
        }
    };
    Ok(LiteralAttribute {
        name: ExpandedName {
            namespace: None,
            local: name.to_owned(),
        },
        value,
        location: document.location(element).clone(),
    })
}
