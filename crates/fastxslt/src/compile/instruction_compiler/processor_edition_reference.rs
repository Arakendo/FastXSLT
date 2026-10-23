//! Bounded selected legacy lowering plus retained AR-0031 test references.
//! The historical module path is private. Non-test entry points admit only the
//! R0 legacy slice; modern selection and the original instruction/root probes
//! remain test-only. No public selector, default change or profile promotion.
#[cfg(test)]
use super::*;
#[cfg(not(test))]
use super::{
    BooleanExpression, CompileFailure, Document, Instruction, NodeId, NodeKind, SourceLocation,
    ValueExpression, XSLT_NAMESPACE, boolean_expression_compiler, compile_apply_imports,
    compile_apply_templates, compile_copy_of, compile_forward_fallbacks_with_body,
    compile_if_with_lowerers, compile_literal_element_parts, compile_local_variable_name,
    compile_message_with_body, compile_select_variable, compile_sequence_with_element_lowerer,
    compile_static_atomic_variable, compile_text, compile_text_after_attribute_validation,
    compile_text_tree_variable_with_mode, compile_value_of_with_validated_options,
    computed_element_compiler, conditional_instruction_compiler, ensure_fallback_attributes,
    ensure_no_meaningful_children, invalid, is_ascii_ncname, is_xslt_element,
    is_xslt10_template_element, literal_attribute_compiler, meaningful_children, number_compiler,
    optional_attribute, parse_literal_document_call, required_attribute,
    template_invocation_compiler, unsupported, validate_exclude_result_prefixes,
    value_expression_compiler, xpath_string_literal, xslt10_static_introspection_compiler,
};
use crate::compile::golden_stylesheet_experiment as compiler;

#[path = "processor_edition_reference/matched_integration.rs"]
mod matched_integration;
pub(crate) use matched_integration::compile_selected_legacy_import_module;
pub(crate) use matched_integration::compile_selected_legacy_include_module;
pub(crate) use matched_integration::compile_selected_legacy_mixed_module;
#[cfg(test)]
pub(crate) use matched_integration::compile_selected_legacy_resolved_chain_for_test;
pub(crate) use matched_integration::compile_selected_legacy_resolved_pair;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Edition {
    Legacy,
    #[cfg(test)]
    Modern,
}

impl Edition {
    fn is_modern(self) -> bool {
        match self {
            Self::Legacy => false,
            #[cfg(test)]
            Self::Modern => true,
        }
    }
}

#[cfg(test)]
fn compilation_context(
    edition: Edition,
    preprocessing: StylesheetPreprocessingProfile,
) -> compiler::stylesheet_preprocessing::StylesheetCompilationContext {
    use crate::xslt::golden_semantics_experiment::ProcessorPolicyStamp;
    compiler::stylesheet_preprocessing::StylesheetCompilationContext::for_reference(
        preprocessing,
        true,
    )
    .with_processor_reference(match edition {
        Edition::Legacy => ProcessorPolicyStamp::Xslt10Reference,
        Edition::Modern => ProcessorPolicyStamp::ModernReference,
    })
}

#[derive(Clone, Copy)]
enum DeclaredVersion {
    One,
    Two,
    Three,
    Four,
}

/// Single root-template module only. All other declaration shapes are refused
/// before any non-profile-aware compiler entry point can consume their bodies.
#[cfg(test)]
pub(super) fn lower_module(
    document: &Document,
    edition: Edition,
    control: &mut crate::execution_control_experiment::InvocationControl,
) -> Result<
    crate::xslt::golden_semantics_experiment::StylesheetProgram,
    compiler::stylesheet_preprocessing::StrictPreprocessingFailure,
> {
    let preprocessing = match edition {
        Edition::Legacy => StylesheetPreprocessingProfile::StrictXslt10,
        Edition::Modern => StylesheetPreprocessingProfile::Modern,
    };
    lower_module_with_preprocessing(document, edition, preprocessing, control)
}

// Orthogonal cross-combinations are differential controls only, not supported
// product profiles. This entry remains inside the test-only reference owner.
#[cfg(test)]
fn lower_module_with_preprocessing(
    document: &Document,
    edition: Edition,
    preprocessing: StylesheetPreprocessingProfile,
    control: &mut crate::execution_control_experiment::InvocationControl,
) -> Result<
    crate::xslt::golden_semantics_experiment::StylesheetProgram,
    compiler::stylesheet_preprocessing::StrictPreprocessingFailure,
> {
    use compiler::stylesheet_preprocessing::stylesheet_visibility_view;
    let root = compiler::document_element(document)?;
    let version = module_version(document, root)?;
    let children = meaningful_children(document, root);
    let [template] = children.as_slice() else {
        return Err(unsupported(
            "FXAR0031",
            "module probe requires exactly one root template",
            document.location(root),
        )
        .into());
    };
    if !is_xslt_element(document, *template, "template")
        || optional_attribute(document, *template, None, "match") != Some("/")
    {
        return Err(unsupported(
            "FXAR0031",
            "declaration outside root-template module probe",
            document.location(*template),
        )
        .into());
    }
    require_probe_attribute_set(document, *template, "match")?;
    let view = stylesheet_visibility_view(document, root, preprocessing, control)?;
    let mut program = compiler::compile_stylesheet_at_excluding_with_template_lowerer(
        &view,
        root,
        &[],
        &mut |document, element| lower_template(document, element, edition, version),
    )?;
    compiler::finalize_decimal_formats(&mut program)?;
    compiler::finalize_attribute_sets(&mut program)?;
    compiler::finalize_character_maps(&mut program)?;
    compiler::validate_named_template_references(&program)?;
    Ok(compilation_context(edition, preprocessing).qualify_module(program))
}

// Probe eligibility is not stylesheet compatibility validation: even attributes
// that the production compiler may ignore must not select an unprofiled route.
fn require_probe_attribute_set(
    document: &Document,
    element: NodeId,
    allowed: &str,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute has a name");
        if name.namespace.is_some() || name.local != allowed {
            return Err(unsupported(
                "FXAR0031",
                "attribute outside bounded module probe",
                document.location(*attribute),
            ));
        }
    }
    Ok(())
}

fn module_version(document: &Document, root: NodeId) -> Result<DeclaredVersion, CompileFailure> {
    compiler::require_stylesheet_root(document, root)?;
    require_probe_attribute_set(document, root, "version")?;
    probe_declared_version(
        compiler::required_attribute(document, root, None, "version")?,
        document.location(root),
    )
}

fn probe_declared_version(
    lexical: &str,
    location: &SourceLocation,
) -> Result<DeclaredVersion, CompileFailure> {
    match lexical {
        "1.0" => Ok(DeclaredVersion::One),
        "2.0" => Ok(DeclaredVersion::Two),
        "3.0" => Ok(DeclaredVersion::Three),
        "4.0" => Ok(DeclaredVersion::Four),
        _ => Err(unsupported(
            "FXAR0031",
            "version lexical form outside module probe",
            location,
        )),
    }
}

#[cfg(test)]
fn lower(
    document: &Document,
    element: NodeId,
    edition: Edition,
    version: DeclaredVersion,
) -> Result<Vec<Instruction>, CompileFailure> {
    lower_at_depth(document, element, edition, version, 0)
}

#[cfg(test)]
fn lower_template(
    document: &Document,
    element: NodeId,
    edition: Edition,
    version: DeclaredVersion,
) -> Result<crate::xslt::golden_semantics_experiment::Template, CompileFailure> {
    if !is_xslt_element(document, element, "template")
        || has_local_version(document, element)
        || meaningful_children(document, element)
            .iter()
            .any(|child| is_xslt_element(document, *child, "param"))
    {
        return Err(unsupported(
            "FXAR0031",
            "template shape or parameter defaults outside probe",
            document.location(element),
        ));
    }
    crate::compile::golden_stylesheet_experiment::compile_template_with_body(
        document,
        element,
        &mut |document, parent, excluded, bindings| {
            if !excluded.is_empty() || !bindings.is_empty() {
                return Err(unsupported(
                    "FXAR0031",
                    "template bindings outside probe",
                    document.location(parent),
                ));
            }
            lower_body(document, parent, edition, version, 0)
        },
    )
}

#[cfg(test)]
fn lower_at_depth(
    document: &Document,
    element: NodeId,
    edition: Edition,
    version: DeclaredVersion,
    depth: usize,
) -> Result<Vec<Instruction>, CompileFailure> {
    if depth >= 16 || has_local_version(document, element) {
        return Err(unsupported(
            "FXAR0031",
            "probe depth or local-version boundary",
            document.location(element),
        ));
    }
    let name = document.name(element).expect("probe receives an element");
    if name.namespace.as_deref() != Some(XSLT_NAMESPACE) {
        return Err(unsupported(
            "FXAR0031",
            "probe requires an XSLT instruction",
            document.location(element),
        ));
    }
    let legacy_known = is_xslt10_template_element(&name.local);
    // This is deliberately not a modern instruction catalog. Only sequence is
    // the known-newer witness; foo is the unknown witness. Refuse other names.
    if !legacy_known && !matches!(name.local.as_str(), "sequence" | "foo") {
        return Err(unsupported(
            "FXAR0031",
            "instruction outside recognition probe",
            document.location(element),
        ));
    }
    let known = legacy_known || matches!(edition, Edition::Modern) && name.local == "sequence";
    if known {
        if matches!(name.local.as_str(), "if" | "choose") {
            let mut condition =
                |document: &Document, element, expression: &str, location: &SourceLocation| {
                    lower_condition(document, element, edition, expression, location)
                };
            let instruction = if name.local == "if" {
                compile_if_with_lowerers(
                    document,
                    element,
                    &mut conditional_instruction_compiler::validate_conditional_attributes,
                    &mut condition,
                    &mut |document, parent| lower_body(document, parent, edition, version, depth),
                )
            } else {
                conditional_instruction_compiler::compile_choose_with_lowerers(
                    document,
                    element,
                    &mut conditional_instruction_compiler::validate_conditional_attributes,
                    &mut condition,
                    &mut |document, parent, _| {
                        lower_body(document, parent, edition, version, depth)
                    },
                )
            };
            return instruction.map(|instruction| vec![instruction]);
        }
        if !matches!(name.local.as_str(), "text" | "sequence") {
            return Err(unsupported(
                "FXAR0031",
                "known instruction outside lowering probe",
                document.location(element),
            ));
        }
        return compile_xslt_instruction(document, element, &name.local, &mut Vec::new())
            .map(|instruction| vec![instruction]);
    }
    let forward = match edition {
        Edition::Legacy => !matches!(version, DeclaredVersion::One),
        Edition::Modern => matches!(version, DeclaredVersion::Four),
    };
    if !forward {
        // Probe classification, not selection of a production diagnostic code.
        return Err(invalid(
            "FXAR0031",
            "unknown instruction outside forward-compatible processing",
            document.location(element),
        ));
    }
    // XSLT 3.0 3.10 does not inherit 1.0's missing-fallback deferral.
    if matches!(edition, Edition::Modern)
        && !document
            .children(element)
            .iter()
            .any(|child| is_xslt_element(document, *child, "fallback"))
    {
        return Err(invalid(
            "FXAR0031",
            "modern unknown instruction requires a direct fallback",
            document.location(element),
        ));
    }
    compile_forward_fallbacks_with_body(
        document,
        element,
        &name.local,
        &mut |document, fallback| {
            if has_local_version(document, fallback) {
                return Err(unsupported(
                    "FXAR0031",
                    "local-version fallback outside probe",
                    document.location(fallback),
                ));
            }
            lower_body(document, fallback, edition, version, depth)
        },
    )
}

#[cfg(test)]
fn lower_body(
    document: &Document,
    parent: NodeId,
    edition: Edition,
    version: DeclaredVersion,
    depth: usize,
) -> Result<Vec<Instruction>, CompileFailure> {
    if has_local_version(document, parent) {
        return Err(unsupported(
            "FXAR0031",
            "local-version body outside probe",
            document.location(parent),
        ));
    }
    compile_sequence_with_element_lowerer(
        document,
        parent,
        &[],
        &[],
        &mut |document, child, _, instructions| {
            instructions.extend(lower_at_depth(
                document,
                child,
                edition,
                version,
                depth + 1,
            )?);
            Ok(())
        },
    )
}

#[cfg(test)]
fn lower_condition(
    document: &Document,
    element: NodeId,
    edition: Edition,
    expression: &str,
    location: &SourceLocation,
) -> Result<BooleanExpression, CompileFailure> {
    if has_local_version(document, element) {
        return Err(unsupported(
            "FXAR0031",
            "local-version condition outside probe",
            location,
        ));
    }
    if let Some(xslt10_static_introspection_compiler::StaticIntrospectionValue::Boolean(value)) =
        introspect(document, element, edition, expression)
    {
        return Ok(BooleanExpression::Constant(value));
    }
    if !matches!(expression.trim(), "true()" | "false()") {
        return Err(unsupported(
            "FXAR0031",
            "conditional expression outside probe",
            location,
        ));
    }
    compile_boolean_test(document, element, expression, location)
}

// Refuse both lexical forms until edition-specific scope is integrated.
fn has_local_version(document: &Document, element: NodeId) -> bool {
    optional_attribute(document, element, Some(XSLT_NAMESPACE), "version").is_some()
        || optional_attribute(document, element, None, "version").is_some()
}

// Hypothetical policy witnesses, not supported processor-version claims or
// a complete availability catalog. Modern sequence capability is unproved.
#[cfg(test)]
fn introspect(
    document: &Document,
    element: NodeId,
    edition: Edition,
    expression: &str,
) -> Option<xslt10_static_introspection_compiler::StaticIntrospectionValue> {
    use xslt10_static_introspection_compiler::{
        StaticIntrospectionValue as Value, fold_with_policy,
    };
    fold_with_policy(
        document,
        element,
        expression,
        |function, namespace, local| match (function, namespace, local) {
            ("system-property", Some(XSLT_NAMESPACE), "version") => Some(Value::String(
                match edition {
                    Edition::Legacy => "1",
                    Edition::Modern => "3.0",
                }
                .to_owned(),
            )),
            ("element-available", Some(XSLT_NAMESPACE), "text")
            | ("function-available", None, "true") => Some(Value::Boolean(true)),
            ("element-available", Some(XSLT_NAMESPACE), "sequence")
                if matches!(edition, Edition::Legacy) =>
            {
                Some(Value::Boolean(false))
            }
            _ => None,
        },
    )
}

#[cfg(test)]
#[path = "processor_edition_reference/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "processor_edition_reference/include_chain.rs"]
mod include_chain;
