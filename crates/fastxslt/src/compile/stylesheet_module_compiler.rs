use std::collections::HashSet;

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::{
    Instruction, MatchPattern, MatchedTemplate, SourceWhitespacePolicy, StylesheetProgram,
    Template, TemplatePriority,
};

use super::instruction_compiler::compile_literal_element;
use super::namespace_alias_compiler;
use super::stylesheet_validation::validate_named_template_references;
use super::{
    CompileFailure, XSLT_NAMESPACE, compile_stylesheet_at_excluding_unvalidated,
    compile_stylesheet_excluding_unvalidated, default_output_settings, document_element,
    ensure_no_meaningful_children, ensure_only_attributes, finalize_character_maps,
    finalize_decimal_formats, invalid, is_xslt_element, meaningful_children, merge_output,
    optional_attribute, output_compiler::OutputDeclaration, require_stylesheet_root,
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
    mut included_program: StylesheetProgram,
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
    apply_principal_namespace_aliases(principal, principal_root, &mut included_program)?;

    merge_included_program(&mut program, included_program, principal.location(*include))?;
    finalize_character_maps(&mut program)?;
    finalize_decimal_formats(&mut program)?;
    validate_named_template_references(&program)?;
    Ok(program)
}

fn merge_included_program(
    program: &mut StylesheetProgram,
    mut included_program: StylesheetProgram,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    merge_attribute_set_declarations(program, &mut included_program, location);
    if program.root_template.is_some() && included_program.root_template.is_some() {
        materialize_root_template_in_declaration_order(program);
        materialize_root_template_in_declaration_order(&mut included_program);
    }
    let combined_import_floor = program
        .matched_templates
        .iter()
        .chain(&included_program.matched_templates)
        .map(|template| template.import_precedence)
        .min()
        .unwrap_or(0);
    for template in program
        .matched_templates
        .iter_mut()
        .chain(&mut included_program.matched_templates)
        .filter(|template| template.import_precedence == 0)
    {
        template.apply_imports_min_precedence = combined_import_floor;
    }
    merge_source_whitespace_policy(
        &mut program.source_whitespace,
        &included_program.source_whitespace,
    );
    program
        .typed_mode_requirements
        .append(&mut included_program.typed_mode_requirements);
    program
        .private_initial_modes
        .append(&mut included_program.private_initial_modes);
    program
        .mode_policies
        .append(&mut included_program.mode_policies);
    merge_included_output(program, &mut included_program, location)?;
    merge_included_character_maps(program, included_program.character_maps, location)?;
    merge_included_decimal_formats(program, included_program.decimal_formats, location)?;
    program
        .key_definitions
        .append(&mut included_program.key_definitions);
    if program.root_template.is_none() {
        program.root_template = included_program.root_template;
        program.root_template_modes = included_program.root_template_modes;
    }
    let insertion_index = program
        .matched_templates
        .iter()
        .position(|matched| {
            matched.template.location.resource == location.resource
                && matched.template.location.span.start > location.span.start
        })
        .unwrap_or(program.matched_templates.len());
    program.matched_templates.splice(
        insertion_index..insertion_index,
        included_program.matched_templates,
    );
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
    super::order_merged_global_dependencies(&mut program.global_bindings, location)?;
    Ok(())
}

fn merge_source_whitespace_policy(
    principal: &mut SourceWhitespacePolicy,
    included: &SourceWhitespacePolicy,
) {
    match included {
        SourceWhitespacePolicy::Preserve => {}
        SourceWhitespacePolicy::StripAllElementWhitespace => {
            *principal = SourceWhitespacePolicy::StripAllElementWhitespace;
        }
        SourceWhitespacePolicy::StripExpandedNames(included_names) => match principal {
            SourceWhitespacePolicy::Preserve => {
                *principal = SourceWhitespacePolicy::StripExpandedNames(included_names.clone());
            }
            SourceWhitespacePolicy::StripAllElementWhitespace => {}
            SourceWhitespacePolicy::StripExpandedNames(names) => {
                for name in included_names {
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                }
            }
        },
    }
}

fn materialize_root_template_in_declaration_order(program: &mut StylesheetProgram) {
    let Some(template) = program.root_template.take() else {
        return;
    };
    let insertion_index = program
        .matched_templates
        .iter()
        .position(|matched| {
            matched.template.location.resource == template.location.resource
                && matched.template.location.span.start > template.location.span.start
        })
        .unwrap_or(program.matched_templates.len());
    program.matched_templates.insert(
        insertion_index,
        MatchedTemplate {
            pattern: MatchPattern::Document,
            import_precedence: 0,
            apply_imports_min_precedence: 0,
            priority: TemplatePriority::ROOT_DEFAULT,
            modes: std::mem::take(&mut program.root_template_modes),
            template,
        },
    );
}

fn merge_included_output(
    program: &mut StylesheetProgram,
    included: &mut StylesheetProgram,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    if included.output_specified_properties.is_empty() {
        return Ok(());
    }
    let mut existing = OutputDeclaration {
        name: None,
        settings: std::mem::replace(&mut program.output, default_output_settings()),
        character_map_names: std::mem::take(&mut program.output_character_map_names),
        specified: std::mem::take(&mut program.output_specified_properties)
            .into_iter()
            .collect(),
        location: program
            .output_character_map_location
            .clone()
            .unwrap_or_else(|| location.clone()),
    };
    let mut next = OutputDeclaration {
        name: None,
        settings: std::mem::replace(&mut included.output, default_output_settings()),
        character_map_names: std::mem::take(&mut included.output_character_map_names),
        specified: std::mem::take(&mut included.output_specified_properties)
            .into_iter()
            .collect(),
        location: included
            .output_character_map_location
            .clone()
            .unwrap_or_else(|| location.clone()),
    };
    let existing_same_precedence = program
        .output_same_precedence_properties
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let included_same_precedence = included
        .output_same_precedence_properties
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let overlaps = existing
        .specified
        .intersection(&next.specified)
        .cloned()
        .collect::<Vec<_>>();
    for property in overlaps {
        match (
            existing_same_precedence.contains(&property),
            included_same_precedence.contains(&property),
        ) {
            (true, false) => {
                next.specified.remove(&property);
                clear_inherited_output_property(&mut next.settings, &property, location)?;
            }
            (false, true) => {
                existing.specified.remove(&property);
                clear_inherited_output_property(&mut existing.settings, &property, location)?;
            }
            (true, true) | (false, false) => {}
        }
    }
    let merged = merge_output(existing, next, false)?;
    program.output = merged.settings;
    program.output_character_map_names = merged.character_map_names;
    program.output_specified_properties = merged.specified.into_iter().collect();
    for property in std::mem::take(&mut included.output_same_precedence_properties) {
        if !program
            .output_same_precedence_properties
            .contains(&property)
        {
            program.output_same_precedence_properties.push(property);
        }
    }
    if program.output_character_map_location.is_none()
        && !program.output_character_map_names.is_empty()
    {
        program.output_character_map_location = included
            .output_character_map_location
            .clone()
            .or_else(|| Some(location.clone()));
    }
    Ok(())
}

fn clear_inherited_output_property(
    settings: &mut crate::xslt::golden_semantics_experiment::OutputSettings,
    property: &str,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    match property {
        "method" => settings.method = None,
        "encoding" => settings.encoding = None,
        "indent" => settings.indent = None,
        _ => {
            return Err(unsupported(
                "FXST1024",
                format!(
                    "included inherited output property {property} is outside the bounded import-precedence slice"
                ),
                location,
            ));
        }
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
    for (mut included, include) in included_programs.into_iter().zip(include_declarations) {
        apply_principal_namespace_aliases(principal, principal_root, &mut included)?;
        merge_included_program(&mut program, included, principal.location(include))?;
    }
    finalize_character_maps(&mut program)?;
    finalize_decimal_formats(&mut program)?;
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
    let imported_program = compile_dependency_module(imported.0, imported.1)?;
    let included_program = compile_dependency_module(included.0, included.1)?;
    compose_imported_and_included_programs(
        principal,
        root,
        *import,
        *include,
        imported_program,
        included_program,
    )
}

pub(crate) fn compile_stylesheet_with_imported_and_included_programs_at(
    principal: &Document,
    principal_root: NodeId,
    imported_program: StylesheetProgram,
    included_program: StylesheetProgram,
) -> Result<StylesheetProgram, CompileFailure> {
    let dependencies = dependency_nodes_at(principal, principal_root)?;
    let [import, include] = dependencies.as_slice() else {
        return Err(invalid(
            "FXST0033",
            "mixed program compilation requires exactly one xsl:import followed by one xsl:include",
            principal.location(principal_root),
        ));
    };
    if !is_xslt_element(principal, *import, "import")
        || !is_xslt_element(principal, *include, "include")
    {
        return Err(invalid(
            "FXST0033",
            "mixed program compilation requires one xsl:import followed by one xsl:include",
            principal.location(*import),
        ));
    }
    compose_imported_and_included_programs(
        principal,
        principal_root,
        *import,
        *include,
        imported_program,
        included_program,
    )
}

fn compose_imported_and_included_programs(
    principal: &Document,
    principal_root: NodeId,
    import: NodeId,
    include: NodeId,
    mut imported_program: StylesheetProgram,
    mut included_program: StylesheetProgram,
) -> Result<StylesheetProgram, CompileFailure> {
    let dependencies = [import, include];
    let mut program =
        compile_stylesheet_at_excluding_unvalidated(principal, principal_root, &dependencies)?;
    let included_minimum_precedence = included_program
        .matched_templates
        .iter()
        .map(|template| template.import_precedence)
        .min()
        .unwrap_or(0)
        .min(0);
    apply_principal_namespace_aliases(principal, principal_root, &mut included_program)?;
    merge_included_program(&mut program, included_program, principal.location(include))?;

    let imported_shift = included_minimum_precedence.checked_sub(1).ok_or_else(|| {
        invalid(
            "FXST0037",
            "stylesheet import precedence exceeds the private integer domain",
            principal.location(import),
        )
    })?;
    rebase_imported_program(
        &mut imported_program,
        imported_shift,
        principal.location(import),
    )?;
    merge_attribute_set_declarations(
        &mut program,
        &mut imported_program,
        principal.location(import),
    );
    validate_fully_shadowed_imported_output(
        &program,
        &imported_program,
        principal.location(import),
    )?;
    materialize_principal_root_template(&mut program);
    let imported_floor = imported_program
        .matched_templates
        .iter()
        .map(|template| template.import_precedence)
        .min()
        .unwrap_or(0);
    set_principal_apply_imports_floor(&mut program, imported_floor);
    imported_program
        .matched_templates
        .append(&mut program.matched_templates);
    program.matched_templates = imported_program.matched_templates;
    merge_imported_named_templates(&mut program, imported_program.named_templates);
    merge_imported_global_bindings(&mut program, imported_program.global_bindings);
    merge_imported_character_maps(&mut program, imported_program.character_maps);
    merge_imported_decimal_formats(&mut program, imported_program.decimal_formats);
    merge_key_definitions(&mut program, imported_program.key_definitions);
    finalize_character_maps(&mut program)?;
    finalize_decimal_formats(&mut program)?;
    validate_named_template_references(&program)?;
    Ok(program)
}

fn apply_principal_namespace_aliases(
    principal: &Document,
    principal_root: NodeId,
    included: &mut StylesheetProgram,
) -> Result<(), CompileFailure> {
    let mut aliases = Vec::new();
    for declaration in meaningful_children(principal, principal_root)
        .into_iter()
        .filter(|child| is_xslt_element(principal, *child, "namespace-alias"))
    {
        namespace_alias_compiler::compile_declaration(principal, declaration, &mut aliases)?;
    }
    namespace_alias_compiler::apply(included, &aliases);
    Ok(())
}

fn namespace_aliases_at(
    document: &Document,
    root: NodeId,
) -> Result<(Vec<NodeId>, Vec<namespace_alias_compiler::NamespaceAlias>), CompileFailure> {
    let nodes = meaningful_children(document, root)
        .into_iter()
        .filter(|child| is_xslt_element(document, *child, "namespace-alias"))
        .collect::<Vec<_>>();
    let mut aliases = Vec::new();
    for declaration in &nodes {
        namespace_alias_compiler::compile_declaration(document, *declaration, &mut aliases)?;
    }
    Ok((nodes, aliases))
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
    let (_, principal_aliases) = namespace_aliases_at(principal, root)?;
    let single_import_aliases = if imported.len() == 1 && !principal_aliases.is_empty() {
        let (nodes, mut aliases) = namespace_aliases_at(imported[0].0, imported[0].1)?;
        namespace_alias_compiler::overlay_higher_precedence(&mut aliases, &principal_aliases);
        Some((nodes, aliases))
    } else {
        None
    };
    let overridden_visibility_modes = explicit_visibility_mode_names(principal, root);
    let import_count = i32::try_from(imported.len()).expect("bounded import count fits i32");
    let mut imported_programs = imported
        .iter()
        .enumerate()
        .map(|(index, (document, root))| {
            let precedence =
                i32::try_from(index).expect("bounded import index fits i32") - import_count;
            let mut excluded =
                shadowed_visibility_only_modes(document, *root, &overridden_visibility_modes);
            if index == 0 {
                if let Some((alias_nodes, _)) = &single_import_aliases {
                    excluded.extend(alias_nodes);
                }
            }
            let mut program =
                compile_imported_program_excluding(document, *root, precedence, &excluded)?;
            if index == 0 {
                if let Some((_, aliases)) = &single_import_aliases {
                    namespace_alias_compiler::apply(&mut program, aliases);
                }
            }
            Ok(program)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for program in &mut imported_programs {
        merge_attribute_set_declarations(&mut principal_program, program, principal.location(root));
    }
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

    materialize_principal_root_template(&mut principal_program);
    let imported_floor = imported_programs
        .iter()
        .flat_map(|program| &program.matched_templates)
        .map(|template| template.import_precedence)
        .min()
        .unwrap_or(0);
    set_principal_apply_imports_floor(&mut principal_program, imported_floor);
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
        merge_imported_decimal_formats(&mut principal_program, program.decimal_formats);
        merge_key_definitions(&mut principal_program, program.key_definitions);
    }
    finalize_character_maps(&mut principal_program)?;
    finalize_decimal_formats(&mut principal_program)?;
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
    for program in &mut imported_programs {
        merge_attribute_set_declarations(
            &mut principal_program,
            program,
            principal.location(principal_root),
        );
    }
    for program in &imported_programs {
        validate_fully_shadowed_imported_output(
            &principal_program,
            program,
            principal.location(principal_root),
        )?;
    }

    materialize_principal_root_template(&mut principal_program);
    let imported_floor = imported_programs
        .iter()
        .flat_map(|program| &program.matched_templates)
        .map(|template| template.import_precedence)
        .min()
        .unwrap_or(0);
    set_principal_apply_imports_floor(&mut principal_program, imported_floor);
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
        merge_imported_decimal_formats(&mut principal_program, program.decimal_formats);
        merge_key_definitions(&mut principal_program, program.key_definitions);
    }
    finalize_character_maps(&mut principal_program)?;
    finalize_decimal_formats(&mut principal_program)?;
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
    merge_attribute_set_declarations(
        &mut principal_program,
        &mut imported_program,
        principal.location(principal_root),
    );
    merge_single_imported_output(
        &mut principal_program,
        &imported_program,
        principal.location(principal_root),
    )?;

    materialize_principal_root_template(&mut principal_program);
    let imported_floor = imported_program
        .matched_templates
        .iter()
        .map(|template| template.import_precedence)
        .min()
        .unwrap_or(0);
    set_principal_apply_imports_floor(&mut principal_program, imported_floor);
    let mut matched_templates = imported_program.matched_templates;
    matched_templates.append(&mut principal_program.matched_templates);
    principal_program.matched_templates = matched_templates;
    merge_imported_named_templates(&mut principal_program, imported_program.named_templates);
    merge_imported_global_bindings(&mut principal_program, imported_program.global_bindings);
    merge_imported_character_maps(&mut principal_program, imported_program.character_maps);
    merge_imported_decimal_formats(&mut principal_program, imported_program.decimal_formats);
    merge_key_definitions(&mut principal_program, imported_program.key_definitions);
    finalize_character_maps(&mut principal_program)?;
    finalize_decimal_formats(&mut principal_program)?;
    validate_named_template_references(&principal_program)?;
    Ok(principal_program)
}

fn materialize_principal_root_template(program: &mut StylesheetProgram) {
    if program
        .root_template
        .as_ref()
        .is_none_or(|template| !contains_apply_imports(&template.body))
    {
        return;
    }
    let Some(template) = program.root_template.take() else {
        return;
    };
    program.matched_templates.push(MatchedTemplate {
        pattern: MatchPattern::Document,
        import_precedence: 0,
        apply_imports_min_precedence: 0,
        priority: TemplatePriority::ROOT_DEFAULT,
        modes: std::mem::take(&mut program.root_template_modes),
        template,
    });
}

fn set_principal_apply_imports_floor(program: &mut StylesheetProgram, floor: i32) {
    for template in program
        .matched_templates
        .iter_mut()
        .filter(|template| template.import_precedence == 0)
    {
        template.apply_imports_min_precedence = floor;
    }
}

fn contains_apply_imports(instructions: &[Instruction]) -> bool {
    instructions.iter().any(|instruction| match instruction {
        Instruction::ApplyImports { .. } => true,
        Instruction::LiteralElement { body, .. }
        | Instruction::ContextNameElement { body, .. }
        | Instruction::DynamicNameElement { body, .. }
        | Instruction::ForEachVariable { body, .. }
        | Instruction::ForEachStaticIntegerRange { body, .. }
        | Instruction::ForEachNodes { body, .. }
        | Instruction::Xslt10SequenceTreeVariable { body, .. }
        | Instruction::If { body, .. } => contains_apply_imports(body),
        Instruction::Xslt10ProcessingInstructionNode { body, .. }
        | Instruction::Xslt10CommentNode { body, .. } => contains_apply_imports(body.as_ref()),
        Instruction::Choose {
            branches,
            otherwise,
            ..
        } => {
            branches
                .iter()
                .any(|branch| contains_apply_imports(&branch.body))
                || contains_apply_imports(otherwise)
        }
        _ => false,
    })
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
    let local_import_floor = program
        .matched_templates
        .iter()
        .map(|template| template.import_precedence)
        .min()
        .unwrap_or(0);
    for template in &mut program.matched_templates {
        template.import_precedence += shift;
        template.apply_imports_min_precedence += shift;
    }
    for declaration in &mut program.attribute_set_declarations {
        declaration.import_precedence = declaration
            .import_precedence
            .checked_add(shift)
            .ok_or_else(|| {
                invalid(
                    "FXST0037",
                    "attribute-set import precedence exceeds the private integer domain",
                    location,
                )
            })?;
    }
    if let Some(template) = program.root_template.take() {
        program.matched_templates.insert(
            0,
            MatchedTemplate {
                pattern: MatchPattern::Document,
                import_precedence: shift,
                apply_imports_min_precedence: local_import_floor + shift,
                priority: TemplatePriority::ROOT_DEFAULT,
                modes: std::mem::take(&mut program.root_template_modes),
                template,
            },
        );
    }
    Ok(())
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
                apply_imports_min_precedence: import_precedence,
                priority: TemplatePriority::ROOT_DEFAULT,
                modes: std::mem::take(&mut imported_program.root_template_modes),
                template,
            },
        );
    }
    for template in &mut imported_program.matched_templates {
        template.import_precedence = import_precedence;
        template.apply_imports_min_precedence = import_precedence;
    }
    for declaration in &mut imported_program.attribute_set_declarations {
        declaration.import_precedence = import_precedence;
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
    super::compile_stylesheet_module_at_unlinked(document, root)
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

fn merge_attribute_set_declarations(
    principal: &mut StylesheetProgram,
    dependency: &mut StylesheetProgram,
    location: &SourceLocation,
) {
    let insertion_index = principal
        .attribute_set_declarations
        .iter()
        .position(|declaration| {
            declaration.location.resource == location.resource
                && declaration.location.span.start > location.span.start
        })
        .unwrap_or(principal.attribute_set_declarations.len());
    principal.attribute_set_declarations.splice(
        insertion_index..insertion_index,
        std::mem::take(&mut dependency.attribute_set_declarations),
    );
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

fn merge_imported_decimal_formats(
    principal: &mut StylesheetProgram,
    imported: Vec<crate::xslt::golden_semantics_experiment::DecimalFormatDefinition>,
) {
    for declaration in imported {
        if !principal
            .decimal_formats
            .iter()
            .any(|existing| existing.name == declaration.name)
        {
            principal.decimal_formats.push(declaration);
        }
    }
}

fn merge_included_decimal_formats(
    principal: &mut StylesheetProgram,
    included: Vec<crate::xslt::golden_semantics_experiment::DecimalFormatDefinition>,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    for declaration in included {
        if principal
            .decimal_formats
            .iter()
            .any(|existing| existing.name == declaration.name)
        {
            return Err(unsupported(
                "FXST1093",
                "same-name decimal-format composition across included modules is outside the bounded module slice",
                location,
            ));
        }
        principal.decimal_formats.push(declaration);
    }
    Ok(())
}

fn merge_key_definitions(
    principal: &mut StylesheetProgram,
    mut definitions: Vec<crate::xslt::golden_semantics_experiment::KeyDefinition>,
) {
    principal.key_definitions.append(&mut definitions);
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

pub(super) fn compile_simplified_stylesheet_at(
    document: &Document,
    root: NodeId,
) -> Result<StylesheetProgram, CompileFailure> {
    let root_name = document.name(root).expect("element nodes have names");
    if root_name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
        return Err(invalid(
            "XTSE0010",
            "a simplified stylesheet must have a literal result element as its document element",
            document.location(root),
        ));
    }
    let declared_version = required_attribute(document, root, Some(XSLT_NAMESPACE), "version")?;
    super::validate_declared_version(document, root, declared_version)?;
    let root_template = Template {
        parameters: Vec::new(),
        body: vec![compile_literal_element(document, root)?],
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
        output_same_precedence_properties: Vec::new(),
        character_maps: Vec::new(),
        decimal_formats: Vec::new(),
        output_character_map_names: Vec::new(),
        output_character_map_location: None,
        attribute_set_declarations: Vec::new(),
        key_definitions: Vec::new(),
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

pub(crate) fn validate_import_order_at(
    document: &Document,
    root: NodeId,
) -> Result<(), CompileFailure> {
    require_stylesheet_root(document, root)?;
    if optional_attribute(document, root, None, "version") != Some("1.0") {
        return Ok(());
    }
    let mut saw_non_import = false;
    for child in meaningful_children(document, root) {
        if is_xslt_element(document, child, "import") {
            if saw_non_import {
                return Err(invalid(
                    "XTSE0200",
                    "xsl:import must precede every other top-level declaration",
                    document.location(child),
                ));
            }
        } else {
            saw_non_import = true;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
    use crate::xslt::golden_semantics_experiment::{Instruction, SourceWhitespacePolicy};

    use super::compile_stylesheet_with_single_include;

    fn stylesheet(identity: &str, bytes: &[u8]) -> Document {
        let parsed = parse_document(
            identity,
            bytes,
            ParseLimits {
                max_events: 128,
                max_depth: 16,
            },
        )
        .expect("parse stylesheet");
        Document::from_parsed(parsed).expect("build stylesheet document")
    }

    #[test]
    fn composes_same_name_attribute_sets_across_includes() {
        let principal = stylesheet(
            "urn:fastxslt:attribute-set-include:principal",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="included.xsl"/><xsl:attribute-set name="common"><xsl:attribute name="a">principal</xsl:attribute></xsl:attribute-set><xsl:template match="/"><out xsl:use-attribute-sets="common"/></xsl:template></xsl:stylesheet>"#,
        );
        let included = stylesheet(
            "urn:fastxslt:attribute-set-include:included",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:attribute-set name="common"><xsl:attribute name="b">included</xsl:attribute></xsl:attribute-set></xsl:stylesheet>"#,
        );
        let root = included
            .children(included.document_node())
            .iter()
            .copied()
            .find(|node| included.name(*node).is_some())
            .expect("included stylesheet root");

        let mut program = compile_stylesheet_with_single_include(&principal, &included, root)
            .expect("same-name attribute sets should compose across includes");
        super::super::finalize_attribute_sets(&mut program)
            .expect("composed attribute sets should link");
        let template = program.root_template.expect("principal root template");
        let [
            Instruction::LiteralElement {
                computed_attributes,
                ..
            },
        ] = template.body.as_slice()
        else {
            panic!("expected one literal result element")
        };
        assert_eq!(computed_attributes.len(), 2);
        assert_eq!(computed_attributes[0].name.local, "b");
        assert_eq!(computed_attributes[1].name.local, "a");
    }

    #[test]
    fn composes_same_name_key_declarations_across_includes() {
        let principal = stylesheet(
            "urn:fastxslt:key-include:principal",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="included.xsl"/><xsl:key name="shared" match="principal" use="@code"/><xsl:template match="/"/></xsl:stylesheet>"#,
        );
        let included = stylesheet(
            "urn:fastxslt:key-include:included",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:key name="shared" match="included" use="@code"/></xsl:stylesheet>"#,
        );
        let root = included
            .children(included.document_node())
            .iter()
            .copied()
            .find(|node| included.name(*node).is_some())
            .expect("included stylesheet root");

        let program = compile_stylesheet_with_single_include(&principal, &included, root)
            .expect("same-name key declarations compose additively");

        assert_eq!(program.key_definitions.len(), 2);
        assert!(
            program
                .key_definitions
                .iter()
                .all(|definition| definition.name.local == "shared")
        );
    }

    #[test]
    fn composes_exact_strip_names_across_includes() {
        let principal = stylesheet(
            "urn:fastxslt:whitespace-include:principal",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="included.xsl"/><xsl:strip-space elements="principal"/><xsl:template match="/"/></xsl:stylesheet>"#,
        );
        let included = stylesheet(
            "urn:fastxslt:whitespace-include:included",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:strip-space elements="included"/></xsl:stylesheet>"#,
        );
        let root = included
            .children(included.document_node())
            .iter()
            .copied()
            .find(|node| included.name(*node).is_some())
            .expect("included stylesheet root");

        let program = compile_stylesheet_with_single_include(&principal, &included, root)
            .expect("exact strip names should compose additively across includes");

        assert!(matches!(
            program.source_whitespace,
            SourceWhitespacePolicy::StripExpandedNames(ref names)
                if names.iter().map(|name| name.local.as_str()).collect::<Vec<_>>()
                    == ["principal", "included"]
        ));
    }

    #[test]
    fn rejects_same_name_decimal_format_composition_across_includes() {
        let principal = stylesheet(
            "urn:fastxslt:decimal-format-include:principal",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="included.xsl"/><xsl:decimal-format decimal-separator="." grouping-separator=","/><xsl:template match="/"/></xsl:stylesheet>"#,
        );
        let included = stylesheet(
            "urn:fastxslt:decimal-format-include:included",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:decimal-format decimal-separator="," grouping-separator="."/></xsl:stylesheet>"#,
        );
        let root = included
            .children(included.document_node())
            .iter()
            .copied()
            .find(|node| included.name(*node).is_some())
            .expect("included stylesheet root");

        let failure = compile_stylesheet_with_single_include(&principal, &included, root)
            .expect_err("same-name decimal formats at one precedence remain explicit");

        assert_eq!(failure.code, "FXST1093");
        assert!(failure.detail.contains("across included modules"));
    }
}
