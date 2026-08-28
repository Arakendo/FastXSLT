//! Adapts an admitted stylesheet resource into the private compiled program.

use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::resources::{ResolutionFailure, ResolutionLimits, ResourceSnapshot, SnapshotResolver};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::parse_document;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::{ExecutionFailure, FailureCategory, XML_LIMITS, failure, failure_at};

pub(in crate::runtime) fn compile_resource(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let mut resolver =
        SnapshotResolver::new(snapshot, std::iter::empty(), ResolutionLimits::new(1));
    let bytes = resolver
        .resolve(stylesheet_id)
        .map_err(resolution_failure)?;
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

fn resolution_failure(error: ResolutionFailure) -> ExecutionFailure {
    match error {
        ResolutionFailure::Missing { identity } => failure(
            "FXRS0002",
            FailureCategory::MissingResource,
            None,
            format!("stylesheet is not admitted: {identity}"),
        ),
        ResolutionFailure::Denied { identity } => failure(
            "FXRS0003",
            FailureCategory::Denied,
            None,
            format!("stylesheet authority is denied: {identity}"),
        ),
        ResolutionFailure::AttemptLimit { maximum } => failure(
            "FXRS0006",
            FailureCategory::Limit,
            None,
            format!("stylesheet resolution attempt limit is {maximum}"),
        ),
        ResolutionFailure::EmptyReference => failure(
            "FXRS0004",
            FailureCategory::Invalid,
            None,
            "stylesheet identity is empty",
        ),
        ResolutionFailure::InvalidReference { reference } => failure(
            "FXRS0004",
            FailureCategory::Invalid,
            None,
            format!("stylesheet identity is not a valid absolute resource URI: {reference}"),
        ),
        ResolutionFailure::RelativeReferenceUnsupported { reference }
        | ResolutionFailure::FragmentUnsupported { reference } => failure(
            "FXRS1001",
            FailureCategory::Unsupported,
            None,
            format!(
                "relative or fragment stylesheet resolution is outside the private snapshot slice: {reference}"
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::resources::{ResourceLimits, ResourceSetBuilder};

    use super::compile_resource;
    use crate::runtime::golden_runtime_experiment::FailureCategory;

    #[test]
    fn compilation_rejects_unqualified_admitted_identity_without_ambient_fallback() {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 512, 512));
        resources
            .admit(
                "stylesheet.xsl",
                br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#
                    .to_vec(),
            )
            .expect("admit unqualified logical identity");

        let failure = compile_resource(&resources.seal(), "stylesheet.xsl")
            .expect_err("compilation must require a qualified identity");

        assert_eq!(failure.code, "FXRS1001");
        assert_eq!(failure.category, FailureCategory::Unsupported);
        assert!(failure.detail.contains("stylesheet.xsl"));
    }
}
