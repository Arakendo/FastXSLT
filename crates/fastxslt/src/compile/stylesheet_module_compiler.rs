use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::{
    Instruction, MatchPattern, MatchedTemplate, StylesheetProgram, Template, TemplatePriority,
};

use super::instruction_compiler::{compile_sequence_excluding, literal_result_namespaces};
use super::stylesheet_validation::validate_named_template_references;
use super::{
    CompileFailure, XSLT_NAMESPACE, compile_stylesheet_excluding_unvalidated,
    default_output_settings, document_element, ensure_no_meaningful_children,
    ensure_only_attributes, invalid, is_xslt_element, meaningful_children, require_stylesheet_root,
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

pub(crate) fn discovered_stylesheet_dependencies(
    document: &Document,
) -> Result<Vec<StylesheetDependencyReference>, CompileFailure> {
    let root = document_element(document)?;
    let is_standard_stylesheet = document.name(root).is_some_and(|name| {
        name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            && matches!(name.local.as_str(), "stylesheet" | "transform")
    });
    if !is_standard_stylesheet {
        return Ok(Vec::new());
    }
    dependency_nodes(document)?
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
) -> Result<StylesheetProgram, CompileFailure> {
    let include_declarations = include_nodes(principal)?;
    let [include] = include_declarations.as_slice() else {
        return Err(invalid(
            "FXST0027",
            "single-include compilation requires exactly one xsl:include",
            principal.location(principal.document_node()),
        ));
    };
    let mut program = compile_stylesheet_excluding_unvalidated(principal, &[*include])?;
    let included_program = compile_simplified_stylesheet(included)?;

    if included_program.output != default_output_settings() {
        return Err(unsupported(
            "FXST1019",
            "included output declarations are outside the single-include slice",
            included.location(included.document_node()),
        ));
    }
    if program.root_template.is_some() && included_program.root_template.is_some() {
        return Err(unsupported(
            "FXST1020",
            "template priority across duplicate root matches is outside the single-include slice",
            included.location(included.document_node()),
        ));
    }
    if program.root_template.is_none() {
        program.root_template = included_program.root_template;
        program.root_template_modes = included_program.root_template_modes;
    }
    for matched in included_program.matched_templates {
        if program
            .matched_templates
            .iter()
            .any(|existing| existing.pattern == matched.pattern && existing.modes == matched.modes)
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
                included.location(included.document_node()),
            ));
        }
        program.global_bindings.push(binding);
    }
    validate_named_template_references(&program)?;
    Ok(program)
}

pub(crate) fn compile_stylesheet_with_imports(
    principal: &Document,
    imported: &[&Document],
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
    let children = meaningful_children(principal, root);
    if children
        .iter()
        .take(import_declarations.len())
        .ne(import_declarations.iter())
    {
        return Err(invalid(
            "XTSE0200",
            "xsl:import must precede every other top-level declaration",
            principal.location(import_declarations[0]),
        ));
    }
    let mut principal_program =
        compile_stylesheet_excluding_unvalidated(principal, &import_declarations)?;
    let import_count = i32::try_from(imported.len()).expect("bounded import count fits i32");
    let mut imported_programs = imported
        .iter()
        .enumerate()
        .map(|(index, document)| {
            let precedence =
                i32::try_from(index).expect("bounded import index fits i32") - import_count;
            compile_imported_program(document, precedence)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut matched_templates = Vec::new();
    for program in &mut imported_programs {
        matched_templates.append(&mut program.matched_templates);
    }
    matched_templates.append(&mut principal_program.matched_templates);
    principal_program.matched_templates = matched_templates;
    for program in imported_programs.into_iter().rev() {
        merge_imported_named_templates(&mut principal_program, program.named_templates)?;
        merge_imported_global_bindings(&mut principal_program, program.global_bindings);
    }
    validate_named_template_references(&principal_program)?;
    Ok(principal_program)
}

fn compile_imported_program(
    imported: &Document,
    import_precedence: i32,
) -> Result<StylesheetProgram, CompileFailure> {
    let mut imported_program = compile_imported_module(imported)?;
    if imported_program.output != default_output_settings() {
        return Err(unsupported(
            "FXST1024",
            "imported output declarations are outside the bounded import slice",
            imported.location(imported.document_node()),
        ));
    }
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

fn compile_imported_module(document: &Document) -> Result<StylesheetProgram, CompileFailure> {
    let root = document_element(document)?;
    if document
        .name(root)
        .is_some_and(|name| name.namespace.as_deref() == Some(XSLT_NAMESPACE))
    {
        super::compile_stylesheet(document)
    } else {
        compile_simplified_stylesheet(document)
    }
}

fn merge_imported_named_templates(
    principal: &mut StylesheetProgram,
    imported: Vec<crate::xslt::golden_semantics_experiment::NamedTemplate>,
) -> Result<(), CompileFailure> {
    for template in imported {
        if principal
            .named_templates
            .iter()
            .any(|existing| existing.name == template.name)
        {
            return Err(unsupported(
                "FXST1026",
                "duplicate named templates across import precedence are outside the bounded import slice",
                &template.template.location,
            ));
        }
        principal.named_templates.push(template);
    }
    Ok(())
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

fn compile_simplified_stylesheet(document: &Document) -> Result<StylesheetProgram, CompileFailure> {
    let root = document_element(document)?;
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
        output: default_output_settings(),
        root_template: Some(root_template),
        root_template_modes: Vec::new(),
        matched_templates: Vec::new(),
        named_templates: Vec::new(),
        global_bindings: Vec::new(),
    })
}

fn include_nodes(document: &Document) -> Result<Vec<NodeId>, CompileFailure> {
    let root = document_element(document)?;
    require_stylesheet_root(document, root)?;
    Ok(meaningful_children(document, root)
        .into_iter()
        .filter(|child| is_xslt_element(document, *child, "include"))
        .collect())
}

fn import_nodes(document: &Document) -> Result<Vec<NodeId>, CompileFailure> {
    let root = document_element(document)?;
    require_stylesheet_root(document, root)?;
    Ok(meaningful_children(document, root)
        .into_iter()
        .filter(|child| is_xslt_element(document, *child, "import"))
        .collect())
}

fn dependency_nodes(document: &Document) -> Result<Vec<NodeId>, CompileFailure> {
    let root = document_element(document)?;
    require_stylesheet_root(document, root)?;
    Ok(meaningful_children(document, root)
        .into_iter()
        .filter(|child| {
            is_xslt_element(document, *child, "include")
                || is_xslt_element(document, *child, "import")
        })
        .collect())
}
