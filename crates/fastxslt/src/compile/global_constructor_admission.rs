//! Narrow legacy shared-constructor admission; no second sequence compiler.
use super::{
    CompileFailure, Document, GlobalBindingDefault, MatchPattern, NodeId, StylesheetProgram,
    XSLT_NAMESPACE, compile_global_default, document_element, instruction_compiler,
    optional_attribute, unsupported,
};

pub(super) fn validate_program(program: &StylesheetProgram) -> Result<(), CompileFailure> {
    let Some(location) = program.global_bindings.iter().find_map(|binding| {
        if let GlobalBindingDefault::SharedConstructor { location, .. } = &binding.default {
            Some(location)
        } else {
            None
        }
    }) else {
        return Ok(());
    };
    // These consumers still read completed maps directly. Do not let them
    // mistake the constructor's deliberately empty map for a completed store.
    let parameterized = program
        .root_template
        .iter()
        .chain(program.matched_templates.iter().map(|t| &t.template))
        .chain(program.named_templates.iter().map(|t| &t.template))
        .any(|t| !t.parameters.is_empty());
    let unproven_match = program.matched_templates.iter().any(|t| {
        !matches!(
            t.pattern,
            MatchPattern::Document
                | MatchPattern::DocumentElement(_)
                | MatchPattern::DocumentChildElement(_)
                | MatchPattern::Element(_)
                | MatchPattern::ElementLocal(_)
                | MatchPattern::ElementNamespace(_)
                | MatchPattern::DescendantAnyElement
                | MatchPattern::AnyElement
                | MatchPattern::QualifiedElementPathAlternatives(_)
        )
    });
    if parameterized || unproven_match {
        return Err(unsupported(
            "FXST1015",
            "shared global constructors do not yet admit template parameters or this match pattern",
            location,
        ));
    }
    Ok(())
}

pub(super) fn compile_default(
    document: &Document,
    element: NodeId,
    label: &str,
) -> Result<GlobalBindingDefault, CompileFailure> {
    match compile_global_default(document, element, label) {
        Err(original) if original.code == "FXST1015" => {
            let root = document_element(document)?;
            if optional_attribute(document, root, None, "version") != Some("1.0")
                || optional_attribute(document, element, None, "select").is_some()
                || optional_attribute(document, element, None, "as").is_some()
                || document.children(root).iter().any(|child| {
                    document.name(*child).is_some_and(|name| {
                        name.namespace.as_deref() == Some(XSLT_NAMESPACE)
                            && matches!(
                                name.local.as_str(),
                                "include" | "import" | "namespace-alias" | "attribute-set"
                            )
                    })
                })
            {
                return Err(original);
            }
            let body = instruction_compiler::compile_sequence(document, element)?;
            if body.is_empty() {
                return Err(original);
            }
            Ok(GlobalBindingDefault::SharedConstructor {
                body: body.into_boxed_slice(),
                location: document.location(element).clone(),
            })
        }
        result => result,
    }
}
