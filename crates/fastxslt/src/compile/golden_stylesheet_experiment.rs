use std::collections::BTreeMap;

use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::path_experiment::{PathFailure, parse_location_path};
use crate::xslt::golden_semantics_experiment::{
    CharacterMapDefinition, ConstructedAttribute, ConstructedElement, ConstructedNode,
    GlobalBinding, GlobalBindingDefault, GlobalBindingKind, Instruction, LiteralAttributeValue,
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
    compile_stylesheet_with_imports, compile_stylesheet_with_single_imported_program_at,
    compile_stylesheet_with_single_include, compile_stylesheet_with_single_include_program_at,
    compile_stylesheet_with_two_imported_programs_at,
    compile_stylesheet_with_two_included_programs_at, discovered_stylesheet_dependencies_at,
};
use stylesheet_validation::validate_named_template_references;
use template_pattern_compiler::compile_match_pattern;

use instruction_compiler::{
    compile_comment, compile_literal_result_attributes, compile_processing_instruction,
    compile_sequence_excluding, compile_text, literal_result_namespaces, parse_template_modes,
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
    let mut program = compile_stylesheet_at_excluding_unvalidated(document, root, &[])?;
    finalize_character_maps(&mut program)?;
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
    let mut named_output_names = Vec::new();
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
                if let Some(name) = &declaration.name {
                    if named_output_names.contains(name) {
                        return Err(unsupported(
                            "FXST1055",
                            "merging duplicate named output declarations is outside the private slice",
                            document.location(child),
                        ));
                    }
                    if !declaration.character_map_names.is_empty() {
                        return Err(unsupported(
                            "FXST1056",
                            "character maps on unused named output declarations are outside the private slice",
                            document.location(child),
                        ));
                    }
                    named_output_names.push(name.clone());
                } else {
                    output = Some(match output {
                        Some(existing) => merge_output(existing, declaration)?,
                        None => declaration,
                    });
                }
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
    let output_character_map_names = output
        .as_ref()
        .map(|declaration| declaration.character_map_names.clone())
        .unwrap_or_default();
    let output_character_map_location = output.as_ref().and_then(|declaration| {
        (!declaration.character_map_names.is_empty()).then(|| declaration.location.clone())
    });
    let output_specified_properties = output
        .as_ref()
        .map(|declaration| declaration.specified.iter().cloned().collect())
        .unwrap_or_default();

    Ok(StylesheetProgram {
        declared_version,
        default_initial_mode,
        source_whitespace,
        typed_mode_requirements: modes.typed,
        private_initial_modes: modes.private_initial,
        mode_policies: modes.policies,
        output: output.map_or_else(default_output_settings, |declaration| declaration.settings),
        output_specified_properties,
        character_maps,
        output_character_map_names,
        output_character_map_location,
        root_template,
        root_template_modes,
        matched_templates,
        named_templates,
        global_bindings,
    })
}

fn compile_character_map(
    document: &Document,
    element: NodeId,
) -> Result<CharacterMapDefinition, CompileFailure> {
    let Some(name) = optional_attribute(document, element, None, "name") else {
        return Err(invalid(
            "XTSE0010",
            "xsl:character-map requires a name attribute",
            document.location(element),
        ));
    };
    let name = compile_expanded_qname(document, element, name, "xsl:character-map name")?;
    ensure_only_attributes(
        document,
        element,
        &["name", "use-character-maps"],
        "xsl:character-map",
    )?;
    let referenced_map_names = optional_attribute(document, element, None, "use-character-maps")
        .map(|value| {
            let names: Vec<_> = value.split_whitespace().collect();
            if names.is_empty() {
                return Err(invalid(
                    "XTSE0020",
                    "xsl:character-map use-character-maps must not be empty",
                    document.location(element),
                ));
            }
            names
                .into_iter()
                .map(|name| {
                    compile_expanded_qname(
                        document,
                        element,
                        name,
                        "xsl:character-map use-character-maps",
                    )
                })
                .collect()
        })
        .transpose()?
        .unwrap_or_default();
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
    Ok(CharacterMapDefinition {
        name,
        referenced_map_names,
        entries,
        location: document.location(element).clone(),
    })
}

pub(super) fn compile_expanded_qname(
    document: &Document,
    element: NodeId,
    lexical: &str,
    role: &str,
) -> Result<ExpandedName, CompileFailure> {
    let (prefix, local) = lexical
        .split_once(':')
        .map_or((None, lexical), |(prefix, local)| (Some(prefix), local));
    if !is_ascii_ncname(local)
        || prefix.is_some_and(|prefix| !is_ascii_ncname(prefix))
        || local.contains(':')
    {
        return Err(invalid(
            "XTSE0020",
            format!("invalid QName for {role}: {lexical}"),
            document.location(element),
        ));
    }
    let namespace = if let Some(prefix) = prefix {
        namespace_for_prefix(document, element, prefix)
            .ok_or_else(|| {
                invalid(
                    "XTSE0280",
                    format!("unbound prefix in {role}: {lexical}"),
                    document.location(element),
                )
            })?
            .to_owned()
    } else {
        String::new()
    };
    Ok(ExpandedName {
        namespace: (!namespace.is_empty()).then_some(namespace),
        local: local.to_owned(),
    })
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

fn resolved_character_map(
    map: &CharacterMapDefinition,
    maps: &[CharacterMapDefinition],
) -> Result<Vec<(char, String)>, CompileFailure> {
    let mut resolved = BTreeMap::new();
    for reference in &map.referenced_map_names {
        let referenced = maps
            .iter()
            .find(|candidate| &candidate.name == reference)
            .ok_or_else(|| invalid("XTSE1590", "unknown character map", &map.location))?;
        if !referenced.referenced_map_names.is_empty() {
            return Err(unsupported(
                "FXST1047",
                "character-map composition chains remain outside the first composition slice",
                &map.location,
            ));
        }
        merge_character_map_entries(&mut resolved, &referenced.entries);
    }
    merge_character_map_entries(&mut resolved, &map.entries);
    Ok(resolved.into_iter().collect())
}

pub(super) fn finalize_character_maps(
    program: &mut StylesheetProgram,
) -> Result<(), CompileFailure> {
    let mut output_character_map = BTreeMap::new();
    for name in &program.output_character_map_names {
        let map = program
            .character_maps
            .iter()
            .find(|map| map.name == *name)
            .ok_or_else(|| {
                invalid(
                    "XTSE1590",
                    "unknown character map",
                    program
                        .output_character_map_location
                        .as_ref()
                        .expect("character-map output reference retains its location"),
                )
            })?;
        let resolved = resolved_character_map(map, &program.character_maps)?;
        merge_character_map_entries(&mut output_character_map, &resolved);
    }
    program.output.character_map = output_character_map.into_iter().collect();
    Ok(())
}

fn merge_character_map_entries(target: &mut BTreeMap<char, String>, entries: &[(char, String)]) {
    for (character, replacement) in entries {
        target.insert(*character, replacement.clone());
    }
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
    private_initial: Vec<crate::xslt::golden_semantics_experiment::PrivateInitialMode>,
    policies: Vec<crate::xslt::golden_semantics_experiment::ModePolicy>,
}

impl CompiledModes {
    fn push(&mut self, declaration: mode_declaration_compiler::CompiledModeDeclaration) {
        self.typed.extend(declaration.typed_requirement);
        self.private_initial
            .extend(declaration.private_initial_mode);
        self.policies.extend(declaration.policy);
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
        GlobalBindingKind::Variable => &["name", "select", "as"][..],
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
    let default = compile_global_default(document, element, label)?;
    Ok(GlobalBinding {
        kind,
        name: name.to_owned(),
        required,
        default,
    })
}

fn compile_global_default(
    document: &Document,
    element: NodeId,
    label: &str,
) -> Result<GlobalBindingDefault, CompileFailure> {
    let declared_type = optional_attribute(document, element, None, "as");
    if let Some(select) = optional_attribute(document, element, None, "select") {
        if let Some(declared_type) = declared_type {
            ensure_no_meaningful_children(document, element, label)?;
            return compile_typed_atomic_global(document, element, declared_type, select, true);
        }
        ensure_no_meaningful_children(document, element, label)?;
        if let Some(division) = compile_double_division_global(document, element, select)? {
            Ok(division)
        } else if let Some(atomic) = compile_atomic_constructor_global(document, element, select)? {
            Ok(atomic)
        } else if let Some(variable) = select.strip_prefix('$') {
            if !is_ascii_ncname(variable) {
                return Err(invalid(
                    "FXXP0002",
                    format!("invalid variable reference: {select}"),
                    document.location(element),
                ));
            }
            Ok(GlobalBindingDefault::Variable(variable.to_owned()))
        } else if let Some(value) = select
            .strip_prefix('\'')
            .and_then(|value| value.strip_suffix('\''))
        {
            Ok(GlobalBindingDefault::Text(value.to_owned()))
        } else if let Ok(value) = select.parse::<i64>() {
            Ok(GlobalBindingDefault::Integer(value))
        } else if let Some(path) = select
            .strip_prefix("generate-id(")
            .and_then(|path| path.strip_suffix(')'))
        {
            Ok(GlobalBindingDefault::SourceNodeIdentity(
                parse_location_path(path.trim(), document.location(element).clone())
                    .map_err(map_path_failure)?,
            ))
        } else if select.trim_start().starts_with("QName(") {
            Err(unsupported(
                "FXXP1011",
                "QName construction is outside the admitted global expression slice",
                document.location(element),
            ))
        } else {
            Ok(GlobalBindingDefault::LocationPath(
                parse_location_path(select, document.location(element).clone())
                    .map_err(map_path_failure)?,
            ))
        }
    } else if document
        .children(element)
        .iter()
        .any(|node| document.kind(*node) == NodeKind::Element)
    {
        if let Some(temporary) =
            compile_parentless_temporary_node(document, element, declared_type)?
        {
            Ok(temporary)
        } else {
            let elements = compile_constructed_elements(document, element)?;
            if declared_type.is_some_and(|declared| declared != "element()")
                || declared_type == Some("element()") && elements.len() != 1
            {
                return Err(unsupported(
                    "FXST1016",
                    "the private typed global-variable slice requires one static node constructor matching its declared type",
                    document.location(element),
                ));
            }
            Ok(GlobalBindingDefault::TemporaryTree(elements))
        }
    } else {
        if let Some(declared_type) = declared_type {
            return compile_typed_atomic_global(
                document,
                element,
                declared_type,
                document.string_value(element).as_str(),
                false,
            );
        }
        Ok(GlobalBindingDefault::TemporaryText(
            document.string_value(element),
        ))
    }
}

fn compile_typed_atomic_global(
    document: &Document,
    element: NodeId,
    declared_type: &str,
    expression_or_content: &str,
    is_select: bool,
) -> Result<GlobalBindingDefault, CompileFailure> {
    let Some((prefix, local)) = declared_type.split_once(':') else {
        return Err(unsupported_typed_global(document, element, declared_type));
    };
    if namespace_for_prefix(document, element, prefix) != Some(XML_SCHEMA_NAMESPACE) {
        return Err(unsupported_typed_global(document, element, declared_type));
    }
    let atomic_type = match local {
        "string" => BuiltinAtomicType::String,
        "untypedAtomic" => BuiltinAtomicType::UntypedAtomic,
        "boolean" => BuiltinAtomicType::Boolean,
        "integer" => BuiltinAtomicType::Integer,
        "double" => BuiltinAtomicType::Double,
        _ => return Err(unsupported_typed_global(document, element, declared_type)),
    };
    let lexical = if is_select {
        if atomic_type == BuiltinAtomicType::Boolean {
            match expression_or_content.trim() {
                "true()" => "true",
                "false()" => "false",
                _ => {
                    return Err(unsupported(
                        "FXST1016",
                        "the private typed boolean global slice requires true() or false()",
                        document.location(element),
                    ));
                }
            }
        } else {
            typed_atomic_select_lexical(document, element, local, expression_or_content)
                .ok_or_else(|| {
                    unsupported(
                        "FXST1016",
                        "the private typed atomic global slice requires a matching constructor or string literal",
                        document.location(element),
                    )
                })?
        }
    } else {
        expression_or_content
    };
    if !typed_atomic_lexical_is_admitted(atomic_type, lexical) {
        return Err(unsupported(
            "FXST1016",
            "the typed atomic global literal is outside the admitted lexical value space",
            document.location(element),
        ));
    }
    Ok(GlobalBindingDefault::Atomic(
        AtomicValue::from_validated_lexical(atomic_type, lexical),
    ))
}

fn typed_atomic_lexical_is_admitted(atomic_type: BuiltinAtomicType, lexical: &str) -> bool {
    match atomic_type {
        BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic => true,
        BuiltinAtomicType::Integer => lexical.parse::<i64>().is_ok(),
        BuiltinAtomicType::Double => lexical.parse::<f64>().is_ok(),
        BuiltinAtomicType::Boolean => matches!(lexical, "true" | "false" | "1" | "0"),
        _ => false,
    }
}

fn compile_atomic_constructor_global(
    document: &Document,
    element: NodeId,
    expression: &str,
) -> Result<Option<GlobalBindingDefault>, CompileFailure> {
    let Some((constructor, argument)) = expression.split_once('(') else {
        return Ok(None);
    };
    let Some(argument) = argument.strip_suffix(')') else {
        return Ok(None);
    };
    let Some((prefix, local)) = constructor.split_once(':') else {
        return Ok(None);
    };
    if namespace_for_prefix(document, element, prefix) != Some(XML_SCHEMA_NAMESPACE) {
        return Ok(None);
    }
    let atomic_type = match local {
        "string" => BuiltinAtomicType::String,
        "untypedAtomic" => BuiltinAtomicType::UntypedAtomic,
        "boolean" => BuiltinAtomicType::Boolean,
        "integer" => BuiltinAtomicType::Integer,
        "double" => BuiltinAtomicType::Double,
        _ => return Ok(None),
    };
    let Some(lexical) = xpath_string_literal(argument.trim()) else {
        return Ok(None);
    };
    if !typed_atomic_lexical_is_admitted(atomic_type, lexical) {
        return Err(unsupported(
            "FXST1016",
            "the atomic constructor literal is outside the admitted lexical value space",
            document.location(element),
        ));
    }
    Ok(Some(GlobalBindingDefault::Atomic(
        AtomicValue::from_validated_lexical(atomic_type, lexical),
    )))
}

fn compile_double_division_global(
    document: &Document,
    element: NodeId,
    expression: &str,
) -> Result<Option<GlobalBindingDefault>, CompileFailure> {
    let Some((constructor, argument)) = expression.split_once('(') else {
        return Ok(None);
    };
    let Some(argument) = argument.strip_suffix(')') else {
        return Ok(None);
    };
    let Some((prefix, local)) = constructor.split_once(':') else {
        return Ok(None);
    };
    if local != "double"
        || namespace_for_prefix(document, element, prefix) != Some(XML_SCHEMA_NAMESPACE)
    {
        return Ok(None);
    }
    let Some((numerator, denominator)) = argument.split_once(" div ") else {
        return Ok(None);
    };
    let location = document.location(element).clone();
    Ok(Some(GlobalBindingDefault::DoubleDivision {
        numerator: parse_location_path(numerator.trim(), location.clone())
            .map_err(map_path_failure)?,
        denominator: parse_location_path(denominator.trim(), location).map_err(map_path_failure)?,
    }))
}

fn typed_atomic_select_lexical<'a>(
    document: &Document,
    element: NodeId,
    declared_local: &str,
    expression: &'a str,
) -> Option<&'a str> {
    if let Some(literal) = xpath_string_literal(expression) {
        return Some(literal);
    }
    let (constructor, argument) = expression.split_once('(')?;
    let argument = argument.strip_suffix(')')?;
    let (prefix, local) = constructor.split_once(':')?;
    (local == declared_local
        && namespace_for_prefix(document, element, prefix) == Some(XML_SCHEMA_NAMESPACE))
    .then(|| xpath_string_literal(argument))
    .flatten()
}

fn unsupported_typed_global(
    document: &Document,
    element: NodeId,
    declared_type: &str,
) -> CompileFailure {
    unsupported(
        "FXST1016",
        format!("unsupported typed global variable: {declared_type}"),
        document.location(element),
    )
}

fn xpath_string_literal(expression: &str) -> Option<&str> {
    expression
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            expression
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
}

fn compile_parentless_temporary_node(
    document: &Document,
    element: NodeId,
    declared_type: Option<&str>,
) -> Result<Option<GlobalBindingDefault>, CompileFailure> {
    let children = meaningful_children(document, element);
    let [constructor] = children.as_slice() else {
        return Ok(None);
    };
    if is_xslt_element(document, *constructor, "sequence") {
        if declared_type.is_some() {
            return Err(unsupported(
                "FXST1016",
                "the private empty-sequence global slice is untyped",
                document.location(element),
            ));
        }
        ensure_only_attributes(document, *constructor, &["select"], "xsl:sequence")?;
        ensure_no_meaningful_children(document, *constructor, "xsl:sequence")?;
        if required_attribute(document, *constructor, None, "select")? != "()" {
            return Err(unsupported(
                "FXST1016",
                "the private global xsl:sequence slice admits only the empty sequence",
                document.location(*constructor),
            ));
        }
        return Ok(Some(GlobalBindingDefault::EmptySequence));
    }
    if is_xslt_element(document, *constructor, "attribute") {
        if declared_type != Some("attribute()") {
            return Err(unsupported(
                "FXST1016",
                "the private parentless attribute slice requires as='attribute()'",
                document.location(element),
            ));
        }
        ensure_only_attributes(document, *constructor, &["name"], "xsl:attribute")?;
        let name = required_attribute(document, *constructor, None, "name")?;
        if !is_ascii_ncname(name) {
            return Err(unsupported(
                "FXST1016",
                "the private parentless attribute slice requires an unprefixed static NCName",
                document.location(*constructor),
            ));
        }
        if document
            .children(*constructor)
            .iter()
            .any(|child| document.kind(*child) == NodeKind::Element)
        {
            return Err(unsupported(
                "FXST1016",
                "the private parentless attribute slice requires static text content",
                document.location(*constructor),
            ));
        }
        return Ok(Some(GlobalBindingDefault::TemporaryAttribute {
            name: ExpandedName {
                namespace: None,
                local: name.to_owned(),
            },
            value: document.string_value(*constructor),
        }));
    }
    let (expected_type, instruction) = if is_xslt_element(document, *constructor, "text") {
        ("text()", compile_text(document, *constructor)?)
    } else if is_xslt_element(document, *constructor, "comment") {
        ("comment()", compile_comment(document, *constructor)?)
    } else if is_xslt_element(document, *constructor, "processing-instruction") {
        (
            "processing-instruction()",
            compile_processing_instruction(document, *constructor)?,
        )
    } else {
        return Ok(None);
    };
    if declared_type != Some(expected_type) {
        return Err(unsupported(
            "FXST1016",
            format!("the private parentless-node global slice requires as='{expected_type}'"),
            document.location(element),
        ));
    }
    let default = match instruction {
        Instruction::Text { value, .. } => GlobalBindingDefault::TemporaryText(value),
        Instruction::CommentNode { value, .. } => GlobalBindingDefault::TemporaryComment(value),
        Instruction::ProcessingInstructionNode { target, value, .. } => {
            GlobalBindingDefault::TemporaryProcessingInstruction { target, value }
        }
        _ => unreachable!("the selected static constructor has one known instruction shape"),
    };
    Ok(Some(default))
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
    if name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
        return Err(unsupported(
            "FXST1015",
            "XSLT instructions are outside the global temporary-tree constructor slice",
            document.location(element),
        ));
    }
    let attributes = compile_literal_result_attributes(document, element)?
        .into_iter()
        .map(|attribute| match attribute.value {
            LiteralAttributeValue::Text(value) => Ok(ConstructedAttribute {
                name: attribute.name,
                value,
            }),
            _ => Err(unsupported(
                "FXST1015",
                "dynamic literal attributes are outside the global temporary-tree constructor slice",
                &attribute.location,
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ConstructedElement {
        name: name.clone(),
        namespaces: literal_result_namespaces(document, element),
        attributes,
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
    let normalized_pattern = strip_outer_pattern_parentheses(lexical_pattern);
    let alternatives = normalized_pattern
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    if alternatives.len() == 1 {
        return compile_matched_template(document, element, lexical_pattern).map(|rule| vec![rule]);
    }
    if template_pattern_compiler::is_homogeneous_qualified_path_union(normalized_pattern) {
        return compile_matched_template(document, element, normalized_pattern)
            .map(|rule| vec![rule]);
    }
    let patterns = alternatives
        .into_iter()
        .map(|alternative| compile_match_pattern(document, element, alternative))
        .collect::<Result<Vec<_>, _>>()?;
    if !template_pattern_compiler::alternatives_are_pairwise_disjoint(&patterns) {
        let Some(priority) = patterns.first().map(|(_, priority)| *priority) else {
            unreachable!("union pattern has more than one alternative")
        };
        if patterns
            .iter()
            .any(|(_, candidate_priority)| *candidate_priority != priority)
        {
            return Err(unsupported(
                "FXST1005",
                "overlapping union alternatives with different default priorities are outside the private slice",
                document.location(element),
            ));
        }
        return Ok(vec![MatchedTemplate {
            pattern: MatchPattern::UnionAlternatives(
                patterns.into_iter().map(|(pattern, _)| pattern).collect(),
            ),
            import_precedence: 0,
            priority,
            modes: compile_template_modes_for_rule(document, element)?,
            template: compile_template(document, element)?,
        }]);
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

fn strip_outer_pattern_parentheses(pattern: &str) -> &str {
    let trimmed = pattern.trim();
    trimmed
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
        .map_or(trimmed, str::trim)
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
            ensure_only_attributes(
                document,
                child,
                &["name", "tunnel", "select", "required"],
                "xsl:param",
            )?;
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
            let required = parse_template_parameter_required(document, child)?;
            let children = meaningful_children(document, child);
            if required
                && (optional_attribute(document, child, None, "select").is_some()
                    || !children.is_empty())
            {
                return Err(invalid(
                    "XTSE0010",
                    "a required template parameter cannot declare a default value",
                    document.location(child),
                ));
            }
            let default = compile_template_parameter_default(document, child, &children)?;
            parameters.push(TemplateParameter {
                name,
                tunnel,
                required,
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
                    "xsl:param must precede the named-template body",
                    document.location(child),
                ));
            }
            parameters.push(compile_named_template_parameter(
                document,
                child,
                &parameters,
            )?);
            parameter_nodes.push(child);
        } else {
            body_started = true;
        }
    }
    Ok(NamedTemplate {
        name: name.to_owned(),
        parameters: parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect(),
        template: Template {
            parameters,
            body: compile_sequence_excluding(document, element, &parameter_nodes)?,
            location: document.location(element).clone(),
        },
    })
}

fn compile_named_template_parameter(
    document: &Document,
    child: NodeId,
    preceding: &[TemplateParameter],
) -> Result<TemplateParameter, CompileFailure> {
    ensure_only_attributes(
        document,
        child,
        &["name", "tunnel", "select", "required"],
        "xsl:param",
    )?;
    let parameter = required_attribute(document, child, None, "name")?;
    if !is_ascii_ncname(parameter) || preceding.iter().any(|existing| existing.name == parameter) {
        return Err(invalid(
            "FXST0012",
            format!("invalid or duplicate named-template parameter: {parameter}"),
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
    let required = parse_template_parameter_required(document, child)?;
    let children = meaningful_children(document, child);
    if required
        && (optional_attribute(document, child, None, "select").is_some() || !children.is_empty())
    {
        return Err(invalid(
            "XTSE0010",
            "a required named-template parameter cannot declare a default value",
            document.location(child),
        ));
    }
    let default = compile_template_parameter_default(document, child, &children)?;
    Ok(TemplateParameter {
        name: parameter.to_owned(),
        tunnel,
        required,
        default,
    })
}

fn compile_template_parameter_default(
    document: &Document,
    child: NodeId,
    children: &[NodeId],
) -> Result<TemplateParameterDefault, CompileFailure> {
    let Some(select) = optional_attribute(document, child, None, "select") else {
        if children
            .iter()
            .any(|node| document.kind(*node) != NodeKind::Text)
        {
            return Err(unsupported(
                "FXST1032",
                "the private template parameter default slice permits only literal text",
                document.location(child),
            ));
        }
        return Ok(TemplateParameterDefault::Text(document.string_value(child)));
    };
    ensure_no_meaningful_children(document, child, "xsl:param")?;
    if let Ok(value) = select.parse::<i64>() {
        return Ok(TemplateParameterDefault::Integer(value));
    }
    if let Some(value) = static_string_literal(select) {
        return Ok(TemplateParameterDefault::Text(value.to_owned()));
    }
    Err(unsupported(
        "FXST1032",
        format!("unsupported template parameter default: {select}"),
        document.location(child),
    ))
}

fn parse_template_parameter_required(
    document: &Document,
    parameter: NodeId,
) -> Result<bool, CompileFailure> {
    match optional_attribute(document, parameter, None, "required") {
        None | Some("no") => Ok(false),
        Some("yes") => Ok(true),
        Some(value) => Err(invalid(
            "XTSE0020",
            format!("invalid xsl:param required value: {value}"),
            document.location(parameter),
        )),
    }
}

pub(super) fn normalize_named_template_name(
    document: &Document,
    element: NodeId,
    name: &str,
) -> Result<String, CompileFailure> {
    let name = name.trim();
    if is_ascii_ncname(name) {
        return Ok(name.to_owned());
    }
    if let Some(qualified) = name.strip_prefix("Q{") {
        let Some((namespace, local)) = qualified.split_once('}') else {
            return Err(unsupported(
                "FXST1013",
                format!("unsupported named-template name: {name}"),
                document.location(element),
            ));
        };
        if namespace.contains(['{', '}']) || !is_ascii_ncname(local) {
            return Err(unsupported(
                "FXST1013",
                format!("unsupported named-template name: {name}"),
                document.location(element),
            ));
        }
        return normalize_expanded_named_template_name(document, element, namespace, local, name);
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
            return normalize_expanded_named_template_name(
                document,
                element,
                &binding.namespace,
                local,
                name,
            );
        }
        current = document.parent(node);
    }
    Err(unsupported(
        "FXST1013",
        format!("unsupported named-template name: {name}"),
        document.location(element),
    ))
}

fn normalize_expanded_named_template_name(
    document: &Document,
    element: NodeId,
    namespace: &str,
    local: &str,
    lexical_name: &str,
) -> Result<String, CompileFailure> {
    if namespace == XSLT_NAMESPACE {
        if local == "initial-template" {
            return Ok(STANDARD_INITIAL_TEMPLATE_NAME.to_owned());
        }
        return Err(invalid(
            "XTSE0080",
            format!("reserved named-template name: {lexical_name}"),
            document.location(element),
        ));
    }
    if namespace.is_empty() {
        Ok(local.to_owned())
    } else {
        Ok(format!("Q{{{namespace}}}{local}"))
    }
}

fn static_string_literal(expression: &str) -> Option<&str> {
    expression
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            expression
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
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
        PathFailure::Invalid {
            standard_code,
            detail,
            location,
        } => CompileFailure {
            code: standard_code,
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
#[path = "golden_stylesheet_tests.rs"]
mod tests;
