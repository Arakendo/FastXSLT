//! Compile-time validation and expansion of local static attribute sets.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{
    AttributeSetDeclaration, ComputedAttribute, LiteralAttributeValue,
};

use super::super::{
    CompileFailure, ensure_only_attributes, invalid, is_xslt_element, meaningful_children,
    optional_attribute, required_attribute, unsupported,
};
use super::compile_computed_attribute;

pub(super) fn validate_local_attribute_set(
    document: &Document,
    element: NodeId,
) -> Result<ExpandedName, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["name", "use-attribute-sets"],
        "xsl:attribute-set",
    )?;
    let name = required_attribute(document, element, None, "name")?;
    let name =
        super::super::compile_expanded_qname(document, element, name, "xsl:attribute-set name")?;
    attribute_set_references(document, element)?;
    for child in meaningful_children(document, element) {
        if !is_xslt_element(document, child, "attribute") {
            return Err(invalid(
                "XTSE0010",
                "xsl:attribute-set content must contain only xsl:attribute instructions",
                document.location(child),
            ));
        }
        compile_attribute_set_attribute(document, child)?;
    }
    Ok(name)
}

pub(super) fn compile_attribute_set_use_names(
    document: &Document,
    element: NodeId,
    attribute_namespace: Option<&str>,
) -> Result<Vec<ExpandedName>, CompileFailure> {
    let Some(names) =
        optional_attribute(document, element, attribute_namespace, "use-attribute-sets")
    else {
        return Ok(Vec::new());
    };
    let requested = names
        .split_whitespace()
        .map(|name| {
            super::super::compile_expanded_qname(document, element, name, "xsl:use-attribute-sets")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if requested.is_empty() {
        return Err(invalid(
            "XTSE0020",
            "xsl:use-attribute-sets must name at least one attribute set",
            document.location(element),
        ));
    }
    Ok(requested)
}

pub(super) fn compile_attribute_set_declaration(
    document: &Document,
    element: NodeId,
) -> Result<AttributeSetDeclaration, CompileFailure> {
    let name = validate_local_attribute_set(document, element)?;
    let referenced_names = attribute_set_references(document, element)?;
    let mut dependency_names = Vec::new();
    collect_attribute_set_references(document, element, &mut dependency_names)?;
    let attributes = meaningful_children(document, element)
        .into_iter()
        .map(|child| compile_attribute_set_attribute(document, child))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(AttributeSetDeclaration {
        name,
        referenced_names,
        dependency_names,
        attributes,
        import_precedence: 0,
        location: document.location(element).clone(),
    })
}

fn compile_attribute_set_attribute(
    document: &Document,
    element: NodeId,
) -> Result<ComputedAttribute, CompileFailure> {
    let mut attribute = compile_computed_attribute(document, element)?;
    match &attribute.value {
        LiteralAttributeValue::Text(_) | LiteralAttributeValue::ContextStringValue => {}
        LiteralAttributeValue::Xslt10SequenceConstructor(_)
            if constructor_has_no_variable_reference(document, element) => {}
        LiteralAttributeValue::Variable(name) => {
            let stylesheet = containing_stylesheet(document, element)
                .expect("attribute-set attribute has stylesheet");
            let declared_globally = meaningful_children(document, stylesheet)
                .into_iter()
                .filter(|child| {
                    is_xslt_element(document, *child, "variable")
                        || is_xslt_element(document, *child, "param")
                })
                .any(|binding| {
                    optional_attribute(document, binding, None, "name") == Some(name.as_str())
                });
            if !declared_globally {
                return Err(invalid(
                    "XPST0008",
                    format!(
                        "attribute-set value references an undeclared global variable: ${name}"
                    ),
                    document.location(element),
                ));
            }
            attribute.value = LiteralAttributeValue::GlobalVariable(name.clone());
        }
        _ => {
            return Err(unsupported(
                "FXST1065",
                "the local attribute-set slice admits static text, the current source string value, a variable-free XSLT 1.0 constructor, or one global atomic variable value",
                document.location(element),
            ));
        }
    }
    Ok(attribute)
}

fn constructor_has_no_variable_reference(document: &Document, node: NodeId) -> bool {
    if document
        .attributes(node)
        .iter()
        .filter_map(|attribute| document.value(*attribute))
        .any(|value| value.contains('$'))
    {
        return false;
    }
    document
        .children(node)
        .iter()
        .all(|child| constructor_has_no_variable_reference(document, *child))
}

fn attribute_set_references(
    document: &Document,
    declaration: NodeId,
) -> Result<Vec<ExpandedName>, CompileFailure> {
    let Some(names) = optional_attribute(document, declaration, None, "use-attribute-sets") else {
        return Ok(Vec::new());
    };
    compile_attribute_set_reference_list(document, declaration, names)
}

fn compile_attribute_set_reference_list(
    document: &Document,
    element: NodeId,
    names: &str,
) -> Result<Vec<ExpandedName>, CompileFailure> {
    let names = names
        .split_whitespace()
        .map(|name| {
            super::super::compile_expanded_qname(document, element, name, "xsl:use-attribute-sets")
        })
        .collect::<Result<Vec<_>, _>>()?;
    if names.is_empty() {
        return Err(invalid(
            "XTSE0020",
            "xsl:use-attribute-sets must name at least one attribute set",
            document.location(element),
        ));
    }
    Ok(names)
}

fn collect_attribute_set_references(
    document: &Document,
    element: NodeId,
    references: &mut Vec<ExpandedName>,
) -> Result<(), CompileFailure> {
    const XSLT_NAMESPACE: &str = "http://www.w3.org/1999/XSL/Transform";
    for namespace in [None, Some(XSLT_NAMESPACE)] {
        let Some(names) = optional_attribute(document, element, namespace, "use-attribute-sets")
        else {
            continue;
        };
        references.extend(compile_attribute_set_reference_list(
            document, element, names,
        )?);
    }
    for child in document.children(element) {
        collect_attribute_set_references(document, *child, references)?;
    }
    Ok(())
}

fn containing_stylesheet(document: &Document, element: NodeId) -> Option<NodeId> {
    let mut current = document.parent(element);
    while let Some(node) = current {
        if is_xslt_element(document, node, "stylesheet")
            || is_xslt_element(document, node, "transform")
        {
            return Some(node);
        }
        current = document.parent(node);
    }
    None
}
