//! ADR-0025 matched-template integration checkpoint. Private until profile gates
//! pass; ordinary entry points retain their existing semantics. Uses shared
//! declaration, sequence, instruction, fallback and result construction owners.
use super::*;
use crate::execution_control_experiment::InvocationControl;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use compiler::stylesheet_preprocessing::{StrictPreprocessingFailure, stylesheet_visibility_view};

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
            require_probe_attribute_set(document, child, "match")?;
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
            for attr in document.attributes(child) {
                let name = document.name(*attr).expect("attribute name");
                if name.namespace.is_some()
                    || !matches!(
                        name.local.as_str(),
                        "method" | "indent" | "omit-xml-declaration"
                    )
                {
                    return Err(refusal(document, *attr).into());
                }
            }
        } else if matches!(edition, Edition::Legacy) && is_xslt_element(document, child, "fallback")
        {
            if matches!(version, LocalVersion::LegacyNormal) {
                return Err(invalid(
                    "FXST1002",
                    "xsl:fallback is not an XSLT 1.0 top-level declaration",
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
    let program = compiler::compile_stylesheet_with_validation_and_template_lowerer(
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
            compiler::compile_template_with_body(
                document,
                template,
                &mut |document, parent, excluded, bindings| {
                    if !excluded.is_empty() || !bindings.is_empty() {
                        return Err(refusal(document, parent));
                    }
                    body(document, parent, edition, version, 0, &[])
                },
            )
        },
    )?;
    let mut program = compilation_context(edition, preprocessing).qualify_module(program);
    compiler::finalize_decimal_formats(&mut program)?;
    compiler::finalize_attribute_sets(&mut program)?;
    compiler::finalize_character_maps(&mut program)?;
    compiler::validate_named_template_references(&program)?;
    Ok(program)
}

fn refusal(document: &Document, node: NodeId) -> CompileFailure {
    unsupported(
        "FXAR0031",
        "processor policy not yet propagated to this compilation shape",
        document.location(node),
    )
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
                    "fallback" if matches!(edition, Edition::Legacy) => {
                        // Section 15: ordinary instantiation is a no-op. Only
                        // the parent instruction's fallback path lowers content.
                        // Attribute/local-version policy is not widened here.
                        if !document.attributes(child).is_empty() {
                            return Err(refusal(document, child));
                        }
                        return Ok(());
                    }
                    "text" | "value-of" => {}
                    "element" => {
                        out.push(computed_element(
                            document, child, edition, version, depth, bindings,
                        )?);
                        return Ok(());
                    }
                    "variable" if matches!(edition, Edition::Legacy) => {
                        out.push(text_variable(document, child, bindings)?);
                        return Ok(());
                    }
                    _ => return Err(refusal(document, child)),
                }
                if name.local == "value-of" {
                    out.push(value_of(document, child, child_version, bindings)?);
                } else {
                    if !document.attributes(child).is_empty() {
                        return Err(refusal(document, child));
                    }
                    out.push(compile_xslt_instruction(
                        document,
                        child,
                        &name.local,
                        bindings,
                    )?);
                }
            }
            Ok(())
        },
    )
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

fn text_variable(
    document: &Document,
    element: NodeId,
    bindings: &mut Vec<String>,
) -> Result<Instruction, CompileFailure> {
    require_probe_attribute_set(document, element, "name")?;
    let name = compile_local_variable_name(document, element, document.location(element))?;
    // Only text constructors enter this slice; other local constructors and
    // select expressions still require their own policy propagation.
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
    if bindings.contains(&name) {
        return Err(invalid(
            "FXST0017",
            format!("duplicate local variable binding: ${name}"),
            document.location(element),
        ));
    }
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
            .split('/')
            .all(|step| step == "text()" || is_ascii_ncname(step))
}

// This entire select is one call with no argument evaluation effects. Do not
// promote this instruction-level failure to nested/short-circuited calls.
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
