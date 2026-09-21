//! Bounded XSLT 1.0 global sequence-constructor specializations.
//!
//! These compile-time forms are deliberately narrower than ordinary template
//! execution. They retain only stylesheet-derived values and lower into the
//! existing immutable temporary-tree representation.

use std::collections::BTreeMap;

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xslt::golden_semantics_experiment::{
    ConstructedAttribute, ConstructedElement, ConstructedNode, GlobalBindingDefault,
    LiteralAttributeValue,
};

use super::instruction_compiler::{
    compile_literal_result_attributes, compile_text_value, literal_result_namespaces,
};
use super::{
    CompileFailure, XSLT_NAMESPACE, ensure_no_meaningful_children, ensure_only_attributes, invalid,
    is_xslt_element, meaningful_children, normalize_variable_qname, optional_attribute,
    required_attribute, xpath_string_literal,
};

pub(super) fn compile_static_local_tree(
    document: &Document,
    binding: NodeId,
    declared_type: Option<&str>,
) -> Result<Option<GlobalBindingDefault>, CompileFailure> {
    let Some(stylesheet) = document.parent(binding) else {
        return Ok(None);
    };
    if declared_type.is_some()
        || optional_attribute(document, stylesheet, None, "version") != Some("1.0")
    {
        return Ok(None);
    }
    let children = meaningful_children(document, binding);
    if !children
        .iter()
        .any(|child| is_xslt_element(document, *child, "variable"))
    {
        return Ok(None);
    }

    let mut locals = BTreeMap::new();
    let mut nodes = Vec::new();
    for child in children {
        if is_xslt_element(document, child, "variable") {
            let Some((name, value)) = compile_static_local(document, child)? else {
                return Ok(None);
            };
            if locals.insert(name.clone(), value).is_some() {
                return Err(invalid(
                    "FXST0017",
                    format!("duplicate local variable binding: ${name}"),
                    document.location(child),
                ));
            }
            continue;
        }
        let Some(node) = compile_node(document, child, &locals)? else {
            return Ok(None);
        };
        nodes.push(node);
    }
    if nodes.is_empty() {
        return Ok(None);
    }
    Ok(Some(GlobalBindingDefault::TemporaryTree(nodes)))
}

fn compile_static_local(
    document: &Document,
    variable: NodeId,
) -> Result<Option<(String, String)>, CompileFailure> {
    ensure_only_attributes(document, variable, &["name", "select"], "xsl:variable")?;
    ensure_no_meaningful_children(document, variable, "xsl:variable")?;
    let name = normalize_variable_qname(
        document,
        variable,
        required_attribute(document, variable, None, "name")?,
    )?;
    let select = required_attribute(document, variable, None, "select")?;
    Ok(xpath_string_literal(select.trim()).map(|value| (name, value.to_owned())))
}

fn compile_node(
    document: &Document,
    node: NodeId,
    locals: &BTreeMap<String, String>,
) -> Result<Option<ConstructedNode>, CompileFailure> {
    match document.kind(node) {
        NodeKind::Text => Ok(Some(ConstructedNode::Text(
            document.value(node).unwrap_or_default().to_owned(),
        ))),
        NodeKind::Element if is_xslt_element(document, node, "value-of") => {
            compile_local_value_of(document, node, locals)
                .map(|value| value.map(ConstructedNode::Text))
        }
        NodeKind::Element if is_xslt_element(document, node, "text") => {
            compile_text_value(document, node).map(|value| Some(ConstructedNode::Text(value)))
        }
        NodeKind::Element
            if document
                .name(node)
                .is_some_and(|name| name.namespace.as_deref() == Some(XSLT_NAMESPACE)) =>
        {
            Ok(None)
        }
        NodeKind::Element => compile_element(document, node, locals)
            .map(|element| element.map(ConstructedNode::Element)),
        NodeKind::Comment | NodeKind::ProcessingInstruction => {
            unreachable!("meaningful_children excludes comments and processing instructions")
        }
        NodeKind::Document | NodeKind::Attribute => Err(invalid(
            "FXST0006",
            "unexpected node kind in a temporary-tree constructor",
            document.location(node),
        )),
    }
}

fn compile_local_value_of(
    document: &Document,
    element: NodeId,
    locals: &BTreeMap<String, String>,
) -> Result<Option<String>, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    let select = required_attribute(document, element, None, "select")?.trim();
    let Some(variable) = select.strip_prefix('$') else {
        return Ok(None);
    };
    let variable = normalize_variable_qname(document, element, variable)?;
    Ok(locals.get(&variable).cloned())
}

fn compile_element(
    document: &Document,
    element: NodeId,
    locals: &BTreeMap<String, String>,
) -> Result<Option<ConstructedElement>, CompileFailure> {
    let name = document.name(element).expect("element nodes have names");
    let mut attributes = Vec::new();
    for attribute in compile_literal_result_attributes(document, element)? {
        let value = match attribute.value {
            LiteralAttributeValue::Text(value) => value,
            LiteralAttributeValue::Variable(variable) => {
                let Some(value) = locals.get(&variable) else {
                    return Ok(None);
                };
                value.clone()
            }
            _ => return Ok(None),
        };
        attributes.push(ConstructedAttribute {
            name: attribute.name,
            value,
        });
    }
    let mut children = Vec::new();
    for child in meaningful_children(document, element) {
        let Some(child) = compile_node(document, child, locals)? else {
            return Ok(None);
        };
        children.push(child);
    }
    Ok(Some(ConstructedElement {
        name: name.clone(),
        namespaces: literal_result_namespaces(document, element),
        attributes,
        children,
    }))
}
