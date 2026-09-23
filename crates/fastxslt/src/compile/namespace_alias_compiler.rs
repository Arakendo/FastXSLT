//! Static XSLT 1.0 namespace-alias compilation and result-name rewriting.

use std::sync::Arc;

use super::{
    CompileFailure, Document, NodeId, XSLT_NAMESPACE, ensure_no_meaningful_children,
    ensure_only_attributes, invalid, is_ascii_ncname, namespace_for_prefix, optional_attribute,
};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xslt::golden_semantics_experiment::{
    ConstructedElement, ConstructedNode, ElementConstructorOrigin, GlobalBindingDefault,
    Instruction, StylesheetProgram,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NamespaceAlias {
    stylesheet_namespace: Option<String>,
    result_namespace: Option<String>,
    result_prefix: Option<String>,
}

pub(super) fn compile_declaration(
    document: &Document,
    element: NodeId,
    aliases: &mut Vec<NamespaceAlias>,
) -> Result<(), CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["stylesheet-prefix", "result-prefix"],
        "xsl:namespace-alias",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:namespace-alias")?;
    let stylesheet_prefix = required_prefix(document, element, "stylesheet-prefix")?;
    let result_prefix = required_prefix(document, element, "result-prefix")?;
    let stylesheet_namespace = resolve_prefix(document, element, &stylesheet_prefix)?;
    let result_namespace = resolve_prefix(document, element, &result_prefix)?;
    let result_prefix = (result_prefix != "#default").then_some(result_prefix);
    let alias = NamespaceAlias {
        stylesheet_namespace,
        result_namespace,
        result_prefix,
    };
    if let Some(existing) = aliases
        .iter()
        .find(|existing| existing.stylesheet_namespace == alias.stylesheet_namespace)
    {
        if existing != &alias {
            return Err(invalid(
                "XTSE0810",
                "conflicting namespace aliases have the same import precedence",
                document.location(element),
            ));
        }
        return Ok(());
    }
    aliases.push(alias);
    Ok(())
}

fn required_prefix(
    document: &Document,
    element: NodeId,
    local: &str,
) -> Result<String, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, local) else {
        return Err(invalid(
            "XTSE0010",
            format!("xsl:namespace-alias requires {local}"),
            document.location(element),
        ));
    };
    if value != "#default" && !is_ascii_ncname(value) {
        return Err(invalid(
            "XTSE0020",
            format!("invalid xsl:namespace-alias {local}: {value}"),
            document.location(element),
        ));
    }
    Ok(value.to_owned())
}

fn resolve_prefix(
    document: &Document,
    element: NodeId,
    prefix: &str,
) -> Result<Option<String>, CompileFailure> {
    if prefix == "#default" {
        return Ok(namespace_for_prefix(document, element, "").map(str::to_owned));
    }
    let namespace = if prefix == "xsl" {
        Some(XSLT_NAMESPACE)
    } else {
        namespace_for_prefix(document, element, prefix)
    };
    namespace
        .map(|namespace| Some(namespace.to_owned()))
        .ok_or_else(|| {
            invalid(
                "XTSE0812",
                format!("xsl:namespace-alias prefix is not bound: {prefix}"),
                document.location(element),
            )
        })
}

pub(super) fn apply(program: &mut StylesheetProgram, aliases: &[NamespaceAlias]) {
    if aliases.is_empty() {
        return;
    }
    if let Some(template) = &mut program.root_template {
        apply_instructions(&mut template.body, aliases);
    }
    for template in &mut program.matched_templates {
        apply_instructions(&mut template.template.body, aliases);
    }
    for template in &mut program.named_templates {
        apply_instructions(&mut template.template.body, aliases);
    }
    for binding in &mut program.global_bindings {
        if let GlobalBindingDefault::TemporaryTree(nodes) = &mut binding.default {
            for node in nodes {
                apply_constructed_node(node, aliases);
            }
        }
    }
}

pub(super) fn overlay_higher_precedence(
    aliases: &mut Vec<NamespaceAlias>,
    higher_precedence: &[NamespaceAlias],
) {
    aliases.retain(|alias| {
        !higher_precedence
            .iter()
            .any(|higher| higher.stylesheet_namespace == alias.stylesheet_namespace)
    });
    aliases.extend_from_slice(higher_precedence);
}

fn apply_instructions(instructions: &mut [Instruction], aliases: &[NamespaceAlias]) {
    for instruction in instructions {
        match instruction {
            Instruction::LiteralElement {
                origin,
                name,
                namespaces,
                attributes,
                body,
                ..
            } => {
                if *origin == ElementConstructorOrigin::Literal {
                    apply_name(name, aliases);
                    for attribute in attributes.iter_mut() {
                        apply_attribute_name(&mut attribute.name, aliases);
                    }
                    let mut owned = namespaces.to_vec();
                    apply_namespaces(&mut owned, aliases);
                    ensure_name_namespace(&mut owned, name, aliases);
                    for attribute in attributes.iter() {
                        ensure_name_namespace(&mut owned, &attribute.name, aliases);
                    }
                    *namespaces = Arc::from(owned);
                }
                apply_instructions(body, aliases);
            }
            Instruction::TemporaryTreeVariable { elements, .. } => {
                for element in elements {
                    apply_constructed_element(element, aliases);
                }
            }
            Instruction::ForEachVariable { body, .. }
            | Instruction::ForEachStaticIntegerRange { body, .. }
            | Instruction::ForEachNodes { body, .. }
            | Instruction::Xslt10SequenceTreeVariable { body, .. }
            | Instruction::If { body, .. }
            | Instruction::Copy { body, .. } => apply_instructions(body, aliases),
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                for branch in branches {
                    apply_instructions(&mut branch.body, aliases);
                }
                apply_instructions(otherwise, aliases);
            }
            _ => {}
        }
    }
}

fn apply_constructed_element(element: &mut ConstructedElement, aliases: &[NamespaceAlias]) {
    apply_name(&mut element.name, aliases);
    for attribute in &mut element.attributes {
        apply_attribute_name(&mut attribute.name, aliases);
    }
    apply_namespaces(&mut element.namespaces, aliases);
    ensure_name_namespace(&mut element.namespaces, &element.name, aliases);
    for attribute in &element.attributes {
        ensure_name_namespace(&mut element.namespaces, &attribute.name, aliases);
    }
    for child in &mut element.children {
        if let ConstructedNode::Element(child) = child {
            apply_constructed_element(child, aliases);
        }
    }
}

fn apply_constructed_node(node: &mut ConstructedNode, aliases: &[NamespaceAlias]) {
    if let ConstructedNode::Element(element) = node {
        apply_constructed_element(element, aliases);
    }
}

fn apply_name(name: &mut ExpandedName, aliases: &[NamespaceAlias]) {
    if let Some(alias) = aliases
        .iter()
        .find(|alias| alias.stylesheet_namespace == name.namespace)
    {
        name.namespace.clone_from(&alias.result_namespace);
    }
}

fn apply_attribute_name(name: &mut ExpandedName, aliases: &[NamespaceAlias]) {
    if name.namespace.is_some() {
        apply_name(name, aliases);
    }
}

fn apply_namespaces(namespaces: &mut Vec<NamespaceBinding>, aliases: &[NamespaceAlias]) {
    for binding in namespaces.iter_mut() {
        if let Some(alias) = aliases
            .iter()
            .find(|alias| alias.stylesheet_namespace.as_deref() == Some(&binding.namespace))
        {
            binding.prefix.clone_from(&alias.result_prefix);
            if let Some(result_namespace) = &alias.result_namespace {
                binding.namespace.clone_from(result_namespace);
            }
        }
    }
    namespaces.dedup();
}

fn ensure_name_namespace(
    namespaces: &mut Vec<NamespaceBinding>,
    name: &ExpandedName,
    aliases: &[NamespaceAlias],
) {
    let Some(namespace) = name.namespace.as_deref() else {
        return;
    };
    let Some(alias) = aliases
        .iter()
        .find(|alias| alias.result_namespace.as_deref() == Some(namespace))
    else {
        return;
    };
    if !namespaces.iter().any(|binding| {
        binding.prefix == alias.result_prefix
            && alias.result_namespace.as_deref() == Some(binding.namespace.as_str())
    }) {
        namespaces.push(NamespaceBinding {
            prefix: alias.result_prefix.clone(),
            namespace: namespace.to_owned(),
        });
    }
}
