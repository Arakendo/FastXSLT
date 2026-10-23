//! Bounded selected-policy composition; no acquisition or relative resolution.
use super::*;

pub(crate) fn compile_selected_legacy_mixed_module_for_test(
    document: &Document,
    imported: StylesheetProgram,
    included: StylesheetProgram,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(document)?;
    let children = meaningful_children(document, root);
    let Some(edges) = children.get(..2) else {
        return Err(refusal(document, root).into());
    };
    for (&edge, kind) in edges.iter().zip(["import", "include"]) {
        if !is_xslt_element(document, edge, kind) {
            return Err(refusal(document, edge).into());
        }
        require_probe_attribute_set(document, edge, "href")?;
        required_attribute(document, edge, None, "href")?;
        ensure_no_meaningful_children(document, edge, kind)?;
    }
    let globals: Vec<_> = imported
        .global_bindings
        .iter()
        .chain(&included.global_bindings)
        .map(|binding| binding.name.clone())
        .collect();
    let principal = lower_module_with_globals(document, Edition::Legacy, control, edges, &globals)?;
    Ok(
        compiler::stylesheet_module_compiler::merge_imported_and_included_programs(
            document, root, edges[0], edges[1], principal, imported, included,
        )?,
    )
}

/// Lower one resolved import-only module; the loader owns recursion and limits.
/// Imported programs must already carry the selected compilation qualification.
pub(crate) fn compile_selected_legacy_import_module_for_test(
    document: &Document,
    mut imports: Vec<StylesheetProgram>,
    control: &mut InvocationControl,
    complete_graph: bool,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(document)?;
    let children = meaningful_children(document, root);
    if imports.len() > 3 || children.len() < imports.len() {
        return Err(refusal(document, root).into());
    }
    let edges = &children[..imports.len()];
    for &edge in edges {
        if !is_xslt_element(document, edge, "import") {
            return Err(refusal(document, edge).into());
        }
        require_probe_attribute_set(document, edge, "href")?;
        required_attribute(document, edge, None, "href")?;
        ensure_no_meaningful_children(document, edge, "xsl:import")?;
    }
    let globals: Vec<_> = imports
        .iter()
        .flat_map(|program| &program.global_bindings)
        .map(|binding| binding.name.clone())
        .collect();
    let principal = lower_module_with_globals(document, Edition::Legacy, control, edges, &globals)?;
    let mut program = match imports.len() {
        0 => principal,
        1 => compiler::stylesheet_module_compiler::assemble_single_imported_program(
            principal,
            imports.pop().expect("one imported program"),
            document.location(edges[0]),
            document.location(root),
        )?,
        _ => compiler::stylesheet_module_compiler::assemble_imported_programs_at(
            document, root, principal, imports,
        )?,
    };
    if complete_graph {
        compiler::finalize_decimal_formats(&mut program)?;
        compiler::finalize_attribute_sets(&mut program)?;
        compiler::finalize_character_maps(&mut program)?;
        compiler::validate_named_template_references(&program)?;
    }
    Ok(program)
}

/// Includes retain their declaration positions; each dependency is already lowered.
pub(crate) fn compile_selected_legacy_include_module_for_test(
    document: &Document,
    includes: Vec<StylesheetProgram>,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(document)?;
    let edges: Vec<_> = meaningful_children(document, root)
        .into_iter()
        .filter(|node| is_xslt_element(document, *node, "include"))
        .collect();
    if includes.is_empty() || includes.len() > 3 || edges.len() != includes.len() {
        return Err(refusal(document, root).into());
    }
    for &edge in &edges {
        require_probe_attribute_set(document, edge, "href")?;
        required_attribute(document, edge, None, "href")?;
        ensure_no_meaningful_children(document, edge, "xsl:include")?;
    }
    let globals: Vec<_> = includes
        .iter()
        .flat_map(|program| &program.global_bindings)
        .map(|binding| binding.name.clone())
        .collect();
    let mut program =
        lower_module_with_globals(document, Edition::Legacy, control, &edges, &globals)?;
    for (included, edge) in includes.into_iter().zip(edges) {
        compiler::stylesheet_module_compiler::merge_included_program(
            &mut program,
            included,
            document.location(edge),
        )?;
    }
    compiler::finalize_decimal_formats(&mut program)?;
    compiler::finalize_attribute_sets(&mut program)?;
    compiler::finalize_character_maps(&mut program)?;
    compiler::validate_named_template_references(&program)?;
    Ok(program)
}

/// Test-only compiler seam after the existing loader has resolved one dependency.
/// The caller owns resolution; compilation never relabels the dependency identity.
pub(crate) fn compile_selected_legacy_resolved_pair_for_test(
    principal: &Document,
    included: &Document,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(principal)?;
    let children = meaningful_children(principal, root);
    let Some(&edge) = children.first() else {
        return Err(refusal(principal, root).into());
    };
    let is_include = is_xslt_element(principal, edge, "include");
    if !is_include && !is_xslt_element(principal, edge, "import") {
        return Err(refusal(principal, edge).into());
    }
    require_probe_attribute_set(principal, edge, "href")?;
    required_attribute(principal, edge, None, "href")?;
    ensure_no_meaningful_children(principal, edge, "stylesheet dependency")?;
    // The complete pair is known before either body is lowered. Collect only
    // static names; value evaluation and precedence remain shared runtime/linker work.
    let mut globals = Vec::new();
    for document in [principal, included] {
        let module_root = compiler::document_element(document)?;
        for child in meaningful_children(document, module_root) {
            if is_xslt_element(document, child, "param")
                || is_xslt_element(document, child, "variable")
            {
                validate_selected_global_binding(document, child)?;
                globals.push(compiler::normalize_variable_qname(
                    document,
                    child,
                    required_attribute(document, child, None, "name")?,
                )?);
            }
        }
    }
    globals.sort();
    globals.dedup();
    let mut program =
        lower_module_with_globals(principal, Edition::Legacy, control, &[edge], &globals)?;
    let dependency = lower_module_with_globals(included, Edition::Legacy, control, &[], &globals)?;
    if is_include {
        compiler::stylesheet_module_compiler::merge_included_program(
            &mut program,
            dependency,
            principal.location(edge),
        )?;
    } else {
        program = compiler::stylesheet_module_compiler::assemble_single_imported_program(
            program,
            dependency,
            principal.location(edge),
            principal.location(root),
        )?;
    }
    compiler::finalize_decimal_formats(&mut program)?;
    compiler::finalize_attribute_sets(&mut program)?;
    compiler::finalize_character_maps(&mut program)?;
    compiler::validate_named_template_references(&program)?;
    Ok(program)
}

fn compile_supplied_siblings(
    principal: &Document,
    imported: &Document,
    included: &Document,
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(principal)?;
    let children = meaningful_children(principal, root);
    let Some(edges) = children.get(..2) else {
        return Err(refusal(principal, root).into());
    };
    let documents = [principal, imported, included];
    for (index, document) in documents.iter().enumerate() {
        if documents[..index].iter().any(|prior| {
            prior.location(prior.document_node()).resource
                == document.location(document.document_node()).resource
        }) {
            return Err(refusal(principal, root).into());
        }
    }
    for ((edge, kind), dependency) in edges
        .iter()
        .zip(["import", "include"])
        .zip([imported, included])
    {
        if !is_xslt_element(principal, *edge, kind) {
            return Err(refusal(principal, *edge).into());
        }
        require_probe_attribute_set(principal, *edge, "href")?;
        ensure_no_meaningful_children(principal, *edge, kind)?;
        if required_attribute(principal, *edge, None, "href")?
            != dependency.location(dependency.document_node()).resource
        {
            return Err(refusal(principal, *edge).into());
        }
    }
    let mut program = lower_module(principal, edition, control, edges)?;
    let included_program = lower_module(included, edition, control, &[])?;
    let imported_program = lower_module(imported, edition, control, &[])?;
    // Includes share principal precedence; merge them before resolving imported
    // declarations or validating names that either sibling may supply.
    compiler::stylesheet_module_compiler::merge_included_program(
        &mut program,
        included_program,
        principal.location(edges[1]),
    )?;
    Ok(
        compiler::stylesheet_module_compiler::merge_single_imported_program(
            program,
            imported_program,
            principal.location(edges[0]),
            principal.location(root),
        )?,
    )
}

// A bounded supplied chain, not a resolver or a general dependency graph.
fn compile_supplied_chain(
    documents: &[&Document],
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    for pair in documents.windows(2) {
        let root = compiler::document_element(pair[0])?;
        let children = meaningful_children(pair[0], root);
        let Some(&edge) = children.first() else {
            return Err(refusal(pair[0], root).into());
        };
        if required_attribute(pair[0], edge, None, "href")?
            != pair[1].location(pair[1].document_node()).resource
        {
            return Err(refusal(pair[0], edge).into());
        }
    }
    compile_resolved_chain(documents, edition, control)
}

/// Existing loader owns the edges; this seam retains the resolved document IDs.
pub(crate) fn compile_selected_legacy_resolved_chain_for_test(
    documents: &[&Document],
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    compile_resolved_chain(documents, Edition::Legacy, control)
}

fn compile_resolved_chain(
    documents: &[&Document],
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let Some(principal) = documents.first() else {
        unreachable!("private chain caller supplies a principal")
    };
    let root = compiler::document_element(principal)?;
    if !(2..=4).contains(&documents.len()) {
        return Err(refusal(principal, root).into());
    }
    let mut modules = Vec::new();
    let mut edges = Vec::new();
    for (index, document) in documents.iter().enumerate() {
        let root = compiler::document_element(document)?;
        let identity = &document.location(root).resource;
        if documents[..index]
            .iter()
            .any(|prior| prior.location(prior.document_node()).resource == *identity)
        {
            return Err(refusal(document, root).into());
        }
        let excluded = if index + 1 < documents.len() {
            let children = meaningful_children(document, root);
            let Some(&edge) = children.first() else {
                return Err(refusal(document, root).into());
            };
            let is_include = is_xslt_element(document, edge, "include");
            if !is_include && !is_xslt_element(document, edge, "import") {
                return Err(refusal(document, edge).into());
            }
            require_probe_attribute_set(document, edge, "href")?;
            ensure_no_meaningful_children(document, edge, "xsl:import")?;
            required_attribute(document, edge, None, "href")?;
            edges.push((
                document.location(edge).clone(),
                document.location(root).clone(),
                is_include,
            ));
            vec![edge]
        } else {
            Vec::new()
        };
        modules.push(lower_module(document, edition, control, &excluded)?);
    }
    let mut program = modules.pop().expect("bounded nonempty chain");
    for (mut parent, (edge, root, is_include)) in modules.into_iter().zip(edges).rev() {
        if is_include {
            compiler::stylesheet_module_compiler::merge_included_program(
                &mut parent,
                program,
                &edge,
            )?;
            program = parent;
        } else {
            program = compiler::stylesheet_module_compiler::assemble_single_imported_program(
                parent, program, &edge, &root,
            )?;
        }
    }
    compiler::finalize_decimal_formats(&mut program)?;
    compiler::finalize_attribute_sets(&mut program)?;
    compiler::finalize_character_maps(&mut program)?;
    compiler::validate_named_template_references(&program)?;
    Ok(program)
}

fn compile_supplied(
    principal: &Document,
    imported: &Document,
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    compile_supplied_leaves(principal, &[imported], edition, control)
}

fn compile_supplied_leaves(
    principal: &Document,
    imports: &[&Document],
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(principal)?;
    let children = meaningful_children(principal, root);
    if imports.is_empty() || imports.len() > 3 || children.len() < imports.len() {
        return Err(refusal(principal, root).into());
    }
    let edges = &children[..imports.len()];
    for (index, (&edge, imported)) in edges.iter().zip(imports).enumerate() {
        // Enforce first-position imports for every admitted version spelling.
        if !is_xslt_element(principal, edge, "import") {
            return Err(refusal(principal, edge).into());
        }
        require_probe_attribute_set(principal, edge, "href")?;
        ensure_no_meaningful_children(principal, edge, "xsl:import")?;
        let identity = &imported.location(imported.document_node()).resource;
        if required_attribute(principal, edge, None, "href")? != identity
            || principal.location(principal.document_node()).resource == *identity
            || imports[..index]
                .iter()
                .any(|prior| prior.location(prior.document_node()).resource == *identity)
        {
            return Err(refusal(principal, edge).into());
        }
    }
    // Additional and nested dependencies remain outside this leaf experiment.
    let principal_program = lower_module(principal, edition, control, edges)?;
    let imported_programs = imports
        .iter()
        .map(|document| lower_module(document, edition, control, &[]))
        .collect::<Result<Vec<_>, _>>()?;
    if imported_programs.len() == 1 {
        Ok(
            compiler::stylesheet_module_compiler::merge_single_imported_program(
                principal_program,
                imported_programs.into_iter().next().expect("one import"),
                principal.location(edges[0]),
                principal.location(root),
            )?,
        )
    } else {
        Ok(
            compiler::stylesheet_module_compiler::merge_imported_programs_at(
                principal,
                root,
                principal_program,
                imported_programs,
            )?,
        )
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod global_visibility_tests;
