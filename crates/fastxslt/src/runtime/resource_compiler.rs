//! Adapts an admitted stylesheet resource into the private compiled program.

use crate::compile::golden_stylesheet_experiment::{
    CompileCategory, CompileFailure, compile_stylesheet, compile_stylesheet_with_single_include,
    single_include_reference,
};
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
        SnapshotResolver::new(snapshot, std::iter::empty(), ResolutionLimits::new(2));
    compile_resource_with_resolver(&mut resolver, stylesheet_id)
}

fn compile_resource_with_resolver(
    resolver: &mut SnapshotResolver<'_>,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let resource = resolver
        .resolve_from(stylesheet_id, "")
        .map_err(resolution_failure)?;
    debug_assert!(resource.fragment.is_none());
    let parsed =
        parse_document(&resource.identity, resource.bytes, XML_LIMITS).map_err(|error| {
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
    let Some(include) = single_include_reference(&document).map_err(compile_failure)? else {
        return compile_stylesheet(&document).map_err(compile_failure);
    };
    let included_resource = resolver
        .resolve_from(&resource.identity, &include.href)
        .map_err(resolution_failure)?;
    if included_resource.fragment.is_some() {
        return Err(failure_at(
            "FXRS1001",
            FailureCategory::Unsupported,
            None,
            include.location.clone(),
            "fragment selection for included stylesheet modules is outside the private slice",
        ));
    }
    let included_parsed = parse_document(
        &included_resource.identity,
        included_resource.bytes,
        XML_LIMITS,
    )
    .map_err(|error| {
        failure_at(
            "FXXM0001",
            FailureCategory::Invalid,
            None,
            include.location.clone(),
            format!("included stylesheet XML is invalid: {error:?}"),
        )
    })?;
    let included_document = Document::from_parsed(included_parsed).map_err(|error| {
        failure_at(
            "FXXD0001",
            FailureCategory::Invalid,
            None,
            include.location.clone(),
            format!("included stylesheet XDM construction failed: {error:?}"),
        )
    })?;
    compile_stylesheet_with_single_include(&document, &included_document).map_err(compile_failure)
}

fn compile_failure(error: CompileFailure) -> ExecutionFailure {
    let detail = format!(
        "{} at {}:{}..{}",
        error.detail, error.location.resource, error.location.span.start, error.location.span.end
    );
    failure_at(
        error.code,
        match error.category {
            CompileCategory::Invalid => FailureCategory::Invalid,
            CompileCategory::Unsupported => FailureCategory::Unsupported,
        },
        None,
        error.location,
        detail,
    )
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
        ResolutionFailure::InvalidReference { reference } => failure(
            "FXRS0004",
            FailureCategory::Invalid,
            None,
            format!("stylesheet identity is not a valid absolute resource URI: {reference}"),
        ),
        ResolutionFailure::InvalidBase { base } => failure(
            "FXRS1001",
            FailureCategory::Unsupported,
            None,
            format!("stylesheet base identity is not a supported absolute IRI: {base}"),
        ),
        ResolutionFailure::ResolutionFailed { base, reference } => failure(
            "FXRS0004",
            FailureCategory::Invalid,
            None,
            format!(
                "stylesheet reference cannot be resolved using RFC 3986: {reference} against {base}"
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::resources::{
        ResolutionLimits, ResourceLimits, ResourceSetBuilder, SnapshotResolver,
    };

    use super::{compile_resource, compile_resource_with_resolver};
    use crate::runtime::golden_runtime_experiment::FailureCategory;

    const STYLESHEET_ID: &str = "urn:fastxslt:test:stylesheet";
    const STYLESHEET: &[u8] = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#;

    fn stylesheet_snapshot() -> crate::resources::ResourceSnapshot {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 512, 512));
        resources
            .admit(STYLESHEET_ID, STYLESHEET.to_vec())
            .expect("admit qualified stylesheet");
        resources.seal()
    }

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

    #[test]
    fn compilation_preserves_explicit_denial_without_revealing_admission() {
        let snapshot = stylesheet_snapshot();
        let mut admitted_denial = SnapshotResolver::new(
            &snapshot,
            [STYLESHEET_ID.to_owned()],
            ResolutionLimits::new(1),
        );
        let admitted_failure = compile_resource_with_resolver(&mut admitted_denial, STYLESHEET_ID)
            .expect_err("explicitly denied admitted stylesheet must fail");

        let mut missing_denial = SnapshotResolver::new(
            &snapshot,
            ["urn:fastxslt:test:not-admitted".to_owned()],
            ResolutionLimits::new(1),
        );
        let missing_failure =
            compile_resource_with_resolver(&mut missing_denial, "urn:fastxslt:test:not-admitted")
                .expect_err("explicitly denied missing stylesheet must fail identically");

        for failure in [admitted_failure, missing_failure] {
            assert_eq!(failure.code, "FXRS0003");
            assert_eq!(failure.category, FailureCategory::Denied);
        }
    }

    #[test]
    fn compilation_preserves_resolution_attempt_exhaustion() {
        let snapshot = stylesheet_snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(1));

        let missing =
            compile_resource_with_resolver(&mut resolver, "urn:fastxslt:test:not-admitted")
                .expect_err("first lookup must report the missing resource");
        assert_eq!(missing.code, "FXRS0002");
        assert_eq!(missing.category, FailureCategory::MissingResource);

        let exhausted = compile_resource_with_resolver(&mut resolver, STYLESHEET_ID)
            .expect_err("second lookup must fail before accessing admitted bytes");
        assert_eq!(exhausted.code, "FXRS0006");
        assert_eq!(exhausted.category, FailureCategory::Limit);
    }

    #[test]
    fn compilation_reports_the_resolved_missing_include_without_ambient_fallback() {
        const PRINCIPAL: &str = "https://example.invalid/styles/main.xsl";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 512, 512));
        resources
            .admit(
                PRINCIPAL,
                br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="missing.xsl"/></xsl:stylesheet>"#
                    .to_vec(),
            )
            .expect("admit principal stylesheet only");

        let failure = compile_resource(&resources.seal(), PRINCIPAL)
            .expect_err("missing included stylesheet must remain an operation failure");

        assert_eq!(failure.code, "FXRS0002");
        assert_eq!(failure.category, FailureCategory::MissingResource);
        assert!(
            failure
                .detail
                .contains("https://example.invalid/styles/missing.xsl")
        );
    }
}
