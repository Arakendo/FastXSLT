//! Selected-profile routing only; conditional assembly and execution stay shared.
use super::*;

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    edition: Edition,
    version: LocalVersion,
    depth: usize,
    bindings: &[String],
) -> Result<Instruction, CompileFailure> {
    if matches!(version, LocalVersion::ModernWitness(DeclaredVersion::Four)) {
        return Err(refusal(document, element));
    }
    let is_if = is_xslt_element(document, element, "if");
    // Prevent baseline local-version/optional-attribute helpers from silently
    // selecting policy. Broader conditional static context is not admitted here.
    require_probe_attribute_set(document, element, if is_if { "test" } else { "" })?;
    if !is_if {
        for child in meaningful_children(document, element) {
            if is_xslt_element(document, child, "when") {
                require_probe_attribute_set(document, child, "test")?;
            } else if is_xslt_element(document, child, "otherwise") {
                require_probe_attribute_set(document, child, "")?;
            }
        }
    }
    let mut condition =
        |document: &Document, element: NodeId, expression: &str, location: &SourceLocation| {
            boolean_call_timing::condition(expression, version, location, bindings)?
                .ok_or_else(|| refusal(document, element))
        };
    if is_if {
        compile_if_with_lowerers(
            document,
            element,
            &mut condition,
            &mut |document, parent| body(document, parent, edition, version, depth + 1, bindings),
        )
    } else {
        conditional_instruction_compiler::compile_choose_with_lowerers(
            document,
            element,
            &mut condition,
            &mut |document, parent, _| {
                // Compile every branch under the selected policy. A false test
                // does not erase static lexical errors from its body.
                body(document, parent, edition, version, depth + 1, bindings)
            },
        )
    }
}

#[path = "conditional_integration_tests.rs"]
mod tests;
