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
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{
    BooleanExpression, ChooseBranch, EqualityTest, Instruction, SequenceItemExpression,
    TemplateArgument, ValueExpression,
};

#[path = "instruction_compiler/computed_attribute_compiler.rs"]
mod computed_attribute_compiler;
use computed_attribute_compiler::compile_computed_attributes;
#[path = "instruction_compiler/literal_attribute_compiler.rs"]
mod literal_attribute_compiler;
use literal_attribute_compiler::compile_literal_result_attributes;
#[path = "instruction_compiler/source_copy_compiler.rs"]
mod source_copy_compiler;
use source_copy_compiler::compile_copy;
#[path = "instruction_compiler/template_invocation_compiler.rs"]
mod template_invocation_compiler;

use super::{
    CompileCategory, CompileFailure, XML_SCHEMA_NAMESPACE, XSLT_NAMESPACE,
    effective_xpath_default_namespace, ensure_no_meaningful_children, ensure_only_attributes,
    invalid, is_ascii_ncname, is_xslt_element, map_path_failure, meaningful_children,
    normalize_variable_qname, optional_attribute, required_attribute, unsupported,
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
                    } else if name.local == "value-of" {
                        instructions.push(compile_value_of(document, child)?);
                    } else if name.local == "variable" {
                        let variable = compile_variable(document, child)?;
                        let (Instruction::Variable { name, .. }
                        | Instruction::IntegerRangeVariable { name, .. }
                        | Instruction::TemporaryTreeVariable { name, .. }) = &variable
                        else {
                            unreachable!("compile_variable returns a variable instruction")
                        };
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
    if select.trim() != "." {
        return Err(unsupported(
            "FXXP1003",
            format!("unsupported xsl:copy-of selection: {select}"),
            document.location(element),
        ));
    }
    Ok(Instruction::CopyOfCurrent {
        location: document.location(element).clone(),
    })
}

fn compile_for_each(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    let select = required_attribute(document, element, None, "select")?;
    if is_static_integer_range(select) {
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
    }
    Err(unsupported(
        "FXST1006",
        "xsl:for-each is outside the private instruction slice",
        document.location(element),
    ))
}

fn is_static_integer_range(expression: &str) -> bool {
    expression.split_once(" to ").is_some_and(|(start, end)| {
        start.trim().parse::<i64>().is_ok() && end.trim().parse::<i64>().is_ok()
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
            && name.local != "xpath-default-namespace"
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

fn compile_text(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
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
            let required_for_element = document
                .name(element)
                .is_some_and(|name| name.namespace.as_deref() == Some(binding.namespace.as_str()));
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
    mode: &str,
    location: &SourceLocation,
) -> Result<Vec<String>, CompileFailure> {
    template_invocation_compiler::parse_template_modes(mode, location)
}

fn compile_value_of(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select", "separator"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
    let select = if recognizes_deep_equal(expression) {
        ValueExpression::DeepEqual(Box::new(parse_deep_equal(expression, &location).map_err(
            |failure| {
                let (code, category) = match failure.kind {
                    DeepEqualFailureKind::InvalidArity { .. } => {
                        ("FXXP0005", CompileCategory::Invalid)
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
        ValueExpression::Castable(Box::new(parse_castable(expression, &location).map_err(
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
            parse_decimal_sum_for(expression, &location).map_err(|failure| CompileFailure {
                code: "FXXP1006",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("format-number(") {
        ValueExpression::FormatNumber(Box::new(
            parse_format_number(expression, &location).map_err(|failure| CompileFailure {
                code: "FXXP1009",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("sum(for $") {
        ValueExpression::FocusSumFor(Box::new(
            parse_focus_sum_for(expression, &location).map_err(|failure| CompileFailure {
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
    } else if expression.trim() == "name(.)" {
        ValueExpression::ContextNodeName
    } else if let Some(variable) = expression.strip_prefix('$') {
        if !is_ascii_ncname(variable) {
            return Err(invalid(
                "FXXP0002",
                format!("invalid variable reference: {expression}"),
                &location,
            ));
        }
        ValueExpression::Variable(variable.to_owned())
    } else {
        ValueExpression::LocationPath(
            parse_location_path(expression, location.clone()).map_err(map_path_failure)?,
        )
    };
    let separator = optional_attribute(document, element, None, "separator")
        .unwrap_or(" ")
        .to_owned();
    Ok(Instruction::ValueOf {
        select,
        separator,
        location,
    })
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
    ensure_only_attributes(document, element, &["test"], "xsl:if")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "test")?;
    Ok(Instruction::If {
        test: parse_boolean_expression(expression, &location)?,
        body: compile_sequence(document, element)?,
        location,
    })
}

fn compile_choose(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &[], "xsl:choose")?;
    let mut branches = Vec::new();
    let mut otherwise = None;
    for child in meaningful_children(document, element) {
        if is_xslt_element(document, child, "when") {
            if otherwise.is_some() {
                return Err(invalid(
                    "FXST0018",
                    "xsl:when cannot follow xsl:otherwise",
                    document.location(child),
                ));
            }
            ensure_only_attributes(document, child, &["test"], "xsl:when")?;
            let expression = required_attribute(document, child, None, "test")?;
            branches.push(ChooseBranch {
                test: parse_boolean_expression(expression, document.location(child))?,
                body: compile_sequence(document, child)?,
            });
        } else if is_xslt_element(document, child, "otherwise") {
            if otherwise.is_some() {
                return Err(invalid(
                    "FXST0019",
                    "xsl:choose permits at most one xsl:otherwise",
                    document.location(child),
                ));
            }
            ensure_only_attributes(document, child, &[], "xsl:otherwise")?;
            otherwise = Some(compile_sequence(document, child)?);
        } else {
            return Err(invalid(
                "FXST0020",
                "xsl:choose permits only xsl:when and xsl:otherwise children",
                document.location(child),
            ));
        }
    }
    if branches.is_empty() {
        return Err(invalid(
            "FXST0021",
            "xsl:choose requires at least one xsl:when",
            document.location(element),
        ));
    }
    Ok(Instruction::Choose {
        branches,
        otherwise: otherwise.unwrap_or_default(),
        location: document.location(element).clone(),
    })
}

fn parse_boolean_expression(
    expression: &str,
    location: &SourceLocation,
) -> Result<BooleanExpression, CompileFailure> {
    let parsed = strip_enclosing_parentheses(expression.trim());
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
