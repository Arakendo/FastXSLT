//! Compiles template invocation, arguments, selection, and mode controls.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, Instruction, NodeTest, TemplateArgument, TemplateArgumentValue,
};

use super::super::variable_filtered_path_compiler::parse as parse_variable_filtered_path;
use super::{
    CompileFailure, effective_xpath_default_namespace, ensure_no_meaningful_children,
    ensure_only_attributes, invalid, is_ascii_ncname, is_xslt_element, map_path_failure,
    meaningful_children, optional_attribute, required_attribute, unsupported,
};

pub(super) fn compile_apply_imports(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &[], "xsl:apply-imports")?;
    Ok(Instruction::ApplyImports {
        arguments: compile_with_params(document, element, "xsl:apply-imports", false)?,
        location: document.location(element).clone(),
    })
}

pub(super) fn compile_apply_templates(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["select", "mode"],
        "xsl:apply-templates",
    )?;
    let location = document.location(element).clone();
    let select = optional_attribute(document, element, None, "select")
        .map(|expression| parse_apply_selection(document, element, expression, location.clone()))
        .transpose()?;
    let mode = optional_attribute(document, element, None, "mode")
        .map(|mode| parse_apply_mode(document, element, mode))
        .transpose()?;
    Ok(Instruction::ApplyTemplates {
        select,
        mode,
        arguments: compile_with_params(document, element, "xsl:apply-templates", false)?,
        location,
    })
}

pub(super) fn compile_with_params(
    document: &Document,
    parent: NodeId,
    parent_label: &str,
    allow_fallback: bool,
) -> Result<Vec<TemplateArgument>, CompileFailure> {
    let mut arguments = Vec::new();
    for child in meaningful_children(document, parent) {
        if allow_fallback && is_xslt_element(document, child, "fallback") {
            ensure_only_attributes(document, child, &[], "xsl:fallback")?;
            continue;
        }
        if !is_xslt_element(document, child, "with-param") {
            return Err(unsupported(
                "FXST1014",
                format!("the private {parent_label} slice permits only xsl:with-param children"),
                document.location(child),
            ));
        }
        ensure_only_attributes(document, child, &["name", "select"], "xsl:with-param")?;
        ensure_no_meaningful_children(document, child, "xsl:with-param")?;
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
        let select = required_attribute(document, child, None, "select")?;
        let value = if let Some(variable) = select.strip_prefix('$') {
            if !is_ascii_ncname(variable) {
                return Err(invalid(
                    "FXXP0002",
                    format!("invalid variable reference: {select}"),
                    document.location(child),
                ));
            }
            TemplateArgumentValue::Variable(variable.to_owned())
        } else {
            TemplateArgumentValue::Integer(select.parse::<i64>().map_err(|_| {
                unsupported(
                    "FXXP1011",
                    format!("unsupported template argument expression: {select}"),
                    document.location(child),
                )
            })?)
        };
        arguments.push(TemplateArgument {
            name: argument_name.to_owned(),
            value,
            location: document.location(child).clone(),
        });
    }
    Ok(arguments)
}

pub(super) fn parse_apply_selection(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: SourceLocation,
) -> Result<ApplySelection, CompileFailure> {
    if let Some(path) = parse_variable_filtered_path(expression) {
        return Ok(ApplySelection::VariableFilteredElementPath(path));
    }
    if let Some(variable) = expression
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        return Ok(ApplySelection::TemporaryRoot(variable.to_owned()));
    }
    if let Some(variable) = expression
        .strip_prefix('$')
        .and_then(|value| value.strip_suffix("/*"))
        .filter(|name| is_ascii_ncname(name))
    {
        return Ok(ApplySelection::GlobalTemporaryChildren(variable.to_owned()));
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
            return Ok(ApplySelection::ChildElement(expanded_name(
                Some(namespace),
                expression,
            )));
        }
    }
    if let Some(local) = expression
        .strip_prefix("//")
        .filter(|name| is_ascii_ncname(name))
    {
        if let Some(namespace) = effective_xpath_default_namespace(document, element) {
            return Ok(ApplySelection::DescendantElement(expanded_name(
                Some(namespace),
                local,
            )));
        }
    }
    if let Some(attribute) = expression
        .strip_prefix('@')
        .filter(|name| is_ascii_ncname(name))
    {
        return Ok(ApplySelection::Attribute(expanded_name(None, attribute)));
    }
    if expression == "/" {
        return parse_location_path(expression, location)
            .map(ApplySelection::LocationPath)
            .map_err(map_path_failure);
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

fn expanded_name(
    namespace: Option<&str>,
    local: &str,
) -> crate::xml::quick_xml_experiment::ExpandedName {
    crate::xml::quick_xml_experiment::ExpandedName {
        namespace: namespace.map(str::to_owned),
        local: local.to_owned(),
    }
}

pub(super) fn parse_apply_mode(
    document: &Document,
    element: NodeId,
    mode: &str,
) -> Result<String, CompileFailure> {
    if matches!(mode, "#current" | "#default") {
        Ok(mode.to_owned())
    } else {
        parse_mode(document, element, mode)
    }
}

pub(super) fn parse_mode(
    document: &Document,
    element: NodeId,
    mode: &str,
) -> Result<String, CompileFailure> {
    if is_ascii_ncname(mode) {
        return Ok(mode.to_owned());
    }
    if let Some((prefix, local)) = mode.split_once(':').filter(|(prefix, local)| {
        is_ascii_ncname(prefix) && is_ascii_ncname(local) && !local.contains(':')
    }) {
        let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
            invalid(
                "FXST0031",
                format!("unbound prefix in mode name: {prefix}"),
                document.location(element),
            )
        })?;
        return Ok(format!("Q{{{namespace}}}{local}"));
    }
    Err(unsupported(
        "FXST1012",
        format!("unsupported mode name: {mode}"),
        document.location(element),
    ))
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

pub(super) fn parse_template_modes(
    document: &Document,
    element: NodeId,
    mode: &str,
) -> Result<Vec<String>, CompileFailure> {
    let modes = mode
        .split_whitespace()
        .map(|name| {
            if matches!(name, "#all" | "#default") {
                Ok(name.to_owned())
            } else {
                parse_mode(document, element, name)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if modes.is_empty() {
        return Err(unsupported(
            "FXST1012",
            "template mode list is empty",
            document.location(element),
        ));
    }
    Ok(modes)
}

pub(super) fn compile_call_template(
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
            value: TemplateArgumentValue::Text(document.string_value(child)),
            location: document.location(child).clone(),
        });
    }
    Ok(Instruction::CallTemplate {
        name: name.to_owned(),
        arguments,
        location: document.location(element).clone(),
    })
}
