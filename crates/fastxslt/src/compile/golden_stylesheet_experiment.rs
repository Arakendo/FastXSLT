use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::NamespaceBinding;
use crate::xpath::castable_experiment::{parse as parse_castable, parse_cast};
use crate::xpath::constant_numeric_experiment::{self, ConstantNumericFailure};
use crate::xpath::decimal_sum_for_experiment::parse as parse_decimal_sum_for;
use crate::xpath::focus_sum_for_experiment::parse as parse_focus_sum_for;
use crate::xpath::for_distinct_values_experiment::{
    ForExpressionFailure, parse as parse_for_distinct_values,
};
use crate::xpath::format_number_experiment::parse as parse_format_number;
use crate::xpath::integer_for_experiment::parse as parse_integer_for;
use crate::xpath::path_experiment::{PathFailure, parse_child_path};
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, BooleanExpression, ChooseBranch, EqualityTest, GlobalBinding,
    GlobalBindingDefault, GlobalBindingKind, Instruction, MatchPattern, MatchedTemplate,
    NamedTemplate, NodeTest, OutputSettings, STANDARD_INITIAL_TEMPLATE_NAME, StylesheetProgram,
    Template, TemplateArgument, ValueExpression,
};

const XSLT_NAMESPACE: &str = "http://www.w3.org/1999/XSL/Transform";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompileCategory {
    Invalid,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompileFailure {
    pub(crate) code: &'static str,
    pub(crate) category: CompileCategory,
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn compile_stylesheet(document: &Document) -> Result<StylesheetProgram, CompileFailure> {
    let root = document_element(document)?;
    require_name(document, root, Some(XSLT_NAMESPACE), "stylesheet")?;
    let declared_version = required_attribute(document, root, None, "version")?.to_owned();

    let mut output = None;
    let mut root_template = None;
    let mut matched_templates = Vec::new();
    let mut named_templates = Vec::new();
    let mut global_bindings = Vec::new();
    for child in meaningful_children(document, root) {
        let Some(name) = document.name(child) else {
            continue;
        };
        match (name.namespace.as_deref(), name.local.as_str()) {
            (Some(XSLT_NAMESPACE), "output") => {
                if output.is_some() {
                    return Err(invalid(
                        "FXST0002",
                        "the private slice permits one xsl:output declaration",
                        document.location(child),
                    ));
                }
                output = Some(compile_output(document, child)?);
            }
            (Some(XSLT_NAMESPACE), "template") => {
                compile_top_level_template(
                    document,
                    child,
                    &mut root_template,
                    &mut matched_templates,
                    &mut named_templates,
                )?;
            }
            (Some(XSLT_NAMESPACE), "variable" | "param") => {
                let kind = if name.local == "variable" {
                    GlobalBindingKind::Variable
                } else {
                    GlobalBindingKind::Parameter
                };
                let binding = compile_global_binding(document, child, kind)?;
                if global_bindings
                    .iter()
                    .any(|existing: &GlobalBinding| existing.name == binding.name)
                {
                    return Err(invalid(
                        "FXST0022",
                        format!("duplicate global binding: ${}", binding.name),
                        document.location(child),
                    ));
                }
                global_bindings.push(binding);
            }
            (Some(XSLT_NAMESPACE), local) => {
                return Err(unsupported(
                    "FXST1002",
                    format!("unsupported top-level XSLT declaration: xsl:{local}"),
                    document.location(child),
                ));
            }
            _ => {
                return Err(unsupported(
                    "FXST1003",
                    "literal top-level elements are outside the private slice",
                    document.location(child),
                ));
            }
        }
    }

    let program = StylesheetProgram {
        declared_version,
        output: output.unwrap_or(OutputSettings {
            method: None,
            omit_xml_declaration: false,
        }),
        root_template,
        matched_templates,
        named_templates,
        global_bindings,
    };
    validate_named_template_references(&program)?;
    Ok(program)
}

fn compile_top_level_template(
    document: &Document,
    element: NodeId,
    root_template: &mut Option<Template>,
    matched_templates: &mut Vec<MatchedTemplate>,
    named_templates: &mut Vec<NamedTemplate>,
) -> Result<(), CompileFailure> {
    if let Some(name) = optional_attribute(document, element, None, "name") {
        let name = normalize_named_template_name(document, element, name)?;
        if named_templates.iter().any(|template| template.name == name) {
            return Err(invalid(
                "FXST0010",
                format!("duplicate named template: {name}"),
                document.location(element),
            ));
        }
        named_templates.push(compile_named_template(document, element, &name)?);
        return Ok(());
    }

    let pattern = required_attribute(document, element, None, "match")?;
    if pattern == "/" {
        if optional_attribute(document, element, None, "mode").is_some() {
            return Err(unsupported(
                "FXST1011",
                "a mode on the root match pattern is outside the private slice",
                document.location(element),
            ));
        }
        if root_template.is_some() {
            return Err(unsupported(
                "FXST1001",
                "the private slice permits one root template",
                document.location(element),
            ));
        }
        *root_template = Some(compile_template(document, element)?);
        return Ok(());
    }

    let matched_template = compile_matched_template(document, element, pattern)?;
    if matched_templates.iter().any(|existing| {
        existing.pattern == matched_template.pattern && existing.mode == matched_template.mode
    }) {
        return Err(unsupported(
            "FXST1008",
            format!(
                "template priority for duplicate match pattern is outside the private slice: {pattern}"
            ),
            document.location(element),
        ));
    }
    matched_templates.push(matched_template);
    Ok(())
}

fn compile_global_binding(
    document: &Document,
    element: NodeId,
    kind: GlobalBindingKind,
) -> Result<GlobalBinding, CompileFailure> {
    let label = match kind {
        GlobalBindingKind::Variable => "xsl:variable",
        GlobalBindingKind::Parameter => "xsl:param",
    };
    ensure_only_attributes(document, element, &["name", "select"], label)?;
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(invalid(
            "FXST0023",
            format!("invalid global binding name: ${name}"),
            document.location(element),
        ));
    }
    if document
        .children(element)
        .iter()
        .any(|node| document.kind(*node) == NodeKind::Element)
    {
        return Err(unsupported(
            "FXST1015",
            format!("{label} sequence constructors with elements are outside the private slice"),
            document.location(element),
        ));
    }
    let default = if let Some(select) = optional_attribute(document, element, None, "select") {
        ensure_no_meaningful_children(document, element, label)?;
        if let Some(variable) = select.strip_prefix('$') {
            if !is_ascii_ncname(variable) {
                return Err(invalid(
                    "FXXP0002",
                    format!("invalid variable reference: {select}"),
                    document.location(element),
                ));
            }
            GlobalBindingDefault::Variable(variable.to_owned())
        } else {
            GlobalBindingDefault::ChildPath(
                parse_child_path(select, document.location(element).clone())
                    .map_err(map_path_failure)?,
            )
        }
    } else {
        GlobalBindingDefault::Text(document.string_value(element))
    };
    Ok(GlobalBinding {
        kind,
        name: name.to_owned(),
        default,
    })
}

fn compile_output(document: &Document, element: NodeId) -> Result<OutputSettings, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["method", "omit-xml-declaration"],
        "xsl:output",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:output")?;
    let method = required_attribute(document, element, None, "method")?;
    if method != "xml" {
        return Err(unsupported(
            "FXST1004",
            format!("unsupported output method: {method}"),
            document.location(element),
        ));
    }
    let omit = required_attribute(document, element, None, "omit-xml-declaration")?;
    let omit_xml_declaration = match omit {
        "yes" => true,
        "no" => false,
        _ => {
            return Err(invalid(
                "FXST0005",
                "omit-xml-declaration must be 'yes' or 'no'",
                document.location(element),
            ));
        }
    };
    Ok(OutputSettings {
        method: Some(method.to_owned()),
        omit_xml_declaration,
    })
}

fn compile_matched_template(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> Result<MatchedTemplate, CompileFailure> {
    let pattern = match pattern {
        "comment()" => MatchPattern::Comment,
        "processing-instruction()" => MatchPattern::ProcessingInstruction,
        "node()" => MatchPattern::AnyNode,
        attribute if attribute.starts_with('@') && is_ascii_ncname(&attribute[1..]) => {
            MatchPattern::Attribute(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: attribute[1..].to_owned(),
            })
        }
        name if is_ascii_ncname(name) => {
            MatchPattern::Element(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: name.to_owned(),
            })
        }
        path if path.contains('/') && !path.starts_with('/') => MatchPattern::Path(
            parse_child_path(path, document.location(element).clone()).map_err(map_path_failure)?,
        ),
        _ => {
            return Err(unsupported(
                "FXST1005",
                format!("unsupported template match pattern: {pattern}"),
                document.location(element),
            ));
        }
    };
    let mode = optional_attribute(document, element, None, "mode")
        .map(|mode| parse_mode(mode, document.location(element)))
        .transpose()?;
    Ok(MatchedTemplate {
        pattern,
        mode,
        template: compile_template(document, element)?,
    })
}

fn compile_template(document: &Document, element: NodeId) -> Result<Template, CompileFailure> {
    ensure_only_attributes(document, element, &["match", "mode"], "xsl:template")?;
    Ok(Template {
        body: compile_sequence(document, element)?,
        location: document.location(element).clone(),
    })
}

fn compile_named_template(
    document: &Document,
    element: NodeId,
    name: &str,
) -> Result<NamedTemplate, CompileFailure> {
    ensure_only_attributes(document, element, &["name"], "xsl:template")?;
    let mut parameters = Vec::new();
    let mut parameter_nodes = Vec::new();
    let mut body_started = false;
    for child in meaningful_children(document, element) {
        if is_xslt_element(document, child, "param") {
            if body_started {
                return Err(invalid(
                    "FXST0011",
                    "xsl:param must precede the named-template body",
                    document.location(child),
                ));
            }
            ensure_only_attributes(document, child, &["name"], "xsl:param")?;
            ensure_no_meaningful_children(document, child, "xsl:param")?;
            let parameter = required_attribute(document, child, None, "name")?;
            if !is_ascii_ncname(parameter) || parameters.iter().any(|name| name == parameter) {
                return Err(invalid(
                    "FXST0012",
                    format!("invalid or duplicate named-template parameter: {parameter}"),
                    document.location(child),
                ));
            }
            parameters.push(parameter.to_owned());
            parameter_nodes.push(child);
        } else {
            body_started = true;
        }
    }
    Ok(NamedTemplate {
        name: name.to_owned(),
        parameters,
        template: Template {
            body: compile_sequence_excluding(document, element, &parameter_nodes)?,
            location: document.location(element).clone(),
        },
    })
}

fn normalize_named_template_name(
    document: &Document,
    element: NodeId,
    name: &str,
) -> Result<String, CompileFailure> {
    if is_ascii_ncname(name) {
        return Ok(name.to_owned());
    }
    let Some((prefix, local)) = name.split_once(':') else {
        return Err(unsupported(
            "FXST1013",
            format!("unsupported named-template name: {name}"),
            document.location(element),
        ));
    };
    if !is_ascii_ncname(prefix) || !is_ascii_ncname(local) {
        return Err(unsupported(
            "FXST1013",
            format!("unsupported named-template name: {name}"),
            document.location(element),
        ));
    }
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(binding) = document
            .namespace_declarations(node)
            .iter()
            .find(|binding| binding.prefix.as_deref() == Some(prefix))
        {
            if binding.namespace == XSLT_NAMESPACE && local == "initial-template" {
                return Ok(STANDARD_INITIAL_TEMPLATE_NAME.to_owned());
            }
            break;
        }
        current = document.parent(node);
    }
    Err(unsupported(
        "FXST1013",
        format!("unsupported named-template name: {name}"),
        document.location(element),
    ))
}

fn compile_sequence(
    document: &Document,
    parent: NodeId,
) -> Result<Vec<Instruction>, CompileFailure> {
    compile_sequence_excluding(document, parent, &[])
}

fn compile_sequence_excluding(
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
                    if name.local == "value-of" {
                        instructions.push(compile_value_of(document, child)?);
                    } else if name.local == "variable" {
                        let variable = compile_variable(document, child)?;
                        let Instruction::Variable { name, .. } = &variable else {
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

fn literal_result_namespaces(document: &Document, element: NodeId) -> Vec<NamespaceBinding> {
    let mut namespaces = Vec::new();
    let mut current = Some(element);
    while let Some(node) = current {
        for binding in document.namespace_declarations(node) {
            let Some(prefix) = binding.prefix.as_deref() else {
                continue;
            };
            if prefix != "xml"
                && binding.namespace != XSLT_NAMESPACE
                && !binding.namespace.is_empty()
                && !namespaces
                    .iter()
                    .any(|existing: &NamespaceBinding| existing.prefix.as_deref() == Some(prefix))
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
        .map(|expression| parse_apply_selection(expression, location.clone()))
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
    expression: &str,
    location: SourceLocation,
) -> Result<ApplySelection, CompileFailure> {
    let node_test = match expression {
        "comment()" => Some(NodeTest::Comment),
        "processing-instruction()" => Some(NodeTest::ProcessingInstruction),
        "node()" => Some(NodeTest::AnyNode),
        _ => None,
    };
    if let Some(node_test) = node_test {
        return Ok(ApplySelection::ChildNodes(node_test));
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
    parse_child_path(expression, location)
        .map(ApplySelection::ChildPath)
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

fn compile_value_of(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select", "separator"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
    let select = if expression.contains(" castable as ") {
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
        ValueExpression::ChildPath(
            parse_child_path(expression, location.clone()).map_err(map_path_failure)?,
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
    ensure_only_attributes(document, element, &["name", "select"], "xsl:variable")?;
    ensure_no_meaningful_children(document, element, "xsl:variable")?;
    let location = document.location(element).clone();
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(invalid(
            "FXST0016",
            format!("invalid local variable name: {name}"),
            &location,
        ));
    }
    let expression = required_attribute(document, element, None, "select")?;
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

fn compile_sequence_nodes(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:sequence")?;
    ensure_no_meaningful_children(document, element, "xsl:sequence")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
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

fn is_xslt_element(document: &Document, node: NodeId, local: &str) -> bool {
    document.name(node).is_some_and(|name| {
        document.kind(node) == NodeKind::Element
            && name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            && name.local == local
    })
}

fn validate_named_template_references(program: &StylesheetProgram) -> Result<(), CompileFailure> {
    if let Some(root) = &program.root_template {
        validate_named_calls(program, &root.body)?;
    }
    for template in &program.matched_templates {
        validate_named_calls(program, &template.template.body)?;
    }
    for template in &program.named_templates {
        validate_named_calls(program, &template.template.body)?;
    }
    Ok(())
}

fn validate_named_calls(
    program: &StylesheetProgram,
    instructions: &[Instruction],
) -> Result<(), CompileFailure> {
    for instruction in instructions {
        match instruction {
            Instruction::LiteralElement { body, .. } | Instruction::If { body, .. } => {
                validate_named_calls(program, body)?;
            }
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                for branch in branches {
                    validate_named_calls(program, &branch.body)?;
                }
                validate_named_calls(program, otherwise)?;
            }
            Instruction::CallTemplate {
                name,
                arguments,
                location,
            } => {
                let target = program
                    .named_templates
                    .iter()
                    .find(|template| template.name == *name)
                    .ok_or_else(|| {
                        invalid(
                            "FXST0014",
                            format!("unknown named template: {name}"),
                            location,
                        )
                    })?;
                if let Some(argument) = arguments
                    .iter()
                    .find(|argument| !target.parameters.contains(&argument.name))
                {
                    return Err(invalid(
                        "FXST0015",
                        format!(
                            "unknown parameter {} for named template {name}",
                            argument.name
                        ),
                        location,
                    ));
                }
            }
            Instruction::Text { .. }
            | Instruction::ValueOf { .. }
            | Instruction::Variable { .. }
            | Instruction::SequenceNodes { .. }
            | Instruction::ApplyTemplates { .. } => {}
        }
    }
    Ok(())
}

fn ensure_only_attributes(
    document: &Document,
    element: NodeId,
    allowed: &[&str],
    display_name: &str,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.is_some() || !allowed.contains(&name.local.as_str()) {
            return Err(unsupported(
                "FXST1009",
                format!(
                    "unsupported attribute on {display_name}: {{{}}}{}",
                    name.namespace.as_deref().unwrap_or(""),
                    name.local
                ),
                document.location(*attribute),
            ));
        }
    }
    Ok(())
}

fn document_element(document: &Document) -> Result<NodeId, CompileFailure> {
    let root = document.document_node();
    let elements: Vec<_> = document
        .children(root)
        .iter()
        .copied()
        .filter(|node| document.kind(*node) == NodeKind::Element)
        .collect();
    if let [element] = elements.as_slice() {
        return Ok(*element);
    }
    Err(invalid(
        "FXST0001",
        "a stylesheet document must contain exactly one document element",
        document.location(root),
    ))
}

fn meaningful_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|child| match document.kind(*child) {
            NodeKind::Comment | NodeKind::ProcessingInstruction => false,
            NodeKind::Text => !document
                .value(*child)
                .unwrap_or_default()
                .chars()
                .all(char::is_whitespace),
            _ => true,
        })
        .collect()
}

fn ensure_no_meaningful_children(
    document: &Document,
    element: NodeId,
    display_name: &str,
) -> Result<(), CompileFailure> {
    if meaningful_children(document, element).is_empty() {
        Ok(())
    } else {
        Err(invalid(
            "FXST0007",
            format!("{display_name} must be empty in the private slice"),
            document.location(element),
        ))
    }
}

fn required_attribute<'a>(
    document: &'a Document,
    element: NodeId,
    namespace: Option<&str>,
    local: &str,
) -> Result<&'a str, CompileFailure> {
    document
        .attributes(element)
        .iter()
        .copied()
        .find(|attribute| {
            document
                .name(*attribute)
                .is_some_and(|name| name.namespace.as_deref() == namespace && name.local == local)
        })
        .and_then(|attribute| document.value(attribute))
        .ok_or_else(|| {
            invalid(
                "FXST0008",
                format!("missing required attribute: {local}"),
                document.location(element),
            )
        })
}

fn optional_attribute<'a>(
    document: &'a Document,
    element: NodeId,
    namespace: Option<&str>,
    local: &str,
) -> Option<&'a str> {
    document
        .attributes(element)
        .iter()
        .copied()
        .find(|attribute| {
            document
                .name(*attribute)
                .is_some_and(|name| name.namespace.as_deref() == namespace && name.local == local)
        })
        .and_then(|attribute| document.value(attribute))
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

fn require_name(
    document: &Document,
    node: NodeId,
    namespace: Option<&str>,
    local: &str,
) -> Result<(), CompileFailure> {
    if document
        .name(node)
        .is_some_and(|name| name.namespace.as_deref() == namespace && name.local == local)
    {
        Ok(())
    } else {
        Err(invalid(
            "FXST0009",
            format!("expected element: {{{}}}{local}", namespace.unwrap_or("")),
            document.location(node),
        ))
    }
}

fn map_path_failure(failure: PathFailure) -> CompileFailure {
    match failure {
        PathFailure::Invalid { detail, location } => CompileFailure {
            code: "FXXP0001",
            category: CompileCategory::Invalid,
            detail,
            location,
        },
        PathFailure::Unsupported { detail, location } => CompileFailure {
            code: "FXXP1001",
            category: CompileCategory::Unsupported,
            detail,
            location,
        },
    }
}

fn invalid(
    code: &'static str,
    detail: impl Into<String>,
    location: &SourceLocation,
) -> CompileFailure {
    CompileFailure {
        code,
        category: CompileCategory::Invalid,
        detail: detail.into(),
        location: location.clone(),
    }
}

fn unsupported(
    code: &'static str,
    detail: impl Into<String>,
    location: &SourceLocation,
) -> CompileFailure {
    CompileFailure {
        code,
        category: CompileCategory::Unsupported,
        detail: detail.into(),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
    use crate::xslt::golden_semantics_experiment::{
        Instruction, STANDARD_INITIAL_TEMPLATE_NAME, ValueExpression,
    };

    use super::{CompileCategory, compile_stylesheet};

    const LIMITS: ParseLimits = ParseLimits {
        max_events: 256,
        max_depth: 32,
    };

    fn parse_stylesheet(resource: &str, bytes: &[u8]) -> Document {
        let parsed = parse_document(resource, bytes, LIMITS).expect("stylesheet XML should parse");
        Document::from_parsed(parsed).expect("stylesheet XDM should build")
    }

    #[test]
    fn compiles_the_golden_stylesheet_into_owned_semantics() {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/golden/hello/stylesheet.xsl"
        ));
        let document = parse_stylesheet("golden:hello/stylesheet.xsl", bytes);

        let program = compile_stylesheet(&document).expect("golden stylesheet should compile");

        assert_eq!(program.declared_version, "1.0");
        assert_eq!(program.output.method.as_deref(), Some("xml"));
        assert!(program.output.omit_xml_declaration);
        let [Instruction::LiteralElement { name, body, .. }] = program
            .root_template
            .as_ref()
            .expect("root template")
            .body
            .as_slice()
        else {
            panic!("root template should contain one literal result element");
        };
        assert_eq!(name.namespace, None);
        assert_eq!(name.local, "message");
        assert!(matches!(
            body.as_slice(),
            [
                Instruction::Text { value: first, .. },
                Instruction::ValueOf { select, .. },
                Instruction::Text { value: last, .. }
            ] if first == "Hello, "
                && matches!(select, ValueExpression::ChildPath(path)
                    if path.steps == ["greeting", "name"])
                && last == "!"
        ));
        assert_eq!(
            program
                .root_template
                .as_ref()
                .expect("root template")
                .location
                .resource,
            "golden:hello/stylesheet.xsl"
        );
    }

    #[test]
    fn preserves_absent_output_declaration_for_runtime_method_inference() {
        let stylesheet = parse_stylesheet(
            "memory:default-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");

        assert_eq!(program.output.method, None);
        assert!(!program.output.omit_xml_declaration);
    }

    #[test]
    fn compiles_exact_element_template_dispatch_without_priority_semantics() {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/golden/template-dispatch/stylesheet.xsl"
        ));
        let document = parse_stylesheet("golden:template-dispatch/stylesheet.xsl", bytes);

        let program = compile_stylesheet(&document).expect("dispatch stylesheet should compile");

        assert_eq!(program.matched_templates.len(), 1);
        assert!(matches!(
            &program.matched_templates[0].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
                if name.local == "item"
        ));
        assert!(matches!(
            program
                .root_template
                .as_ref()
                .expect("root template")
                .body
                .as_slice(),
            [Instruction::LiteralElement { body, .. }]
                if matches!(body.as_slice(), [Instruction::ApplyTemplates { select: Some(select), .. }]
                    if matches!(select,
                        crate::xslt::golden_semantics_experiment::ApplySelection::ChildPath(path)
                            if path.steps == ["catalog", "item"]))
        ));

        let duplicate = parse_stylesheet(
            "memory:duplicate-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out/></xsl:template><xsl:template match="item"><a/></xsl:template><xsl:template match="item"><b/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&duplicate)
            .expect_err("priority conflict must remain visibly unsupported");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXST1008");

        let mode = parse_stylesheet(
            "memory:mode.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:apply-templates select="root/item" mode="detail"/></xsl:template><xsl:template match="item" mode="detail"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&mode).expect("unprefixed modes should compile");
        assert_eq!(program.matched_templates[0].mode.as_deref(), Some("detail"));
    }

    #[test]
    fn distinguishes_invalid_stylesheet_from_unsupported_instruction() {
        let invalid = parse_stylesheet(
            "memory:invalid.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:value-of/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&invalid).expect_err("missing select should fail");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "FXST0008");
        assert_eq!(failure.location.resource, "memory:invalid.xsl");

        let unsupported = parse_stylesheet(
            "memory:unsupported.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:for-each select="item"/></xsl:template></xsl:stylesheet>"#,
        );
        let failure =
            compile_stylesheet(&unsupported).expect_err("unsupported instruction should fail");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXST1006");
        assert_eq!(failure.location.resource, "memory:unsupported.xsl");

        let named_template = parse_stylesheet(
            "memory:named-template.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="worker"><out/></xsl:template><xsl:template match="/"><xsl:call-template name="worker"/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&named_template).expect("named template should compile");
        assert_eq!(program.named_templates.len(), 1);
        assert_eq!(program.named_templates[0].name, "worker");

        let standard_initial_template = parse_stylesheet(
            "memory:standard-initial-template.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="xsl:initial-template"><out>ok</out></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&standard_initial_template)
            .expect("the reserved standard initial-template name should compile");
        assert_eq!(
            program.named_templates[0].name,
            STANDARD_INITIAL_TEMPLATE_NAME
        );

        let unknown_call = parse_stylesheet(
            "memory:unknown-template.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:call-template name="missing"/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&unknown_call)
            .expect_err("unknown named-template references are statically invalid");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "FXST0014");
    }

    #[test]
    fn classifies_xpath_outside_the_private_child_path_slice_as_unsupported() {
        let stylesheet = parse_stylesheet(
            "memory:path.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><value><xsl:value-of select="greeting//name"/></value></xsl:template></xsl:stylesheet>"#,
        );

        let failure = compile_stylesheet(&stylesheet).expect_err("unsupported XPath should fail");

        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXXP1001");
        assert_eq!(failure.location.resource, "memory:path.xsl");
    }
}
