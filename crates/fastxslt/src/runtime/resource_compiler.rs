//! Adapts an admitted stylesheet resource into the private compiled program.

use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{ExecutionFailure, FailureCategory, XML_LIMITS, failure, failure_at};

pub(in crate::runtime) fn compile_resource(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let bytes = snapshot.get(stylesheet_id).ok_or_else(|| {
        failure(
            "FXRS0002",
            FailureCategory::MissingResource,
            None,
            format!("stylesheet is not admitted: {stylesheet_id}"),
        )
    })?;
    let parsed = parse_document(stylesheet_id, bytes, XML_LIMITS).map_err(|error| {
        failure(
            "FXXM0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XML is invalid: {error:?}"),
        )
    })?;
    let document = Document::from_parsed(parsed).map_err(|error| {
        failure(
            "FXXD0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XDM construction failed: {error:?}"),
        )
    })?;
    compile_stylesheet(&document).map_err(|error| {
        failure_at(
            error.code,
            match error.category {
                crate::compile::golden_stylesheet_experiment::CompileCategory::Invalid => {
                    FailureCategory::Invalid
                }
                crate::compile::golden_stylesheet_experiment::CompileCategory::Unsupported => {
                    FailureCategory::Unsupported
                }
            },
            None,
            error.location.clone(),
            format!(
                "{} at {}:{}..{}",
                error.detail,
                error.location.resource,
                error.location.span.start,
                error.location.span.end
            ),
        )
    })
}
