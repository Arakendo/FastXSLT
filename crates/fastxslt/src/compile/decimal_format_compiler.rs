//! Static XSLT 1.0 decimal-format compilation.

use std::collections::BTreeSet;

use super::{
    CompileFailure, Document, NodeId, compile_expanded_qname, ensure_no_meaningful_children,
    ensure_only_attributes, invalid, optional_attribute, unsupported,
};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::format_number_experiment::DecimalFormat;
use crate::xslt::golden_semantics_experiment::{
    DecimalFormatDefinition, Instruction, StylesheetProgram, TemplateArgument,
    TemplateArgumentValue, ValueExpression,
};

const PROPERTIES: [&str; 10] = [
    "decimal-separator",
    "grouping-separator",
    "infinity",
    "minus-sign",
    "NaN",
    "percent",
    "per-mille",
    "zero-digit",
    "digit",
    "pattern-separator",
];

#[derive(Debug)]
struct DecimalFormatDeclaration {
    format: DecimalFormat,
    specified: BTreeSet<&'static str>,
    location: SourceLocation,
}

#[derive(Debug, Default)]
pub(super) struct DecimalFormats {
    default: Option<DecimalFormatDeclaration>,
    named: Vec<(ExpandedName, DecimalFormatDeclaration)>,
}

impl DecimalFormats {
    pub(super) fn into_definitions(self) -> Vec<DecimalFormatDefinition> {
        self.default
            .into_iter()
            .map(|declaration| DecimalFormatDefinition {
                name: None,
                format: declaration.format,
                location: declaration.location,
            })
            .chain(
                self.named
                    .into_iter()
                    .map(|(name, declaration)| DecimalFormatDefinition {
                        name: Some(name),
                        format: declaration.format,
                        location: declaration.location,
                    }),
            )
            .collect()
    }
}

pub(super) fn compile_declaration(
    document: &Document,
    element: NodeId,
    declarations: &mut DecimalFormats,
) -> Result<(), CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &[
            "name",
            "decimal-separator",
            "grouping-separator",
            "infinity",
            "minus-sign",
            "NaN",
            "percent",
            "per-mille",
            "zero-digit",
            "digit",
            "pattern-separator",
        ],
        "xsl:decimal-format",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:decimal-format")?;
    let name = optional_attribute(document, element, None, "name")
        .map(|name| compile_expanded_qname(document, element, name, "xsl:decimal-format name"))
        .transpose()?;
    let target = declaration_for(declarations, name, document.location(element));
    for property in PROPERTIES {
        let Some(value) = optional_attribute(document, element, None, property) else {
            continue;
        };
        if target.specified.contains(property) && property_value(&target.format, property) != value
        {
            return Err(invalid(
                "XTSE1290",
                format!("conflicting default decimal-format property: {property}"),
                document.location(element),
            ));
        }
        set_property(document, element, &mut target.format, property, value)?;
        target.specified.insert(property);
    }
    Ok(())
}

fn declaration_for<'a>(
    declarations: &'a mut DecimalFormats,
    name: Option<ExpandedName>,
    location: &SourceLocation,
) -> &'a mut DecimalFormatDeclaration {
    let create = || DecimalFormatDeclaration {
        format: DecimalFormat::default(),
        specified: BTreeSet::new(),
        location: location.clone(),
    };
    let Some(name) = name else {
        return declarations.default.get_or_insert_with(create);
    };
    if let Some(index) = declarations
        .named
        .iter()
        .position(|(existing, _)| existing == &name)
    {
        return &mut declarations.named[index].1;
    }
    declarations.named.push((name, create()));
    &mut declarations
        .named
        .last_mut()
        .expect("inserted declaration")
        .1
}

fn set_property(
    document: &Document,
    element: NodeId,
    format: &mut DecimalFormat,
    property: &'static str,
    value: &str,
) -> Result<(), CompileFailure> {
    match property {
        "infinity" => value.clone_into(&mut format.infinity),
        "NaN" => value.clone_into(&mut format.nan),
        "decimal-separator" => {
            format.decimal_separator = one_character(document, element, property, value)?;
        }
        "grouping-separator" => {
            format.grouping_separator = one_character(document, element, property, value)?;
        }
        "minus-sign" => format.minus_sign = one_character(document, element, property, value)?,
        "percent" => format.percent = one_character(document, element, property, value)?,
        "per-mille" => format.per_mille = one_character(document, element, property, value)?,
        "zero-digit" => {
            let zero = one_character(document, element, property, value)?;
            let last = u32::from(zero).checked_add(9).and_then(char::from_u32);
            if last.is_none() {
                return Err(invalid(
                    "XTSE0020",
                    "xsl:decimal-format zero-digit must begin a ten-character scalar family",
                    document.location(element),
                ));
            }
            format.zero_digit = zero;
        }
        "digit" => format.digit = one_character(document, element, property, value)?,
        "pattern-separator" => {
            format.pattern_separator = one_character(document, element, property, value)?;
        }
        _ => unreachable!("known decimal-format property"),
    }
    Ok(())
}

fn one_character(
    document: &Document,
    element: NodeId,
    property: &str,
    value: &str,
) -> Result<char, CompileFailure> {
    let mut characters = value.chars();
    let Some(character) = characters.next() else {
        return Err(invalid(
            "XTSE0020",
            format!("xsl:decimal-format {property} must contain exactly one character"),
            document.location(element),
        ));
    };
    if characters.next().is_some() {
        return Err(invalid(
            "XTSE0020",
            format!("xsl:decimal-format {property} must contain exactly one character"),
            document.location(element),
        ));
    }
    Ok(character)
}

fn property_value(format: &DecimalFormat, property: &str) -> String {
    match property {
        "decimal-separator" => format.decimal_separator.to_string(),
        "grouping-separator" => format.grouping_separator.to_string(),
        "infinity" => format.infinity.clone(),
        "minus-sign" => format.minus_sign.to_string(),
        "NaN" => format.nan.clone(),
        "percent" => format.percent.to_string(),
        "per-mille" => format.per_mille.to_string(),
        "zero-digit" => format.zero_digit.to_string(),
        "digit" => format.digit.to_string(),
        "pattern-separator" => format.pattern_separator.to_string(),
        _ => unreachable!("known decimal-format property"),
    }
}

fn validate_distinct_symbols(
    format: &DecimalFormat,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    let symbols = [
        format.decimal_separator,
        format.grouping_separator,
        format.minus_sign,
        format.percent,
        format.per_mille,
        format.zero_digit,
        format.digit,
        format.pattern_separator,
    ];
    let distinct = symbols.iter().copied().collect::<BTreeSet<_>>();
    if distinct.len() != symbols.len() {
        return Err(invalid(
            "XTSE0020",
            "xsl:decimal-format active characters must be distinct",
            location,
        ));
    }
    Ok(())
}

pub(super) fn apply(
    program: &mut StylesheetProgram,
    declarations: &[DecimalFormatDefinition],
) -> Result<(), CompileFailure> {
    for declaration in declarations {
        validate_distinct_symbols(&declaration.format, &declaration.location)?;
    }
    if let Some(template) = &mut program.root_template {
        apply_instructions(&mut template.body, declarations)?;
    }
    for template in &mut program.matched_templates {
        apply_instructions(&mut template.template.body, declarations)?;
    }
    for template in &mut program.named_templates {
        apply_instructions(&mut template.template.body, declarations)?;
    }
    Ok(())
}

fn apply_instructions(
    instructions: &mut [Instruction],
    declarations: &[DecimalFormatDefinition],
) -> Result<(), CompileFailure> {
    for instruction in instructions {
        match instruction {
            Instruction::ValueOf { select, .. } => apply_value(select, declarations)?,
            Instruction::LiteralElement {
                computed_attributes,
                body,
                ..
            }
            | Instruction::ContextNameElement {
                computed_attributes,
                body,
                ..
            }
            | Instruction::DynamicNameElement {
                computed_attributes,
                body,
                ..
            } => {
                apply_computed_attributes(computed_attributes, declarations)?;
                apply_instructions(body, declarations)?;
            }
            Instruction::Attribute { attribute, .. } => {
                apply_computed_attribute(attribute, declarations)?;
            }
            Instruction::ForEachVariable { body, .. }
            | Instruction::ForEachStaticIntegerRange { body, .. }
            | Instruction::ForEachNodes { body, .. }
            | Instruction::If { body, .. }
            | Instruction::Xslt10SequenceTreeVariable { body, .. }
            | Instruction::Copy { body, .. } => apply_instructions(body, declarations)?,
            Instruction::Xslt10ProcessingInstructionNode { body, .. } => {
                apply_instructions(body.as_mut(), declarations)?;
            }
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                for branch in branches {
                    apply_instructions(&mut branch.body, declarations)?;
                }
                apply_instructions(otherwise, declarations)?;
            }
            Instruction::ApplyTemplates { arguments, .. }
            | Instruction::NextMatch { arguments, .. }
            | Instruction::ApplyImports { arguments, .. }
            | Instruction::CallTemplate { arguments, .. } => {
                apply_arguments(arguments, declarations)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn apply_computed_attributes(
    attributes: &mut [crate::xslt::golden_semantics_experiment::ComputedAttribute],
    declarations: &[DecimalFormatDefinition],
) -> Result<(), CompileFailure> {
    for attribute in attributes {
        apply_computed_attribute(attribute, declarations)?;
    }
    Ok(())
}

fn apply_computed_attribute(
    attribute: &mut crate::xslt::golden_semantics_experiment::ComputedAttribute,
    declarations: &[DecimalFormatDefinition],
) -> Result<(), CompileFailure> {
    if let crate::xslt::golden_semantics_experiment::LiteralAttributeValue::Xslt10SequenceConstructor(
        instructions,
    ) = &mut attribute.value
    {
        apply_instructions(instructions, declarations)?;
    }
    Ok(())
}

fn apply_arguments(
    arguments: &mut [TemplateArgument],
    declarations: &[DecimalFormatDefinition],
) -> Result<(), CompileFailure> {
    for argument in arguments {
        match &mut argument.value {
            TemplateArgumentValue::Xslt10Content(content) => {
                apply_value(&mut content.value, declarations)?;
            }
            TemplateArgumentValue::Xslt10SequenceConstructor(instructions) => {
                apply_instructions(instructions, declarations)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn apply_value(
    value: &mut ValueExpression,
    declarations: &[DecimalFormatDefinition],
) -> Result<(), CompileFailure> {
    if let ValueExpression::FormatNumber(expression)
    | ValueExpression::Xslt10NumberOfFormatNumber(expression) = value
    {
        let declaration = if let Some(name) = expression.requested_format() {
            declarations
                .iter()
                .find(|candidate| candidate.name.as_ref() == Some(name))
                .ok_or_else(|| {
                    unsupported(
                        "FXST1092",
                        format!(
                            "the requested decimal format is not declared: {}",
                            expression.requested_format_lexical().unwrap_or(&name.local)
                        ),
                        expression.location(),
                    )
                })?
        } else if let Some(declaration) = declarations
            .iter()
            .find(|candidate| candidate.name.is_none())
        {
            declaration
        } else {
            return Ok(());
        };
        expression.set_default_decimal_format(&declaration.format);
    }
    Ok(())
}
