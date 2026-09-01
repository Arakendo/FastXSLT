use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xpath::path_experiment::{PathFailure, parse_location_path};
use crate::xslt::golden_semantics_experiment::{
    ConstructedElement, ConstructedNode, GlobalBinding, GlobalBindingDefault, GlobalBindingKind,
    MatchPattern, MatchedTemplate, NamedTemplate, STANDARD_INITIAL_TEMPLATE_NAME,
    SourceWhitespacePolicy, StylesheetProgram, Template, TemplateParameter,
    TemplateParameterDefault, TemplatePriority,
};

#[path = "instruction_compiler.rs"]
mod instruction_compiler;
#[path = "mode_declaration_compiler.rs"]
mod mode_declaration_compiler;
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
    compile_stylesheet_with_single_include_program_at,
    compile_stylesheet_with_two_imported_programs_at,
    compile_stylesheet_with_two_included_programs_at, discovered_stylesheet_dependencies_at,
};
use stylesheet_validation::validate_named_template_references;
use template_pattern_compiler::compile_match_pattern;

use instruction_compiler::{
    compile_sequence_excluding, literal_result_namespaces, parse_template_modes,
};
use mode_declaration_compiler::{
    validate_mode_declaration as validate_mode, validate_same_precedence_mode_declaration_conflicts,
};
pub(super) use output_compiler::default_output_settings;
use output_compiler::{compile_output, merge_output};

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

#[allow(
    clippy::too_many_lines,
    reason = "single-document top-level composition remains one cohesive compiler responsibility"
)]
pub(super) fn compile_stylesheet_at_excluding_unvalidated(
    document: &Document,
    root: NodeId,
    excluded_top_level: &[NodeId],
) -> Result<StylesheetProgram, CompileFailure> {
    require_stylesheet_root(document, root)?;
    let declared_version = required_attribute(document, root, None, "version")?.to_owned();
    let default_initial_mode = optional_attribute(document, root, None, "default-mode")
        .map(str::trim)
        .map(|mode| match mode {
            "#unnamed" => Ok(None),
            mode => instruction_compiler::parse_mode(document, root, mode).map(Some),
        })
        .transpose()?
        .flatten();
    let top_level_children = meaningful_children(document, root)
        .into_iter()
        .filter(|child| !excluded_top_level.contains(child))
        .collect::<Vec<_>>();
    validate_same_precedence_mode_declaration_conflicts(document, &top_level_children)?;

    let mut output = None;
    let mut source_whitespace = SourceWhitespacePolicy::Preserve;
    let mut modes = CompiledModes::default();
    let mut root_template = None;
    let mut root_template_modes = Vec::new();
    let mut matched_templates = Vec::new();
    let mut named_templates = Vec::new();
    let mut global_bindings = Vec::new();
    let mut global_binding_locations = Vec::new();
    let mut character_maps = Vec::new();
    for child in top_level_children {
        let Some(name) = document.name(child) else {
            continue;
        };
        match (name.namespace.as_deref(), name.local.as_str()) {
            (Some(XSLT_NAMESPACE), "output") => {
                let declaration = compile_output(document, child, &declared_version)?;
                output = Some(match output {
                    Some(existing) => merge_output(existing, declaration)?,
                    None => declaration,
                });
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
            (Some(XSLT_NAMESPACE), "mode") => {
                modes.push(validate_mode(document, child, &declared_version)?);
            }
            (Some(XSLT_NAMESPACE), "character-map") => {
                character_maps.push(compile_character_map(document, child)?);
            }
            (Some(XSLT_NAMESPACE), "strip-space") => {
                ensure_only_attributes(document, child, &["elements"], "xsl:strip-space")?;
                ensure_no_meaningful_children(document, child, "xsl:strip-space")?;
                let elements = required_attribute(document, child, None, "elements")?;
                if elements != "*" {
                    return Err(unsupported(
                        "FXST1043",
                        "the private whitespace-policy reference supports only xsl:strip-space elements='*'",
                        document.location(child),
                    ));
                }
                source_whitespace = SourceWhitespacePolicy::StripAllElementWhitespace;
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
                global_binding_locations.push(document.location(child).clone());
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
    reject_unordered_global_dependencies(&global_bindings, &global_binding_locations)?;
    if let Some(declaration) = output.as_mut()
        && let Some(name) = declaration.character_map_name.as_deref()
    {
        let map = character_maps
            .iter()
            .find(|map| map.name == name)
            .ok_or_else(|| invalid("XTSE1590", "unknown character map", &declaration.location))?;
        declaration.settings.character_map = resolved_character_map(map, &character_maps)?;
    }

    Ok(StylesheetProgram {
        declared_version,
        default_initial_mode,
        source_whitespace,
        typed_mode_requirements: modes.typed,
        mode_on_no_match: modes.on_no_match,
        output: output.map_or_else(default_output_settings, |declaration| declaration.settings),
        root_template,
        root_template_modes,
        matched_templates,
        named_templates,
        global_bindings,
    })
}

struct CompiledCharacterMap {
    name: String,
    referenced_map_name: Option<String>,
    entries: Vec<(char, String)>,
    location: SourceLocation,
}

fn compile_character_map(
    document: &Document,
    element: NodeId,
) -> Result<CompiledCharacterMap, CompileFailure> {
    let Some(name) = optional_attribute(document, element, None, "name") else {
        return Err(invalid(
            "XTSE0010",
            "xsl:character-map requires a name attribute",
            document.location(element),
        ));
    };
    if !is_ascii_ncname(name) {
        return Err(unsupported(
            "FXST1047",
            "the first character-map slice requires an unprefixed NCName",
            document.location(element),
        ));
    }
    ensure_only_attributes(
        document,
        element,
        &["name", "use-character-maps"],
        "xsl:character-map",
    )?;
    let referenced_map_name = optional_attribute(document, element, None, "use-character-maps")
        .map(|value| {
            if value.split_whitespace().count() == 1 && is_ascii_ncname(value) {
                Ok(value.to_owned())
            } else {
                Err(unsupported(
                    "FXST1047",
                    "the first character-map composition slice requires one unprefixed name",
                    document.location(element),
                ))
            }
        })
        .transpose()?;
    let children = meaningful_children(document, element);
    let mut entries = Vec::new();
    for child in children {
        if !is_xslt_element(document, child, "output-character") {
            return Err(unsupported(
                "FXST1047",
                "character-map children must be xsl:output-character",
                document.location(child),
            ));
        }
        ensure_only_attributes(
            document,
            child,
            &["character", "string"],
            "xsl:output-character",
        )?;
        ensure_no_meaningful_children(document, child, "xsl:output-character")?;
        let lexical = required_attribute(document, child, None, "character")?;
        let mut characters = lexical.chars();
        let character = characters
            .next()
            .filter(|_| characters.next().is_none())
            .ok_or_else(|| {
                invalid(
                    "XTSE0020",
                    "output-character requires exactly one character",
                    document.location(child),
                )
            })?;
        let replacement = required_attribute(document, child, None, "string")?.to_owned();
        entries.push((character, replacement));
    }
    Ok(CompiledCharacterMap {
        name: name.to_owned(),
        referenced_map_name,
        entries,
        location: document.location(element).clone(),
    })
}

fn resolved_character_map(
    map: &CompiledCharacterMap,
    maps: &[CompiledCharacterMap],
) -> Result<Vec<(char, String)>, CompileFailure> {
    let mut resolved = if let Some(reference) = map.referenced_map_name.as_deref() {
        let referenced = maps
            .iter()
            .find(|candidate| candidate.name == reference)
            .ok_or_else(|| invalid("XTSE1590", "unknown character map", &map.location))?;
        if referenced.referenced_map_name.is_some() {
            return Err(unsupported(
                "FXST1047",
                "character-map composition chains remain outside the first composition slice",
                &map.location,
            ));
        }
        referenced.entries.clone()
    } else {
        Vec::new()
    };
    for (character, replacement) in &map.entries {
        if let Some((_, inherited)) = resolved
            .iter_mut()
            .find(|(candidate, _)| candidate == character)
        {
            inherited.clone_from(replacement);
        } else {
            resolved.push((*character, replacement.clone()));
        }
    }
    Ok(resolved)
}

fn reject_unordered_global_dependencies(
    bindings: &[GlobalBinding],
    locations: &[SourceLocation],
) -> Result<(), CompileFailure> {
    for (index, binding) in bindings.iter().enumerate() {
        let GlobalBindingDefault::Variable(dependency) = &binding.default else {
            continue;
        };
        if bindings[..index]
            .iter()
            .any(|candidate| candidate.name == *dependency)
        {
            continue;
        }
        if bindings[index..]
            .iter()
            .any(|candidate| candidate.name == *dependency)
        {
            return Err(unsupported(
                "FXST1044",
                format!(
                    "global dependency ordering for ${} -> ${dependency} is outside the private slice",
                    binding.name
                ),
                &locations[index],
            ));
        }
    }
    Ok(())
}

#[derive(Default)]
struct CompiledModes {
    typed: Vec<crate::xslt::golden_semantics_experiment::TypedModeRequirement>,
    on_no_match: Vec<crate::xslt::golden_semantics_experiment::ModeOnNoMatch>,
}

impl CompiledModes {
    fn push(&mut self, declaration: mode_declaration_compiler::CompiledModeDeclaration) {
        self.typed.extend(declaration.typed_requirement);
        self.on_no_match.extend(declaration.on_no_match);
    }
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
        if optional_attribute(document, element, None, "match").is_none() {
            return Ok(());
        }
    }

    let pattern = required_attribute(document, element, None, "match")?;
    if pattern == "/" {
        let has_priority = optional_attribute(document, element, None, "priority").is_some();
        let has_mode = optional_attribute(document, element, None, "mode").is_some()
            || effective_default_mode(document, element).is_some_and(|mode| mode != "#unnamed");
        let has_competing_default = matched_templates.iter().any(|existing| {
            existing.pattern == MatchPattern::Document && existing.modes.is_empty()
        });
        if has_priority || has_mode || root_template.is_some() || has_competing_default {
            let matched_template = compile_matched_template(document, element, pattern)?;
            let competes_with_default = matched_template.modes.is_empty()
                || matched_template
                    .modes
                    .iter()
                    .any(|mode| matches!(mode.as_str(), "#default" | "#unnamed" | "#all"));
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

    matched_templates.extend(compile_matched_templates(document, element, pattern)?);
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
        elements.push(compile_constructed_element(document, child)?);
    }
    Ok(elements)
}

fn compile_constructed_element(
    document: &Document,
    element: NodeId,
) -> Result<ConstructedElement, CompileFailure> {
    let name = document.name(element).expect("element nodes have names");
    if name.namespace.as_deref() == Some(XSLT_NAMESPACE) || !document.attributes(element).is_empty()
    {
        return Err(unsupported(
            "FXST1015",
            "only attribute-free literal elements are admitted in global temporary trees",
            document.location(element),
        ));
    }
    Ok(ConstructedElement {
        name: name.clone(),
        namespaces: literal_result_namespaces(document, element),
        children: compile_constructed_children(document, element)?,
    })
}

fn compile_constructed_children(
    document: &Document,
    parent: NodeId,
) -> Result<Vec<ConstructedNode>, CompileFailure> {
    meaningful_children(document, parent)
        .into_iter()
        .map(|child| match document.kind(child) {
            NodeKind::Text => Ok(ConstructedNode::Text(
                document.value(child).unwrap_or_default().to_owned(),
            )),
            NodeKind::Element => {
                compile_constructed_element(document, child).map(ConstructedNode::Element)
            }
            NodeKind::Comment | NodeKind::ProcessingInstruction => {
                unreachable!("meaningful_children excludes comments and processing instructions")
            }
            NodeKind::Document | NodeKind::Attribute => Err(invalid(
                "FXST0006",
                "unexpected node kind in a temporary-tree constructor",
                document.location(child),
            )),
        })
        .collect()
}

fn compile_matched_template(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> Result<MatchedTemplate, CompileFailure> {
    let (pattern, priority) = compile_match_pattern(document, element, pattern)?;
    Ok(MatchedTemplate {
        pattern,
        import_precedence: 0,
        priority,
        modes: compile_template_modes_for_rule(document, element)?,
        template: compile_template(document, element)?,
    })
}

fn compile_matched_templates(
    document: &Document,
    element: NodeId,
    lexical_pattern: &str,
) -> Result<Vec<MatchedTemplate>, CompileFailure> {
    let alternatives = lexical_pattern
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    if alternatives.len() == 1 {
        return compile_matched_template(document, element, lexical_pattern).map(|rule| vec![rule]);
    }
    if template_pattern_compiler::is_homogeneous_qualified_path_union(lexical_pattern) {
        return compile_matched_template(document, element, lexical_pattern).map(|rule| vec![rule]);
    }
    let patterns = alternatives
        .into_iter()
        .map(|alternative| compile_match_pattern(document, element, alternative))
        .collect::<Result<Vec<_>, _>>()?;
    if !template_pattern_compiler::alternatives_are_pairwise_disjoint(&patterns) {
        return Err(unsupported(
            "FXST1005",
            "union match patterns whose alternatives can overlap are outside the private slice",
            document.location(element),
        ));
    }
    let modes = compile_template_modes_for_rule(document, element)?;
    let template = compile_template(document, element)?;
    Ok(patterns
        .into_iter()
        .map(|(pattern, priority)| MatchedTemplate {
            pattern,
            import_precedence: 0,
            priority,
            modes: modes.clone(),
            template: template.clone(),
        })
        .collect())
}

fn compile_template_modes_for_rule(
    document: &Document,
    element: NodeId,
) -> Result<Vec<String>, CompileFailure> {
    let modes = match optional_attribute(document, element, None, "mode") {
        Some(mode) => Some(parse_template_modes(document, element, mode)?),
        None => match effective_default_mode(document, element) {
            Some("#unnamed") | None => None,
            Some(mode) => Some(parse_template_modes(document, element, mode)?),
        },
    };
    Ok(modes.unwrap_or_default())
}

fn compile_template(document: &Document, element: NodeId) -> Result<Template, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &[
            "name",
            "match",
            "mode",
            "priority",
            "xpath-default-namespace",
            "default-mode",
            "exclude-result-prefixes",
        ],
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
    ensure_only_attributes(
        document,
        element,
        &[
            "name",
            "match",
            "mode",
            "priority",
            "xpath-default-namespace",
            "default-mode",
        ],
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

pub(super) fn effective_default_mode(document: &Document, element: NodeId) -> Option<&str> {
    let mut current = Some(element);
    while let Some(node) = current {
        let is_xslt = document
            .name(node)
            .is_some_and(|name| name.namespace.as_deref() == Some(XSLT_NAMESPACE));
        let lexical = if is_xslt {
            optional_attribute(document, node, None, "default-mode")
        } else {
            optional_attribute(document, node, Some(XSLT_NAMESPACE), "default-mode")
        };
        if lexical.is_some() {
            return lexical.map(str::trim);
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
        Instruction, MatchPattern, STANDARD_INITIAL_TEMPLATE_NAME, TemplatePriority,
        ValueExpression,
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
    fn forward_and_cyclic_global_dependencies_are_explicitly_unsupported() {
        for (label, declarations) in [
            (
                "forward",
                r#"<xsl:variable name="first" select="$later"/><xsl:variable name="later" select="7"/>"#,
            ),
            (
                "cycle",
                r#"<xsl:variable name="first" select="$later"/><xsl:variable name="later" select="$first"/>"#,
            ),
        ] {
            let stylesheet = format!(
                r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{declarations}<xsl:template match="/"/></xsl:stylesheet>"#
            );
            let document = parse_stylesheet(
                &format!("memory:{label}-global-dependency.xsl"),
                stylesheet.as_bytes(),
            );

            let failure = compile_stylesheet(&document)
                .expect_err("unordered global dependency should remain explicit");

            assert_eq!(failure.code, "FXST1044");
            assert_eq!(failure.category, CompileCategory::Unsupported);
            assert!(failure.detail.contains("$first -> $later"));
        }
    }

    #[test]
    fn backward_global_dependencies_remain_in_the_admitted_slice() {
        let document = parse_stylesheet(
            "memory:backward-global-dependency.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="earlier" select="7"/><xsl:variable name="later" select="$earlier"/><xsl:template match="/"/></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&document).expect("backward dependency should compile");

        assert_eq!(program.global_bindings.len(), 2);
    }

    #[test]
    fn preserves_absent_output_declaration_for_runtime_method_inference() {
        let stylesheet = parse_stylesheet(
            "memory:default-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");

        assert_eq!(program.output.method, None);
        assert_eq!(program.output.version, None);
        assert_eq!(program.output.encoding, None);
        assert_eq!(program.output.media_type, None);
        assert_eq!(program.output.doctype_system, None);
        assert_eq!(program.output.doctype_public, None);
        assert_eq!(program.output.include_content_type, None);
        assert_eq!(program.output.byte_order_mark, None);
        assert_eq!(program.output.normalization_form, None);
        assert_eq!(program.output.standalone, None);
        assert!(program.output.cdata_section_elements.is_empty());
        assert!(!program.output.omit_xml_declaration);
    }

    #[test]
    fn retains_requested_normalization_for_serializer_capability_selection() {
        let none = parse_stylesheet(
            "memory:no-normalization.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" normalization-form="none"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
        let nfc = parse_stylesheet(
            "memory:nfc-normalization.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" normalization-form="NFC"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&none).expect("none should preserve result characters");
        assert_eq!(program.output.normalization_form.as_deref(), Some("none"));
        let program = compile_stylesheet(&nfc)
            .expect("the compiler should retain rather than implement normalization");
        assert_eq!(program.output.normalization_form.as_deref(), Some("NFC"));
    }

    #[test]
    fn retains_xml_10_serialization_version_and_rejects_unadmitted_versions() {
        let xml_10 = parse_stylesheet(
            "memory:xml-10-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" version="1.0"/><xsl:template match="/"><html/></xsl:template></xsl:stylesheet>"#,
        );
        let xml_11 = parse_stylesheet(
            "memory:xml-11-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" version="1.1"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&xml_10).expect("XML 1.0 serialization should compile");
        assert_eq!(program.output.version.as_deref(), Some("1.0"));
        let failure = compile_stylesheet(&xml_11).expect_err("XML 1.1 remains unadmitted");
        assert_eq!(failure.code, "FXST1021");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }

    #[test]
    fn retains_doctype_identifiers_as_owned_serialization_metadata() {
        let stylesheet = parse_stylesheet(
            "memory:doctype-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" doctype-system="out.dtd" doctype-public="-//EXAMPLE//DTD Test//EN"/><xsl:template match="/"><html xmlns="http://www.w3.org/1999/xhtml"/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&stylesheet).expect("DOCTYPE metadata should compile");

        assert_eq!(program.output.doctype_system.as_deref(), Some("out.dtd"));
        assert_eq!(
            program.output.doctype_public.as_deref(),
            Some("-//EXAMPLE//DTD Test//EN")
        );
    }

    #[test]
    fn output_ignores_only_the_admitted_xml_space_control_attribute() {
        let stylesheet = parse_stylesheet(
            "memory:foreign-output-attribute.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:e="urn:example"><xsl:output e:unknown="value"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let failure = compile_stylesheet(&stylesheet)
            .expect_err("an arbitrary foreign output attribute remains unsupported");
        assert_eq!(failure.code, "FXST1009");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }

    #[test]
    fn rejects_overlapping_output_properties_during_bounded_merge() {
        let stylesheet = parse_stylesheet(
            "memory:overlapping-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml"/><xsl:output method="xhtml"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&stylesheet)
            .expect_err("repeated scalar properties remain outside bounded merging");
        assert_eq!(failure.code, "FXST1018");
        assert_eq!(failure.category, CompileCategory::Unsupported);
    }

    #[test]
    fn retains_requested_encoding_for_serializer_capability_selection() {
        let iso_8859_1 = parse_stylesheet(
            "memory:iso-8859-1-output.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" encoding="ISO-8859-1"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&iso_8859_1)
            .expect("the bounded byte lane should retain ISO-8859-1 metadata");
        assert_eq!(program.output.encoding.as_deref(), Some("ISO-8859-1"));

        let utf_16 = parse_stylesheet(
            "memory:unsupported-encoding.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" encoding="UTF-16"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&utf_16)
            .expect("the compiler should retain rather than implement the requested encoding");
        assert_eq!(program.output.encoding.as_deref(), Some("UTF-16"));
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
        assert_eq!(failure.code, "XTSE0020");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    #[test]
    fn validates_escape_uri_attributes_only_for_the_inert_xml_slice() {
        let xml = parse_stylesheet(
            "memory:xml-escape-uri.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" escape-uri-attributes="yes"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
        compile_stylesheet(&xml).expect("the explicit XML property should be inert");

        let xhtml = parse_stylesheet(
            "memory:xhtml-escape-uri.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xhtml" escape-uri-attributes="yes"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&xhtml).expect_err("XHTML URI escaping remains open");
        assert_eq!(failure.code, "FXST1036");
        assert_eq!(failure.category, CompileCategory::Unsupported);

        let invalid = parse_stylesheet(
            "memory:invalid-escape-uri.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" escape-uri-attributes="true"/><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&invalid).expect_err("XSLT 2.0 requires yes or no");
        assert_eq!(failure.code, "XTSE0020");
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
    fn compiles_inherited_default_mode_without_overriding_explicit_mode() {
        let stylesheet = parse_stylesheet(
            "memory:inherited-default-mode.xsl",
            br##"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" default-mode="a"><out xsl:default-mode="#unnamed"><xsl:apply-templates select="doc/a"/><xsl:apply-templates select="doc/a" mode="b"/></out></xsl:template><xsl:template match="a" mode="a b"/></xsl:stylesheet>"##,
        );
        let program = compile_stylesheet(&stylesheet).expect("default mode should compile");
        assert!(program.root_template.is_none());
        let document_rule = program
            .matched_templates
            .iter()
            .find(|template| template.pattern == MatchPattern::Document)
            .expect("default-mode applies to the template rule");
        assert_eq!(document_rule.modes, ["a"]);
        assert!(matches!(
            document_rule.template.body.as_slice(),
            [Instruction::LiteralElement { body, .. }]
                if matches!(body.as_slice(), [
                    Instruction::ApplyTemplates { mode: None, .. },
                    Instruction::ApplyTemplates { mode: Some(mode), .. }
                ] if mode == "b")
        ));

        let stylesheet = parse_stylesheet(
            "memory:default-initial-mode.xsl",
            br##"<xsl:stylesheet version="3.0" default-mode="a" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/" mode="#unnamed a"><out/></xsl:template></xsl:stylesheet>"##,
        );
        let program = compile_stylesheet(&stylesheet).expect("default initial mode should compile");
        assert_eq!(program.default_initial_mode.as_deref(), Some("a"));
        assert_eq!(program.matched_templates[0].modes, ["#unnamed", "a"]);
    }

    #[test]
    fn compiles_only_provably_disjoint_union_rules_with_individual_priorities() {
        let stylesheet = parse_stylesheet(
            "memory:disjoint-union.xsl",
            br##"<xsl:stylesheet version="3.0" default-mode=" a " xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="v | chapter/text()" mode="#unnamed"><xsl:apply-templates mode="#unnamed"/></xsl:template></xsl:stylesheet>"##,
        );
        let program = compile_stylesheet(&stylesheet).expect("disjoint union should compile");
        assert_eq!(program.default_initial_mode.as_deref(), Some("a"));
        assert_eq!(program.matched_templates.len(), 2);
        assert_eq!(
            program.matched_templates[0].priority,
            TemplatePriority::EXACT_NAME_DEFAULT
        );
        assert_eq!(
            program.matched_templates[1].priority,
            TemplatePriority::PATH_DEFAULT
        );
        assert!(
            program
                .matched_templates
                .iter()
                .all(|rule| rule.modes == ["#unnamed"])
        );
        assert!(program.matched_templates.iter().all(|rule| matches!(
            rule.template.body.as_slice(),
            [Instruction::ApplyTemplates { mode: None, .. }]
        )));

        let overlapping = parse_stylesheet(
            "memory:overlapping-union.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="text() | chapter/text()"/></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&overlapping)
            .expect_err("potentially overlapping alternatives must remain unsupported");
        assert_eq!(failure.code, "FXST1005");
        assert_eq!(failure.category, CompileCategory::Unsupported);
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

        let document_rooted = parse_stylesheet(
            "memory:document-rooted-descendant-wildcard-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="foo"><exact/></xsl:template><xsl:template match="/root//*"><descendant/></xsl:template></xsl:stylesheet>"#,
        );
        let document_rooted_program = compile_stylesheet(&document_rooted)
            .expect("document-rooted descendant wildcard should compile");
        assert!(matches!(
            document_rooted_program.matched_templates[1].pattern,
            crate::xslt::golden_semantics_experiment::MatchPattern::Path(_)
        ));
        assert!(
            document_rooted_program.matched_templates[1].priority
                > document_rooted_program.matched_templates[0].priority
        );

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
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:message>unsupported</xsl:message></xsl:template></xsl:stylesheet>"#,
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

        let named_and_matched = parse_stylesheet(
            "memory:named-and-matched-template.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="scan" match="*" mode="a" priority="2"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&named_and_matched)
            .expect("one template may be both named and matched");
        assert_eq!(program.named_templates.len(), 1);
        assert_eq!(program.named_templates[0].name, "scan");
        assert_eq!(program.matched_templates.len(), 1);
        assert_eq!(program.matched_templates[0].modes, ["a"]);
        assert_eq!(
            program.matched_templates[0].priority,
            TemplatePriority::explicit_integer(2)
        );

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
    fn compiles_only_the_exact_strip_all_whitespace_reference_policy() {
        let stylesheet = parse_stylesheet(
            "memory:strip-all.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:strip-space elements="*"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let program =
            compile_stylesheet(&stylesheet).expect("exact strip-all policy should compile");
        assert_eq!(
            program.source_whitespace,
            crate::xslt::golden_semantics_experiment::SourceWhitespacePolicy::StripAllElementWhitespace
        );

        let unsupported = parse_stylesheet(
            "memory:selective-strip.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:strip-space elements="item"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&unsupported)
            .expect_err("selective whitespace rules remain outside the reference slice");
        assert_eq!(failure.code, "FXST1043");
        assert_eq!(failure.category, CompileCategory::Unsupported);
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
    fn processing_instruction_compiles_static_target_and_literal_data() {
        let stylesheet = parse_stylesheet(
            "memory:processing-instruction.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:processing-instruction name="my-pi">href="book.css" type="text/css"</xsl:processing-instruction></xsl:template></xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&stylesheet).expect("static PI should compile");
        let root_template = program.root_template.expect("root template");
        let [Instruction::ProcessingInstructionNode { target, value, .. }] =
            root_template.body.as_slice()
        else {
            panic!("xsl:processing-instruction should lower to one PI instruction");
        };
        assert_eq!(target, "my-pi");
        assert_eq!(value, "href=\"book.css\" type=\"text/css\"");

        let invalid = parse_stylesheet(
            "memory:invalid-processing-instruction.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:processing-instruction name="xml">data</xsl:processing-instruction></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&invalid).expect_err("reserved target should fail");
        assert_eq!(failure.code, "FXST0036");
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
