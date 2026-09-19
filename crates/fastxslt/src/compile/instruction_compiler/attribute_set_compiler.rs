//! Compile-time validation and expansion of local static attribute sets.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{ComputedAttribute, LiteralAttributeValue};

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

pub(super) fn validate_local_attribute_set_graph(
    document: &Document,
    stylesheet: NodeId,
) -> Result<(), CompileFailure> {
    let mut validated = Vec::new();
    for declaration in meaningful_children(document, stylesheet)
        .into_iter()
        .filter(|child| is_xslt_element(document, *child, "attribute-set"))
    {
        let name = required_attribute(document, declaration, None, "name")?;
        let name = super::super::compile_expanded_qname(
            document,
            declaration,
            name,
            "xsl:attribute-set name",
        )?;
        if validated.contains(&name) {
            continue;
        }
        validate_dependencies(
            document,
            stylesheet,
            &name,
            &mut Vec::new(),
            document.location(declaration),
        )?;
        validated.push(name);
    }
    Ok(())
}

pub(super) fn compile_local_attribute_sets(
    document: &Document,
    element: NodeId,
    attribute_namespace: Option<&str>,
) -> Result<Vec<ComputedAttribute>, CompileFailure> {
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
    let stylesheet =
        containing_stylesheet(document, element).expect("result element has stylesheet");
    let mut values = Vec::new();
    for requested_name in requested {
        apply(
            document,
            stylesheet,
            &requested_name,
            &mut values,
            &mut Vec::new(),
            document.location(element),
        )?;
    }
    Ok(values)
}

fn apply(
    document: &Document,
    stylesheet: NodeId,
    requested_name: &ExpandedName,
    values: &mut Vec<ComputedAttribute>,
    active: &mut Vec<ExpandedName>,
    use_location: &SourceLocation,
) -> Result<(), CompileFailure> {
    if active.contains(requested_name) {
        return Err(invalid(
            "XTSE0720",
            "circular xsl:attribute-set references are not permitted",
            use_location,
        ));
    }
    let declarations = declarations(document, stylesheet, requested_name);
    if declarations.is_empty() {
        return Err(unsupported(
            "FXST1065",
            "the local attribute-set slice requires a declaration for every referenced name",
            use_location,
        ));
    }
    active.push(requested_name.clone());
    for declaration in declarations {
        for referenced_name in attribute_set_references(document, declaration)? {
            apply(
                document,
                stylesheet,
                &referenced_name,
                values,
                active,
                document.location(declaration),
            )?;
        }
        for child in meaningful_children(document, declaration) {
            let attribute = compile_attribute_set_attribute(document, child)?;
            if attribute.dynamic_name.is_none()
                && let Some(index) = values.iter().position(|existing| {
                    existing.dynamic_name.is_none() && existing.name == attribute.name
                })
            {
                values.remove(index);
            }
            values.push(attribute);
        }
    }
    active.pop();
    Ok(())
}

fn compile_attribute_set_attribute(
    document: &Document,
    element: NodeId,
) -> Result<ComputedAttribute, CompileFailure> {
    let mut attribute = compile_computed_attribute(document, element)?;
    match &attribute.value {
        LiteralAttributeValue::Text(_) => {}
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
                "the local attribute-set slice admits static text or one global atomic variable value",
                document.location(element),
            ));
        }
    }
    Ok(attribute)
}

fn validate_dependencies(
    document: &Document,
    stylesheet: NodeId,
    requested_name: &ExpandedName,
    active: &mut Vec<ExpandedName>,
    reference_location: &SourceLocation,
) -> Result<(), CompileFailure> {
    if active.contains(requested_name) {
        return Err(invalid(
            "XTSE0720",
            "circular xsl:attribute-set references are not permitted",
            reference_location,
        ));
    }
    let declarations = declarations(document, stylesheet, requested_name);
    if declarations.is_empty() {
        return Err(invalid(
            "XTSE0710",
            "xsl:attribute-set references an undefined attribute set",
            reference_location,
        ));
    }
    active.push(requested_name.clone());
    for declaration in declarations {
        for referenced_name in attribute_set_references(document, declaration)? {
            validate_dependencies(
                document,
                stylesheet,
                &referenced_name,
                active,
                document.location(declaration),
            )?;
        }
    }
    active.pop();
    Ok(())
}

fn declarations(
    document: &Document,
    stylesheet: NodeId,
    requested_name: &ExpandedName,
) -> Vec<NodeId> {
    meaningful_children(document, stylesheet)
        .into_iter()
        .filter(|child| is_xslt_element(document, *child, "attribute-set"))
        .filter(|candidate| {
            let Some(lexical) = optional_attribute(document, *candidate, None, "name") else {
                return false;
            };
            super::super::compile_expanded_qname(
                document,
                *candidate,
                lexical,
                "xsl:attribute-set name",
            )
            .is_ok_and(|name| name == *requested_name)
        })
        .collect()
}

fn attribute_set_references(
    document: &Document,
    declaration: NodeId,
) -> Result<Vec<ExpandedName>, CompileFailure> {
    let Some(names) = optional_attribute(document, declaration, None, "use-attribute-sets") else {
        return Ok(Vec::new());
    };
    let names = names
        .split_whitespace()
        .map(|name| {
            super::super::compile_expanded_qname(
                document,
                declaration,
                name,
                "xsl:use-attribute-sets",
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if names.is_empty() {
        return Err(invalid(
            "XTSE0020",
            "xsl:use-attribute-sets must name at least one attribute set",
            document.location(declaration),
        ));
    }
    Ok(names)
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
