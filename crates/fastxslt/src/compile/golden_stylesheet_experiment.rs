use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xpath::path_experiment::{PathFailure, parse_location_path};
use crate::xslt::golden_semantics_experiment::{
    ConstructedElement, GlobalBinding, GlobalBindingDefault, GlobalBindingKind, MatchPattern,
    MatchedTemplate, NamedTemplate, STANDARD_INITIAL_TEMPLATE_NAME, StylesheetProgram, Template,
    TemplateParameter, TemplateParameterDefault, TemplatePriority,
};

#[path = "instruction_compiler.rs"]
mod instruction_compiler;
#[path = "output_compiler.rs"]
mod output_compiler;
#[path = "stylesheet_module_compiler.rs"]
mod stylesheet_module_compiler;
#[path = "stylesheet_validation.rs"]
mod stylesheet_validation;
#[path = "template_pattern_compiler.rs"]
mod template_pattern_compiler;
#[path = "variable_filtered_path_compiler.rs"]
mod variable_filtered_path_compiler;

pub(crate) use stylesheet_module_compiler::{
    StylesheetDependencyKind, compile_stylesheet_with_import_and_include,
    compile_stylesheet_with_imports, compile_stylesheet_with_single_include,
    compile_stylesheet_with_single_include_program_at, discovered_stylesheet_dependencies_at,
};
use stylesheet_validation::validate_named_template_references;
use template_pattern_compiler::compile_match_pattern;

use instruction_compiler::{
    compile_sequence_excluding, literal_result_namespaces, parse_template_modes,
};
use output_compiler::compile_output;
pub(super) use output_compiler::default_output_settings;

pub(super) const XSLT_NAMESPACE: &str = "http://www.w3.org/1999/XSL/Transform";
const XML_SCHEMA_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";

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
    compile_stylesheet_at(document, root)
}

pub(crate) fn compile_stylesheet_at(
    document: &Document,
    root: NodeId,
) -> Result<StylesheetProgram, CompileFailure> {
    let program = compile_stylesheet_at_excluding_unvalidated(document, root, &[])?;
    validate_named_template_references(&program)?;
    Ok(program)
}

pub(super) fn compile_stylesheet_excluding_unvalidated(
    document: &Document,
    excluded_top_level: &[NodeId],
) -> Result<StylesheetProgram, CompileFailure> {
    let root = document_element(document)?;
    compile_stylesheet_at_excluding_unvalidated(document, root, excluded_top_level)
}

pub(super) fn compile_stylesheet_at_excluding_unvalidated(
    document: &Document,
    root: NodeId,
    excluded_top_level: &[NodeId],
) -> Result<StylesheetProgram, CompileFailure> {
    require_stylesheet_root(document, root)?;
    let declared_version = required_attribute(document, root, None, "version")?.to_owned();

    let mut output = None;
    let mut root_template = None;
    let mut root_template_modes = Vec::new();
    let mut matched_templates = Vec::new();
    let mut named_templates = Vec::new();
    let mut global_bindings = Vec::new();
    for child in meaningful_children(document, root)
        .into_iter()
        .filter(|child| !excluded_top_level.contains(child))
    {
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
                output = Some(compile_output(document, child, &declared_version)?);
            }
            (Some(XSLT_NAMESPACE), "template") => {
                compile_top_level_template(
                    document,
                    child,
                    &mut root_template,
                    &mut root_template_modes,
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

    Ok(StylesheetProgram {
        declared_version,
        output: output.unwrap_or_else(default_output_settings),
        root_template,
        root_template_modes,
        matched_templates,
        named_templates,
        global_bindings,
    })
}

pub(super) fn require_stylesheet_root(
    document: &Document,
    root: NodeId,
) -> Result<(), CompileFailure> {
    if document.name(root).is_some_and(|name| {
        name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            && matches!(name.local.as_str(), "stylesheet" | "transform")
    }) {
        Ok(())
    } else {
        Err(invalid(
            "FXST0009",
            "expected xsl:stylesheet or its xsl:transform synonym",
            document.location(root),
        ))
    }
}

fn compile_top_level_template(
    document: &Document,
    element: NodeId,
    root_template: &mut Option<Template>,
    root_template_modes: &mut Vec<String>,
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
        let has_priority = optional_attribute(document, element, None, "priority").is_some();
        let has_mode = optional_attribute(document, element, None, "mode").is_some();
        let has_competing_default = matched_templates.iter().any(|existing| {
            existing.pattern == MatchPattern::Document && existing.modes.is_empty()
        });
        if has_priority || has_mode || root_template.is_some() || has_competing_default {
            let matched_template = compile_matched_template(document, element, pattern)?;
            let competes_with_default = matched_template.modes.is_empty()
                || matched_template
                    .modes
                    .iter()
                    .any(|mode| matches!(mode.as_str(), "#default" | "#all"));
            if competes_with_default {
                if let Some(previous) = root_template.take() {
                    matched_templates.push(MatchedTemplate {
                        pattern: MatchPattern::Document,
                        import_precedence: 0,
                        priority: TemplatePriority::ROOT_DEFAULT,
                        modes: Vec::new(),
                        template: previous,
                    });
                    root_template_modes.clear();
                }
                matched_templates.push(matched_template);
                return Ok(());
            }
            if matched_templates.iter().any(|existing| {
                existing.pattern == matched_template.pattern
                    && existing.modes == matched_template.modes
            }) {
                return Err(unsupported(
                    "FXST1008",
                    "template priority for duplicate mode-qualified root pattern is outside the private slice",
                    document.location(element),
                ));
            }
            matched_templates.push(matched_template);
            return Ok(());
        }
        root_template_modes.clear();
        *root_template = Some(compile_template(document, element)?);
        return Ok(());
    }

    let matched_template = compile_matched_template(document, element, pattern)?;
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
    let allowed_attributes = match kind {
        GlobalBindingKind::Variable => &["name", "select"][..],
        GlobalBindingKind::Parameter => &["name", "select", "required"][..],
    };
    ensure_only_attributes(document, element, allowed_attributes, label)?;
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(invalid(
            "FXST0023",
            format!("invalid global binding name: ${name}"),
            document.location(element),
        ));
    }
    let required = match optional_attribute(document, element, None, "required") {
        None | Some("no") => false,
        Some("yes") => true,
        Some(value) => {
            return Err(invalid(
                "FXST0025",
                format!("invalid xsl:param required value: {value}"),
                document.location(element),
            ));
        }
    };
    if required
        && (optional_attribute(document, element, None, "select").is_some()
            || !document.string_value(element).trim().is_empty())
    {
        return Err(invalid(
            "FXST0026",
            "a required global parameter cannot declare a default value",
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
        } else if let Some(value) = select
            .strip_prefix('\'')
            .and_then(|value| value.strip_suffix('\''))
        {
            GlobalBindingDefault::Text(value.to_owned())
        } else if let Ok(value) = select.parse::<i64>() {
            GlobalBindingDefault::Integer(value)
        } else {
            GlobalBindingDefault::LocationPath(
                parse_location_path(select, document.location(element).clone())
                    .map_err(map_path_failure)?,
            )
        }
    } else if document
        .children(element)
        .iter()
        .any(|node| document.kind(*node) == NodeKind::Element)
    {
        GlobalBindingDefault::TemporaryTree(compile_constructed_elements(document, element)?)
    } else {
        GlobalBindingDefault::Text(document.string_value(element))
    };
    Ok(GlobalBinding {
        kind,
        name: name.to_owned(),
        required,
        default,
    })
}

fn compile_constructed_elements(
    document: &Document,
    parent: NodeId,
) -> Result<Vec<ConstructedElement>, CompileFailure> {
    let mut elements = Vec::new();
    for child in meaningful_children(document, parent) {
        if document.kind(child) != NodeKind::Element {
            return Err(unsupported(
                "FXST1015",
                "mixed-content global sequence constructors are outside the private slice",
                document.location(child),
            ));
        }
        let name = document.name(child).expect("element nodes have names");
        if name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            || !document.attributes(child).is_empty()
        {
            return Err(unsupported(
                "FXST1015",
                "only attribute-free literal elements are admitted in global temporary trees",
                document.location(child),
            ));
        }
        elements.push(ConstructedElement {
            name: name.clone(),
            namespaces: literal_result_namespaces(document, child),
            children: compile_constructed_elements(document, child)?,
        });
    }
    Ok(elements)
}

fn compile_matched_template(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> Result<MatchedTemplate, CompileFailure> {
    let (pattern, priority) = compile_match_pattern(document, element, pattern)?;
    let modes = optional_attribute(document, element, None, "mode")
        .map(|mode| parse_template_modes(mode, document.location(element)))
        .transpose()?;
    Ok(MatchedTemplate {
        pattern,
        import_precedence: 0,
        priority,
        modes: modes.unwrap_or_default(),
        template: compile_template(document, element)?,
    })
}

fn compile_template(document: &Document, element: NodeId) -> Result<Template, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["match", "mode", "priority", "xpath-default-namespace"],
        "xsl:template",
    )?;
    let mut parameters = Vec::new();
    let mut parameter_nodes = Vec::new();
    let mut body_started = false;
    for child in meaningful_children(document, element) {
        if is_xslt_element(document, child, "param") {
            if body_started {
                return Err(invalid(
                    "FXST0011",
                    "xsl:param must precede the template body",
                    document.location(child),
                ));
            }
            ensure_only_attributes(document, child, &["name", "tunnel", "select"], "xsl:param")?;
            ensure_no_meaningful_children(document, child, "xsl:param")?;
            let lexical_name = required_attribute(document, child, None, "name")?;
            let name = normalize_variable_qname(document, child, lexical_name)?;
            if parameters
                .iter()
                .any(|parameter: &TemplateParameter| parameter.name == name)
            {
                return Err(invalid(
                    "FXST0012",
                    format!("duplicate template parameter: {lexical_name}"),
                    document.location(child),
                ));
            }
            let tunnel = match optional_attribute(document, child, None, "tunnel") {
                None | Some("no") => false,
                Some("yes") => true,
                Some(value) => {
                    return Err(invalid(
                        "FXST0024",
                        format!("invalid xsl:param tunnel value: {value}"),
                        document.location(child),
                    ));
                }
            };
            let default = optional_attribute(document, child, None, "select").map_or_else(
                || Ok(TemplateParameterDefault::Text(String::new())),
                |select| {
                    select
                        .parse::<i64>()
                        .map(TemplateParameterDefault::Integer)
                        .map_err(|_| {
                            unsupported(
                                "FXST1032",
                                format!("unsupported template parameter default: {select}"),
                                document.location(child),
                            )
                        })
                },
            )?;
            parameters.push(TemplateParameter {
                name,
                tunnel,
                default,
            });
            parameter_nodes.push(child);
        } else {
            body_started = true;
        }
    }
    Ok(Template {
        parameters,
        body: compile_sequence_excluding(document, element, &parameter_nodes)?,
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
            parameters: Vec::new(),
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

fn normalize_variable_qname(
    document: &Document,
    element: NodeId,
    lexical_name: &str,
) -> Result<String, CompileFailure> {
    if is_ascii_ncname(lexical_name) {
        return Ok(lexical_name.to_owned());
    }
    let Some((prefix, local)) = lexical_name.split_once(':') else {
        return Err(invalid(
            "FXST0012",
            format!("invalid template parameter name: {lexical_name}"),
            document.location(element),
        ));
    };
    if !is_ascii_ncname(prefix) || !is_ascii_ncname(local) || local.contains(':') {
        return Err(invalid(
            "FXST0012",
            format!("invalid template parameter name: {lexical_name}"),
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
            return Ok(format!("Q{{{}}}{local}", binding.namespace));
        }
        current = document.parent(node);
    }
    Err(invalid(
        "FXST0012",
        format!("unbound prefix in template parameter name: {lexical_name}"),
        document.location(element),
    ))
}

pub(super) fn is_xslt_element(document: &Document, node: NodeId, local: &str) -> bool {
    document.name(node).is_some_and(|name| {
        document.kind(node) == NodeKind::Element
            && name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            && name.local == local
    })
}

pub(super) fn effective_xpath_default_namespace(
    document: &Document,
    element: NodeId,
) -> Option<&str> {
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(namespace) = optional_attribute(document, node, None, "xpath-default-namespace")
            .or_else(|| {
                optional_attribute(
                    document,
                    node,
                    Some(XSLT_NAMESPACE),
                    "xpath-default-namespace",
                )
            })
        {
            return (!namespace.is_empty()).then_some(namespace);
        }
        current = document.parent(node);
    }
    None
}

pub(super) fn ensure_only_attributes(
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

pub(super) fn document_element(document: &Document) -> Result<NodeId, CompileFailure> {
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

pub(super) fn meaningful_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
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

pub(super) fn ensure_no_meaningful_children(
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

pub(super) fn required_attribute<'a>(
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

pub(super) fn invalid(
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

pub(super) fn unsupported(
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
        Instruction, STANDARD_INITIAL_TEMPLATE_NAME, TemplatePriority, ValueExpression,
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
                && matches!(select, ValueExpression::LocationPath(path)
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
        assert_eq!(program.output.encoding, None);
        assert_eq!(program.output.media_type, None);
        assert_eq!(program.output.include_content_type, None);
        assert_eq!(program.output.byte_order_mark, None);
        assert!(!program.output.omit_xml_declaration);
    }

    #[test]
    fn rejects_non_utf8_encoding_in_the_private_string_result_lane() {
        let stylesheet = parse_stylesheet(
            "memory:unsupported-encoding.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" encoding="UTF-16"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let failure = compile_stylesheet(&stylesheet).expect_err("UTF-16 needs a byte result lane");

        assert_eq!(failure.code, "FXST1016");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }

    #[test]
    fn xslt30_boolean_output_lexicals_do_not_widen_xslt20_yes_no_values() {
        let xslt30 = parse_stylesheet(
            "memory:xslt30-output-boolean.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration=" 1 "/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
        let xslt20 = parse_stylesheet(
            "memory:xslt20-output-boolean.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="true"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&xslt30).expect("XSLT 3.0 boolean should compile");
        let failure = compile_stylesheet(&xslt20).expect_err("XSLT 2.0 requires yes or no");

        assert!(program.output.omit_xml_declaration);
        assert_eq!(failure.code, "FXST0005");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    #[test]
    fn preserves_output_media_type_as_owned_serialization_metadata() {
        let stylesheet = parse_stylesheet(
            "memory:media-type.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" media-type="application/x-fastxslt-test+xml"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&stylesheet).expect("media type should compile");

        assert_eq!(program.output.method.as_deref(), Some("xml"));
        assert_eq!(
            program.output.media_type.as_deref(),
            Some("application/x-fastxslt-test+xml")
        );
    }

    #[test]
    fn compiles_exact_element_template_dispatch_and_modes() {
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
                        crate::xslt::golden_semantics_experiment::ApplySelection::LocationPath(path)
                            if path.steps == ["catalog", "item"]))
        ));

        let duplicate = parse_stylesheet(
            "memory:duplicate-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out/></xsl:template><xsl:template match="item"><a/></xsl:template><xsl:template match="item"><b/></xsl:template></xsl:stylesheet>"#,
        );
        let duplicate_program =
            compile_stylesheet(&duplicate).expect("XSLT 3.0 use-last conflict should compile");
        assert_eq!(duplicate_program.matched_templates.len(), 2);
        assert_eq!(
            duplicate_program.matched_templates[0].priority,
            duplicate_program.matched_templates[1].priority
        );

        let mode = parse_stylesheet(
            "memory:mode.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:apply-templates select="root/item" mode="detail"/></xsl:template><xsl:template match="item" mode="detail"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&mode).expect("unprefixed modes should compile");
        assert_eq!(program.matched_templates[0].modes, ["detail"]);

        let current_mode = parse_stylesheet(
            "memory:current-mode.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xpath-default-namespace="http://example.test/"><xsl:template match="doc"><xsl:apply-templates select="item" mode="detail"/></xsl:template><xsl:template match="item" mode="detail"><xsl:call-template name="common"/></xsl:template><xsl:template name="common"><xsl:apply-templates select="/" mode="#current"/></xsl:template><xsl:template match="/" mode="detail"><out/></xsl:template></xsl:stylesheet>"##,
        );
        let program = compile_stylesheet(&current_mode)
            .expect("current mode and namespace-insensitive root path should compile");
        assert!(matches!(
            program.named_templates[0].template.body.as_slice(),
            [Instruction::ApplyTemplates {
                select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::LocationPath(path)),
                mode: Some(mode),
                ..
            }] if path.steps.is_empty() && mode == "#current"
        ));

        let default_mode = parse_stylesheet(
            "memory:default-mode.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xpath-default-namespace="http://example.test/"><xsl:template match="doc"><xsl:apply-templates select="item" mode="#default"/></xsl:template><xsl:template match="item" mode="a b #default"><xsl:call-template name="common"/></xsl:template><xsl:template name="common"><xsl:apply-templates select="//tail" mode="#current"/></xsl:template><xsl:template match="tail"><out/></xsl:template></xsl:stylesheet>"##,
        );
        let program = compile_stylesheet(&default_mode)
            .expect("default and current mode forms should compile");
        assert_eq!(program.matched_templates[1].modes, ["a", "b", "#default"]);
        assert!(matches!(
            program.named_templates[0].template.body.as_slice(),
            [Instruction::ApplyTemplates {
                select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::DescendantElement(name)),
                mode: Some(mode),
                ..
            }] if name.namespace.as_deref() == Some("http://example.test/")
                && name.local == "tail"
                && mode == "#current"
        ));
    }

    #[test]
    fn retains_bounded_exact_template_priority_and_classifies_other_lexicals() {
        let stylesheet = parse_stylesheet(
            "memory:priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc" priority="10"><out/></xsl:template><xsl:template match="node()" priority="1"><fallback/></xsl:template><xsl:template match="*"><wildcard/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&stylesheet).expect("integer priorities should compile");
        assert!(program.matched_templates[0].priority > program.matched_templates[1].priority);
        assert!(program.matched_templates[1].priority > program.matched_templates[2].priority);

        let fractional = parse_stylesheet(
            "memory:fractional-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item" priority=".5"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let fractional_program =
            compile_stylesheet(&fractional).expect("bounded fractional priority should compile");
        assert_eq!(
            fractional_program.matched_templates[0].priority,
            TemplatePriority::PATH_DEFAULT
        );

        let overprecision = parse_stylesheet(
            "memory:overprecision-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item" priority=".1234567"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&overprecision)
            .expect_err("priority beyond the fixed-point domain should remain unsupported");
        assert_eq!(failure.code, "FXST1025");
        assert_eq!(failure.category, CompileCategory::Unsupported);

        let invalid = parse_stylesheet(
            "memory:invalid-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item" priority="high"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&invalid).expect_err("invalid priority should fail");
        assert_eq!(failure.code, "FXST0030");
        assert_eq!(failure.category, CompileCategory::Invalid);

        let root = parse_stylesheet(
            "memory:root-priority.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" priority="1"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let root_program =
            compile_stylesheet(&root).expect("explicit root priority should use typed selection");
        assert!(root_program.root_template.is_none());
        assert_eq!(root_program.matched_templates.len(), 1);
        assert_eq!(
            root_program.matched_templates[0].priority,
            TemplatePriority::explicit_integer(1)
        );

        let default_mode_root = parse_stylesheet(
            "memory:default-mode-root.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><first/></xsl:template><xsl:template match="/" mode="#default"><second/></xsl:template></xsl:stylesheet>"##,
        );
        let default_mode_program = compile_stylesheet(&default_mode_root)
            .expect("#default root should compete through typed selection");
        assert!(default_mode_program.root_template.is_none());
        assert_eq!(default_mode_program.matched_templates.len(), 2);
    }

    #[test]
    fn compiles_bounded_attribute_presence_match_predicate() {
        let stylesheet = parse_stylesheet(
            "memory:attribute-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc/foo"><path/></xsl:template><xsl:template match="foo[@test]"><predicate/></xsl:template></xsl:stylesheet>"#,
        );
        let program =
            compile_stylesheet(&stylesheet).expect("attribute presence pattern should compile");
        assert!(matches!(
            &program.matched_templates[1].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::ElementWithAttribute {
                element,
                attribute
            } if element.local == "foo" && attribute.local == "test"
        ));
        assert_eq!(
            program.matched_templates[0].priority,
            program.matched_templates[1].priority
        );

        let comparison = parse_stylesheet(
            "memory:attribute-comparison-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo[@test='true']"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let comparison_program = compile_stylesheet(&comparison)
            .expect("exact single-quoted attribute value predicate should compile");
        assert!(matches!(
            &comparison_program.matched_templates[0].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::ElementWithAttributeValue {
                element,
                attribute,
                value
            } if element.local == "foo" && attribute.local == "test" && value == "true"
        ));

        let general_comparison = parse_stylesheet(
            "memory:general-attribute-comparison-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo[@test!='true']"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&general_comparison)
            .expect_err("general attribute comparisons must remain unsupported");
        assert_eq!(failure.code, "FXST1005");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }

    #[test]
    fn compiles_exact_descendant_wildcard_with_non_simple_priority() {
        let stylesheet = parse_stylesheet(
            "memory:descendant-wildcard-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo"><exact/></xsl:template><xsl:template match="//*"><descendant/></xsl:template></xsl:stylesheet>"#,
        );
        let program =
            compile_stylesheet(&stylesheet).expect("exact descendant wildcard should compile");
        assert!(matches!(
            program.matched_templates[1].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::DescendantAnyElement
        ));
        assert!(program.matched_templates[1].priority > program.matched_templates[0].priority);

        let named_descendant = parse_stylesheet(
            "memory:named-descendant-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="//foo"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&named_descendant)
            .expect_err("general descendant patterns must remain unsupported");
        assert_eq!(failure.code, "FXST1005");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }

    #[test]
    fn compiles_prefixed_element_and_explicit_namespace_wildcard_patterns() {
        let stylesheet = parse_stylesheet(
            "memory:namespace-wildcard-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:bar="http://bar.example/"><xsl:template match="bar:foo" priority="5"><exact/></xsl:template><xsl:template match="bar:*" priority="5"><wildcard/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&stylesheet)
            .expect("prefixed element and explicit namespace wildcard should compile");
        assert!(matches!(
            &program.matched_templates[0].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
                if name.namespace.as_deref() == Some("http://bar.example/") && name.local == "foo"
        ));
        assert!(matches!(
            &program.matched_templates[1].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::ElementNamespace(namespace)
                if namespace == "http://bar.example/"
        ));
        assert_eq!(
            program.matched_templates[0].priority,
            program.matched_templates[1].priority
        );

        let implicit = parse_stylesheet(
            "memory:implicit-namespace-wildcard.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:bar="http://bar.example/"><xsl:template match="bar:*"><namespace/></xsl:template><xsl:template match="*:foo"><local/></xsl:template></xsl:stylesheet>"#,
        );
        let implicit_program = compile_stylesheet(&implicit)
            .expect("namespace and local-name wildcards should retain exact quarter priority");
        assert!(matches!(
            &implicit_program.matched_templates[1].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::ElementLocal(local)
                if local == "foo"
        ));
        assert_eq!(
            implicit_program.matched_templates[0].priority,
            implicit_program.matched_templates[1].priority
        );

        let unbound = parse_stylesheet(
            "memory:unbound-match-prefix.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="bar:foo"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&unbound).expect_err("unbound prefix should be invalid");
        assert_eq!(failure.code, "FXST0031");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    #[test]
    fn compiles_xpath_default_namespace_for_simple_pattern_and_selection() {
        let stylesheet = parse_stylesheet(
            "memory:xpath-default-namespace.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc" xpath-default-namespace="http://example.test/"><out><xsl:apply-templates select="item"/></out></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&stylesheet)
            .expect("simple default-namespace pattern and selection should compile");
        assert!(matches!(
            &program.matched_templates[0].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
                if name.namespace.as_deref() == Some("http://example.test/") && name.local == "doc"
        ));
        assert!(matches!(
            program.matched_templates[0].template.body.as_slice(),
            [Instruction::LiteralElement { body, .. }]
                if matches!(body.as_slice(), [Instruction::ApplyTemplates {
                    select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::ChildElement(name)),
                    ..
                }] if name.namespace.as_deref() == Some("http://example.test/") && name.local == "item")
        ));

        let path_pattern = parse_stylesheet(
            "memory:xpath-default-namespace-pattern-path.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc/item" xpath-default-namespace="http://example.test/"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&path_pattern)
            .expect_err("multi-step default-namespace pattern must not lose expanded names");
        assert_eq!(failure.code, "FXST1027");
        assert_eq!(failure.category, CompileCategory::Unsupported);

        let selection_path = parse_stylesheet(
            "memory:xpath-default-namespace-selection-path.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc" xpath-default-namespace="http://example.test/"><xsl:apply-templates select="item/child"/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&selection_path)
            .expect_err("multi-step default-namespace selection must not lose expanded names");
        assert_eq!(failure.code, "FXST1027");
        assert_eq!(failure.category, CompileCategory::Unsupported);

        let literal_context = parse_stylesheet(
            "memory:literal-xpath-default-namespace.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc"><out xsl:xpath-default-namespace="http://example.test/"><xsl:apply-templates select="item"/></out></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&literal_context)
            .expect("literal result static-context attribute should compile");
        assert!(matches!(
            program.matched_templates[0].template.body.as_slice(),
            [Instruction::LiteralElement { body, .. }]
                if matches!(body.as_slice(), [Instruction::ApplyTemplates {
                    select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::ChildElement(name)),
                    ..
                }] if name.namespace.as_deref() == Some("http://example.test/") && name.local == "item")
        ));

        let stylesheet_context = parse_stylesheet(
            "memory:stylesheet-xpath-default-namespace.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xpath-default-namespace="http://example.test/"><xsl:template match="doc"><xsl:apply-templates select="item"/></xsl:template><xsl:template match="@code"><xsl:value-of select="."/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&stylesheet_context)
            .expect("stylesheet-wide default element namespace should compile");
        assert!(matches!(
            &program.matched_templates[0].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::Element(name)
                if name.namespace.as_deref() == Some("http://example.test/") && name.local == "doc"
        ));
        assert!(matches!(
            program.matched_templates[0].template.body.as_slice(),
            [Instruction::ApplyTemplates {
                select: Some(crate::xslt::golden_semantics_experiment::ApplySelection::ChildElement(name)),
                ..
            }] if name.namespace.as_deref() == Some("http://example.test/") && name.local == "item"
        ));
        assert!(matches!(
            &program.matched_templates[1].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::Attribute(name)
                if name.namespace.is_none() && name.local == "code"
        ));
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
    fn classifies_xpath_outside_the_private_location_path_slice_as_unsupported() {
        let stylesheet = parse_stylesheet(
            "memory:path.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><value><xsl:value-of select="greeting///name"/></value></xsl:template></xsl:stylesheet>"#,
        );

        let failure = compile_stylesheet(&stylesheet).expect_err("unsupported XPath should fail");

        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXXP1001");
        assert_eq!(failure.location.resource, "memory:path.xsl");
    }

    #[test]
    fn xsl_text_preserves_explicit_whitespace_and_rejects_element_content() {
        let stylesheet = parse_stylesheet(
            "memory:text.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text>  kept  </xsl:text></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&stylesheet).expect("xsl:text should compile");
        let root_template = program.root_template.expect("root template");
        let [Instruction::Text { value, .. }] = root_template.body.as_slice() else {
            panic!("xsl:text should lower to one owned text instruction");
        };
        assert_eq!(value, "  kept  ");

        let invalid_text = parse_stylesheet(
            "memory:invalid-text.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:text><bad/></xsl:text></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&invalid_text).expect_err("element content must fail");
        assert_eq!(failure.code, "FXST0026");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    #[test]
    fn separates_invalid_deep_equal_arity_from_unsupported_collation_semantics() {
        let invalid = parse_stylesheet(
            "memory:deep-equal-arity.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="deep-equal()"/></xsl:template></xsl:stylesheet>"#,
        );
        let failure =
            compile_stylesheet(&invalid).expect_err("invalid deep-equal arity should fail");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "FXXP0005");
        assert_eq!(failure.location.resource, "memory:deep-equal-arity.xsl");
        assert!(!failure.location.span.is_empty());

        let unsupported = parse_stylesheet(
            "memory:deep-equal-collation.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="deep-equal(1, 1, ())"/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&unsupported)
            .expect_err("unimplemented deep-equal collation semantics should fail");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXXP1010");
        assert_eq!(failure.location.resource, "memory:deep-equal-collation.xsl");
        assert!(!failure.location.span.is_empty());

        let composed = parse_stylesheet(
            "memory:deep-equal-composed.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="not(deep-equal((1, 2), (2, 1)))"/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&composed)
            .expect("composed deep-equal expression should use the shared owner");
        assert!(matches!(
            program
                .root_template
                .expect("root template")
                .body
                .as_slice(),
            [Instruction::ValueOf {
                select: ValueExpression::DeepEqual(_),
                ..
            }]
        ));
    }
}
