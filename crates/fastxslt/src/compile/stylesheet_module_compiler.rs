use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::{
    Instruction, MatchPattern, MatchedTemplate, SourceWhitespacePolicy, StylesheetProgram,
    Template, TemplatePriority,
};

use super::instruction_compiler::{compile_sequence_excluding, literal_result_namespaces};
use super::stylesheet_validation::validate_named_template_references;
use super::{
    CompileFailure, XSLT_NAMESPACE, compile_stylesheet_at_excluding_unvalidated,
    compile_stylesheet_excluding_unvalidated, default_output_settings, document_element,
    ensure_no_meaningful_children, ensure_only_attributes, finalize_character_maps, invalid,
    is_xslt_element, meaningful_children, optional_attribute, require_stylesheet_root,
    required_attribute, unsupported,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StylesheetDependencyKind {
    Include,
    Import,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StylesheetDependencyReference {
    pub(crate) kind: StylesheetDependencyKind,
    pub(crate) href: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn discovered_stylesheet_dependencies_at(
    document: &Document,
    root: NodeId,
) -> Result<Vec<StylesheetDependencyReference>, CompileFailure> {
    let is_standard_stylesheet = document.name(root).is_some_and(|name| {
        name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            && matches!(name.local.as_str(), "stylesheet" | "transform")
    });
    if !is_standard_stylesheet {
        return Ok(Vec::new());
    }
    dependency_nodes_at(document, root)?
        .into_iter()
        .map(|declaration| {
            let kind = if is_xslt_element(document, declaration, "include") {
                StylesheetDependencyKind::Include
            } else {
                StylesheetDependencyKind::Import
            };
            let label = match kind {
                StylesheetDependencyKind::Include => "xsl:include",
                StylesheetDependencyKind::Import => "xsl:import",
            };
            ensure_only_attributes(document, declaration, &["href"], label)?;
            ensure_no_meaningful_children(document, declaration, label)?;
            Ok(StylesheetDependencyReference {
                kind,
                href: required_attribute(document, declaration, None, "href")?.to_owned(),
                location: document.location(declaration).clone(),
            })
        })
        .collect()
}

pub(crate) fn compile_stylesheet_with_single_include(
    principal: &Document,
    included: &Document,
    included_root: NodeId,
) -> Result<StylesheetProgram, CompileFailure> {
    let principal_root = document_element(principal)?;
    let included_program = compile_dependency_module(included, included_root)?;
    compile_stylesheet_with_single_include_program_at(principal, principal_root, included_program)
}

pub(crate) fn compile_stylesheet_with_single_include_program_at(
    principal: &Document,
    principal_root: NodeId,
    included_program: StylesheetProgram,
) -> Result<StylesheetProgram, CompileFailure> {
    let include_declarations = include_nodes_at(principal, principal_root)?;
    let [include] = include_declarations.as_slice() else {
        return Err(invalid(
            "FXST0027",
            "single-include compilation requires exactly one xsl:include",
            principal.location(principal.document_node()),
        ));
    };
    let mut program =
        compile_stylesheet_at_excluding_unvalidated(principal, principal_root, &[*include])?;

    merge_included_program(
        &mut program,
        included_program,
        principal.location(principal_root),
        false,
    )?;
    finalize_character_maps(&mut program)?;
    validate_named_template_references(&program)?;
    Ok(program)
}

fn merge_included_program(
    program: &mut StylesheetProgram,
    mut included_program: StylesheetProgram,
    location: &SourceLocation,
    allow_duplicate_matches: bool,
) -> Result<(), CompileFailure> {
    if included_program.source_whitespace == SourceWhitespacePolicy::StripAllElementWhitespace {
        program.source_whitespace = SourceWhitespacePolicy::StripAllElementWhitespace;
    }
    program
        .typed_mode_requirements
        .append(&mut included_program.typed_mode_requirements);
    program
        .private_initial_modes
        .append(&mut included_program.private_initial_modes);
    program
        .mode_policies
        .append(&mut included_program.mode_policies);
    merge_included_character_maps(program, included_program.character_maps, location)?;
    if included_program.output != default_output_settings() {
        return Err(unsupported(
            "FXST1019",
            "included output declarations are outside the single-include slice",
            location,
        ));
    }
    if program.root_template.is_some() && included_program.root_template.is_some() {
        return Err(unsupported(
            "FXST1020",
            "template priority across duplicate root matches is outside the single-include slice",
            location,
        ));
    }
    if program.root_template.is_none() {
        program.root_template = included_program.root_template;
        program.root_template_modes = included_program.root_template_modes;
    }
    for matched in included_program.matched_templates {
        if !allow_duplicate_matches
            && program.matched_templates.iter().any(|existing| {
                existing.pattern == matched.pattern && existing.modes == matched.modes
            })
        {
            return Err(unsupported(
                "FXST1021",
                "template priority across duplicate included matches is outside the single-include slice",
                &matched.template.location,
            ));
        }
        program.matched_templates.push(matched);
    }
    for named in included_program.named_templates {
        if program
            .named_templates
            .iter()
            .any(|existing| existing.name == named.name)
        {
            return Err(invalid(
                "FXST0028",
                format!(
                    "duplicate named template across included modules: {}",
                    named.name
                ),
                &named.template.location,
            ));
        }
        program.named_templates.push(named);
    }
    for binding in included_program.global_bindings {
        if program
            .global_bindings
            .iter()
            .any(|existing| existing.name == binding.name)
        {
            return Err(invalid(
                "FXST0029",
                format!(
                    "duplicate global binding across included modules: ${}",
                    binding.name
                ),
                location,
            ));
        }
        program.global_bindings.push(binding);
    }
    Ok(())
}

pub(crate) fn compile_stylesheet_with_two_included_programs_at(
    principal: &Document,
    principal_root: NodeId,
    included_programs: [StylesheetProgram; 2],
) -> Result<StylesheetProgram, CompileFailure> {
    let include_declarations = include_nodes_at(principal, principal_root)?;
    if include_declarations.len() != included_programs.len() {
        return Err(invalid(
            "FXST0034",
            "two-include compilation requires exactly two supplied included modules",
            principal.location(principal_root),
        ));
    }
    let mut program = compile_stylesheet_at_excluding_unvalidated(
        principal,
        principal_root,
        &include_declarations,
    )?;
    for included in included_programs {
        merge_included_program(
            &mut program,
            included,
            principal.location(principal_root),
            true,
        )?;
    }
    finalize_character_maps(&mut program)?;
    validate_named_template_references(&program)?;
    Ok(program)
}

pub(crate) fn compile_stylesheet_with_import_and_include(
    principal: &Document,
    imported: (&Document, NodeId),
    included: (&Document, NodeId),
) -> Result<StylesheetProgram, CompileFailure> {
    let root = document_element(principal)?;
    let dependencies = dependency_nodes_at(principal, root)?;
    let [import, include] = dependencies.as_slice() else {
        return Err(invalid(
            "FXST0033",
            "mixed compilation requires exactly one xsl:import followed by one xsl:include",
            principal.location(root),
        ));
    };
    if !is_xslt_element(principal, *import, "import")
        || !is_xslt_element(principal, *include, "include")
    {
        return Err(invalid(
            "FXST0033",
            "mixed compilation requires one xsl:import followed by one xsl:include",
            principal.location(*import),
        ));
    }
    let mut program = compile_stylesheet_at_excluding_unvalidated(principal, root, &dependencies)?;
    let included_program = compile_dependency_module(included.0, included.1)?;
    merge_included_program(
        &mut program,
        included_program,
        principal.location(*include),
        false,
    )?;

    let mut imported_program = compile_imported_program(imported.0, imported.1, -1)?;
    validate_fully_shadowed_imported_output(
        &program,
        &imported_program,
        principal.location(*import),
    )?;
    imported_program
        .matched_templates
        .append(&mut program.matched_templates);
    program.matched_templates = imported_program.matched_templates;
    merge_imported_named_templates(&mut program, imported_program.named_templates);
    merge_imported_global_bindings(&mut program, imported_program.global_bindings);
    merge_imported_character_maps(&mut program, imported_program.character_maps);
    finalize_character_maps(&mut program)?;
    validate_named_template_references(&program)?;
    Ok(program)
}

pub(crate) fn compile_stylesheet_with_imports(
    principal: &Document,
    imported: &[(&Document, NodeId)],
) -> Result<StylesheetProgram, CompileFailure> {
    let import_declarations = import_nodes(principal)?;
    if import_declarations.is_empty() || import_declarations.len() != imported.len() {
        return Err(invalid(
            "FXST0031",
            "import compilation requires one supplied module per xsl:import",
            principal.location(principal.document_node()),
        ));
    }
    if imported.len() > 2 {
        return Err(unsupported(
            "FXST1028",
            "the private import slice permits at most two sibling imports",
            principal.location(principal.document_node()),
        ));
    }
    let root = document_element(principal)?;
    let mut principal_program =
        compile_stylesheet_excluding_unvalidated(principal, &import_declarations)?;
    let overridden_visibility_modes = explicit_visibility_mode_names(principal, root);
    let import_count = i32::try_from(imported.len()).expect("bounded import count fits i32");
    let mut imported_programs = imported
        .iter()
        .enumerate()
        .map(|(index, (document, root))| {
            let precedence =
                i32::try_from(index).expect("bounded import index fits i32") - import_count;
            let excluded_modes =
                shadowed_visibility_only_modes(document, *root, &overridden_visibility_modes);
            compile_imported_program_excluding(document, *root, precedence, &excluded_modes)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if imported_programs.len() == 1 {
        merge_single_imported_output(
            &mut principal_program,
            &imported_programs[0],
            principal.location(root),
        )?;
    } else {
        for program in &imported_programs {
            validate_fully_shadowed_imported_output(
                &principal_program,
                program,
                principal.location(root),
            )?;
        }
    }

    let mut matched_templates = Vec::new();
    for program in &mut imported_programs {
        matched_templates.append(&mut program.matched_templates);
    }
    matched_templates.append(&mut principal_program.matched_templates);
    principal_program.matched_templates = matched_templates;
    for program in imported_programs.into_iter().rev() {
        merge_imported_named_templates(&mut principal_program, program.named_templates);
        merge_imported_global_bindings(&mut principal_program, program.global_bindings);
        merge_imported_character_maps(&mut principal_program, program.character_maps);
    }
    finalize_character_maps(&mut principal_program)?;
    validate_named_template_references(&principal_program)?;
    Ok(principal_program)
}

pub(crate) fn compile_stylesheet_with_two_imported_programs_at(
    principal: &Document,
    principal_root: NodeId,
    imported_programs: [StylesheetProgram; 2],
) -> Result<StylesheetProgram, CompileFailure> {
    let import_declarations = import_nodes_at(principal, principal_root)?;
    if import_declarations.len() != imported_programs.len() {
        return Err(invalid(
            "FXST0035",
            "two-program import compilation requires exactly two xsl:import declarations",
            principal.location(principal_root),
        ));
    }
    let mut principal_program = compile_stylesheet_at_excluding_unvalidated(
        principal,
        principal_root,
        &import_declarations,
    )?;
    let mut imported_programs = imported_programs;
    for (program, shift) in imported_programs.iter_mut().zip([-3, -1]) {
        rebase_imported_program(program, shift, principal.location(principal_root))?;
    }
    for program in &imported_programs {
        validate_fully_shadowed_imported_output(
            &principal_program,
            program,
            principal.location(principal_root),
        )?;
    }

    let mut matched_templates = Vec::new();
    for program in &mut imported_programs {
        matched_templates.append(&mut program.matched_templates);
    }
    matched_templates.append(&mut principal_program.matched_templates);
    principal_program.matched_templates = matched_templates;
    for program in imported_programs.into_iter().rev() {
        merge_imported_named_templates(&mut principal_program, program.named_templates);
        merge_imported_global_bindings(&mut principal_program, program.global_bindings);
        merge_imported_character_maps(&mut principal_program, program.character_maps);
    }
    finalize_character_maps(&mut principal_program)?;
    validate_named_template_references(&principal_program)?;
    Ok(principal_program)
}

pub(crate) fn compile_stylesheet_with_single_imported_program_at(
    principal: &Document,
    principal_root: NodeId,
    mut imported_program: StylesheetProgram,
) -> Result<StylesheetProgram, CompileFailure> {
    let import_declarations = import_nodes_at(principal, principal_root)?;
    if import_declarations.len() != 1 {
        return Err(invalid(
            "FXST0035",
            "single-program import compilation requires exactly one xsl:import declaration",
            principal.location(principal_root),
        ));
    }
    let mut principal_program = compile_stylesheet_at_excluding_unvalidated(
        principal,
        principal_root,
        &import_declarations,
    )?;
    rebase_imported_program(
        &mut imported_program,
        -1,
        principal.location(principal_root),
    )?;
    merge_single_imported_output(
        &mut principal_program,
        &imported_program,
        principal.location(principal_root),
    )?;

    let mut matched_templates = imported_program.matched_templates;
    matched_templates.append(&mut principal_program.matched_templates);
    principal_program.matched_templates = matched_templates;
    merge_imported_named_templates(&mut principal_program, imported_program.named_templates);
    merge_imported_global_bindings(&mut principal_program, imported_program.global_bindings);
    merge_imported_character_maps(&mut principal_program, imported_program.character_maps);
    finalize_character_maps(&mut principal_program)?;
    validate_named_template_references(&principal_program)?;
    Ok(principal_program)
}

fn rebase_imported_program(
    program: &mut StylesheetProgram,
    shift: i32,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    if program
        .matched_templates
        .iter()
        .any(|template| !matches!(template.import_precedence, -1 | 0))
    {
        return Err(unsupported(
            "FXST1030",
            "the private nested-import slice requires one precedence level below each imported branch",
            location,
        ));
    }
    for template in &mut program.matched_templates {
        template.import_precedence += shift;
    }
    if let Some(template) = program.root_template.take() {
        program.matched_templates.insert(
            0,
            MatchedTemplate {
                pattern: MatchPattern::Document,
                import_precedence: shift,
                priority: TemplatePriority::ROOT_DEFAULT,
                modes: std::mem::take(&mut program.root_template_modes),
                template,
            },
        );
    }
    Ok(())
}

fn compile_imported_program(
    imported: &Document,
    imported_root: NodeId,
    import_precedence: i32,
) -> Result<StylesheetProgram, CompileFailure> {
    compile_imported_program_excluding(imported, imported_root, import_precedence, &[])
}

fn compile_imported_program_excluding(
    imported: &Document,
    imported_root: NodeId,
    import_precedence: i32,
    excluded: &[NodeId],
) -> Result<StylesheetProgram, CompileFailure> {
    let mut imported_program = if excluded.is_empty() {
        compile_dependency_module(imported, imported_root)?
    } else {
        compile_stylesheet_at_excluding_unvalidated(imported, imported_root, excluded)?
    };
    if let Some(template) = imported_program.root_template.take() {
        imported_program.matched_templates.insert(
            0,
            MatchedTemplate {
                pattern: MatchPattern::Document,
                import_precedence,
                priority: TemplatePriority::ROOT_DEFAULT,
                modes: std::mem::take(&mut imported_program.root_template_modes),
                template,
            },
        );
    }
    for template in &mut imported_program.matched_templates {
        template.import_precedence = import_precedence;
    }
    Ok(imported_program)
}

fn explicit_visibility_mode_names(document: &Document, root: NodeId) -> Vec<String> {
    meaningful_children(document, root)
        .into_iter()
        .filter(|node| is_xslt_element(document, *node, "mode"))
        .filter_map(|node| {
            optional_attribute(document, node, None, "visibility")?;
            let name = optional_attribute(document, node, None, "name")?;
            (!name.contains(':') && !name.starts_with('#')).then(|| name.to_owned())
        })
        .collect()
}

fn shadowed_visibility_only_modes(
    document: &Document,
    root: NodeId,
    overridden_names: &[String],
) -> Vec<NodeId> {
    meaningful_children(document, root)
        .into_iter()
        .filter(|node| is_xslt_element(document, *node, "mode"))
        .filter(|node| {
            let Some(name) = optional_attribute(document, *node, None, "name") else {
                return false;
            };
            overridden_names.iter().any(|candidate| candidate == name)
                && optional_attribute(document, *node, None, "visibility").is_some()
                && document.attributes(*node).iter().all(|attribute| {
                    document.name(*attribute).is_some_and(|name| {
                        name.namespace.is_none()
                            && matches!(name.local.as_str(), "name" | "visibility")
                    })
                })
        })
        .collect()
}

fn validate_fully_shadowed_imported_output(
    principal: &StylesheetProgram,
    imported: &StylesheetProgram,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    let unshadowed = imported
        .output_specified_properties
        .iter()
        .filter(|property| !principal.output_specified_properties.contains(property))
        .cloned()
        .collect::<Vec<_>>();
    if unshadowed.is_empty() {
        return Ok(());
    }
    Err(unsupported(
        "FXST1024",
        format!(
            "imported output properties not explicitly shadowed by the principal declaration are outside the bounded import slice: {}",
            unshadowed.join(", ")
        ),
        location,
    ))
}

fn merge_single_imported_output(
    principal: &mut StylesheetProgram,
    imported: &StylesheetProgram,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    let unshadowed = imported
        .output_specified_properties
        .iter()
        .filter(|property| !principal.output_specified_properties.contains(property))
        .cloned()
        .collect::<Vec<_>>();
    if unshadowed
        .iter()
        .any(|property| !matches!(property.as_str(), "method" | "encoding" | "indent"))
    {
        return validate_fully_shadowed_imported_output(principal, imported, location);
    }
    for property in unshadowed {
        match property.as_str() {
            "method" => principal.output.method.clone_from(&imported.output.method),
            "encoding" => principal
                .output
                .encoding
                .clone_from(&imported.output.encoding),
            "indent" => principal.output.indent = imported.output.indent,
            _ => unreachable!("unadmitted output properties were rejected"),
        }
        principal.output_specified_properties.push(property);
    }
    Ok(())
}

fn compile_dependency_module(
    document: &Document,
    root: NodeId,
) -> Result<StylesheetProgram, CompileFailure> {
    if document
        .name(root)
        .is_some_and(|name| name.namespace.as_deref() == Some(XSLT_NAMESPACE))
    {
        super::compile_stylesheet_at(document, root)
    } else {
        compile_simplified_stylesheet_at(document, root)
    }
}

fn merge_imported_named_templates(
    principal: &mut StylesheetProgram,
    imported: Vec<crate::xslt::golden_semantics_experiment::NamedTemplate>,
) {
    for template in imported {
        if principal
            .named_templates
            .iter()
            .any(|existing| existing.name == template.name)
        {
            continue;
        }
        principal.named_templates.push(template);
    }
}

fn merge_imported_global_bindings(
    principal: &mut StylesheetProgram,
    mut imported: Vec<crate::xslt::golden_semantics_experiment::GlobalBinding>,
) {
    imported.retain(|binding| {
        !principal
            .global_bindings
            .iter()
            .any(|existing| existing.name == binding.name)
    });
    imported.append(&mut principal.global_bindings);
    principal.global_bindings = imported;
}

fn merge_imported_character_maps(
    principal: &mut StylesheetProgram,
    mut imported: Vec<crate::xslt::golden_semantics_experiment::CharacterMapDefinition>,
) {
    imported.retain(|map| {
        !principal
            .character_maps
            .iter()
            .any(|existing| existing.name == map.name)
    });
    imported.append(&mut principal.character_maps);
    principal.character_maps = imported;
}

fn merge_included_character_maps(
    principal: &mut StylesheetProgram,
    included: Vec<crate::xslt::golden_semantics_experiment::CharacterMapDefinition>,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    for map in included {
        if principal
            .character_maps
            .iter()
            .any(|existing| existing.name == map.name)
        {
            return Err(invalid(
                "XTSE1580",
                "duplicate character map at one import precedence",
                location,
            ));
        }
        principal.character_maps.push(map);
    }
    Ok(())
}

fn compile_simplified_stylesheet_at(
    document: &Document,
    root: NodeId,
) -> Result<StylesheetProgram, CompileFailure> {
    let root_name = document.name(root).expect("element nodes have names");
    if root_name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
        return Err(unsupported(
            "FXST1022",
            "the first included-module slice requires a simplified stylesheet",
            document.location(root),
        ));
    }
    let declared_version = required_attribute(document, root, Some(XSLT_NAMESPACE), "version")?;
    for attribute in document.attributes(root) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have names");
        if name.namespace.as_deref() != Some(XSLT_NAMESPACE) || name.local != "version" {
            return Err(unsupported(
                "FXST1023",
                "literal attributes on the simplified stylesheet root are outside the first include slice",
                document.location(*attribute),
            ));
        }
    }
    let root_template = Template {
        parameters: Vec::new(),
        body: vec![Instruction::LiteralElement {
            name: root_name.clone(),
            namespaces: literal_result_namespaces(document, root),
            attributes: Vec::new(),
            computed_attributes: Vec::new(),
            body: compile_sequence_excluding(document, root, &[])?,
            location: document.location(root).clone(),
        }],
        location: document.location(root).clone(),
    };
    Ok(StylesheetProgram {
        declared_version: declared_version.to_owned(),
        default_initial_mode: None,
        source_whitespace: SourceWhitespacePolicy::Preserve,
        typed_mode_requirements: Vec::new(),
        private_initial_modes: Vec::new(),
        mode_policies: Vec::new(),
        output: default_output_settings(),
        output_specified_properties: Vec::new(),
        character_maps: Vec::new(),
        output_character_map_names: Vec::new(),
        output_character_map_location: None,
        root_template: Some(root_template),
        root_template_modes: Vec::new(),
        matched_templates: Vec::new(),
        named_templates: Vec::new(),
        global_bindings: Vec::new(),
    })
}

fn include_nodes_at(document: &Document, root: NodeId) -> Result<Vec<NodeId>, CompileFailure> {
    require_stylesheet_root(document, root)?;
    Ok(meaningful_children(document, root)
        .into_iter()
        .filter(|child| is_xslt_element(document, *child, "include"))
        .collect())
}

fn import_nodes(document: &Document) -> Result<Vec<NodeId>, CompileFailure> {
    let root = document_element(document)?;
    import_nodes_at(document, root)
}

fn import_nodes_at(document: &Document, root: NodeId) -> Result<Vec<NodeId>, CompileFailure> {
    require_stylesheet_root(document, root)?;
    Ok(meaningful_children(document, root)
        .into_iter()
        .filter(|child| is_xslt_element(document, *child, "import"))
        .collect())
}

fn dependency_nodes_at(document: &Document, root: NodeId) -> Result<Vec<NodeId>, CompileFailure> {
    require_stylesheet_root(document, root)?;
    Ok(meaningful_children(document, root)
        .into_iter()
        .filter(|child| {
            is_xslt_element(document, *child, "include")
                || is_xslt_element(document, *child, "import")
        })
        .collect())
}
