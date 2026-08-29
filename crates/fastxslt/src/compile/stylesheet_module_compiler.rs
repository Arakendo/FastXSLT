use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::{Instruction, StylesheetProgram, Template};

use super::instruction_compiler::{compile_sequence_excluding, literal_result_namespaces};
use super::stylesheet_validation::validate_named_template_references;
use super::{
    CompileFailure, XSLT_NAMESPACE, compile_stylesheet_excluding, default_output_settings,
    document_element, ensure_no_meaningful_children, ensure_only_attributes, invalid,
    is_xslt_element, meaningful_children, require_stylesheet_root, required_attribute, unsupported,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IncludeReference {
    pub(crate) href: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn single_include_reference(
    document: &Document,
) -> Result<Option<IncludeReference>, CompileFailure> {
    let include_declarations = include_nodes(document)?;
    let Some(include) = include_declarations.first().copied() else {
        return Ok(None);
    };
    if include_declarations.len() != 1 {
        return Err(unsupported(
            "FXST1018",
            "the private slice permits one xsl:include dependency",
            document.location(include),
        ));
    }
    ensure_only_attributes(document, include, &["href"], "xsl:include")?;
    ensure_no_meaningful_children(document, include, "xsl:include")?;
    Ok(Some(IncludeReference {
        href: required_attribute(document, include, None, "href")?.to_owned(),
        location: document.location(include).clone(),
    }))
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
    let mut program = compile_stylesheet_excluding(principal, &[*include])?;
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
