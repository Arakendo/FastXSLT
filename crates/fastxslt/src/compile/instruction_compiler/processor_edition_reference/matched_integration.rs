//! ADR-0025 matched-template integration checkpoint. Private until profile gates
//! pass; ordinary entry points retain their existing semantics. Uses shared
//! declaration, sequence, instruction, fallback and result construction owners.
use super::*;
use crate::execution_control_experiment::InvocationControl;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use compiler::stylesheet_preprocessing::{StrictPreprocessingFailure, stylesheet_visibility_view};

#[path = "boolean_call_timing.rs"]
mod boolean_call_timing;

#[path = "conditional_integration.rs"]
mod conditional_integration;

#[derive(Clone, Copy)]
enum LocalVersion {
    LegacyNormal,
    LegacyForward,
    ModernWitness(DeclaredVersion),
}

fn compile(
    document: &Document,
    edition: Edition,
    control: &mut InvocationControl,
) -> Result<StylesheetProgram, StrictPreprocessingFailure> {
    let root = compiler::document_element(document)?;
    let version = if matches!(edition, Edition::Legacy) {
        compiler::require_stylesheet_root(document, root)?;
        require_probe_attribute_set(document, root, "version")?;
        legacy_numeric_version(
            required_attribute(document, root, None, "version")?,
            document.location(root),
        )?
    } else {
        LocalVersion::ModernWitness(module_version(document, root)?)
    };
    let mut ignored_top_level = Vec::new();
    // No named templates, defaults, globals or modules may enter an unprofiled
    // lowering route. Their integration is separate from matched body routing.
    for child in meaningful_children(document, root) {
        if is_xslt_element(document, child, "template") {
            validate_template_attributes(document, child, version)?;
            let pattern = required_attribute(document, child, None, "match")?;
            for part in pattern.split('|').map(str::trim) {
                let (compiled, _) = compiler::compile_match_pattern(document, child, part)?;
                if !(matches!(part, "/" | "text()") || is_ascii_ncname(part)
                    || matches!(compiled,
                        crate::xslt::golden_semantics_experiment::MatchPattern::ElementWithAttributeValue { .. })) {
                    return Err(refusal(document, child).into());
                }
            }
        } else if is_xslt_element(document, child, "output") {
            validate_output_attributes(document, child, version)?;
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
    let preprocessing = match edition {
        Edition::Legacy => StylesheetPreprocessingProfile::StrictXslt10,
        Edition::Modern => StylesheetPreprocessingProfile::Modern,
    };
    let view = stylesheet_visibility_view(document, root, preprocessing, control)?;
    let program = compiler::compile_stylesheet_with_declaration_lowerers(
        &view,
        root,
        &ignored_top_level,
        &mut |document, root, lexical| match edition {
            Edition::Legacy => legacy_numeric_version(lexical, document.location(root)).map(|_| ()),
            Edition::Modern => compiler::validate_declared_version(document, root, lexical),
        },
        &mut |document, template| {
            if meaningful_children(document, template)
                .iter()
                .any(|child| is_xslt_element(document, *child, "param"))
            {
                return Err(refusal(document, template));
            }
            compiler::compile_template_with_body_and_validation(
                document,
                template,
                &mut |document, element| validate_template_attributes(document, element, version),
                &mut |document, parent, excluded, bindings| {
                    if !excluded.is_empty() || !bindings.is_empty() {
                        return Err(refusal(document, parent));
                    }
                    body(document, parent, edition, version, 0, &[])
                },
            )
        },
        &mut |document, element, declared, superseded| {
            lower_output_declaration(document, element, declared, superseded, version)
        },
    )?;
    let mut program = compilation_context(edition, preprocessing).qualify_module(program);
    compiler::finalize_decimal_formats(&mut program)?;
    compiler::finalize_attribute_sets(&mut program)?;
    compiler::finalize_character_maps(&mut program)?;
    compiler::validate_named_template_references(&program)?;
    Ok(program)
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
        if name.local == "match" {
            continue;
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
            && !document
                .value(*attr)
                .is_some_and(|v| v.eq_ignore_ascii_case("UTF-8"))
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

fn body(
    document: &Document,
    parent: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    inherited: &[String],
) -> Result<Vec<Instruction>, CompileFailure> {
    if depth >= 16 {
        return Err(refusal(document, parent));
    }
    let version = scoped_version(document, parent, edition, version)?;
    compile_sequence_with_element_lowerer(
        document,
        parent,
        &[],
        inherited,
        &mut |document, child, bindings, out| {
            let child_version = scoped_version(document, child, edition, version)?;
            let name = document.name(child).expect("sequence element");
            if name.namespace.as_deref() != Some(XSLT_NAMESPACE) {
                // Attribute expressions and computed children may re-enter baseline
                // compilation. Refuse them until policy reaches those owners too.
                if document.attributes(child).iter().any(|attr| {
                    let name = document.name(*attr).expect("attribute");
                    name.namespace.as_deref() != Some(XSLT_NAMESPACE) || name.local != "version"
                }) {
                    return Err(refusal(document, child));
                }
                let (mut literal, excluded) = compile_literal_element_parts(document, child)?;
                if !excluded.is_empty() {
                    return Err(refusal(document, child));
                }
                let Instruction::LiteralElement { body: content, .. } = &mut literal else {
                    unreachable!()
                };
                *content = body(document, child, edition, version, depth + 1, bindings)?;
                out.push(literal);
            } else if matches!(edition, Edition::Legacy) && !is_xslt10_template_element(&name.local)
            {
                if matches!(version, LocalVersion::LegacyNormal) {
                    return Err(invalid(
                        "FXAR0031",
                        "unknown instruction outside forward compatibility",
                        document.location(child),
                    ));
                }
                out.extend(compile_forward_fallbacks_with_body(
                    document,
                    child,
                    &name.local,
                    &mut |document, fallback| {
                        body(document, fallback, edition, version, depth + 1, bindings)
                    },
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
                            bindings,
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
                    "value-of" => {}
                    "element" => {
                        out.push(computed_element(
                            document, child, edition, version, depth, bindings,
                        )?);
                        return Ok(());
                    }
                    "variable" if matches!(edition, Edition::Legacy) => {
                        out.push(local_variable(document, child, bindings)?);
                        return Ok(());
                    }
                    _ => return Err(refusal(document, child)),
                }
                out.push(value_of(document, child, child_version, bindings)?);
            }
            Ok(())
        },
    )
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
        {
            return Err(refusal(document, *attribute));
        }
        if name.local != "disable-output-escaping" && matches!(version, LocalVersion::LegacyNormal)
        {
            return Err(invalid(
                "XTSE0090",
                "unknown text attribute outside forward compatibility",
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
    compile_text_after_attribute_validation(document, element, escaping)
}

// This bounded slice admits literal-result xsl:version scope under each policy.
// The processor and preprocessing stamps never change with this local context.
fn scoped_version(
    document: &Document,
    element: NodeId,
    edition: Edition,
    inherited: LocalVersion,
) -> Result<LocalVersion, CompileFailure> {
    if !has_local_version(document, element) {
        return Ok(inherited);
    }
    if matches!(edition, Edition::Modern)
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
// namespace overrides and computed attributes still need policy propagation.
fn computed_element(
    document: &Document,
    element: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    bindings: &[String],
) -> Result<Instruction, CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some()
            || !(name.local == "name"
                || name.local == "version" && matches!(edition, Edition::Modern))
        {
            return Err(refusal(document, *attribute));
        }
    }
    if !is_ascii_ncname(required_attribute(document, element, None, "name")?)
        || meaningful_children(document, element)
            .iter()
            .any(|node| is_xslt_element(document, *node, "attribute"))
    {
        return Err(refusal(document, element));
    }
    computed_element_compiler::compile_computed_element_after_attribute_validation(
        document,
        element,
        &mut |document, parent, excluded| {
            if !excluded.is_empty() {
                return Err(refusal(document, parent));
            }
            body(document, parent, edition, version, depth + 1, bindings)
        },
    )
}

fn local_variable(
    document: &Document,
    element: NodeId,
    bindings: &mut Vec<String>,
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
        if xpath_string_literal(select.trim()).is_none() {
            return Err(refusal(document, element));
        }
        ensure_no_meaningful_children(document, element, "xsl:variable")?;
        let instruction = compile_static_atomic_variable(&name, select, document.location(element))
            .expect("string literal passed shared eligibility");
        bindings.push(name);
        return Ok(instruction);
    }
    // Only text constructors enter this slice; other local constructors and
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
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some()
            || !(name.local == "select"
                || name.local == "version" && matches!(version, LocalVersion::ModernWitness(_)))
        {
            return Err(refusal(document, *attribute));
        }
    }
    let expression = required_attribute(document, element, None, "select")?;
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    if !admitted_value(expression) {
        if matches!(version, LocalVersion::ModernWitness(DeclaredVersion::Four)) {
            return Err(refusal(document, element));
        }
        if let Some(test) =
            boolean_call_timing::compile(expression, version, document.location(element), bindings)?
        {
            return Ok(boolean_call_timing::text_result(
                test,
                document.location(element),
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
    if let Some(name) = expression.strip_prefix('$')
        && !bindings.iter().any(|binding| binding == name)
    {
        // Globals/parameters are excluded here, so the visible set is complete.
        // Unbound names are not an XPath grammar/function deferral case.
        return Err(invalid(
            "XPST0008",
            format!("unbound variable: ${name}"),
            document.location(element),
        ));
    }
    let mode = match version {
        LocalVersion::LegacyNormal
        | LocalVersion::LegacyForward
        | LocalVersion::ModernWitness(DeclaredVersion::One) => {
            value_expression_compiler::ValueCompatibilityMode::Xslt10
        }
        LocalVersion::ModernWitness(_) => value_expression_compiler::ValueCompatibilityMode::Modern,
    };
    compile_value_of_after_attribute_validation(
        document,
        element,
        &mut |document, element, expression, location| {
            value_expression_compiler::compile_value_expression_with_mode(
                document, element, expression, location, mode,
            )
        },
    )
}

fn admitted_value(path: &str) -> bool {
    path.strip_prefix('$').is_some_and(is_ascii_ncname)
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

#[path = "top_level_integration_tests.rs"]
mod top_level_tests;
