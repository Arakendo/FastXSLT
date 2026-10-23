//! Selected-profile routing only; conditional assembly and execution stay shared.
#[cfg(test)]
use super::*;
#[cfg(not(test))]
use super::{
    BooleanExpression, CompileFailure, DeclaredVersion, Document, Edition, Instruction,
    LocalVersion, NodeId, SourceLocation, StaticBindings, body, boolean_call_timing,
    compile_if_with_lowerers, conditional_instruction_compiler, invalid, is_xslt_element, refusal,
    require_probe_attribute_set, selected_introspection, selected_keys,
};

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    bindings: StaticBindings<'_>,
) -> Result<Instruction, CompileFailure> {
    if matches!(version, LocalVersion::ModernWitness(DeclaredVersion::Four)) {
        return Err(refusal(document, element));
    }
    let is_if = is_xslt_element(document, element, "if");
    let mut attributes =
        |document: &Document, element| validate_attributes(document, element, version);
    let mut condition =
        |document: &Document, element: NodeId, expression: &str, location: &SourceLocation| {
            if let Some(lookup) =
                selected_keys::lookup(document, element, expression, version, &bindings.visible())?
            {
                return Ok(BooleanExpression::Xslt10KeyLookupEffectiveBooleanValue(
                    Box::new(lookup),
                ));
            }
            boolean_call_timing::condition(
                expression,
                version,
                location,
                &bindings.visible(),
                &|expression| {
                    selected_introspection::boolean_leaf(document, element, expression, version)
                },
            )?
            .ok_or_else(|| refusal(document, element))
        };
    if is_if {
        compile_if_with_lowerers(
            document,
            element,
            &mut attributes,
            &mut condition,
            &mut |document, parent| body(document, parent, edition, version, depth + 1, bindings),
        )
    } else {
        conditional_instruction_compiler::compile_choose_with_lowerers(
            document,
            element,
            &mut attributes,
            &mut condition,
            &mut |document, parent, _| {
                // Compile every branch under the selected policy. A false test
                // does not erase static lexical errors from its body.
                body(document, parent, edition, version, depth + 1, bindings)
            },
        )
    }
}

fn validate_attributes(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
) -> Result<(), CompileFailure> {
    let has_test =
        is_xslt_element(document, element, "if") || is_xslt_element(document, element, "when");
    if matches!(version, LocalVersion::ModernWitness(_)) {
        return require_probe_attribute_set(document, element, if has_test { "test" } else { "" });
    }
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute name");
        if name.namespace.is_some()
            || matches!(
                name.local.as_str(),
                "version"
                    | "default-mode"
                    | "default-collation"
                    | "xpath-default-namespace"
                    | "exclude-result-prefixes"
            )
        {
            return Err(refusal(document, *attribute));
        }
        if has_test && name.local == "test" {
            continue;
        }
        if matches!(version, LocalVersion::LegacyNormal) {
            return Err(invalid(
                "XTSE0090",
                "unknown conditional attribute outside forward compatibility",
                document.location(*attribute),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "conditional_integration_tests.rs"]
mod tests;
