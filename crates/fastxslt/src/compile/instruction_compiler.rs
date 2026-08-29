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
    ApplySelection, BooleanExpression, ChooseBranch, EqualityTest, Instruction, NodeTest,
    SequenceItemExpression, TemplateArgument, ValueExpression,
};

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
                        | Instruction::IntegerRangeVariable { name, .. }) = &variable
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
                    } else if name.local == "if" {
                        instructions.push(compile_if(document, child)?);
                    } else if name.local == "choose" {
                        instructions.push(compile_choose(document, child)?);
                    } else if name.local == "call-template" {
                        instructions.push(compile_call_template(document, child)?);
                    } else if name.local == "copy" {
                        ensure_only_attributes(document, child, &[], "xsl:copy")?;
                        ensure_no_meaningful_children(document, child, "xsl:copy")?;
                        instructions.push(Instruction::Copy {
                            location: document.location(child).clone(),
                        });
                    } else {
                        return Err(unsupported(
                            "FXST1006",
                            format!("unsupported XSLT instruction: xsl:{}", name.local),
                            document.location(child),
                        ));
                    }
                } else {
                    if !document.attributes(child).is_empty() {
                        return Err(unsupported(
                            "FXST1007",
                            "literal result attributes are outside the private slice",
                            document.location(child),
                        ));
                    }
                    instructions.push(Instruction::LiteralElement {
                        name: name.clone(),
                        namespaces: literal_result_namespaces(document, child),
                        body: compile_sequence(document, child)?,
                        location: document.location(child).clone(),
                    });
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
    ensure_only_attributes(
        document,
        element,
        &["select", "mode"],
        "xsl:apply-templates",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:apply-templates")?;
    let location = document.location(element).clone();
    let select = optional_attribute(document, element, None, "select")
        .map(|expression| parse_apply_selection(document, element, expression, location.clone()))
        .transpose()?;
    let mode = optional_attribute(document, element, None, "mode")
        .map(|mode| parse_mode(mode, document.location(element)))
        .transpose()?;
    Ok(Instruction::ApplyTemplates {
        select,
        mode,
        location,
    })
}

fn parse_apply_selection(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: SourceLocation,
) -> Result<ApplySelection, CompileFailure> {
    if let Some(variable) = expression
        .strip_prefix('$')
        .and_then(|value| value.strip_suffix("/*"))
    {
        if is_ascii_ncname(variable) {
            return Ok(ApplySelection::GlobalTemporaryChildren(variable.to_owned()));
        }
    }
    let node_test = match expression {
        "comment()" => Some(NodeTest::Comment),
        "processing-instruction()" => Some(NodeTest::ProcessingInstruction),
        "node()" => Some(NodeTest::AnyNode),
        _ => None,
    };
    if let Some(node_test) = node_test {
        return Ok(ApplySelection::ChildNodes(node_test));
    }
    if is_ascii_ncname(expression) {
        if let Some(namespace) = effective_xpath_default_namespace(document, element) {
            return Ok(ApplySelection::ChildElement(
                crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: Some(namespace.to_owned()),
                    local: expression.to_owned(),
                },
            ));
        }
    }
    if let Some(attribute) = expression.strip_prefix('@') {
        if is_ascii_ncname(attribute) {
            return Ok(ApplySelection::Attribute(
                crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: attribute.to_owned(),
                },
            ));
        }
    }
    if effective_xpath_default_namespace(document, element).is_some() {
        return Err(unsupported(
            "FXST1027",
            "xpath-default-namespace on non-simple apply-templates paths is outside the private expanded-name path slice",
            &location,
        ));
    }
    parse_location_path(expression, location)
        .map(ApplySelection::LocationPath)
        .map_err(map_path_failure)
}

fn parse_mode(mode: &str, location: &SourceLocation) -> Result<String, CompileFailure> {
    if is_ascii_ncname(mode) {
        Ok(mode.to_owned())
    } else {
        Err(unsupported(
            "FXST1012",
            format!("unsupported mode name: {mode}"),
            location,
        ))
    }
}

pub(super) fn parse_template_modes(
    mode: &str,
    location: &SourceLocation,
) -> Result<Vec<String>, CompileFailure> {
    let modes = mode
        .split_whitespace()
        .map(|name| {
            if name == "#all" {
                Ok(name.to_owned())
            } else {
                parse_mode(name, location)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if modes.is_empty() {
        return Err(unsupported(
            "FXST1012",
            "template mode list is empty",
            location,
        ));
    }
    Ok(modes)
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
    ensure_only_attributes(document, element, &["name"], "xsl:call-template")?;
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(unsupported(
            "FXST1013",
            format!("unsupported named-template name: {name}"),
            document.location(element),
        ));
    }
    let mut arguments = Vec::new();
    for child in meaningful_children(document, element) {
        if !is_xslt_element(document, child, "with-param") {
            return Err(unsupported(
                "FXST1014",
                "the private call-template slice permits only xsl:with-param children",
                document.location(child),
            ));
        }
        ensure_only_attributes(document, child, &["name"], "xsl:with-param")?;
        let argument_name = required_attribute(document, child, None, "name")?;
        if !is_ascii_ncname(argument_name)
            || arguments
                .iter()
                .any(|argument: &TemplateArgument| argument.name == argument_name)
        {
            return Err(invalid(
                "FXST0013",
                format!("invalid or duplicate template argument: {argument_name}"),
                document.location(child),
            ));
        }
        arguments.push(TemplateArgument {
            name: argument_name.to_owned(),
            value: document.string_value(child),
        });
    }
    Ok(Instruction::CallTemplate {
        name: name.to_owned(),
        arguments,
        location: document.location(element).clone(),
    })
}
