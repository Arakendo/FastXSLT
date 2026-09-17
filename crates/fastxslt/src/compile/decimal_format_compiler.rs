//! Static unnamed XSLT 1.0 decimal-format compilation.

use std::collections::BTreeSet;

use super::{
    CompileFailure, Document, NodeId, ensure_no_meaningful_children, ensure_only_attributes,
    invalid, optional_attribute, unsupported,
};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xpath::format_number_experiment::DecimalFormat;
use crate::xslt::golden_semantics_experiment::{
    Instruction, StylesheetProgram, TemplateArgument, TemplateArgumentValue, ValueExpression,
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
pub(super) struct DefaultDecimalFormat {
    format: DecimalFormat,
    specified: BTreeSet<&'static str>,
    location: SourceLocation,
}

pub(super) fn compile_default_declaration(
    document: &Document,
    element: NodeId,
    declaration: &mut Option<DefaultDecimalFormat>,
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
    if let Some(name) = optional_attribute(document, element, None, "name") {
        return Err(unsupported(
            "FXST1090",
            format!("named decimal formats remain outside the private slice: {name}"),
            document.location(element),
        ));
    }

    let target = declaration.get_or_insert_with(|| DefaultDecimalFormat {
        format: DecimalFormat::default(),
        specified: BTreeSet::new(),
        location: document.location(element).clone(),
    });
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
        "zero-digit" if value != "0" => {
            return Err(unsupported(
                "FXST1091",
                "non-ASCII zero-digit formatting remains outside the private slice",
                document.location(element),
            ));
        }
        "decimal-separator" => {
            format.decimal_separator = one_character(document, element, property, value)?;
        }
        "grouping-separator" => {
            format.grouping_separator = one_character(document, element, property, value)?;
        }
        "minus-sign" => format.minus_sign = one_character(document, element, property, value)?,
        "percent" => format.percent = one_character(document, element, property, value)?,
        "per-mille" => format.per_mille = one_character(document, element, property, value)?,
        "zero-digit" => format.zero_digit = '0',
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
    declaration: &DefaultDecimalFormat,
) -> Result<(), CompileFailure> {
    validate_distinct_symbols(&declaration.format, &declaration.location)?;
    if let Some(template) = &mut program.root_template {
        apply_instructions(&mut template.body, &declaration.format);
    }
    for template in &mut program.matched_templates {
        apply_instructions(&mut template.template.body, &declaration.format);
    }
    for template in &mut program.named_templates {
        apply_instructions(&mut template.template.body, &declaration.format);
    }
    Ok(())
}

fn apply_instructions(instructions: &mut [Instruction], format: &DecimalFormat) {
    for instruction in instructions {
        match instruction {
            Instruction::ValueOf { select, .. } => apply_value(select, format),
            Instruction::LiteralElement { body, .. }
            | Instruction::ForEachVariable { body, .. }
            | Instruction::ForEachStaticIntegerRange { body, .. }
            | Instruction::ForEachNodes { body, .. }
            | Instruction::If { body, .. }
            | Instruction::Copy { body, .. } => apply_instructions(body, format),
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                for branch in branches {
                    apply_instructions(&mut branch.body, format);
                }
                apply_instructions(otherwise, format);
            }
            Instruction::ApplyTemplates { arguments, .. }
            | Instruction::NextMatch { arguments, .. }
            | Instruction::ApplyImports { arguments, .. }
            | Instruction::CallTemplate { arguments, .. } => apply_arguments(arguments, format),
            _ => {}
        }
    }
}

fn apply_arguments(arguments: &mut [TemplateArgument], format: &DecimalFormat) {
    for argument in arguments {
        if let TemplateArgumentValue::Xslt10Content(content) = &mut argument.value {
            apply_value(&mut content.value, format);
        }
    }
}

fn apply_value(value: &mut ValueExpression, format: &DecimalFormat) {
    if let ValueExpression::FormatNumber(expression) = value {
        expression.set_default_decimal_format(format);
    }
}
