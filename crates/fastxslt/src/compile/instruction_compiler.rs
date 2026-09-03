//! Private compilation of XSLT sequence constructors and instructions.

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::NamespaceBinding;
use crate::xpath::castable_experiment::{parse as parse_castable, parse_cast};
use crate::xpath::constant_numeric_experiment::{self, ConstantNumericFailure};
use crate::xpath::decimal_sum_for_experiment::parse as parse_decimal_sum_for;
use crate::xpath::deep_equal_boolean_experiment::{
    parse as parse_deep_equal, recognizes as recognizes_deep_equal,
};
use crate::xpath::deep_equal_experiment::DeepEqualFailureKind;
use crate::xpath::focus_sum_for_experiment::parse as parse_focus_sum_for;
use crate::xpath::for_distinct_values_experiment::{
    ForExpressionFailure, parse as parse_for_distinct_values,
};
use crate::xpath::format_number_experiment::parse as parse_format_number;
use crate::xpath::integer_for_experiment::parse as parse_integer_for;
use crate::xpath::path_experiment::{PathFailure, parse_location_path, parse_qualified_child_path};
use crate::xslt::golden_semantics_experiment::{
    BooleanExpression, ChooseBranch, ComputedAttribute, EqualityTest, Instruction,
    LiteralAttributeValue, SequenceItemExpression, TemplateArgument, ValueExpression,
};

#[path = "instruction_compiler/computed_attribute_compiler.rs"]
mod computed_attribute_compiler;
use computed_attribute_compiler::compile_computed_attributes;
#[path = "instruction_compiler/literal_attribute_compiler.rs"]
mod literal_attribute_compiler;
pub(super) use literal_attribute_compiler::compile_literal_result_attributes;
#[path = "instruction_compiler/source_copy_compiler.rs"]
mod source_copy_compiler;
use source_copy_compiler::compile_copy;
#[path = "instruction_compiler/template_invocation_compiler.rs"]
mod template_invocation_compiler;

pub(super) fn parse_mode(
    document: &Document,
    element: NodeId,
    mode: &str,
) -> Result<String, CompileFailure> {
    template_invocation_compiler::parse_mode(document, element, mode)
}

use super::{
    CompileCategory, CompileFailure, XML_SCHEMA_NAMESPACE, XSLT_NAMESPACE, effective_default_mode,
    effective_xpath_default_namespace, ensure_no_meaningful_children, ensure_only_attributes,
    invalid, is_ascii_ncname, is_xslt_element, map_path_failure, meaningful_children,
    normalize_named_template_name, normalize_variable_qname, optional_attribute,
    required_attribute, unsupported,
};

fn compile_sequence(
    document: &Document,
    parent: NodeId,
) -> Result<Vec<Instruction>, CompileFailure> {
    compile_sequence_excluding(document, parent, &[])
}

pub(super) fn compile_sequence_excluding(
    document: &Document,
    parent: NodeId,
    excluded: &[NodeId],
) -> Result<Vec<Instruction>, CompileFailure> {
    let mut instructions = Vec::new();
    let mut local_variables = Vec::new();
    for child in document
        .children(parent)
        .iter()
        .copied()
        .filter(|child| !excluded.contains(child))
    {
        match document.kind(child) {
            NodeKind::Text => {
                let value = document.value(child).unwrap_or_default();
                if !value.chars().all(char::is_whitespace) {
                    instructions.push(Instruction::Text {
                        value: value.to_owned(),
                        location: document.location(child).clone(),
                    });
                }
            }
            NodeKind::Comment | NodeKind::ProcessingInstruction => {}
            NodeKind::Element => {
                let name = document.name(child).expect("element nodes have names");
                if name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
                    if name.local == "text" {
                        instructions.push(compile_text(document, child)?);
                    } else if name.local == "comment" {
                        instructions.push(compile_comment(document, child)?);
                    } else if name.local == "attribute" {
                        instructions.push(compile_attribute(document, child)?);
                    } else if name.local == "processing-instruction" {
                        instructions.push(compile_processing_instruction(document, child)?);
                    } else if name.local == "value-of" {
                        instructions.push(compile_value_of(document, child)?);
                    } else if name.local == "variable" {
                        let variable = compile_variable(document, child)?;
                        let name = local_variable_name(&variable);
                        if local_variables.contains(name) {
                            return Err(invalid(
                                "FXST0017",
                                format!("duplicate local variable binding: ${name}"),
                                document.location(child),
                            ));
                        }
                        local_variables.push(name.clone());
                        instructions.push(variable);
                    } else if name.local == "sequence" {
                        instructions.push(compile_sequence_nodes(document, child)?);
                    } else if name.local == "apply-templates" {
                        instructions.push(compile_apply_templates(document, child)?);
                    } else if name.local == "next-match" {
                        ensure_only_attributes(document, child, &[], "xsl:next-match")?;
                        instructions.push(Instruction::NextMatch {
                            arguments: compile_with_params(
                                document,
                                child,
                                "xsl:next-match",
                                true,
                            )?,
                            location: document.location(child).clone(),
                        });
                    } else if name.local == "apply-imports" {
                        instructions.push(compile_apply_imports(document, child)?);
                    } else if name.local == "for-each" {
                        instructions.push(compile_for_each(document, child)?);
                    } else if name.local == "if" {
                        instructions.push(compile_if(document, child)?);
                    } else if name.local == "choose" {
                        instructions.push(compile_choose(document, child)?);
                    } else if name.local == "call-template" {
                        instructions.push(compile_call_template(document, child)?);
                    } else if name.local == "copy" {
                        instructions.push(compile_copy(document, child)?);
                    } else if name.local == "copy-of" {
                        instructions.push(compile_copy_of(document, child)?);
                    } else {
                        return Err(unsupported(
                            "FXST1006",
                            format!("unsupported XSLT instruction: xsl:{}", name.local),
                            document.location(child),
                        ));
                    }
                } else {
                    instructions.push(compile_literal_element(document, child)?);
                }
            }
            NodeKind::Document | NodeKind::Attribute => {
                return Err(invalid(
                    "FXST0006",
                    "unexpected node kind in stylesheet sequence",
                    document.location(child),
                ));
            }
        }
    }
    Ok(instructions)
}

fn local_variable_name(variable: &Instruction) -> &String {
    let (Instruction::Variable { name, .. }
    | Instruction::ContextPositionVariable { name, .. }
    | Instruction::SourceNodeVariable { name, .. }
    | Instruction::IntegerRangeVariable { name, .. }
    | Instruction::TemporaryTreeVariable { name, .. }) = variable
    else {
        unreachable!("compile_variable returns a variable instruction")
    };
    name
}

fn compile_attribute(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["name", "select"], "xsl:attribute")?;
    ensure_no_meaningful_children(document, element, "xsl:attribute")?;
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(unsupported(
            "FXST1033",
            "the private standalone xsl:attribute slice requires an unprefixed static NCName",
            document.location(element),
        ));
    }
    let select = required_attribute(document, element, None, "select")?;
    let value = match select.split_whitespace().collect::<String>().as_str() {
        ".+1" => LiteralAttributeValue::ContextIntegerIncrement(1),
        _ => {
            return Err(unsupported(
                "FXXP1012",
                format!("unsupported standalone xsl:attribute value expression: {select}"),
                document.location(element),
            ));
        }
    };
    Ok(Instruction::Attribute {
        attribute: ComputedAttribute {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: name.to_owned(),
            },
            value,
            location: document.location(element).clone(),
        },
        location: document.location(element).clone(),
    })
}

fn compile_literal_element(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_literal_result_control_attributes(document, element)?;
    let (computed_attributes, computed_attribute_nodes) =
        compile_computed_attributes(document, element)?;
    let attributes = compile_literal_result_attributes(document, element)?;
    ensure_distinct_result_attributes(
        &attributes,
        &computed_attributes,
        document.location(element),
    )?;
    Ok(Instruction::LiteralElement {
        name: document
            .name(element)
            .expect("literal result element has a name")
            .clone(),
        namespaces: literal_result_namespaces(document, element),
        attributes,
        computed_attributes,
        body: compile_sequence_excluding(document, element, &computed_attribute_nodes)?,
        location: document.location(element).clone(),
    })
}

fn ensure_distinct_result_attributes(
    literal: &[crate::xslt::golden_semantics_experiment::LiteralAttribute],
    computed: &[crate::xslt::golden_semantics_experiment::ComputedAttribute],
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    for (index, attribute) in computed.iter().enumerate() {
        if literal
            .iter()
            .any(|existing| existing.name == attribute.name)
            || computed[..index]
                .iter()
                .any(|existing| existing.name == attribute.name)
        {
            return Err(invalid(
                "XTDE0410",
                format!("duplicate result attribute: {}", attribute.name.local),
                location,
            ));
        }
    }
    Ok(())
}

fn compile_apply_imports(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    template_invocation_compiler::compile_apply_imports(document, element)
}

fn compile_copy_of(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:copy-of")?;
    ensure_no_meaningful_children(document, element, "xsl:copy-of")?;
    let select = required_attribute(document, element, None, "select")?;
    match select.trim() {
        "." => Ok(Instruction::CopyOfCurrent {
            location: document.location(element).clone(),
        }),
        "*" => Ok(Instruction::CopyOfChildElements {
            location: document.location(element).clone(),
        }),
        "ancestor-or-self::*" => Ok(Instruction::CopyOfAncestorOrSelfElements {
            location: document.location(element).clone(),
        }),
        _ => Err(unsupported(
            "FXXP1003",
            format!("unsupported xsl:copy-of selection: {select}"),
            document.location(element),
        )),
    }
}

fn compile_for_each(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    let select = required_attribute(document, element, None, "select")?;
    if let Some(variable) = select
        .trim()
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        ensure_only_attributes(
            document,
            element,
            &["select", "default-mode"],
            "xsl:for-each",
        )?;
        return Ok(Instruction::ForEachTemporaryRoot {
            variable: variable.to_owned(),
            body: compile_sequence(document, element)?,
            location: document.location(element).clone(),
        });
    }
    if let Some((start, end)) = parse_static_integer_range(select) {
        for child in meaningful_children(document, element) {
            if is_xslt_element(document, child, "apply-templates") {
                let apply_select = optional_attribute(document, child, None, "select");
                if apply_select.is_none_or(|value| value.trim() == ".") {
                    return Err(invalid(
                        "XTTE0510",
                        "xsl:apply-templates requires nodes, but the statically known focus is an integer",
                        document.location(child),
                    ));
                }
            }
        }
        ensure_only_attributes(
            document,
            element,
            &["select", "default-mode"],
            "xsl:for-each",
        )?;
        let body = compile_sequence(document, element)?;
        if !is_context_independent_static_range_body(&body) {
            return Err(unsupported(
                "FXST1007",
                "the admitted static integer-range xsl:for-each body is limited to literal result elements and text",
                document.location(element),
            ));
        }
        return Ok(Instruction::ForEachStaticIntegerRange {
            start,
            end,
            body,
            location: document.location(element).clone(),
        });
    }
    ensure_only_attributes(
        document,
        element,
        &["select", "default-mode"],
        "xsl:for-each",
    )?;
    let location = document.location(element).clone();
    Ok(Instruction::ForEachNodes {
        select: template_invocation_compiler::parse_apply_selection(
            document,
            element,
            select,
            location.clone(),
        )?,
        body: compile_sequence(document, element)?,
        location,
    })
}

fn parse_static_integer_range(expression: &str) -> Option<(i64, i64)> {
    let (start, end) = expression.split_once(" to ")?;
    Some((
        start.trim().parse::<i64>().ok()?,
        end.trim().parse::<i64>().ok()?,
    ))
}

fn is_context_independent_static_range_body(body: &[Instruction]) -> bool {
    body.iter().all(|instruction| match instruction {
        Instruction::Text { .. } => true,
        Instruction::LiteralElement { body, .. } => is_context_independent_static_range_body(body),
        _ => false,
    })
}

fn ensure_literal_result_control_attributes(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            && !matches!(
                name.local.as_str(),
                "xpath-default-namespace" | "default-mode"
            )
        {
            return Err(unsupported(
                "FXST1007",
                "unsupported XSLT control attribute on a literal result element",
                document.location(*attribute),
            ));
        }
    }
    Ok(())
}

pub(super) fn compile_text(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &[], "xsl:text")?;
    let mut value = String::new();
    for child in document.children(element).iter().copied() {
        match document.kind(child) {
            NodeKind::Text => value.push_str(document.value(child).unwrap_or_default()),
            NodeKind::Comment | NodeKind::ProcessingInstruction => {}
            NodeKind::Element | NodeKind::Document | NodeKind::Attribute => {
                return Err(invalid(
                    "FXST0026",
                    "the private xsl:text slice permits character content only",
                    document.location(child),
                ));
            }
        }
    }
    Ok(Instruction::Text {
        value,
        location: document.location(element).clone(),
    })
}

pub(super) fn compile_processing_instruction(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["name"], "xsl:processing-instruction")?;
    let target = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(target) || target.eq_ignore_ascii_case("xml") {
        return Err(invalid(
            "FXST0036",
            "the static processing-instruction target must be an NCName other than XML",
            document.location(element),
        ));
    }
    let mut value = String::new();
    for child in document.children(element).iter().copied() {
        match document.kind(child) {
            NodeKind::Text => value.push_str(document.value(child).unwrap_or_default()),
            NodeKind::Comment | NodeKind::ProcessingInstruction => {}
            NodeKind::Element => {
                return Err(unsupported(
                    "FXST1034",
                    "computed processing-instruction content is outside the private slice",
                    document.location(child),
                ));
            }
            NodeKind::Document | NodeKind::Attribute => {
                return Err(invalid(
                    "FXST0006",
                    "unexpected node kind in xsl:processing-instruction",
                    document.location(child),
                ));
            }
        }
    }
    if value.contains("?>") {
        return Err(unsupported(
            "FXST1035",
            "processing-instruction data containing ?> requires recovery outside the private slice",
            document.location(element),
        ));
    }
    Ok(Instruction::ProcessingInstructionNode {
        target: target.to_owned(),
        value,
        location: document.location(element).clone(),
    })
}

pub(super) fn compile_comment(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:comment")?;
    let value = if let Some(select) = optional_attribute(document, element, None, "select") {
        ensure_no_meaningful_children(document, element, "xsl:comment")?;
        crate::xpath::static_string_experiment::fold(select).ok_or_else(|| {
            unsupported(
                "FXXP1013",
                format!("unsupported static comment expression: {select}"),
                document.location(element),
            )
        })?
    } else {
        let mut value = String::new();
        for child in document.children(element).iter().copied() {
            match document.kind(child) {
                NodeKind::Text => value.push_str(document.value(child).unwrap_or_default()),
                NodeKind::Comment | NodeKind::ProcessingInstruction => {}
                NodeKind::Element => {
                    return Err(unsupported(
                        "FXST1036",
                        "computed comment content is outside the private slice",
                        document.location(child),
                    ));
                }
                NodeKind::Document | NodeKind::Attribute => {
                    return Err(invalid(
                        "FXST0006",
                        "unexpected node kind in xsl:comment",
                        document.location(child),
                    ));
                }
            }
        }
        value
    };
    if value.contains("--") || value.ends_with('-') {
        return Err(unsupported(
            "FXST1037",
            "comment content requiring lexical recovery is outside the private slice",
            document.location(element),
        ));
    }
    Ok(Instruction::CommentNode {
        value,
        location: document.location(element).clone(),
    })
}

pub(super) fn literal_result_namespaces(
    document: &Document,
    element: NodeId,
) -> Vec<NamespaceBinding> {
    let mut namespaces = Vec::new();
    let mut excluded_prefixes = Vec::new();
    let mut exclude_all = false;
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(exclusions) =
            optional_attribute(document, node, None, "exclude-result-prefixes")
        {
            for prefix in exclusions.split_whitespace() {
                if prefix == "#all" {
                    exclude_all = true;
                } else if !excluded_prefixes.contains(&prefix) {
                    excluded_prefixes.push(prefix);
                }
            }
        }
        current = document.parent(node);
    }
    let mut current = Some(element);
    while let Some(node) = current {
        for binding in document.namespace_declarations(node) {
            let prefix = binding.prefix.as_deref();
            let required_for_element = document.name(element).is_some_and(|name| {
                name.namespace.as_deref() == Some(binding.namespace.as_str())
                    && document.prefix(element) == prefix
            });
            let excluded = match prefix {
                Some(prefix) => excluded_prefixes.contains(&prefix),
                None => excluded_prefixes.contains(&"#default"),
            };
            if prefix != Some("xml")
                && binding.namespace != XSLT_NAMESPACE
                && !binding.namespace.is_empty()
                && (required_for_element || (!exclude_all && !excluded))
                && !namespaces
                    .iter()
                    .any(|existing: &NamespaceBinding| existing.prefix.as_deref() == prefix)
            {
                namespaces.push(binding.clone());
            }
        }
        current = document.parent(node);
    }
    if let Some(preferred_prefix) = document.prefix(element)
        && let Some(index) = namespaces
            .iter()
            .position(|binding| binding.prefix.as_deref() == Some(preferred_prefix))
    {
        namespaces[..=index].rotate_right(1);
    }
    namespaces
}

fn compile_apply_templates(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    template_invocation_compiler::compile_apply_templates(document, element)
}

fn compile_with_params(
    document: &Document,
    parent: NodeId,
    parent_label: &str,
    allow_fallback: bool,
) -> Result<Vec<TemplateArgument>, CompileFailure> {
    template_invocation_compiler::compile_with_params(
        document,
        parent,
        parent_label,
        allow_fallback,
    )
}

pub(super) fn parse_template_modes(
    document: &Document,
    element: NodeId,
    mode: &str,
) -> Result<Vec<String>, CompileFailure> {
    template_invocation_compiler::parse_template_modes(document, element, mode)
}

fn compile_value_of(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select", "separator"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
    let select = compile_value_expression(document, element, expression, &location)?;
    let separator = optional_attribute(document, element, None, "separator")
        .unwrap_or(" ")
        .to_owned();
    Ok(Instruction::ValueOf {
        select,
        separator,
        location,
    })
}

fn compile_value_expression(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    Ok(if recognizes_deep_equal(expression) {
        ValueExpression::DeepEqual(Box::new(parse_deep_equal(expression, location).map_err(
            |failure| {
                let (code, category) = match failure.kind {
                    DeepEqualFailureKind::InvalidArity { .. } => {
                        ("FXXP0005", CompileCategory::Invalid)
                    }
                    DeepEqualFailureKind::InvalidCollation { standard_code }
                    | DeepEqualFailureKind::InvalidCollationType { standard_code } => {
                        (standard_code, CompileCategory::Invalid)
                    }
                    DeepEqualFailureKind::Unsupported => ("FXXP1010", CompileCategory::Unsupported),
                };
                CompileFailure {
                    code,
                    category,
                    detail: failure.detail,
                    location: failure.location,
                }
            },
        )?))
    } else if expression.contains(" castable as ") {
        ValueExpression::Castable(Box::new(parse_castable(expression, location).map_err(
            |failure| CompileFailure {
                code: "FXXP1007",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            },
        )?))
    } else if expression.trim_start().starts_with("format-number(")
        && expression.contains("sum(for $")
    {
        ValueExpression::DecimalSumFor(Box::new(
            parse_decimal_sum_for(expression, location).map_err(|failure| CompileFailure {
                code: "FXXP1006",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("format-number(") {
        ValueExpression::FormatNumber(Box::new(
            parse_format_number(expression, location).map_err(|failure| CompileFailure {
                code: "FXXP1009",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("sum(for $") {
        ValueExpression::FocusSumFor(Box::new(
            parse_focus_sum_for(expression, location).map_err(|failure| CompileFailure {
                code: "FXXP1005",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("for $") {
        ValueExpression::IntegerFor(Box::new(
            parse_integer_for(expression, location.clone()).map_err(|failure| CompileFailure {
                code: "FXXP1004",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if let Some(root) = compile_root_value(document, element, expression, location)? {
        root
    } else if expression.trim() == "name(.)" {
        ValueExpression::ContextNodeName
    } else if expression.trim() == "upper-case(.)" {
        ValueExpression::UpperCaseContextString
    } else if let Some((literal, variable)) = parse_literal_variable_concat(expression) {
        ValueExpression::LiteralVariableConcat { literal, variable }
    } else if let Some(variable) = expression.strip_prefix('$') {
        if !is_ascii_ncname(variable) {
            return Err(invalid(
                "FXXP0002",
                format!("invalid variable reference: {expression}"),
                location,
            ));
        }
        ValueExpression::Variable(variable.to_owned())
    } else {
        ValueExpression::LocationPath(
            parse_location_path(expression, location.clone()).map_err(map_path_failure)?,
        )
    })
}

fn parse_literal_variable_concat(expression: &str) -> Option<(String, String)> {
    let arguments = expression
        .trim()
        .strip_prefix("concat(")?
        .strip_suffix(')')?;
    let (literal, variable) = arguments.rsplit_once(',')?;
    let literal = literal.trim();
    let literal = literal
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            literal
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })?;
    let variable = variable.trim().strip_prefix('$')?;
    is_ascii_ncname(variable).then(|| (literal.to_owned(), variable.to_owned()))
}

fn root_argument(expression: &str) -> Option<&str> {
    expression
        .trim()
        .strip_prefix("root(")?
        .strip_suffix(')')
        .map(str::trim)
}

fn compile_root_value(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    if let Some((reference, descendant_local)) = parse_generated_document_root(expression) {
        return Ok(Some(ValueExpression::GeneratedDocumentRootIdentity(
            crate::xslt::golden_semantics_experiment::DocumentRootReference {
                base: location.resource.clone(),
                reference: reference.to_owned(),
                descendant_local: descendant_local.map(str::to_owned),
            },
        )));
    }
    if let Some((variable, descendant_local)) = parse_generated_temporary_root(expression) {
        return Ok(Some(ValueExpression::GeneratedTemporaryRootIdentity {
            variable: variable.to_owned(),
            descendant_local: descendant_local.map(str::to_owned),
        }));
    }
    if let Some(argument) = generated_root_argument(expression) {
        return parse_location_path(argument, location.clone())
            .map(ValueExpression::GeneratedRootIdentity)
            .map(Some)
            .map_err(map_path_failure);
    }
    root_argument(expression)
        .map(|argument| compile_root_expression(document, element, argument, location))
        .transpose()
}

fn generated_root_argument(expression: &str) -> Option<&str> {
    expression
        .trim()
        .strip_prefix("generate-id(root(")?
        .strip_suffix("))")
        .map(str::trim)
}

fn compile_root_expression(
    document: &Document,
    element: NodeId,
    argument: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    if let Some(variable) = argument.strip_prefix('$') {
        if !is_ascii_ncname(variable) {
            return Err(invalid(
                "XPST0003",
                format!("invalid variable reference in root(): {argument}"),
                location,
            ));
        }
        return Ok(ValueExpression::RootVariable(variable.to_owned()));
    }
    let path = match parse_location_path(argument, location.clone()) {
        Ok(path) => Ok(path),
        Err(PathFailure::Unsupported { .. }) if argument.contains(':') => {
            parse_qualified_child_path(argument, location.clone(), |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
        }
        Err(failure) => Err(failure),
    };
    path.map(ValueExpression::RootPath)
        .map_err(map_path_failure)
}

fn compile_variable(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["name", "select", "as"], "xsl:variable")?;
    let location = document.location(element).clone();
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(invalid(
            "FXST0016",
            format!("invalid local variable name: {name}"),
            &location,
        ));
    }
    let Some(expression) = optional_attribute(document, element, None, "select") else {
        if optional_attribute(document, element, None, "as").is_none()
            && meaningful_children(document, element)
                .iter()
                .all(|child| document.kind(*child) == NodeKind::Element)
        {
            return Ok(Instruction::TemporaryTreeVariable {
                name: name.to_owned(),
                elements: super::compile_constructed_elements(document, element)?,
                location,
            });
        }
        return compile_integer_range_variable(document, element, name, &location);
    };
    ensure_no_meaningful_children(document, element, "xsl:variable")?;
    if optional_attribute(document, element, None, "as").is_some() {
        return Err(unsupported(
            "FXST1016",
            "typed select-based local variables are outside the private slice",
            &location,
        ));
    }
    if expression.trim() == "position()" {
        return Ok(Instruction::ContextPositionVariable {
            name: name.to_owned(),
            location,
        });
    }
    let node_path = match parse_location_path(expression, location.clone()) {
        Ok(path) => Some(path),
        Err(PathFailure::Unsupported { .. }) if expression.contains(':') => Some(
            parse_qualified_child_path(expression, location.clone(), |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
            .map_err(map_path_failure)?,
        ),
        Err(_) => None,
    };
    if let Some(select) = node_path {
        return Ok(Instruction::SourceNodeVariable {
            name: name.to_owned(),
            select,
            location,
        });
    }
    let select = parse_cast(expression, &location).map_err(|failure| CompileFailure {
        code: "FXXP1008",
        category: CompileCategory::Unsupported,
        detail: failure.detail,
        location: failure.location,
    })?;
    Ok(Instruction::Variable {
        name: name.to_owned(),
        select: Box::new(select),
        location,
    })
}

fn compile_integer_range_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    location: &SourceLocation,
) -> Result<Instruction, CompileFailure> {
    let sequence_type = optional_attribute(document, element, None, "as").ok_or_else(|| {
        unsupported(
            "FXST1016",
            "constructed local variables require an admitted sequence type",
            location,
        )
    })?;
    if sequence_type != "xs:integer *"
        || namespace_for_prefix(document, element, "xs") != Some(XML_SCHEMA_NAMESPACE)
    {
        return Err(unsupported(
            "FXST1016",
            format!("unsupported constructed local variable type: {sequence_type}"),
            location,
        ));
    }
    let children = meaningful_children(document, element);
    let [for_each] = children.as_slice() else {
        return Err(unsupported(
            "FXST1017",
            "the admitted constructed integer sequence requires one xsl:for-each",
            location,
        ));
    };
    if !is_xslt_element(document, *for_each, "for-each") {
        return Err(unsupported(
            "FXST1017",
            "the admitted constructed integer sequence requires xsl:for-each",
            document.location(*for_each),
        ));
    }
    ensure_only_attributes(document, *for_each, &["select"], "xsl:for-each")?;
    let range = required_attribute(document, *for_each, None, "select")?;
    let (start, end) = parse_integer_range(range, document.location(*for_each))?;
    validate_atomized_range_body(document, *for_each)?;
    Ok(Instruction::IntegerRangeVariable {
        name: name.to_owned(),
        start,
        end,
        location: location.clone(),
    })
}

fn parse_integer_range(
    expression: &str,
    location: &SourceLocation,
) -> Result<(i64, i64), CompileFailure> {
    let Some((start, end)) = expression.split_once(" to ") else {
        return Err(unsupported(
            "FXXP1010",
            format!("unsupported integer range: {expression}"),
            location,
        ));
    };
    let start = start.trim().parse::<i64>().map_err(|_| {
        invalid(
            "FXXP0004",
            format!("invalid integer range start: {start}"),
            location,
        )
    })?;
    let end = end.trim().parse::<i64>().map_err(|_| {
        invalid(
            "FXXP0004",
            format!("invalid integer range end: {end}"),
            location,
        )
    })?;
    Ok((start, end))
}

fn validate_atomized_range_body(
    document: &Document,
    for_each: NodeId,
) -> Result<(), CompileFailure> {
    let children = meaningful_children(document, for_each);
    let [wrapper] = children.as_slice() else {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range body requires one literal wrapper",
            document.location(for_each),
        ));
    };
    if document
        .name(*wrapper)
        .is_none_or(|name| name.namespace.as_deref() == Some(XSLT_NAMESPACE))
        || !document.attributes(*wrapper).is_empty()
    {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range body requires an attribute-free literal wrapper",
            document.location(*wrapper),
        ));
    }
    let body = meaningful_children(document, *wrapper);
    let [value_of] = body.as_slice() else {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range wrapper requires one xsl:value-of",
            document.location(*wrapper),
        ));
    };
    if !is_xslt_element(document, *value_of, "value-of")
        || required_attribute(document, *value_of, None, "select")? != "."
    {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range wrapper must atomize the range item",
            document.location(*value_of),
        ));
    }
    ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, *value_of, "xsl:value-of")
}

fn namespace_for_prefix<'a>(
    document: &'a Document,
    element: NodeId,
    prefix: &str,
) -> Option<&'a str> {
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(binding) = document
            .namespace_declarations(node)
            .iter()
            .find(|binding| binding.prefix.as_deref() == Some(prefix))
        {
            return Some(binding.namespace.as_str());
        }
        current = document.parent(node);
    }
    None
}

fn compile_sequence_nodes(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:sequence")?;
    ensure_no_meaningful_children(document, element, "xsl:sequence")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
    let sequence_items: Vec<_> = expression.split(',').map(str::trim).collect();
    if sequence_items
        .iter()
        .all(|item| *item == "*" || item.starts_with('$'))
    {
        let mut select = Vec::new();
        for item in sequence_items {
            if item == "*" {
                select.push(SequenceItemExpression::ChildElements);
            } else if let Some(variable) = item.strip_prefix('$') {
                select.push(SequenceItemExpression::Variable(normalize_variable_qname(
                    document, element, variable,
                )?));
            } else {
                return Err(unsupported(
                    "FXXP1003",
                    format!("unsupported xsl:sequence item expression: {item}"),
                    &location,
                ));
            }
        }
        return Ok(Instruction::SequenceItems { select, location });
    }
    let select =
        parse_for_distinct_values(expression, location.clone()).map_err(
            |failure| match failure {
                ForExpressionFailure::Invalid { detail, location } => CompileFailure {
                    code: "FXXP0003",
                    category: CompileCategory::Invalid,
                    detail,
                    location,
                },
                ForExpressionFailure::Unsupported { detail, location } => CompileFailure {
                    code: "FXXP1003",
                    category: CompileCategory::Unsupported,
                    detail,
                    location,
                },
            },
        )?;
    Ok(Instruction::SequenceNodes {
        select: Box::new(select),
        location,
    })
}

fn compile_if(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["test", "default-mode"], "xsl:if")?;
    let location = document.location(element).clone();
    let expression = required_conditional_test(document, element)?;
    Ok(Instruction::If {
        test: parse_boolean_expression(expression, &location)?,
        body: compile_sequence(document, element)?,
        location,
    })
}

fn compile_choose(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &[], "xsl:choose")?;
    let children = meaningful_children(document, element);
    validate_choose_structure(document, element, &children)?;
    let mut branches = Vec::new();
    let mut otherwise = None;
    for child in children {
        if is_xslt_element(document, child, "when") {
            ensure_only_attributes(document, child, &["test"], "xsl:when")?;
            let expression = required_conditional_test(document, child)?;
            branches.push(ChooseBranch {
                test: parse_boolean_expression(expression, document.location(child))?,
                body: compile_sequence(document, child)?,
            });
        } else if is_xslt_element(document, child, "otherwise") {
            ensure_only_attributes(document, child, &[], "xsl:otherwise")?;
            otherwise = Some(compile_sequence(document, child)?);
        } else {
            unreachable!("validate_choose_structure rejects other children")
        }
    }
    Ok(Instruction::Choose {
        branches,
        otherwise: otherwise.unwrap_or_default(),
        location: document.location(element).clone(),
    })
}

fn validate_choose_structure(
    document: &Document,
    element: NodeId,
    children: &[NodeId],
) -> Result<(), CompileFailure> {
    let mut when_count = 0_usize;
    let mut saw_otherwise = false;
    for child in children {
        if is_xslt_element(document, *child, "when") {
            if saw_otherwise {
                return Err(invalid(
                    "XTSE0010",
                    "xsl:when cannot follow xsl:otherwise",
                    document.location(*child),
                ));
            }
            required_conditional_test(document, *child)?;
            when_count += 1;
        } else if is_xslt_element(document, *child, "otherwise") {
            if saw_otherwise {
                return Err(invalid(
                    "XTSE0010",
                    "xsl:choose permits at most one xsl:otherwise",
                    document.location(*child),
                ));
            }
            saw_otherwise = true;
        } else {
            return Err(invalid(
                "XTSE0010",
                "xsl:choose permits only xsl:when and xsl:otherwise children",
                document.location(*child),
            ));
        }
    }
    if when_count == 0 {
        return Err(invalid(
            "XTSE0010",
            "xsl:choose requires at least one xsl:when",
            document.location(element),
        ));
    }
    Ok(())
}

fn required_conditional_test(document: &Document, element: NodeId) -> Result<&str, CompileFailure> {
    optional_attribute(document, element, None, "test").ok_or_else(|| {
        invalid(
            "XTSE0010",
            "xsl:if and xsl:when require a test attribute",
            document.location(element),
        )
    })
}

fn parse_boolean_expression(
    expression: &str,
    location: &SourceLocation,
) -> Result<BooleanExpression, CompileFailure> {
    let parsed = strip_enclosing_parentheses(expression.trim());
    if let Some((left, right)) = split_top_level_or(parsed) {
        return Ok(BooleanExpression::Or {
            left: Box::new(parse_boolean_expression(left, location)?),
            right: Box::new(parse_boolean_expression(right, location)?),
        });
    }
    if let Some(inner) = parsed
        .strip_prefix("not(")
        .and_then(|inner| inner.strip_suffix(')'))
    {
        return Ok(BooleanExpression::Not(Box::new(parse_boolean_expression(
            inner, location,
        )?)));
    }
    if let Some((left, right)) = parse_document_root_identity_test(parsed) {
        return Ok(BooleanExpression::DocumentRootIdentityEqual {
            left: crate::xslt::golden_semantics_experiment::DocumentRootReference {
                base: location.resource.clone(),
                reference: left.0.to_owned(),
                descendant_local: left.1.map(str::to_owned),
            },
            right: crate::xslt::golden_semantics_experiment::DocumentRootReference {
                base: location.resource.clone(),
                reference: right.0.to_owned(),
                descendant_local: right.1.map(str::to_owned),
            },
        });
    }
    if let Some((variable, descendant_local)) = parse_temporary_root_identity_test(parsed) {
        return Ok(BooleanExpression::TemporaryRootIdentityEqual {
            variable: variable.to_owned(),
            descendant_local: descendant_local.to_owned(),
        });
    }
    if let Some((path, variable)) = parse_generated_root_identity_test(parsed) {
        return Ok(BooleanExpression::RootIdentityEqualsVariable {
            path: parse_location_path(path, location.clone()).map_err(map_path_failure)?,
            variable: variable.to_owned(),
        });
    }
    parse_scalar_boolean_expression(parsed, expression, location)
}

fn split_top_level_or(expression: &str) -> Option<(&str, &str)> {
    let bytes = expression.as_bytes();
    let mut quote = None;
    let mut parentheses = 0_usize;
    let mut brackets = 0_usize;
    let mut index = 0_usize;
    while index + 1 < bytes.len() {
        let byte = bytes[index];
        if let Some(expected) = quote {
            if byte == expected {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => parentheses += 1,
            b')' => parentheses = parentheses.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'o' if bytes[index + 1] == b'r' && parentheses == 0 && brackets == 0 => {
                let left_boundary = index == 0 || !is_xpath_name_byte(bytes[index - 1]);
                let right_boundary = index + 2 == bytes.len()
                    || !is_xpath_name_byte(bytes.get(index + 2).copied().unwrap_or_default());
                if left_boundary && right_boundary {
                    let left = expression[..index].trim();
                    let right = expression[index + 2..].trim();
                    if !left.is_empty() && !right.is_empty() {
                        return Some((left, right));
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn is_xpath_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':')
}

fn parse_scalar_boolean_expression(
    parsed: &str,
    expression: &str,
    location: &SourceLocation,
) -> Result<BooleanExpression, CompileFailure> {
    if let Some(value) = parse_constant_string_boolean(parsed) {
        return Ok(BooleanExpression::Constant(value));
    }
    if let Some(length) = parse_context_string_length_equality(parsed) {
        return Ok(BooleanExpression::ContextStringLengthEquals(length));
    }
    if let Some(expression) = parse_path_boolean_expression(parsed, location)? {
        return Ok(expression);
    }
    if parsed.starts_with('/') || parsed.starts_with('@') {
        return parse_location_path(parsed, location.clone())
            .map(BooleanExpression::NodeExists)
            .map_err(map_path_failure);
    }
    if is_ascii_ncname(parsed) {
        return parse_location_path(parsed, location.clone())
            .map(BooleanExpression::NodeExists)
            .map_err(map_path_failure);
    }
    if let Some((left, right)) = parsed.split_once('=') {
        let (context, literal) = if left.trim() == "." {
            (left, xpath_string_literal(right.trim()))
        } else if right.trim() == "." {
            (right, xpath_string_literal(left.trim()))
        } else {
            (left, None)
        };
        if context.trim() == "."
            && let Some(literal) = literal
        {
            return Ok(BooleanExpression::ContextStringEquals(literal.to_owned()));
        }
    }
    if let Some((variable, integer)) = parsed.split_once('=') {
        let variable = variable.trim().strip_prefix('$').unwrap_or_default();
        if is_ascii_ncname(variable) {
            let integer = integer
                .trim()
                .parse::<i64>()
                .map_err(|_| unsupported_boolean_expression(expression, location))?;
            return Ok(BooleanExpression::VariableEqualsInteger(EqualityTest {
                variable: variable.to_owned(),
                integer,
            }));
        }
    }
    if let Some(variable) = parsed.strip_prefix('$')
        && is_ascii_ncname(variable)
    {
        return Ok(BooleanExpression::VariableEffectiveBooleanValue(
            variable.to_owned(),
        ));
    }
    if !parsed.contains('=') && !parsed.contains('>') {
        let ordering =
            constant_numeric_experiment::compare(parsed, "0").map_err(|failure| match failure {
                ConstantNumericFailure::Invalid => invalid(
                    "FXXP0004",
                    format!("invalid conditional expression: {expression}"),
                    location,
                ),
                ConstantNumericFailure::Unsupported => {
                    unsupported_boolean_expression(expression, location)
                }
            })?;
        return Ok(BooleanExpression::Constant(!ordering.is_eq()));
    }
    let (left, right, greater_than) = if let Some((left, right)) = parsed.split_once('>') {
        (left, right, true)
    } else if let Some((left, right)) = parsed.split_once('=') {
        (left, right, false)
    } else {
        return Err(unsupported_boolean_expression(expression, location));
    };
    let ordering =
        constant_numeric_experiment::compare(left.trim(), right.trim()).map_err(|failure| {
            match failure {
                ConstantNumericFailure::Invalid => invalid(
                    "FXXP0004",
                    format!("invalid conditional expression: {expression}"),
                    location,
                ),
                ConstantNumericFailure::Unsupported => {
                    unsupported_boolean_expression(expression, location)
                }
            }
        })?;
    Ok(BooleanExpression::Constant(if greater_than {
        ordering.is_gt()
    } else {
        ordering.is_eq()
    }))
}

fn parse_context_string_length_equality(expression: &str) -> Option<usize> {
    let (left, right) = expression.split_once('=')?;
    (left.trim() == "string-length(.)")
        .then(|| right.trim().parse().ok())
        .flatten()
}

fn parse_path_boolean_expression(
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<BooleanExpression>, CompileFailure> {
    if let Some((path, value)) = parse_path_context_string_predicate(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::NodeStringEquals {
                    path,
                    value: value.to_owned(),
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, local)) = parse_unqualified_name_equality(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::UnqualifiedNodeNameEquals {
                    path,
                    local: local.to_owned(),
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, value)) = parse_path_string_equality(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::NodeStringEquals {
                    path,
                    value: value.to_owned(),
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, value)) = parse_path_integer_less_than(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| Some(BooleanExpression::NodeIntegerLessThan { path, value }))
            .map_err(map_path_failure);
    }
    Ok(None)
}

fn parse_constant_string_boolean(expression: &str) -> Option<bool> {
    if let Some(value) = xpath_string_literal(expression) {
        return Some(!value.is_empty());
    }
    let (left, right) = expression.split_once('=')?;
    Some(xpath_string_literal(left.trim())? == xpath_string_literal(right.trim())?)
}

fn parse_path_context_string_predicate(expression: &str) -> Option<(&str, &str)> {
    let (path, predicate) = expression.split_once('[')?;
    if path.trim().is_empty() || path.contains(']') {
        return None;
    }
    let predicate = predicate.strip_suffix(']')?;
    if predicate.contains(['[', ']']) {
        return None;
    }
    let (left, right) = predicate.split_once('=')?;
    (left.trim() == ".")
        .then(|| xpath_string_literal(right.trim()))
        .flatten()
        .map(|value| (path.trim(), value))
}

fn parse_path_integer_less_than(expression: &str) -> Option<(&str, i64)> {
    let (path, value) = expression.split_once('<')?;
    let path = path.trim();
    if path.is_empty() || path.contains(['=', '>', '<']) {
        return None;
    }
    Some((path, value.trim().parse().ok()?))
}

fn parse_unqualified_name_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let path = left.trim().strip_prefix("name(")?.strip_suffix(')')?.trim();
    let local = xpath_string_literal(right.trim())?;
    (!path.is_empty() && is_ascii_ncname(local)).then_some((path, local))
}

fn parse_path_string_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let left = left.trim();
    let right = right.trim();
    if let Some(value) = xpath_string_literal(right)
        && left != "."
        && !left.starts_with('$')
    {
        return Some((left, value));
    }
    let value = xpath_string_literal(left)?;
    (right != "." && !right.starts_with('$')).then_some((right, value))
}

fn xpath_string_literal(expression: &str) -> Option<&str> {
    if expression.len() < 2 {
        return None;
    }
    let quote = expression.as_bytes()[0];
    if !matches!(quote, b'\'' | b'"') || expression.as_bytes().last() != Some(&quote) {
        return None;
    }
    let value = &expression[1..expression.len() - 1];
    (!value.as_bytes().contains(&quote)).then_some(value)
}

fn parse_temporary_root_identity_test(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let (left_variable, left_descendant) = parse_generated_temporary_root(left.trim())?;
    let (right_variable, right_descendant) = parse_generated_temporary_root(right.trim())?;
    if left_variable != right_variable || left_descendant.is_some() {
        return None;
    }
    Some((left_variable, right_descendant?))
}

fn parse_generated_temporary_root(expression: &str) -> Option<(&str, Option<&str>)> {
    let argument = generated_root_argument(expression)?;
    let argument = argument.strip_prefix('$')?;
    let (variable, descendant) = argument
        .split_once("//")
        .map_or((argument, None), |(variable, descendant)| {
            (variable, Some(descendant))
        });
    if !is_ascii_ncname(variable)
        || descendant.is_some_and(|descendant| !is_ascii_ncname(descendant))
    {
        return None;
    }
    Some((variable, descendant))
}

type ParsedDocumentRootReference<'a> = (&'a str, Option<&'a str>);

fn parse_document_root_identity_test(
    expression: &str,
) -> Option<(
    ParsedDocumentRootReference<'_>,
    ParsedDocumentRootReference<'_>,
)> {
    let (left, right) = expression.split_once('=')?;
    Some((
        parse_generated_document_root(left.trim())?,
        parse_generated_document_root(right.trim())?,
    ))
}

fn parse_generated_document_root(expression: &str) -> Option<(&str, Option<&str>)> {
    let argument = generated_root_argument(expression)?;
    let argument = argument.strip_prefix("document('")?;
    let (reference, suffix) = argument.split_once("')")?;
    if reference.is_empty() || reference.contains('\'') {
        return None;
    }
    let descendant = if suffix.is_empty() {
        None
    } else {
        Some(suffix.strip_prefix("//")?)
    };
    if descendant.is_some_and(|name| !is_ascii_ncname(name)) {
        return None;
    }
    Some((reference, descendant))
}

fn parse_generated_root_identity_test(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let variable = right.trim().strip_prefix('$')?;
    if !is_ascii_ncname(variable) {
        return None;
    }
    let path = left
        .trim()
        .strip_prefix("generate-id(root(")?
        .strip_suffix("))")?
        .trim();
    Some((path, variable))
}

fn strip_enclosing_parentheses(mut expression: &str) -> &str {
    loop {
        let bytes = expression.as_bytes();
        if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
            return expression;
        }
        let mut depth = 0_usize;
        let mut encloses_all = true;
        for (index, byte) in bytes.iter().copied().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && index + 1 != bytes.len() {
                        encloses_all = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !encloses_all || depth != 0 {
            return expression;
        }
        expression = expression[1..expression.len() - 1].trim();
    }
}

fn unsupported_boolean_expression(expression: &str, location: &SourceLocation) -> CompileFailure {
    unsupported(
        "FXXP1002",
        format!("unsupported conditional expression: {expression}"),
        location,
    )
}

fn compile_call_template(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    template_invocation_compiler::compile_call_template(document, element)
}
