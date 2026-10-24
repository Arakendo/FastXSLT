//! ADR-0025 matched-template integration checkpoint. Private until profile gates
//! pass; ordinary entry points retain their existing semantics. Uses shared
//! declaration, sequence, instruction, fallback and result construction owners.
#[cfg(test)]
use super::*;
#[cfg(not(test))]
use super::{
    BooleanExpression, CompileFailure, DeclaredVersion, Document, Edition, Instruction, NodeId,
    NodeKind, SourceLocation, ValueExpression, XSLT_NAMESPACE, boolean_expression_compiler,
    compile_apply_imports, compile_apply_templates, compile_copy_of,
    compile_forward_fallbacks_with_body, compile_if_with_lowerers, compile_literal_element_parts,
    compile_local_variable_name, compile_message_with_body, compile_select_variable,
    compile_sequence_with_element_lowerer, compile_static_atomic_variable, compile_text,
    compile_text_after_attribute_validation, compile_text_tree_variable_with_mode,
    compile_value_of_with_validated_options, compiler, computed_element_compiler,
    conditional_instruction_compiler, ensure_fallback_attributes, ensure_no_meaningful_children,
    has_local_version, invalid, is_ascii_ncname, is_xslt_element, is_xslt10_template_element,
    literal_attribute_compiler, meaningful_children, module_version, number_compiler,
    optional_attribute, parse_literal_document_call, probe_declared_version,
    require_probe_attribute_set, required_attribute, template_invocation_compiler, unsupported,
    validate_exclude_result_prefixes, value_expression_compiler, xpath_string_literal,
    xslt10_static_introspection_compiler,
};
use crate::execution_control_experiment::InvocationControl;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use compiler::stylesheet_preprocessing::StrictPreprocessingFailure;
#[cfg(test)]
use compiler::stylesheet_preprocessing::stylesheet_visibility_view;

#[path = "boolean_call_timing.rs"]
mod boolean_call_timing;

#[path = "conditional_integration.rs"]
mod conditional_integration;

#[path = "selected_introspection.rs"]
mod selected_introspection;

#[path = "keys/mod.rs"]
mod selected_keys;

#[path = "template_arguments/mod.rs"]
mod template_arguments;

#[path = "module_graph/mod.rs"]
mod module_graph;
pub(crate) use module_graph::compile_selected_legacy_import_module;
pub(crate) use module_graph::compile_selected_legacy_include_module;
pub(crate) use module_graph::compile_selected_legacy_mixed_module;
#[cfg(test)]
pub(crate) use module_graph::compile_selected_legacy_resolved_chain_for_test;
pub(crate) use module_graph::compile_selected_legacy_resolved_pair;

#[derive(Clone, Copy)]
enum LocalVersion {
    LegacyNormal,
    LegacyForward,
    ModernWitness(DeclaredVersion),
}

#[cfg(test)]
fn compile(
    document: &Document,
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let program = lower_module(document, edition, control, &[])?;
    compiler::validate_named_template_references(&program)?;
    Ok(program)
}

// Only a validated supplied dependency edge may be excluded. Named references
// are checked after linking, so imported declarations can satisfy principal reads.
#[cfg(test)]
fn lower_module(
    document: &Document,
    edition: Edition,
    control: &mut InvocationControl,
    dependencies: &[NodeId],
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    lower_module_with_globals(document, edition, control, dependencies, &[])
}

fn lower_module_with_globals(
    document: &Document,
    edition: Edition,
    control: &mut InvocationControl,
    dependencies: &[NodeId],
    dependency_globals: &[String],
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(document)?;
    let (version, ignored_top_level) =
        validate_selected_module(document, root, edition, dependencies)?;
    lower_validated_module(
        document,
        root,
        edition,
        version,
        &ignored_top_level,
        dependency_globals,
        control,
    )
}

fn validate_selected_module(
    document: &Document,
    root: NodeId,
    edition: Edition,
    dependencies: &[NodeId],
) -> Result<(LocalVersion, Vec<NodeId>), StrictPreprocessingFailure> {
    let version = if matches!(edition, Edition::Legacy) {
        compiler::require_stylesheet_root(document, root)?;
        for attribute in document.attributes(root) {
            let name = document.name(*attribute).expect("attribute name");
            if name.namespace.is_some()
                || !matches!(
                    name.local.as_str(),
                    "version" | "id" | "exclude-result-prefixes"
                )
            {
                return Err(refusal(document, *attribute).into());
            }
        }
        if optional_attribute(document, root, None, "exclude-result-prefixes")
            .is_some_and(|list| !list.split_whitespace().all(is_ascii_ncname))
        {
            return Err(refusal(document, root).into());
        }
        validate_exclude_result_prefixes(document, root)?;
        legacy_numeric_version(
            required_attribute(document, root, None, "version")?,
            document.location(root),
        )?
    } else {
        LocalVersion::ModernWitness(module_version(document, root)?)
    };
    let mut ignored_top_level = dependencies.to_vec();
    // Defaults, globals and modules must not enter an unprofiled lowering route.
    for child in meaningful_children(document, root) {
        if dependencies.contains(&child) {
            continue;
        }
        if is_xslt_element(document, child, "template") {
            validate_template_attributes(document, child, version)?;
            for part in optional_attribute(document, child, None, "match")
                .into_iter()
                .flat_map(|pattern| pattern.split('|').map(str::trim))
            {
                let (compiled, _) = compiler::compile_match_pattern(document, child, part)?;
                if !selected_pattern_admitted(part, &compiled, edition) {
                    return Err(refusal(document, child).into());
                }
            }
        } else if matches!(edition, Edition::Legacy)
            && (is_xslt_element(document, child, "strip-space")
                || is_xslt_element(document, child, "preserve-space"))
        {
            validate_selected_source_whitespace(document, child)?;
        } else if matches!(edition, Edition::Legacy) && is_xslt_element(document, child, "key") {
            selected_keys::validate_declaration(document, child)?;
        } else if is_xslt_element(document, child, "output") {
            validate_output_attributes(document, child, version)?;
        } else if matches!(edition, Edition::Legacy)
            && is_xslt_element(document, child, "attribute-set")
        {
            validate_static_attribute_set(document, child)?;
        } else if matches!(edition, Edition::Legacy)
            && (is_xslt_element(document, child, "param")
                || is_xslt_element(document, child, "variable"))
        {
            validate_selected_global_binding(document, child)?;
        } else if matches!(edition, Edition::Legacy)
            && document.name(child).is_some_and(|name| {
                name.namespace.as_deref() == Some(XSLT_NAMESPACE)
                    && !is_legacy_top_level_declaration(&name.local)
            })
        {
            if matches!(version, LocalVersion::LegacyNormal) {
                return Err(invalid(
                    "FXST1002",
                    "element is not an XSLT 1.0 top-level declaration",
                    document.location(child),
                )
                .into());
            }
            ignored_top_level.push(child);
        } else {
            return Err(refusal(document, child).into());
        }
    }
    Ok((version, ignored_top_level))
}

fn lower_validated_module(
    document: &Document,
    root: NodeId,
    edition: Edition,
    version: LocalVersion,
    ignored_top_level: &[NodeId],
    dependency_globals: &[String],
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    #[cfg(test)]
    let preprocessing = match edition {
        Edition::Legacy => StylesheetPreprocessingProfile::StrictXslt10,
        #[cfg(test)]
        Edition::Modern => StylesheetPreprocessingProfile::Modern,
    };
    let view = match edition {
        Edition::Legacy => {
            compiler::stylesheet_preprocessing::strict_content::prepare(document, root, control)?
        }
        #[cfg(test)]
        Edition::Modern => stylesheet_visibility_view(document, root, preprocessing, control)?,
    };
    let mut globals = dependency_globals.to_vec();
    for child in meaningful_children(&view, root) {
        if is_xslt_element(&view, child, "param") || is_xslt_element(&view, child, "variable") {
            globals.push(compiler::normalize_variable_qname(
                &view,
                child,
                required_attribute(&view, child, None, "name")?,
            )?);
        }
    }
    globals.sort();
    globals.dedup();
    let program = compiler::compile_stylesheet_with_declaration_lowerers(
        &view,
        root,
        ignored_top_level,
        &mut |document, root, lexical| match edition {
            Edition::Legacy => legacy_numeric_version(lexical, document.location(root)).map(|_| ()),
            #[cfg(test)]
            Edition::Modern => compiler::validate_declared_version(document, root, lexical),
        },
        &mut |document, template| {
            lower_selected_template(document, template, edition, version, &globals)
        },
        &mut |document, element, declared, superseded| {
            lower_output_declaration(document, element, declared, superseded, version)
        },
        &mut |document, element, name| {
            compiler::compile_named_template_with_body_and_validation(
                document,
                element,
                name,
                &mut |document, element| {
                    validate_template_attributes(document, element, version)?;
                    validate_literal_parameters(document, element, edition)
                },
                &mut |document, parent, excluded, bindings| {
                    body_excluding(
                        document,
                        parent,
                        edition,
                        version,
                        0,
                        StaticBindings {
                            locals: bindings,
                            globals: &globals,
                        },
                        excluded,
                    )
                },
            )
        },
        &mut lower_selected_global,
    )?;
    let context = match edition {
        Edition::Legacy => {
            compiler::stylesheet_preprocessing::StylesheetCompilationContext::selected_legacy()
        }
        #[cfg(test)]
        Edition::Modern => compilation_context(edition, preprocessing),
    };
    let mut program = context.qualify_module(program);
    compiler::finalize_decimal_formats(&mut program)?;
    compiler::finalize_attribute_sets(&mut program)?;
    compiler::finalize_character_maps(&mut program)?;
    Ok(program)
}

fn validate_selected_source_whitespace(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    require_probe_attribute_set(document, element, "elements")?;
    let names = required_attribute(document, element, None, "elements")?;
    if names.split_whitespace().next().is_none()
        || !names.split_whitespace().all(|name| {
            name == "*"
                || is_ascii_ncname(name)
                || name.split_once(':').is_some_and(|(prefix, local)| {
                    is_ascii_ncname(prefix) && (local == "*" || is_ascii_ncname(local))
                })
        })
    {
        return Err(refusal(document, element));
    }
    ensure_no_meaningful_children(document, element, "source whitespace declaration")
}

fn literal_document_global(document: &Document, element: NodeId, select: &str) -> bool {
    is_xslt_element(document, element, "variable")
        && matches!(parse_literal_document_call(select.trim()), Some((reference, None, tail)) if !reference.is_empty() && tail.is_empty())
}

fn lower_selected_global(
    document: &Document,
    element: NodeId,
    kind: compiler::GlobalBindingKind,
) -> Result<compiler::GlobalBinding, CompileFailure> {
    compiler::compile_global_binding_with(document, element, kind, |document, element, label| {
        if let Some(select) = optional_attribute(document, element, None, "select")
            && literal_document_global(document, element, select)
        {
            return compiler::compile_literal_document_global(document, element, select)?
                .ok_or_else(|| refusal(document, element));
        }
        compiler::compile_global_default(document, element, label)
    })
}

// Selected globals admit constant strings and one literal document root only.
// Reads remain limited by the separately guarded selected expressions.
fn validate_selected_global_binding(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some() || !matches!(name.local.as_str(), "name" | "select") {
            return Err(refusal(document, *attribute));
        }
    }
    let Some(select) = optional_attribute(document, element, None, "select") else {
        return Err(refusal(document, element));
    };
    if literal_document_global(document, element, select) {
        return ensure_no_meaningful_children(document, element, "global binding");
    }
    let string_argument = select
        .strip_prefix("string(")
        .and_then(|value| value.strip_suffix(')'));
    // The shared direct-global literal path currently accepts single quotes;
    // its string() folder accepts either delimiter. Do not overstate this guard.
    if xpath_string_literal(string_argument.unwrap_or(select)).is_none()
        || string_argument.is_none() && !select.starts_with('\'')
    {
        return Err(refusal(document, element));
    }
    ensure_no_meaningful_children(document, element, "global binding")
}

fn validate_static_attribute_set(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    require_probe_attribute_set(document, element, "name")?;
    if !is_ascii_ncname(required_attribute(document, element, None, "name")?) {
        return Err(refusal(document, element));
    }
    for child in meaningful_children(document, element) {
        if !is_xslt_element(document, child, "attribute") {
            return Err(refusal(document, child));
        }
        require_probe_attribute_set(document, child, "name")?;
        if !is_ascii_ncname(required_attribute(document, child, None, "name")?)
            || meaningful_children(document, child)
                .iter()
                .any(|node| document.kind(*node) != NodeKind::Text)
        {
            return Err(refusal(document, child));
        }
    }
    Ok(())
}

fn selected_pattern_admitted(
    lexical: &str,
    pattern: &crate::xslt::golden_semantics_experiment::MatchPattern,
    edition: Edition,
) -> bool {
    use crate::xslt::golden_semantics_experiment::MatchPattern;
    if matches!(lexical, "/" | "text()")
        || is_ascii_ncname(lexical)
        || matches!(pattern, MatchPattern::ElementWithAttributeValue { .. })
    {
        return true;
    }
    if !matches!(edition, Edition::Legacy) {
        return false;
    }
    lexical == "*"
        || lexical
            .strip_prefix('/')
            .unwrap_or(lexical)
            .split('/')
            .all(is_ascii_ncname)
        || lexical
            .strip_prefix('/')
            .and_then(|path| path.strip_suffix("/*"))
            .is_some_and(|path| path.split('/').all(is_ascii_ncname))
        || lexical.strip_prefix("//").is_some_and(is_ascii_ncname)
        || matches!(
            pattern,
            MatchPattern::ElementWithAttribute { .. } | MatchPattern::AnyElementNumberEquals(_)
        )
        || matches!(pattern, MatchPattern::AnyElementBooleanPredicate(predicate) if selected_match_boolean_admitted(predicate))
}

fn selected_match_boolean_admitted(
    predicate: &crate::xslt::golden_semantics_experiment::MatchBooleanPredicate,
) -> bool {
    // These shared typed leaves have no declared-version-dependent lowering.
    // Do not equate all implemented predicate variants with selected admission.
    use crate::xslt::golden_semantics_experiment::MatchBooleanPredicate as P;
    match predicate {
        P::ContextNumberEquals(_) | P::Position { .. } | P::AttributeEquals { .. } => true,
        P::And(left, right) | P::Or(left, right) => {
            selected_match_boolean_admitted(left) && selected_match_boolean_admitted(right)
        }
        P::Not(inner) => selected_match_boolean_admitted(inner),
        P::AttributeNumber { .. } => false,
    }
}

fn lower_selected_template(
    document: &Document,
    template: NodeId,
    edition: Edition,
    version: LocalVersion,
    globals: &[String],
) -> Result<crate::xslt::golden_semantics_experiment::Template, CompileFailure> {
    validate_literal_parameters(document, template, edition)?;
    compiler::compile_template_with_body_and_validation(
        document,
        template,
        &mut |document, element| validate_template_attributes(document, element, version),
        &mut |document, parent, excluded, bindings| {
            body_excluding(
                document,
                parent,
                edition,
                version,
                0,
                StaticBindings {
                    locals: bindings,
                    globals,
                },
                excluded,
            )
        },
    )
}

fn validate_literal_parameters(
    document: &Document,
    element: NodeId,
    edition: Edition,
) -> Result<(), CompileFailure> {
    // Named and matched declarations retain their respective shared validation
    // owners. This guard only establishes policy-independent default eligibility.
    for child in meaningful_children(document, element) {
        if !is_xslt_element(document, child, "param") {
            continue;
        }
        if !matches!(edition, Edition::Legacy) {
            return Err(refusal(document, child));
        }
        for attribute in document.attributes(child) {
            let name = document.name(*attribute).expect("attribute");
            if name.namespace.is_some() || !matches!(name.local.as_str(), "name" | "select") {
                return Err(refusal(document, *attribute));
            }
        }
        if let Some(select) = optional_attribute(document, child, None, "select") {
            if xpath_string_literal(select).is_none() {
                return Err(refusal(document, child));
            }
            ensure_no_meaningful_children(document, child, "xsl:param")?;
        }
        if meaningful_children(document, child)
            .iter()
            .any(|node| document.kind(*node) != NodeKind::Text)
        {
            return Err(refusal(document, child));
        }
    }
    Ok(())
}

fn lower_output_declaration(
    document: &Document,
    element: NodeId,
    declared: &str,
    superseded: bool,
    version: LocalVersion,
) -> Result<compiler::output_compiler::OutputDeclaration, CompileFailure> {
    match version {
        LocalVersion::ModernWitness(_) => {
            compiler::output_compiler::compile_output(document, element, declared, superseded)
        }
        LocalVersion::LegacyNormal | LocalVersion::LegacyForward => {
            compiler::output_compiler::compile_output_with_boolean_policy(
                document,
                element,
                declared,
                superseded,
                compiler::output_compiler::OutputBooleanPolicy::selected_legacy(matches!(
                    version,
                    LocalVersion::LegacyForward
                )),
                &mut |document, element| validate_output_attributes(document, element, version),
            )
        }
    }
}

// XSLT 1.0 section 2.2 recognition, not the shared compiler's capability list.
// Recognized but not yet policy-integrated declarations must still be refused;
// newer declarations must not reach modern lowering through a legacy program.
fn is_legacy_top_level_declaration(local: &str) -> bool {
    matches!(
        local,
        "import"
            | "include"
            | "strip-space"
            | "preserve-space"
            | "output"
            | "key"
            | "decimal-format"
            | "namespace-alias"
            | "attribute-set"
            | "variable"
            | "param"
            | "template"
    )
}

fn refusal(document: &Document, node: NodeId) -> CompileFailure {
    unsupported(
        "FXAR0031",
        "processor policy not yet propagated to this compilation shape",
        document.location(node),
    )
}

fn validate_template_attributes(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some() {
            return Err(refusal(document, *attribute));
        }
        if name.local == "match"
            || name.local == "name" && !matches!(version, LocalVersion::ModernWitness(_))
        {
            continue;
        }
        if !matches!(version, LocalVersion::ModernWitness(_))
            && optional_attribute(document, element, None, "match").is_some()
        {
            let value = document.value(*attribute).unwrap_or_default();
            if name.local == "mode" && is_ascii_ncname(value) {
                // A single NCName has identical meaning in both shared mode readers.
                continue;
            }
            if name.local == "priority" && value.trim().parse::<i32>().is_ok() {
                // Exact signed-32-bit integers agree in legacy binary64 and modern
                // decimal priority domains. Fractional/large priorities stay guarded.
                continue;
            }
        }
        // Shared declaration/static-context readers still consume these fields.
        // Do not call them ignored until selected policy reaches those readers.
        if matches!(
            name.local.as_str(),
            "name"
                | "mode"
                | "priority"
                | "xpath-default-namespace"
                | "default-mode"
                | "default-collation"
                | "exclude-result-prefixes"
                | "version"
        ) {
            return Err(refusal(document, *attribute));
        }
        match version {
            LocalVersion::LegacyForward => {}
            LocalVersion::LegacyNormal => {
                return Err(invalid(
                    "XTSE0090",
                    "unknown template attribute outside forward compatibility",
                    document.location(*attribute),
                ));
            }
            LocalVersion::ModernWitness(_) => return Err(refusal(document, *attribute)),
        }
    }
    Ok(())
}

fn validate_output_attributes(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
) -> Result<(), CompileFailure> {
    for attr in document.attributes(element) {
        let name = document.name(*attr).expect("attribute name");
        if name.namespace.is_some() {
            return Err(refusal(document, *attr));
        }
        if !matches!(
            name.local.as_str(),
            "method" | "indent" | "omit-xml-declaration" | "encoding"
        ) {
            if !compiler::output_compiler::is_known_output_attribute(&name.local) {
                match version {
                    LocalVersion::LegacyForward => continue,
                    LocalVersion::LegacyNormal => {
                        return Err(invalid(
                            "XTSE0090",
                            "unknown output attribute outside forward compatibility",
                            document.location(*attr),
                        ));
                    }
                    LocalVersion::ModernWitness(_) => {}
                }
            }
            return Err(refusal(document, *attr));
        }
        if name.local == "encoding"
            && !document.value(*attr).is_some_and(|v| {
                v.eq_ignore_ascii_case("UTF-8")
                    || v.eq_ignore_ascii_case("ISO-8859-1")
                    || v.eq_ignore_ascii_case("UTF-16")
            })
        {
            return Err(refusal(document, *attr));
        }
        if matches!(
            version,
            LocalVersion::LegacyNormal | LocalVersion::LegacyForward
        ) && name.local == "method"
            && !document
                .value(*attr)
                .is_some_and(|value| matches!(value.trim(), "xml" | "html" | "text"))
        {
            return Err(refusal(document, *attr));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct StaticBindings<'a> {
    locals: &'a [String],
    globals: &'a [String],
}

impl StaticBindings<'_> {
    fn visible(self) -> Vec<String> {
        self.locals.iter().chain(self.globals).cloned().collect()
    }
}

fn body(
    document: &Document,
    parent: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    inherited: StaticBindings<'_>,
) -> Result<Vec<Instruction>, CompileFailure> {
    body_excluding(document, parent, edition, version, depth, inherited, &[])
}

fn body_excluding(
    document: &Document,
    parent: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    inherited: StaticBindings<'_>,
    excluded: &[NodeId],
) -> Result<Vec<Instruction>, CompileFailure> {
    if depth >= 16 {
        return Err(refusal(document, parent));
    }
    let version = scoped_version(document, parent, edition, version)?;
    compile_sequence_with_element_lowerer(
        document,
        parent,
        excluded,
        inherited.locals,
        &mut |document, child, bindings, out| {
            lower_sequence_element(
                document,
                child,
                edition,
                version,
                depth,
                bindings,
                inherited.globals,
                out,
            )
        },
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "element lowering keeps mutable lexical bindings distinct from immutable globals and policy"
)]
fn lower_sequence_element(
    document: &Document,
    child: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    bindings: &mut Vec<String>,
    globals: &[String],
    out: &mut Vec<Instruction>,
) -> Result<(), CompileFailure> {
    let scope = StaticBindings {
        locals: bindings,
        globals,
    };
    let visible = scope.visible();
    let child_version = scoped_version(document, child, edition, version)?;
    let name = document.name(child).expect("sequence element");
    if name.namespace.as_deref() != Some(XSLT_NAMESPACE) {
        out.push(selected_literal(
            document, child, edition, version, depth, scope,
        )?);
    } else if matches!(edition, Edition::Legacy) && !is_xslt10_template_element(&name.local) {
        out.extend(selected_unknown_instruction(
            document, child, edition, version, depth, scope,
        )?);
    } else {
        match name.local.as_str() {
            "if" | "choose" => {
                out.push(conditional_integration::compile(
                    document,
                    child,
                    edition,
                    child_version,
                    depth,
                    scope,
                )?);
                return Ok(());
            }
            "fallback" if matches!(edition, Edition::Legacy) => {
                // Section 15: ordinary instantiation is a no-op. Only
                // the parent instruction's fallback path lowers content.
                // Use shared fallback attribute validation; local-version
                // declarations still pass through the guarded scope owner.
                ensure_fallback_attributes(document, child)?;
                return Ok(());
            }
            "text" => {
                out.push(selected_text(document, child, child_version)?);
                return Ok(());
            }
            "message" | "comment" | "processing-instruction"
                if matches!(edition, Edition::Legacy) =>
            {
                let lower = match name.local.as_str() {
                    "comment" => selected_comment,
                    "processing-instruction" => selected_processing_instruction,
                    _ => selected_message,
                };
                out.push(lower(document, child, child_version, depth, scope)?);
                return Ok(());
            }
            "apply-templates" | "call-template" | "apply-imports"
                if matches!(edition, Edition::Legacy) =>
            {
                out.push(template_arguments::compile(
                    document,
                    child,
                    child_version,
                    bindings,
                )?);
                return Ok(());
            }
            "number" if matches!(edition, Edition::Legacy) => {
                out.push(selected_number(document, child, bindings)?);
                return Ok(());
            }
            "copy-of" if matches!(edition, Edition::Legacy) => {
                out.push(selected_child_copy(document, child)?);
                return Ok(());
            }
            "for-each" if matches!(edition, Edition::Legacy) => {
                out.push(selected_child_iteration(
                    document, child, version, depth, scope,
                )?);
                return Ok(());
            }
            "value-of" => {}
            "element" => {
                out.push(computed_element(
                    document, child, edition, version, depth, scope,
                )?);
                return Ok(());
            }
            "variable" if matches!(edition, Edition::Legacy) => {
                out.push(local_variable(
                    document,
                    child,
                    child_version,
                    bindings,
                    globals,
                )?);
                return Ok(());
            }
            _ => return Err(refusal(document, child)),
        }
        out.push(value_of(document, child, child_version, &visible)?);
    }
    Ok(())
}

fn selected_unknown_instruction(
    document: &Document,
    child: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    scope: StaticBindings<'_>,
) -> Result<Vec<Instruction>, CompileFailure> {
    if matches!(version, LocalVersion::LegacyNormal) {
        return Err(invalid(
            "FXAR0031",
            "unknown instruction outside forward compatibility",
            document.location(child),
        ));
    }
    compile_forward_fallbacks_with_body(
        document,
        child,
        &document.name(child).expect("element").local,
        &mut |document, fallback| body(document, fallback, edition, version, depth + 1, scope),
    )
}

fn selected_child_copy(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    require_probe_attribute_set(document, element, "select")?;
    let select = required_attribute(document, element, None, "select")?.trim();
    let string_literal = ['\'', '"'].into_iter().any(|quote| {
        select
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            .is_some_and(|value| !value.contains(quote))
    });
    if string_literal
        || matches!(select, "true()" | "false()")
        || template_arguments::selected_integer_argument(select)
    {
        let instruction = compile_copy_of(document, element)?;
        return if matches!(instruction, Instruction::CopyOfStaticAtomicText { .. }) {
            Ok(instruction)
        } else {
            Err(refusal(document, element))
        };
    }
    let steps = select.strip_prefix('/').unwrap_or(select);
    let named_steps = steps.strip_suffix("/*").unwrap_or(steps);
    let rooted_descendants = select
        .strip_prefix("//")
        .is_some_and(|path| path.split('/').all(is_ascii_ncname));
    let child_comments = steps == "comment()"
        || steps
            .strip_suffix("/comment()")
            .is_some_and(|path| path.split('/').all(is_ascii_ncname));
    let interior_descendants = steps.split_once("//").is_some_and(|(head, tail)| {
        head.split('/').all(is_ascii_ncname) && tail.split('/').all(is_ascii_ncname)
    });
    if !matches!(select, "*" | "." | "/" | "current()")
        && !named_steps.split('/').all(is_ascii_ncname)
        && !rooted_descendants
        && !child_comments
        && !interior_descendants
    {
        return Err(refusal(document, element));
    }
    // The shared compiler owns selection. Named child steps and a final * select
    // child elements, never unattached attributes; still supply selected legacy
    // recovery explicitly instead of inheriting a declared version's flag.
    let mut instruction = compile_copy_of(document, element)?;
    let (Instruction::CopyOfLocationPath {
        recover_unattached_attributes,
        ..
    }
    | Instruction::CopyOfChildElements {
        recover_unattached_attributes,
        ..
    }
    | Instruction::CopyOfCurrent {
        recover_unattached_attributes,
        ..
    }) = &mut instruction
    else {
        return Err(refusal(document, element));
    };
    *recover_unattached_attributes = true;
    Ok(instruction)
}

fn selected_child_iteration(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    depth: usize,
    scope: StaticBindings<'_>,
) -> Result<Instruction, CompileFailure> {
    require_probe_attribute_set(document, element, "select")?;
    let select = required_attribute(document, element, None, "select")?;
    // Relative/rooted named child paths with optional final wildcard, or one root-descendant name. No variable/atomic/
    // secondary-document/namespace selection or sorting is implied.
    if select != "*"
        && !select
            .strip_prefix('/')
            .unwrap_or(select)
            .strip_suffix("/*")
            .unwrap_or(select.strip_prefix('/').unwrap_or(select))
            .split('/')
            .all(is_ascii_ncname)
        && !select.strip_prefix("//").is_some_and(is_ascii_ncname)
        && !select.strip_prefix(".//").is_some_and(is_ascii_ncname)
    {
        return Err(refusal(document, element));
    }
    Ok(Instruction::ForEachNodes {
        select: template_invocation_compiler::parse_apply_selection(
            document,
            element,
            select,
            document.location(element).clone(),
        )?,
        sorts: vec![],
        body: body(
            document,
            element,
            Edition::Legacy,
            version,
            depth + 1,
            scope,
        )?,
        location: document.location(element).clone(),
    })
}

fn selected_number(
    document: &Document,
    element: NodeId,
    bindings: &[String],
) -> Result<Instruction, CompileFailure> {
    require_probe_attribute_set(document, element, "format")?;
    let instruction = number_compiler::compile_format_only(document, element, true)?;
    if let Instruction::Number {
        format: crate::xslt::golden_semantics_experiment::NumberFormatPlan::Xslt10Variable(name),
        ..
    } = &instruction
        && !bindings.contains(name)
    {
        return Err(invalid(
            "XPST0008",
            format!("unbound numbering format variable: ${name}"),
            document.location(element),
        ));
    }
    Ok(instruction)
}

fn selected_literal(
    document: &Document,
    child: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    bindings: StaticBindings<'_>,
) -> Result<Instruction, CompileFailure> {
    // Constant attributes and one whole literal/variable concat AVT only.
    // The shared parser's selected shape is edition-independent; no generic AVT admission.
    for attr in document.attributes(child) {
        let name = document.name(*attr).expect("attribute");
        if name.namespace.as_deref() == Some(XSLT_NAMESPACE) && name.local == "version" {
            continue;
        }
        if matches!(edition, Edition::Legacy)
            && name.namespace.as_deref() == Some("http://www.w3.org/XML/1998/namespace")
            && name.local == "space"
            && matches!(document.value(*attr), Some("default" | "preserve"))
        {
            // The canonical preprocessing view already owns inherited/reset
            // whitespace. Literal construction must also retain the attribute.
            continue;
        }
        if name.namespace.is_some() {
            return Err(refusal(document, child));
        }
        let value = document
            .value(*attr)
            .ok_or_else(|| refusal(document, child))?;
        if value.contains(['{', '}']) {
            let Some(expression) = value
                .strip_prefix('{')
                .and_then(|value| value.strip_suffix('}'))
            else {
                return Err(refusal(document, child));
            };
            if expression.contains(['{', '}'])
                || !selected_literal_variable_concat(
                    expression,
                    version,
                    &bindings.visible(),
                    document.location(*attr),
                )?
            {
                return Err(refusal(document, child));
            }
        }
    }
    let (mut literal, excluded) = compile_literal_element_parts(document, child)?;
    if !excluded.is_empty() {
        return Err(refusal(document, child));
    }
    let Instruction::LiteralElement { body: content, .. } = &mut literal else {
        unreachable!()
    };
    *content = body(document, child, edition, version, depth + 1, bindings)?;
    Ok(literal)
}

fn selected_processing_instruction(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    depth: usize,
    bindings: StaticBindings<'_>,
) -> Result<Instruction, CompileFailure> {
    require_probe_attribute_set(document, element, "name")?;
    let name = required_attribute(document, element, None, "name")?;
    // This slice admits valid static targets only. Dynamic and invalid target
    // timing remain guarded instead of borrowing baseline version semantics.
    if !is_ascii_ncname(name) || name.eq_ignore_ascii_case("xml") {
        return Err(refusal(document, element));
    }
    Ok(Instruction::Xslt10ProcessingInstructionNode {
        target: crate::xslt::golden_semantics_experiment::LiteralAttributeValue::Text(
            name.to_owned(),
        ),
        body: body(
            document,
            element,
            Edition::Legacy,
            version,
            depth + 1,
            bindings,
        )?
        .into_boxed_slice(),
        location: document.location(element).clone(),
    })
}

fn selected_comment(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    depth: usize,
    bindings: StaticBindings<'_>,
) -> Result<Instruction, CompileFailure> {
    // No baseline constructor lowering: expressions and local version
    // interpretation must remain selected throughout the body.
    require_probe_attribute_set(document, element, "")?;
    Ok(Instruction::Xslt10CommentNode {
        body: body(
            document,
            element,
            Edition::Legacy,
            version,
            depth + 1,
            bindings,
        )?
        .into_boxed_slice(),
        location: document.location(element).clone(),
    })
}

fn selected_message(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    depth: usize,
    bindings: StaticBindings<'_>,
) -> Result<Instruction, CompileFailure> {
    // Admit only the already-shared valid legacy attribute subset; forward
    // optional-attribute recovery remains a separate integration gate.
    require_probe_attribute_set(document, element, "terminate")?;
    if !matches!(
        optional_attribute(document, element, None, "terminate"),
        None | Some("yes" | "no")
    ) {
        return Err(refusal(document, element));
    }
    compile_message_with_body(document, element, &mut |document, message| {
        body(
            document,
            message,
            Edition::Legacy,
            version,
            depth + 1,
            bindings,
        )
    })
}

fn selected_text(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
) -> Result<Instruction, CompileFailure> {
    if matches!(version, LocalVersion::ModernWitness(_)) {
        // Keep the preexisting modern witness narrow until its attribute scope
        // is integrated, rather than borrowing legacy optional-attribute rules.
        if !document.attributes(element).is_empty() {
            return Err(refusal(document, element));
        }
        return compile_text(document, element);
    }
    let escaping = legacy_text_output_options(document, element, version, false)?;
    compile_text_after_attribute_validation(document, element, escaping)
}

fn legacy_text_output_options(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    allow_select: bool,
) -> Result<bool, CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some()
            || matches!(
                name.local.as_str(),
                "version"
                    | "default-collation"
                    | "xpath-default-namespace"
                    | "default-mode"
                    | "exclude-result-prefixes"
            )
            || allow_select && name.local == "separator"
        {
            return Err(refusal(document, *attribute));
        }
        if name.local != "disable-output-escaping"
            && !(allow_select && name.local == "select")
            && matches!(version, LocalVersion::LegacyNormal)
        {
            return Err(invalid(
                "XTSE0090",
                "unknown text-output attribute outside forward compatibility",
                document.location(*attribute),
            ));
        }
    }
    let escaping = match optional_attribute(document, element, None, "disable-output-escaping") {
        Some("yes") => true,
        None | Some("no") => false,
        Some(_) if matches!(version, LocalVersion::LegacyForward) => false,
        Some(_) => {
            return Err(invalid(
                "XTSE0020",
                "disable-output-escaping must be 'yes' or 'no'",
                document.location(element),
            ));
        }
    };
    Ok(escaping)
}

// This bounded slice admits literal-result xsl:version scope under each policy.
// The processor and preprocessing stamps never change with this local context.
fn scoped_version(
    document: &Document,
    element: NodeId,
    edition: Edition,
    inherited: LocalVersion,
) -> Result<LocalVersion, CompileFailure> {
    // An instruction unknown to the selected 1.0 processor is not a literal
    // result element. Neither version spelling on it establishes local scope.
    // Recognition is edition-defined, not the modern compiler's capability.
    if matches!(edition, Edition::Legacy)
        && document.name(element).is_some_and(|name| {
            name.namespace.as_deref() == Some(XSLT_NAMESPACE)
                && !is_xslt10_template_element(&name.local)
        })
    {
        return Ok(inherited);
    }
    if !has_local_version(document, element) {
        return Ok(inherited);
    }
    if edition.is_modern()
        && (is_xslt_element(document, element, "value-of")
            || is_xslt_element(document, element, "element"))
        && optional_attribute(document, element, Some(XSLT_NAMESPACE), "version").is_none()
    {
        return probe_declared_version(
            required_attribute(document, element, None, "version")?,
            document.location(element),
        )
        .map(LocalVersion::ModernWitness);
    }
    if document
        .name(element)
        .is_none_or(|name| name.namespace.as_deref() == Some(XSLT_NAMESPACE))
        || optional_attribute(document, element, None, "version").is_some()
    {
        return Err(refusal(document, element));
    }
    let lexical = required_attribute(document, element, Some(XSLT_NAMESPACE), "version")?;
    match edition {
        Edition::Legacy => legacy_numeric_version(lexical, document.location(element)),
        #[cfg(test)]
        Edition::Modern => probe_declared_version(lexical, document.location(element))
            .map(LocalVersion::ModernWitness),
    }
}

// XPath 1.0 Number grammar and IEEE-754 numeric comparison, not modern decimal
// validation or a version interval. Zero and fractions also enable forwarding.
// In particular, do not reuse modern positive-decimal validation: it admits
// a leading plus sign, which is not part of XPath 1.0's Number production.
#[allow(
    clippy::float_cmp,
    reason = "XSLT 1.0 switches on numeric equality to 1.0, not proximity"
)]
fn legacy_numeric_version(
    lexical: &str,
    location: &SourceLocation,
) -> Result<LocalVersion, CompileFailure> {
    let (integer, fraction) = lexical.split_once('.').unwrap_or((lexical, ""));
    if (!integer.is_empty() || !fraction.is_empty())
        && integer.bytes().all(|b| b.is_ascii_digit())
        && fraction.bytes().all(|b| b.is_ascii_digit())
        && let Ok(number) = lexical.parse::<f64>()
    {
        return Ok(if number == 1.0 {
            LocalVersion::LegacyNormal
        } else {
            LocalVersion::LegacyForward
        });
    }
    Err(unsupported(
        "FXAR0031",
        "version outside admitted legacy Number lexical grammar",
        location,
    ))
}

// Keep computed-name eligibility separate from sequence dispatch. Dynamic names,
// dynamic namespace overrides and computed attributes still need policy propagation.
fn computed_element(
    document: &Document,
    element: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    bindings: StaticBindings<'_>,
) -> Result<Instruction, CompileFailure> {
    let name = required_attribute(document, element, None, "name")?;
    let static_legacy_name = matches!(edition, Edition::Legacy) && is_ascii_ncname(name);
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some()
            || !(name.local == "name"
                || static_legacy_name
                    && matches!(name.local.as_str(), "namespace" | "use-attribute-sets")
                || name.local == "version" && edition.is_modern())
        {
            return Err(refusal(document, *attribute));
        }
    }
    if optional_attribute(document, element, None, "namespace")
        .is_some_and(|namespace| namespace.contains(['{', '}']))
        || optional_attribute(document, element, None, "use-attribute-sets").is_some_and(|names| {
            names.split_whitespace().next().is_none()
                || !names.split_whitespace().all(is_ascii_ncname)
        })
    {
        return Err(refusal(document, element));
    }
    let attribute_name = matches!(edition, Edition::Legacy)
        && name
            .strip_prefix("{@")
            .and_then(|name| name.strip_suffix('}'))
            .is_some_and(is_ascii_ncname);
    if (!is_ascii_ncname(name) && !attribute_name)
        || meaningful_children(document, element)
            .iter()
            .any(|node| is_xslt_element(document, *node, "attribute"))
    {
        return Err(refusal(document, element));
    }
    let mut lower_body = |document: &Document, parent, excluded: &[NodeId]| {
        if !excluded.is_empty() {
            return Err(refusal(document, parent));
        }
        body(document, parent, edition, version, depth + 1, bindings)
    };
    if attribute_name {
        computed_element_compiler::compile_selected_legacy_dynamic_element(
            document,
            element,
            &mut lower_body,
        )
    } else {
        computed_element_compiler::compile_computed_element_after_attribute_validation(
            document,
            element,
            &mut lower_body,
        )
    }
}

fn local_variable(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    bindings: &mut Vec<String>,
    globals: &[String],
) -> Result<Instruction, CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute");
        if name.namespace.is_some() || !matches!(name.local.as_str(), "name" | "select") {
            return Err(refusal(document, *attribute));
        }
    }
    let name = compile_local_variable_name(document, element, document.location(element))?;
    if bindings.contains(&name) {
        return Err(invalid(
            "FXST0017",
            format!("duplicate local variable binding: ${name}"),
            document.location(element),
        ));
    }
    if let Some(select) = optional_attribute(document, element, None, "select") {
        if matches!(select.trim(), "name()" | "name(.)")
            || select.strip_prefix('@').is_some_and(is_ascii_ncname)
        {
            ensure_no_meaningful_children(document, element, "xsl:variable")?;
            let instruction = compile_select_variable(
                document,
                element,
                name.clone(),
                select,
                document.location(element).clone(),
            )?;
            bindings.push(name);
            return Ok(instruction);
        }
        if xpath_string_literal(select.trim()).is_none() {
            return Err(refusal(document, element));
        }
        ensure_no_meaningful_children(document, element, "xsl:variable")?;
        let instruction = compile_static_atomic_variable(&name, select, document.location(element))
            .expect("string literal passed shared eligibility");
        bindings.push(name);
        return Ok(instruction);
    }
    let children = meaningful_children(document, element);
    if let [child] = children.as_slice()
        && is_xslt_element(document, *child, "value-of")
    {
        let child_version = scoped_version(document, *child, Edition::Legacy, version)?;
        let Instruction::ValueOf {
            select,
            disable_output_escaping: false,
            ..
        } = value_of(
            document,
            *child,
            child_version,
            &StaticBindings {
                locals: bindings,
                globals,
            }
            .visible(),
        )?
        else {
            return Err(refusal(document, *child));
        };
        bindings.push(name.clone());
        return Ok(Instruction::Xslt10ValueOfTreeVariable {
            name,
            select,
            location: document.location(element).clone(),
        });
    }
    // Only text constructors enter this remaining slice; other local constructors and
    // nonliteral select expressions still require their own policy propagation.
    for child in meaningful_children(document, element) {
        if has_local_version(document, child) {
            return Err(refusal(document, child));
        }
    }
    let variable = compile_text_tree_variable_with_mode(
        document,
        element,
        &name,
        document.location(element),
        true,
    )?
    .ok_or_else(|| refusal(document, element))?;
    bindings.push(name);
    Ok(variable)
}

fn value_of(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    bindings: &[String],
) -> Result<Instruction, CompileFailure> {
    let escaping = if matches!(version, LocalVersion::ModernWitness(_)) {
        validate_modern_value_attributes(document, element)?;
        false
    } else {
        legacy_text_output_options(document, element, version, true)?
    };
    value_of_with_options(document, element, version, bindings, escaping)
}

fn validate_modern_value_attributes(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some() || !matches!(name.local.as_str(), "select" | "version") {
            return Err(refusal(document, *attribute));
        }
    }
    Ok(())
}

fn selected_literal_variable_concat(
    expression: &str,
    version: LocalVersion,
    bindings: &[String],
    location: &SourceLocation,
) -> Result<bool, CompileFailure> {
    if matches!(version, LocalVersion::ModernWitness(_)) {
        return Ok(false);
    }
    let Some((_, name)) = literal_attribute_compiler::parse_literal_variable_concat(expression)
    else {
        return Ok(false);
    };
    if !bindings.iter().any(|binding| binding == name) {
        return Err(invalid(
            "XPST0008",
            format!("unbound variable: ${name}"),
            location,
        ));
    }
    Ok(true)
}

fn selected_variable_concat(
    expression: &str,
    version: LocalVersion,
    bindings: &[String],
    location: &SourceLocation,
) -> Result<bool, CompileFailure> {
    if matches!(version, LocalVersion::ModernWitness(_)) {
        return Ok(false);
    }
    let Some(arguments) = expression
        .strip_prefix("concat(")
        .and_then(|tail| tail.strip_suffix(')'))
    else {
        return Ok(false);
    };
    let names: Option<Vec<_>> = arguments
        .split(',')
        .map(|argument| {
            argument
                .trim()
                .strip_prefix('$')
                .filter(|name| is_ascii_ncname(name))
        })
        .collect();
    let Some(names) = names.filter(|names| names.len() >= 2) else {
        return Ok(false);
    };
    for name in names {
        if !bindings.iter().any(|binding| binding == name) {
            return Err(invalid(
                "XPST0008",
                format!("unbound variable: ${name}"),
                location,
            ));
        }
    }
    Ok(true)
}

fn selected_bound_child_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    version: LocalVersion,
    bindings: &[String],
) -> Result<bool, CompileFailure> {
    if !matches!(version, LocalVersion::ModernWitness(_))
        && let Some((name, path)) = expression
            .strip_prefix('$')
            .and_then(|value| value.split_once('/'))
        && is_ascii_ncname(name)
        && path.split('/').all(is_ascii_ncname)
    {
        validate_value_binding(document, element, name, bindings)?;
        return Ok(true);
    }
    Ok(false)
}

fn value_of_with_options(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    bindings: &[String],
    escaping: bool,
) -> Result<Instruction, CompileFailure> {
    let expression = required_attribute(document, element, None, "select")?;
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    if let Some(argument) = expression
        .strip_prefix("count(")
        .and_then(|value| value.strip_suffix(')'))
        && selected_keys::lookup(document, element, argument, version, bindings)?.is_some()
    {
        return lower_selected_value_of(document, element, version, escaping);
    }
    if selected_bound_child_path(document, element, expression, version, bindings)? {
        return lower_selected_value_of(document, element, version, escaping);
    }
    if selected_variable_concat(expression, version, bindings, document.location(element))?
        || selected_literal_variable_concat(
            expression,
            version,
            bindings,
            document.location(element),
        )?
    {
        return lower_selected_value_of(document, element, version, escaping);
    }
    if let Some(number) =
        selected_introspection::version_number(document, element, expression, version)
    {
        // value-of converts the numeric property to text. This does not admit
        // it as a string-valued variable or into other typed expression owners.
        return compile_value_of_with_validated_options(
            document,
            element,
            escaping,
            &mut |_, _, _, _| Ok(ValueExpression::LiteralString(number.to_string())),
        );
    }
    let numeric_variable = selected_numeric_variable(expression, version);
    if !admitted_value(expression, version) && numeric_variable.is_none() {
        if matches!(version, LocalVersion::ModernWitness(DeclaredVersion::Four)) {
            return Err(refusal(document, element));
        }
        if let Some(test) = boolean_call_timing::value_with_leaf(
            expression,
            version,
            document.location(element),
            bindings,
            &|expression| {
                selected_introspection::boolean_leaf(document, element, expression, version)
            },
        )? {
            return Ok(boolean_call_timing::text_result(
                test,
                document.location(element),
                escaping,
            ));
        }
        if direct_literal_property_arity_error(expression) {
            let detail = "system-property requires exactly one argument";
            if matches!(version, LocalVersion::LegacyForward) {
                return Ok(Instruction::Xslt10DeferredFailure {
                    code: "XPST0017",
                    detail: detail.to_owned(),
                    location: document.location(element).clone(),
                });
            }
            return Err(invalid("XPST0017", detail, document.location(element)));
        }
        // Only unqualified simple path syntax enters this classification. Do
        // not interpret a general expression rejected by a path parser as an
        // invalid XPath expression, nor catch arbitrary compiler failures.
        let path = expression.trim();
        let path_only = path.is_empty()
            || path.bytes().all(|b| b == b'.')
            || path
                .as_bytes()
                .first()
                .is_some_and(|b| b.is_ascii_alphabetic() || matches!(b, b'_' | b'/'))
                && path
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/'));
        if path_only
            && let Err(crate::xpath::path_experiment::PathFailure::Invalid {
                standard_code: "XPST0003",
                detail,
                location,
            }) = crate::xpath::path_experiment::parse_location_path(
                path,
                document.location(element).clone(),
            )
        {
            if matches!(version, LocalVersion::LegacyForward) {
                return Ok(Instruction::Xslt10DeferredFailure {
                    code: "XPST0003",
                    detail,
                    location,
                });
            }
            return Err(invalid("XPST0003", detail, &location));
        }
        return Err(refusal(document, element));
    }
    if let Some(name) = expression.strip_prefix('$').or(numeric_variable) {
        validate_value_binding(document, element, name, bindings)?;
    }
    lower_selected_value_of(document, element, version, escaping)
}

fn validate_value_binding(
    document: &Document,
    element: NodeId,
    name: &str,
    bindings: &[String],
) -> Result<(), CompileFailure> {
    if bindings.iter().any(|binding| binding == name) {
        return Ok(());
    }
    // Some selected consumers still supply locals only. A declared global in
    // those routes is unintegrated, not an invalid stylesheet name.
    let root = compiler::document_element(document)?;
    if meaningful_children(document, root)
        .into_iter()
        .any(|child| {
            (is_xslt_element(document, child, "param")
                || is_xslt_element(document, child, "variable"))
                && optional_attribute(document, child, None, "name") == Some(name)
        })
    {
        return Err(refusal(document, element));
    }
    Err(invalid(
        "XPST0008",
        format!("unbound variable: ${name}"),
        document.location(element),
    ))
}

fn lower_selected_value_of(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    escaping: bool,
) -> Result<Instruction, CompileFailure> {
    let mode = match version {
        LocalVersion::LegacyNormal
        | LocalVersion::LegacyForward
        | LocalVersion::ModernWitness(DeclaredVersion::One) => {
            value_expression_compiler::ValueCompatibilityMode::Xslt10
        }
        LocalVersion::ModernWitness(_) => value_expression_compiler::ValueCompatibilityMode::Modern,
    };
    compile_value_of_with_validated_options(
        document,
        element,
        escaping,
        &mut |document, element, expression, location| {
            value_expression_compiler::compile_value_expression_with_mode(
                document, element, expression, location, mode,
            )
        },
    )
}

fn admitted_value(path: &str, version: LocalVersion) -> bool {
    xpath_string_literal(path).is_some()
        || !matches!(version, LocalVersion::ModernWitness(_))
            && path
                .strip_prefix("./")
                .or_else(|| path.strip_prefix('/'))
                .unwrap_or(path)
                .rsplit_once("/@")
                .is_some_and(|(parent, attribute)| {
                    parent.split('/').all(is_ascii_ncname) && is_ascii_ncname(attribute)
                })
        || !matches!(version, LocalVersion::ModernWitness(_))
            && path.strip_prefix("./").is_some_and(|relative| {
                relative
                    .split('/')
                    .all(|step| step == "text()" || is_ascii_ncname(step))
            })
        || !matches!(version, LocalVersion::ModernWitness(_))
            && matches!(
                path.trim(),
                "name()" | "name(.)" | "self::node()" | "." | "position()" | "last()"
            )
        || !matches!(version, LocalVersion::ModernWitness(_))
            && path
                .strip_prefix("./@")
                .or_else(|| path.strip_prefix('@'))
                .is_some_and(is_ascii_ncname)
        || path.strip_prefix('$').is_some_and(is_ascii_ncname)
        || path
            .strip_prefix("(//")
            .and_then(|tail| tail.split_once(")["))
            .and_then(|(name, rest)| {
                rest.split_once("]/")
                    .map(|(position, suffix)| (name, position, suffix))
            })
            .is_some_and(|(name, position, suffix)| {
                is_ascii_ncname(name)
                    && is_ascii_ncname(suffix.trim())
                    && crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(
                        position,
                    )
                    .is_some()
            })
        || path
            .split('/')
            .all(|step| step == "text()" || is_ascii_ncname(step))
}

fn selected_numeric_variable(expression: &str, version: LocalVersion) -> Option<&str> {
    if matches!(version, LocalVersion::ModernWitness(_)) {
        return None;
    }
    if let Some(variable) =
        value_expression_compiler::parse_xslt10_variable_conversion(expression, "number")
    {
        return Some(variable);
    }
    let (left, operator, right) = crate::xpath::binary_numeric_experiment::split_paths(expression)?;
    if operator != crate::xpath::binary_numeric_experiment::BinaryNumericOperator::Add
        || !template_arguments::selected_integer_argument(left.trim())
    {
        return None;
    }
    right
        .trim()
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
}

// One complete call whose arguments have no evaluation effects. Boolean
// composition uses the same eligibility but puts failure at the call leaf.
fn direct_literal_property_arity_error(expression: &str) -> bool {
    if !super::xslt10_static_introspection_compiler::system_property_arity_error(expression) {
        return false;
    }
    let Some(arguments) = expression
        .trim()
        .strip_prefix("system-property(")
        .and_then(|tail| tail.strip_suffix(')'))
    else {
        return false;
    };
    arguments.trim().is_empty()
        || crate::xpath::static_string_experiment::split_arguments(arguments, 64).is_some_and(
            |arguments| {
                arguments
                    .iter()
                    .all(|argument| xpath_string_literal(argument.trim()).is_some())
            },
        )
}

#[cfg(test)]
#[path = "matched_integration_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "top_level_integration_tests.rs"]
mod top_level_tests;

#[cfg(test)]
#[path = "message_integration_tests.rs"]
mod message_tests;

#[cfg(test)]
#[path = "dispatch_integration_tests.rs"]
mod dispatch_tests;

#[cfg(test)]
#[path = "named_integration_tests.rs"]
mod named_tests;
