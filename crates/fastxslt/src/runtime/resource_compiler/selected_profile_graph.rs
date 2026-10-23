//! Private bounded selected-legacy graph routing; no baseline fallback.
//! The incomplete R0 profile remains gated pending campaign review.
#[cfg(test)]
use super::*;
#[cfg(not(test))]
use super::{
    ExecutionFailure, FailureCategory, InvocationControl, LoadedStylesheetModule,
    StylesheetDependencyKind, StylesheetProgram, compile_failure, failure_at,
};
use crate::compile::golden_stylesheet_experiment::{
    compile_selected_legacy_import_module, compile_selected_legacy_include_module,
    compile_selected_legacy_mixed_module, compile_selected_legacy_resolved_pair,
    stylesheet_preprocessing::StrictPreprocessingFailure,
};

fn limitation(graph: &LoadedStylesheetModule, detail: &str) -> ExecutionFailure {
    failure_at(
        "FXHP0001",
        FailureCategory::Unsupported,
        None,
        graph.document.location(graph.root).clone(),
        detail.to_owned(),
    )
}

fn require_entry(graph: &LoadedStylesheetModule) -> Result<(), ExecutionFailure> {
    if graph.document.parent(graph.root) != Some(graph.document.document_node()) {
        return Err(limitation(
            graph,
            "selected-profile harness: embedded stylesheet root not integrated",
        ));
    }
    // A simplified stylesheet is valid XSLT but outside the frozen entry shape.
    // Do not send it to require_stylesheet_root and report it as invalid XML/XSLT.
    let is_literal = graph.document.name(graph.root).is_some_and(|name| {
        name.namespace.as_deref() != Some("http://www.w3.org/1999/XSL/Transform")
    });
    if is_literal
        && graph
            .document
            .attributes(graph.root)
            .iter()
            .any(|attribute| {
                graph.document.name(*attribute).is_some_and(|name| {
                    name.namespace.as_deref() == Some("http://www.w3.org/1999/XSL/Transform")
                        && name.local == "version"
                })
            })
    {
        return Err(failure_at(
            "FXAR0031",
            FailureCategory::Unsupported,
            None,
            graph.document.location(graph.root).clone(),
            "selected legacy simplified stylesheet entry is not integrated".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn compile(
    graph: &LoadedStylesheetModule,
    control: &mut InvocationControl,
    complete: bool,
) -> Result<StylesheetProgram, ExecutionFailure> {
    require_entry(graph)?;
    let convert = |error| match error {
        StrictPreprocessingFailure::Compile(error) => compile_failure(error),
        StrictPreprocessingFailure::Control(error) => {
            let mut failure = super::super::control_failure(error, &graph.identity);
            failure.request_id = None;
            failure.location = Some(graph.document.location(graph.root).clone());
            failure
        }
    };
    if complete
        && let [child] = graph.dependencies.as_slice()
        && child.dependencies.is_empty()
    {
        require_entry(child)?;
        return compile_selected_legacy_resolved_pair(&graph.document, &child.document, control)
            .map_err(convert);
    }
    let mut children = graph
        .dependencies
        .iter()
        .map(|child| compile(child, control, false))
        .collect::<Result<Vec<_>, _>>()?;
    let kinds: Vec<_> = graph
        .dependencies
        .iter()
        .map(|child| child.dependency_kind)
        .collect();
    let program = if kinds
        .iter()
        .all(|kind| *kind == Some(StylesheetDependencyKind::Import))
    {
        compile_selected_legacy_import_module(&graph.document, children, control, complete)
    } else if kinds
        .iter()
        .all(|kind| *kind == Some(StylesheetDependencyKind::Include))
    {
        compile_selected_legacy_include_module(&graph.document, children, control)
    } else if kinds
        == [
            Some(StylesheetDependencyKind::Import),
            Some(StylesheetDependencyKind::Include),
        ]
    {
        let included = children.pop().unwrap();
        let imported = children.pop().unwrap();
        compile_selected_legacy_mixed_module(&graph.document, imported, included, control)
    } else {
        return Err(limitation(
            graph,
            "selected-profile harness: dependency composition shape not integrated",
        ));
    };
    program.map_err(convert)
}
