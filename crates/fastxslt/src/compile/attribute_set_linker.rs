//! Links stylesheet-package attribute-set declarations into result constructors.

use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xslt::golden_semantics_experiment::{
    AttributeSetDeclaration, ComputedAttribute, Instruction, LiteralAttribute,
    LiteralAttributeValue, StylesheetProgram, TemplateArgument, TemplateArgumentValue,
    TemplateParameterDefault,
};

use super::instruction_compiler::retain_computed_attribute_namespace_bindings;
use super::{CompileFailure, invalid, unsupported};

pub(super) fn link(program: &mut StylesheetProgram) -> Result<(), CompileFailure> {
    let mut declarations = program.attribute_set_declarations.clone();
    declarations.sort_by_key(|declaration| declaration.import_precedence);
    validate_graph(&declarations)?;

    if let Some(template) = &mut program.root_template {
        link_template(template, &declarations)?;
    }
    for matched in &mut program.matched_templates {
        link_template(&mut matched.template, &declarations)?;
    }
    for named in &mut program.named_templates {
        link_template(&mut named.template, &declarations)?;
    }
    Ok(())
}

fn validate_graph(declarations: &[AttributeSetDeclaration]) -> Result<(), CompileFailure> {
    let mut validated = Vec::new();
    for declaration in declarations {
        if validated.contains(&declaration.name) {
            continue;
        }
        validate_dependencies(
            &declaration.name,
            declarations,
            &mut Vec::new(),
            &declaration.location,
        )?;
        validated.push(declaration.name.clone());
    }
    Ok(())
}

fn validate_dependencies(
    requested_name: &ExpandedName,
    declarations: &[AttributeSetDeclaration],
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
    let matching = declarations
        .iter()
        .filter(|declaration| declaration.name == *requested_name)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return Err(invalid(
            "XTSE0710",
            "xsl:attribute-set references an undefined attribute set",
            reference_location,
        ));
    }
    active.push(requested_name.clone());
    for declaration in matching {
        for dependency in &declaration.dependency_names {
            validate_dependencies(dependency, declarations, active, &declaration.location)?;
        }
    }
    active.pop();
    Ok(())
}

fn resolve(
    requested_name: &ExpandedName,
    declarations: &[AttributeSetDeclaration],
    values: &mut Vec<ComputedAttribute>,
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
    let matching = declarations
        .iter()
        .filter(|declaration| declaration.name == *requested_name)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return Err(invalid(
            "XTSE0710",
            "xsl:use-attribute-sets references an undefined attribute set",
            reference_location,
        ));
    }

    active.push(requested_name.clone());
    for declaration in matching {
        for referenced_name in &declaration.referenced_names {
            resolve(
                referenced_name,
                declarations,
                values,
                active,
                &declaration.location,
            )?;
        }
        for attribute in &declaration.attributes {
            push_with_static_override(values, attribute.clone());
        }
    }
    active.pop();
    Ok(())
}

fn push_with_static_override(values: &mut Vec<ComputedAttribute>, attribute: ComputedAttribute) {
    if attribute.dynamic_name.is_none()
        && let Some(index) = values
            .iter()
            .position(|existing| existing.dynamic_name.is_none() && existing.name == attribute.name)
    {
        values.remove(index);
    }
    values.push(attribute);
}

fn resolved_use(
    names: &[ExpandedName],
    declarations: &[AttributeSetDeclaration],
    location: &SourceLocation,
) -> Result<Vec<ComputedAttribute>, CompileFailure> {
    let mut values = Vec::new();
    for name in names {
        resolve(name, declarations, &mut values, &mut Vec::new(), location)?;
    }
    Ok(values)
}

fn link_template(
    template: &mut crate::xslt::golden_semantics_experiment::Template,
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    for parameter in &mut template.parameters {
        if let TemplateParameterDefault::Xslt10SequenceConstructor(body) = &mut parameter.default {
            link_instructions(body, declarations)?;
        }
    }
    link_instructions(&mut template.body, declarations)
}

fn link_instructions(
    instructions: &mut [Instruction],
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    for instruction in instructions {
        match instruction {
            Instruction::LiteralElement {
                namespaces,
                attributes,
                attribute_set_names,
                computed_attributes,
                body,
                location,
                ..
            } => link_literal_constructor(
                namespaces,
                attributes,
                attribute_set_names,
                computed_attributes,
                body,
                location,
                declarations,
            )?,
            Instruction::ContextNameElement {
                static_namespaces,
                attribute_set_names,
                computed_attributes,
                body,
                location,
                ..
            }
            | Instruction::DynamicNameElement {
                static_namespaces,
                attribute_set_names,
                computed_attributes,
                body,
                location,
                ..
            } => link_dynamic_constructor(
                static_namespaces,
                attribute_set_names,
                computed_attributes,
                body,
                location,
                declarations,
            )?,
            Instruction::Copy {
                attributes,
                attribute_set_names,
                body,
                location,
                ..
            } => link_copy_constructor(
                attributes,
                attribute_set_names,
                body,
                location,
                declarations,
            )?,
            Instruction::Attribute { attribute, .. } => {
                link_attribute_value(&mut attribute.value, declarations)?;
            }
            Instruction::ForEachVariable { body, .. }
            | Instruction::ForEachStaticIntegerRange { body, .. }
            | Instruction::ForEachNodes { body, .. }
            | Instruction::Xslt10SequenceTreeVariable { body, .. }
            | Instruction::Xslt10Message { body, .. }
            | Instruction::If { body, .. } => link_instructions(body, declarations)?,
            Instruction::Xslt10ProcessingInstructionNode { body, .. }
            | Instruction::Xslt10CommentNode { body, .. } => {
                link_instructions(body, declarations)?;
            }
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                for branch in branches {
                    link_instructions(&mut branch.body, declarations)?;
                }
                link_instructions(otherwise, declarations)?;
            }
            Instruction::ApplyTemplates { arguments, .. }
            | Instruction::NextMatch { arguments, .. }
            | Instruction::ApplyImports { arguments, .. }
            | Instruction::CallTemplate { arguments, .. } => {
                link_arguments(arguments, declarations)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn link_literal_constructor(
    namespaces: &mut std::sync::Arc<[NamespaceBinding]>,
    attributes: &[LiteralAttribute],
    attribute_set_names: &mut Vec<ExpandedName>,
    computed_attributes: &mut Vec<ComputedAttribute>,
    body: &mut [Instruction],
    location: &SourceLocation,
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    let mut set_values = resolved_use(attribute_set_names, declarations, location)?;
    set_values.retain(|set_attribute| {
        !attributes.iter().any(|attribute| {
            set_attribute.dynamic_name.is_none() && attribute.name == set_attribute.name
        }) && !computed_attributes.iter().any(|attribute| {
            attribute.dynamic_name.is_none()
                && set_attribute.dynamic_name.is_none()
                && attribute.name == set_attribute.name
        })
    });
    set_values.append(computed_attributes);
    *computed_attributes = set_values;
    attribute_set_names.clear();
    retain_namespaces(namespaces, computed_attributes);
    link_computed_attributes(computed_attributes, declarations)?;
    link_instructions(body, declarations)
}

fn link_dynamic_constructor(
    namespaces: &mut std::sync::Arc<[NamespaceBinding]>,
    attribute_set_names: &mut Vec<ExpandedName>,
    computed_attributes: &mut Vec<ComputedAttribute>,
    body: &mut [Instruction],
    location: &SourceLocation,
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    let mut set_values = resolved_use(attribute_set_names, declarations, location)?;
    set_values.retain(|set_attribute| {
        !computed_attributes.iter().any(|attribute| {
            attribute.dynamic_name.is_none()
                && set_attribute.dynamic_name.is_none()
                && attribute.name == set_attribute.name
        })
    });
    set_values.append(computed_attributes);
    *computed_attributes = set_values;
    attribute_set_names.clear();
    retain_namespaces(namespaces, computed_attributes);
    link_computed_attributes(computed_attributes, declarations)?;
    link_instructions(body, declarations)
}

fn link_copy_constructor(
    attributes: &mut Vec<LiteralAttribute>,
    attribute_set_names: &mut Vec<ExpandedName>,
    body: &mut [Instruction],
    location: &SourceLocation,
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    let set_values = resolved_use(attribute_set_names, declarations, location)?;
    let mut linked = set_values
        .into_iter()
        .map(computed_to_literal)
        .collect::<Result<Vec<_>, _>>()?;
    linked.retain(|set_attribute| {
        !attributes
            .iter()
            .any(|attribute| attribute.name == set_attribute.name)
    });
    linked.append(attributes);
    *attributes = linked;
    attribute_set_names.clear();
    link_literal_attributes(attributes, declarations)?;
    link_instructions(body, declarations)
}

fn retain_namespaces(
    namespaces: &mut std::sync::Arc<[NamespaceBinding]>,
    attributes: &[ComputedAttribute],
) {
    let mut owned = namespaces.to_vec();
    retain_computed_attribute_namespace_bindings(&mut owned, attributes);
    *namespaces = owned.into();
}

fn computed_to_literal(attribute: ComputedAttribute) -> Result<LiteralAttribute, CompileFailure> {
    if attribute.dynamic_name.is_some() {
        return Err(unsupported(
            "FXST1065",
            "dynamic attribute-set names on xsl:copy remain outside the linked attribute-set slice",
            &attribute.location,
        ));
    }
    Ok(LiteralAttribute {
        name: attribute.name,
        value: attribute.value,
        location: attribute.location,
    })
}

fn link_computed_attributes(
    attributes: &mut [ComputedAttribute],
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    for attribute in attributes {
        link_attribute_value(&mut attribute.value, declarations)?;
    }
    Ok(())
}

fn link_literal_attributes(
    attributes: &mut [LiteralAttribute],
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    for attribute in attributes {
        link_attribute_value(&mut attribute.value, declarations)?;
    }
    Ok(())
}

fn link_attribute_value(
    value: &mut LiteralAttributeValue,
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    if let LiteralAttributeValue::Xslt10SequenceConstructor(body) = value {
        link_instructions(body, declarations)?;
    }
    Ok(())
}

fn link_arguments(
    arguments: &mut [TemplateArgument],
    declarations: &[AttributeSetDeclaration],
) -> Result<(), CompileFailure> {
    for argument in arguments {
        if let TemplateArgumentValue::Xslt10SequenceConstructor(body) = &mut argument.value {
            link_instructions(body, declarations)?;
        }
    }
    Ok(())
}
