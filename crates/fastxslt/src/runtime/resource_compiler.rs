//! Adapts an admitted stylesheet resource into the private compiled program.

use crate::compile::golden_stylesheet_experiment::{
    CompileCategory, CompileFailure, compile_stylesheet, compile_stylesheet_with_single_include,
    single_include_reference,
};
use crate::resources::{ResolutionFailure, ResolutionLimits, ResourceSnapshot, SnapshotResolver};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::stylesheet_dependency_loader::{
    DependencyFailure, DependencyLimits, load_include_graph,
};
use super::{ExecutionFailure, FailureCategory, failure, failure_at};

const DEPENDENCY_LIMITS: DependencyLimits = DependencyLimits::new(1, 2, 1_048_576);

#[cfg(test)]
pub(in crate::runtime) fn compile_resource(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    compile_resource_with_denied(snapshot, stylesheet_id, std::iter::empty())
}

pub(in crate::runtime) fn compile_resource_with_denied(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
    denied: impl IntoIterator<Item = String>,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let mut resolver = SnapshotResolver::new(snapshot, denied, ResolutionLimits::new(2));
    compile_resource_with_resolver(&mut resolver, stylesheet_id)
}

fn compile_resource_with_resolver(
    resolver: &mut SnapshotResolver<'_>,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let mut graph = load_include_graph(resolver, stylesheet_id, DEPENDENCY_LIMITS)
        .map_err(dependency_failure)?;
    debug_assert_eq!(graph.identity, stylesheet_id);
    if graph.includes.is_empty() {
        return compile_stylesheet(&graph.document).map_err(compile_failure);
    }
    if graph.includes.len() != 1 {
        let failure = single_include_reference(&graph.document)
            .expect_err("multiple includes must remain outside the private compiler slice");
        return Err(compile_failure(failure));
    }
    let included = graph.includes.pop().expect("one included module");
    compile_stylesheet_with_single_include(&graph.document, &included.document)
        .map_err(compile_failure)
}

fn dependency_failure(error: DependencyFailure) -> ExecutionFailure {
    match error {
        DependencyFailure::Resolution { error, location } => resolution_failure(error, location),
        DependencyFailure::Fragment { identity, location } => dependency_failure_at(
            "FXRS1001",
            FailureCategory::Unsupported,
            location,
            format!(
                "fragment selection for included stylesheet modules is unsupported: {identity}"
            ),
        ),
        DependencyFailure::ModuleLimit { maximum, location } => dependency_failure_at(
            "FXRS0006",
            FailureCategory::Limit,
            location,
            format!("stylesheet dependency module limit is {maximum}"),
        ),
        DependencyFailure::DepthLimit { maximum, location } => dependency_failure_at(
            "FXRS0006",
            FailureCategory::Limit,
            location,
            format!("stylesheet dependency depth limit is {maximum}"),
        ),
        DependencyFailure::ByteLimit {
            attempted,
            maximum,
            location,
        } => dependency_failure_at(
            "FXRS0006",
            FailureCategory::Limit,
            location,
            format!("stylesheet dependency bytes {attempted} exceed limit {maximum}"),
        ),
        DependencyFailure::ByteCountOverflow { location } => dependency_failure_at(
            "FXRS0006",
            FailureCategory::Limit,
            location,
            "stylesheet dependency byte accounting overflowed".to_owned(),
        ),
        DependencyFailure::Cycle { identity, location } => dependency_failure_at(
            "FXST0030",
            FailureCategory::Invalid,
            location,
            format!("stylesheet include cycle reaches {identity}"),
        ),
        DependencyFailure::InvalidXml {
            identity,
            detail,
            location,
        } => dependency_failure_at(
            "FXXM0001",
            FailureCategory::Invalid,
            location,
            format!("stylesheet XML is invalid at {identity}: {detail}"),
        ),
        DependencyFailure::InvalidXdm {
            identity,
            detail,
            location,
        } => dependency_failure_at(
            "FXXD0001",
            FailureCategory::Invalid,
            location,
            format!("stylesheet XDM construction failed at {identity}: {detail}"),
        ),
        DependencyFailure::InvalidDeclaration(error) => compile_failure(error),
    }
}

fn dependency_failure_at(
    code: &'static str,
    category: FailureCategory,
    location: Option<crate::xdm::owned_tree_experiment::SourceLocation>,
    detail: String,
) -> ExecutionFailure {
    match location {
        Some(location) => failure_at(code, category, None, location, detail),
        None => failure(code, category, None, detail),
    }
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

fn resolution_failure(
    error: ResolutionFailure,
    location: Option<crate::xdm::owned_tree_experiment::SourceLocation>,
) -> ExecutionFailure {
    match error {
        ResolutionFailure::Missing { identity } => dependency_failure_at(
            "FXRS0002",
            FailureCategory::MissingResource,
            location,
            format!("stylesheet is not admitted: {identity}"),
        ),
        ResolutionFailure::Denied { identity } => dependency_failure_at(
            "FXRS0003",
            FailureCategory::Denied,
            location,
            format!("stylesheet authority is denied: {identity}"),
        ),
        ResolutionFailure::AttemptLimit { maximum } => dependency_failure_at(
            "FXRS0006",
            FailureCategory::Limit,
            location,
            format!("stylesheet resolution attempt limit is {maximum}"),
        ),
        ResolutionFailure::InvalidReference { reference } => dependency_failure_at(
            "FXRS0004",
            FailureCategory::Invalid,
            location,
            format!("stylesheet identity is not a valid absolute resource URI: {reference}"),
        ),
        ResolutionFailure::InvalidBase { base } => dependency_failure_at(
            "FXRS1001",
            FailureCategory::Unsupported,
            location,
            format!("stylesheet base identity is not a supported absolute IRI: {base}"),
        ),
        ResolutionFailure::ResolutionFailed { base, reference } => dependency_failure_at(
            "FXRS0004",
            FailureCategory::Invalid,
            location,
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
