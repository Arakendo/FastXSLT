//! AR-0030 single-module declaration/link probe; never production admission.
//! Reuses declaration checks, ordinary top-level compilation, body lowering and
//! named-call validation. Only untyped content globals in an explicit 1.0 module
//! are admitted. One explicitly supplied include or import can be linked. No
//! transitive graphs, aliases, attribute sets, acquisition or public plan contract.

use super::*;

pub(crate) struct CompiledReference {
    pub(crate) program: StylesheetProgram,
    pub(crate) bodies: BTreeMap<String, Vec<Instruction>>,
}

pub(crate) fn compile(document: &Document) -> Result<CompiledReference, CompileFailure> {
    let mut compiled = compile_unlinked(document, false)?;
    finalize(&mut compiled)?;
    Ok(compiled)
}

/// The caller supplies both effective documents under one compilation profile.
/// This function never resolves or acquires the dependency's href.
pub(crate) fn compile_with_single_include(
    principal: &Document,
    included: &Document,
    context: stylesheet_preprocessing::StylesheetCompilationContext,
) -> Result<CompiledReference, CompileFailure> {
    let root = document_element(principal)?;
    context.require_declaration_driven_lowering(principal.location(root))?;
    let dependencies =
        stylesheet_module_compiler::discovered_stylesheet_dependencies_at(principal, root)?;
    let [dependency] = dependencies.as_slice() else {
        return Err(unsupported(
            "FXAR0030",
            "reference requires exactly one supplied include",
            principal.location(root),
        ));
    };
    if dependency.kind != StylesheetDependencyKind::Include {
        return Err(unsupported(
            "FXAR0030",
            "single-include reference requires an include dependency",
            &dependency.location,
        ));
    }
    let mut compiled = compile_unlinked(principal, true)?;
    let dependency_module = compile_unlinked(included, false)?;
    compiled.program = context.qualify_module(compiled.program);
    let included_program = context.qualify_module(dependency_module.program);
    stylesheet_module_compiler::merge_included_program(
        &mut compiled.program,
        included_program,
        &dependency.location,
    )?;
    // Equal-precedence collisions were rejected by the shared linker above.
    for (name, body) in dependency_module.bodies {
        assert!(compiled.bodies.insert(name, body).is_none());
    }
    finalize(&mut compiled)?;
    Ok(compiled)
}

fn compile_unlinked(
    document: &Document,
    allow_dependency: bool,
) -> Result<CompiledReference, CompileFailure> {
    let root = document_element(document)?;
    require_stylesheet_root(document, root)?;
    if optional_attribute(document, root, None, "version") != Some("1.0") {
        return Err(unsupported(
            "FXAR0030",
            "global declaration reference requires version 1.0",
            document.location(root),
        ));
    }
    let mut declarations = Vec::new();
    let mut bindings = Vec::new();
    let mut bodies = BTreeMap::new();
    for &child in document.children(root) {
        if document
            .name(child)
            .is_none_or(|name| name.namespace.as_deref() != Some(XSLT_NAMESPACE))
        {
            continue;
        }
        let local = &document.name(child).unwrap().local;
        if allow_dependency && matches!(local.as_str(), "include" | "import") {
            declarations.push(child);
            continue;
        }
        if matches!(
            local.as_str(),
            "include" | "import" | "namespace-alias" | "attribute-set"
        ) {
            return Err(unsupported(
                "FXAR0030",
                "module composition and constructor rewrites are outside the declaration reference",
                document.location(child),
            ));
        }
        if !matches!(local.as_str(), "variable" | "param") {
            continue;
        }
        let kind = if local == "param" {
            GlobalBindingKind::Parameter
        } else {
            GlobalBindingKind::Variable
        };
        let mut body = Vec::new();
        let binding = compile_global_binding_with(
            document,
            child,
            kind,
            |document, element, label| {
                if optional_attribute(document, element, None, "select").is_some() {
                    ensure_no_meaningful_children(document, element, label)?;
                    return Err(unsupported(
                        "FXAR0030",
                        "select defaults are outside the declaration reference",
                        document.location(element),
                    ));
                }
                if optional_attribute(document, element, None, "as").is_some()
                    || optional_attribute(document, element, None, "required").is_some()
                {
                    return Err(unsupported(
                        "FXAR0030",
                        "typed/required globals are outside the declaration reference",
                        document.location(element),
                    ));
                }
                body = compile_global_constructor_reference(document, element)?;
                if body.is_empty() {
                    return Err(unsupported(
                        "FXAR0030",
                        "empty defaults require scalar-string semantics outside the declaration reference",
                        document.location(element),
                    ));
                }
                // Private placeholder used only to carry shared binding metadata.
                // The reference driver always reads the corresponding body map.
                Ok(GlobalBindingDefault::TemporaryText(String::new()))
            },
        )?;
        if bodies.contains_key(&binding.name) {
            return Err(invalid(
                "FXST0022",
                format!("duplicate global binding: ${}", binding.name),
                document.location(child),
            ));
        }
        bodies.insert(binding.name.clone(), body);
        bindings.push(binding);
        declarations.push(child);
    }
    let mut program = compile_stylesheet_at_excluding_unvalidated(document, root, &declarations)?;
    program.global_bindings = bindings;
    Ok(CompiledReference { program, bodies })
}

pub(crate) fn compile_with_single_import(
    principal: &Document,
    imported: &Document,
    context: stylesheet_preprocessing::StylesheetCompilationContext,
) -> Result<CompiledReference, CompileFailure> {
    let root = document_element(principal)?;
    context.require_declaration_driven_lowering(principal.location(root))?;
    stylesheet_module_compiler::validate_import_order_at(principal, root)?;
    let dependencies =
        stylesheet_module_compiler::discovered_stylesheet_dependencies_at(principal, root)?;
    if dependencies.len() != 1 || dependencies[0].kind != StylesheetDependencyKind::Import {
        return Err(unsupported(
            "FXAR0030",
            "reference requires exactly one supplied import",
            principal.location(root),
        ));
    }
    let mut principal_module = compile_unlinked(principal, true)?;
    let mut imported_module = compile_unlinked(imported, false)?;
    let program = stylesheet_module_compiler::merge_single_imported_program(
        context.qualify_module(principal_module.program),
        context.qualify_module(imported_module.program),
        &dependencies[0].location,
        principal.location(root),
    )?;
    let mut bodies = BTreeMap::new();
    for binding in &program.global_bindings {
        // This bounded graph has exactly two precedence levels. Choose from the
        // declaration selected by the shared linker, never a last-write name map
        // or a URI/span-as-identity heuristic. Unselected bodies are dropped.
        let candidates = match binding.import_precedence {
            0 => &mut principal_module.bodies,
            -1 => &mut imported_module.bodies,
            _ => unreachable!("single-import reference precedence"),
        };
        bodies.insert(
            binding.name.clone(),
            candidates
                .remove(&binding.name)
                .expect("selected declaration has a compiled body"),
        );
    }
    let mut compiled = CompiledReference { program, bodies };
    finalize(&mut compiled)?;
    Ok(compiled)
}

fn finalize(compiled: &mut CompiledReference) -> Result<(), CompileFailure> {
    finalize_decimal_formats(&mut compiled.program)?;
    finalize_attribute_sets(&mut compiled.program)?;
    finalize_character_maps(&mut compiled.program)?;
    validate_named_template_references(&compiled.program)?;
    for body in compiled.bodies.values() {
        stylesheet_validation::validate_named_calls(&compiled.program, body)?;
    }
    Ok(())
}
