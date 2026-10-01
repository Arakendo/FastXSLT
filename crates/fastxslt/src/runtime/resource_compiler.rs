//! Adapts an admitted stylesheet resource into the private compiled program.

use crate::compile::golden_stylesheet_experiment::{
    CompileCategory, CompileFailure, StylesheetDependencyKind, compile_stylesheet,
    compile_stylesheet_module_at_unlinked, compile_stylesheet_with_import_and_include,
    compile_stylesheet_with_imported_and_included_programs_at,
    compile_stylesheet_with_imported_programs_at, compile_stylesheet_with_imports,
    compile_stylesheet_with_included_programs_at,
    compile_stylesheet_with_single_imported_program_at, compile_stylesheet_with_single_include,
    compile_stylesheet_with_single_include_program_at,
    compile_stylesheet_with_two_included_programs_at, validate_import_order_at,
};
use crate::resources::{ResolutionFailure, ResolutionLimits, ResourceSnapshot, SnapshotResolver};
use crate::xml::internal_subset::InternalSubsetLimits;
use crate::xml::quick_xml_experiment::ParseLimits;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::stylesheet_dependency_loader::{
    DependencyFailure, DependencyLimits, LoadedStylesheetModule, StylesheetExternalSubsetLimits,
    load_stylesheet_dependency_graph,
};
use super::{ExecutionFailure, FailureCategory, failure, failure_at};

#[cfg(test)]
const DEFAULT_DEPENDENCY_LIMITS: DependencyLimits = DependencyLimits::new(2, 5, 1_048_576);
#[cfg(test)]
const DEFAULT_RESOLUTION_ATTEMPTS: usize = 5;

#[derive(Clone, Copy)]
pub(in crate::runtime) struct StylesheetCompileLimits {
    dependency: DependencyLimits,
    resolution_attempts: usize,
    xml: ParseLimits,
    internal_subset: Option<InternalSubsetLimits>,
    external_subset: Option<StylesheetExternalSubsetLimits>,
}

impl StylesheetCompileLimits {
    pub(in crate::runtime) const fn new(
        max_dependency_depth: usize,
        max_modules: usize,
        max_dependency_bytes: usize,
        max_resolution_attempts: usize,
        xml: ParseLimits,
    ) -> Self {
        Self {
            dependency: DependencyLimits::new(
                max_dependency_depth,
                max_modules,
                max_dependency_bytes,
            ),
            resolution_attempts: max_resolution_attempts,
            xml,
            internal_subset: None,
            external_subset: None,
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn with_internal_subset(
        mut self,
        limits: InternalSubsetLimits,
    ) -> Self {
        self.internal_subset = Some(limits);
        self
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn with_external_subset(
        mut self,
        reference: &'static str,
        max_bytes: usize,
    ) -> Self {
        self.external_subset = Some(StylesheetExternalSubsetLimits {
            reference,
            max_bytes,
        });
        self
    }
}

#[cfg(test)]
pub(in crate::runtime) fn compile_resource(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    compile_resource_with_denied(snapshot, stylesheet_id, std::iter::empty())
}

#[cfg(test)]
pub(in crate::runtime) fn compile_resource_with_denied(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
    denied: impl IntoIterator<Item = String>,
) -> Result<StylesheetProgram, ExecutionFailure> {
    compile_resource_with_denied_and_limits(
        snapshot,
        stylesheet_id,
        denied,
        StylesheetCompileLimits::new(
            2,
            5,
            1_048_576,
            DEFAULT_RESOLUTION_ATTEMPTS,
            super::XML_LIMITS,
        ),
    )
}

pub(in crate::runtime) fn compile_resource_with_denied_and_limits(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
    denied: impl IntoIterator<Item = String>,
    limits: StylesheetCompileLimits,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let mut resolver = SnapshotResolver::new(
        snapshot,
        denied,
        ResolutionLimits::new(limits.resolution_attempts),
    );
    compile_resource_with_resolver_and_limits(
        &mut resolver,
        stylesheet_id,
        limits.dependency,
        limits.xml,
        limits.internal_subset,
        limits.external_subset,
    )
}

#[cfg(test)]
fn compile_resource_with_resolver(
    resolver: &mut SnapshotResolver<'_>,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    compile_resource_with_resolver_and_limits(
        resolver,
        stylesheet_id,
        DEFAULT_DEPENDENCY_LIMITS,
        super::XML_LIMITS,
        None,
        None,
    )
}

fn compile_resource_with_resolver_and_limits(
    resolver: &mut SnapshotResolver<'_>,
    stylesheet_id: &str,
    dependency_limits: DependencyLimits,
    xml_limits: ParseLimits,
    internal_subset_limits: Option<InternalSubsetLimits>,
    external_subset_limits: Option<StylesheetExternalSubsetLimits>,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let graph = load_stylesheet_dependency_graph(
        resolver,
        stylesheet_id,
        dependency_limits,
        xml_limits,
        internal_subset_limits,
        external_subset_limits,
    )
    .map_err(dependency_failure)?;
    debug_assert_eq!(graph.identity, stylesheet_id);
    let mut program = compile_loaded_graph(&graph).map_err(compile_failure)?;
    crate::compile::golden_stylesheet_experiment::finalize_attribute_sets(&mut program)
        .map_err(compile_failure)?;
    Ok(program)
}

fn compile_loaded_graph(
    graph: &LoadedStylesheetModule,
) -> Result<StylesheetProgram, CompileFailure> {
    if graph.dependencies.is_empty() {
        return compile_stylesheet(&graph.document);
    }
    validate_loaded_import_order(graph)?;
    let is_include_only = graph
        .dependencies
        .iter()
        .all(|dependency| dependency.dependency_kind == Some(StylesheetDependencyKind::Include));
    validate_sibling_dependency_shape(graph, is_include_only)?;
    if is_include_only {
        if let Some(program) = compile_homogeneous_module(graph) {
            return program;
        }
    }
    if let Some(program) = compile_homogeneous_dependency_tree(graph) {
        return program;
    }
    if let Some(program) = compile_import_then_include_tree(graph) {
        return program;
    }
    if let Some(program) = compile_linear_dependency_chain(graph) {
        return program;
    }
    if let Some(program) = compile_two_include_leaf_import_graph(graph) {
        return program;
    }
    if let Some(program) = compile_two_import_leaf_import_graph(graph) {
        return program;
    }
    if let Some(program) = compile_nested_include_chain(graph) {
        return program;
    }
    if let Some(program) = compile_nested_import_chain(graph) {
        return program;
    }
    for dependency in &graph.dependencies {
        if !dependency.dependencies.is_empty() {
            return Err(CompileFailure {
                code: "FXST1027",
                category: CompileCategory::Unsupported,
                detail: "nested stylesheet dependencies are outside the private compiler slice"
                    .to_owned(),
                location: dependency
                    .document
                    .location(dependency.document.document_node())
                    .clone(),
            });
        }
    }
    let dependency_kinds = graph
        .dependencies
        .iter()
        .map(|dependency| dependency.dependency_kind.expect("dependency kind"))
        .collect::<Vec<_>>();
    match dependency_kinds.as_slice() {
        [StylesheetDependencyKind::Include] => compile_stylesheet_with_single_include(
            &graph.document,
            &graph.dependencies[0].document,
            graph.dependencies[0].root,
        ),
        [
            StylesheetDependencyKind::Import,
            StylesheetDependencyKind::Include,
        ] => compile_stylesheet_with_import_and_include(
            &graph.document,
            (&graph.dependencies[0].document, graph.dependencies[0].root),
            (&graph.dependencies[1].document, graph.dependencies[1].root),
        ),
        kinds
            if kinds
                .iter()
                .all(|kind| *kind == StylesheetDependencyKind::Import) =>
        {
            let imported = graph
                .dependencies
                .iter()
                .map(|dependency| (&dependency.document, dependency.root))
                .collect::<Vec<_>>();
            compile_stylesheet_with_imports(&graph.document, &imported)
        }
        _ => Err(CompileFailure {
            code: "FXST1029",
            category: CompileCategory::Unsupported,
            detail: "mixed include/import assembly is outside the private compiler slice"
                .to_owned(),
            location: graph
                .document
                .location(graph.document.document_node())
                .clone(),
        }),
    }
}

fn validate_sibling_dependency_shape(
    graph: &LoadedStylesheetModule,
    is_include_only: bool,
) -> Result<(), CompileFailure> {
    if graph.dependencies.len() <= 2 || is_include_only {
        return Ok(());
    }
    Err(CompileFailure {
        code: "FXST1018",
        category: CompileCategory::Unsupported,
        detail: "the private slice permits at most two non-include stylesheet dependencies"
            .to_owned(),
        location: graph
            .document
            .location(graph.document.document_node())
            .clone(),
    })
}

fn compile_import_then_include_tree(
    graph: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    let [imported, included] = graph.dependencies.as_slice() else {
        return None;
    };
    if imported.dependency_kind != Some(StylesheetDependencyKind::Import)
        || included.dependency_kind != Some(StylesheetDependencyKind::Include)
        || (imported.dependencies.is_empty() && included.dependencies.is_empty())
    {
        return None;
    }
    let imported_program = compile_homogeneous_module(imported)?;
    let included_program = compile_homogeneous_module(included)?;
    Some(imported_program.and_then(|imported_program| {
        included_program.and_then(|included_program| {
            compile_stylesheet_with_imported_and_included_programs_at(
                &graph.document,
                graph.root,
                imported_program,
                included_program,
            )
        })
    }))
}

fn validate_loaded_import_order(graph: &LoadedStylesheetModule) -> Result<(), CompileFailure> {
    if graph.dependencies.is_empty() {
        return Ok(());
    }
    validate_import_order_at(&graph.document, graph.root)?;
    for dependency in &graph.dependencies {
        validate_loaded_import_order(dependency)?;
    }
    Ok(())
}

fn compile_homogeneous_dependency_tree(
    graph: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    if !graph
        .dependencies
        .iter()
        .any(|dependency| !dependency.dependencies.is_empty())
    {
        return None;
    }
    compile_homogeneous_module(graph)
}

fn compile_homogeneous_module(
    module: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    match module.dependencies.as_slice() {
        [] => Some(compile_stylesheet_module_at_unlinked(
            &module.document,
            module.root,
        )),
        [dependency] => {
            let dependency_program = compile_homogeneous_module(dependency)?;
            Some(dependency_program.and_then(|dependency_program| {
                match dependency
                    .dependency_kind
                    .expect("a loaded dependency has an edge kind")
                {
                    StylesheetDependencyKind::Include => {
                        compile_stylesheet_with_single_include_program_at(
                            &module.document,
                            module.root,
                            dependency_program,
                        )
                    }
                    StylesheetDependencyKind::Import => {
                        compile_stylesheet_with_single_imported_program_at(
                            &module.document,
                            module.root,
                            dependency_program,
                        )
                    }
                }
            }))
        }
        [first, second] if first.dependency_kind == second.dependency_kind => {
            let first_program = compile_homogeneous_module(first)?;
            let second_program = compile_homogeneous_module(second)?;
            Some(first_program.and_then(|first_program| {
                second_program.and_then(|second_program| {
                    match first
                        .dependency_kind
                        .expect("a loaded dependency has an edge kind")
                    {
                        StylesheetDependencyKind::Include => {
                            compile_stylesheet_with_two_included_programs_at(
                                &module.document,
                                module.root,
                                [first_program, second_program],
                            )
                        }
                        StylesheetDependencyKind::Import => {
                            compile_stylesheet_with_imported_programs_at(
                                &module.document,
                                module.root,
                                vec![first_program, second_program],
                            )
                        }
                    }
                })
            }))
        }
        dependencies
            if dependencies.iter().all(|dependency| {
                dependency.dependency_kind == Some(StylesheetDependencyKind::Include)
            }) =>
        {
            let included_programs = dependencies
                .iter()
                .map(compile_homogeneous_module)
                .collect::<Option<Vec<_>>>()?;
            Some(
                included_programs
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .and_then(|included_programs| {
                        compile_stylesheet_with_included_programs_at(
                            &module.document,
                            module.root,
                            included_programs,
                        )
                    }),
            )
        }
        dependencies
            if dependencies.iter().all(|dependency| {
                dependency.dependency_kind == Some(StylesheetDependencyKind::Import)
            }) =>
        {
            compile_homogeneous_import_siblings(module, dependencies)
        }
        _ => None,
    }
}

fn compile_homogeneous_import_siblings(
    module: &LoadedStylesheetModule,
    dependencies: &[LoadedStylesheetModule],
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    let imported_programs = dependencies
        .iter()
        .map(compile_homogeneous_module)
        .collect::<Option<Vec<_>>>()?;
    Some(
        imported_programs
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .and_then(|imported_programs| {
                compile_stylesheet_with_imported_programs_at(
                    &module.document,
                    module.root,
                    imported_programs,
                )
            }),
    )
}

fn compile_linear_dependency_chain(
    graph: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    let [dependency] = graph.dependencies.as_slice() else {
        return None;
    };
    if dependency.dependencies.is_empty() {
        return None;
    }
    Some((|| {
        let dependency_program = compile_linear_module(dependency)?;
        match dependency
            .dependency_kind
            .expect("a loaded dependency has an edge kind")
        {
            StylesheetDependencyKind::Include => compile_stylesheet_with_single_include_program_at(
                &graph.document,
                graph.root,
                dependency_program,
            ),
            StylesheetDependencyKind::Import => compile_stylesheet_with_single_imported_program_at(
                &graph.document,
                graph.root,
                dependency_program,
            ),
        }
    })())
}

fn compile_linear_module(
    module: &LoadedStylesheetModule,
) -> Result<StylesheetProgram, CompileFailure> {
    let [] = module.dependencies.as_slice() else {
        let [dependency] = module.dependencies.as_slice() else {
            return Err(CompileFailure {
                code: "FXST1027",
                category: CompileCategory::Unsupported,
                detail: "branching nested stylesheet dependencies are outside the private compiler slice"
                    .to_owned(),
                location: module
                    .document
                    .location(module.document.document_node())
                    .clone(),
            });
        };
        let dependency_program = compile_linear_module(dependency)?;
        return match dependency
            .dependency_kind
            .expect("a loaded dependency has an edge kind")
        {
            StylesheetDependencyKind::Include => compile_stylesheet_with_single_include_program_at(
                &module.document,
                module.root,
                dependency_program,
            ),
            StylesheetDependencyKind::Import => compile_stylesheet_with_single_imported_program_at(
                &module.document,
                module.root,
                dependency_program,
            ),
        };
    };
    compile_stylesheet_module_at_unlinked(&module.document, module.root)
}

fn compile_nested_import_chain(
    graph: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    let [dependency] = graph.dependencies.as_slice() else {
        return None;
    };
    let [nested] = dependency.dependencies.as_slice() else {
        return None;
    };
    if dependency.dependency_kind != Some(StylesheetDependencyKind::Import)
        || nested.dependency_kind != Some(StylesheetDependencyKind::Import)
        || !nested.dependencies.is_empty()
    {
        return None;
    }
    Some((|| {
        let dependency_program = compile_stylesheet_with_imports(
            &dependency.document,
            &[(&nested.document, nested.root)],
        )?;
        compile_stylesheet_with_single_imported_program_at(
            &graph.document,
            graph.root,
            dependency_program,
        )
    })())
}

fn compile_two_import_leaf_import_graph(
    graph: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    let [first, second] = graph.dependencies.as_slice() else {
        return None;
    };
    let ([first_import], [second_import]) = (
        first.dependencies.as_slice(),
        second.dependencies.as_slice(),
    ) else {
        return None;
    };
    if first.dependency_kind != Some(StylesheetDependencyKind::Import)
        || second.dependency_kind != Some(StylesheetDependencyKind::Import)
        || first_import.dependency_kind != Some(StylesheetDependencyKind::Import)
        || second_import.dependency_kind != Some(StylesheetDependencyKind::Import)
        || !first_import.dependencies.is_empty()
        || !second_import.dependencies.is_empty()
    {
        return None;
    }
    Some((|| {
        let first_program = compile_stylesheet_with_imports(
            &first.document,
            &[(&first_import.document, first_import.root)],
        )?;
        let second_program = compile_stylesheet_with_imports(
            &second.document,
            &[(&second_import.document, second_import.root)],
        )?;
        compile_stylesheet_with_imported_programs_at(
            &graph.document,
            graph.root,
            vec![first_program, second_program],
        )
    })())
}

fn compile_two_include_leaf_import_graph(
    graph: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    let [first, second] = graph.dependencies.as_slice() else {
        return None;
    };
    let ([first_import], [second_import]) = (
        first.dependencies.as_slice(),
        second.dependencies.as_slice(),
    ) else {
        return None;
    };
    if first.dependency_kind != Some(StylesheetDependencyKind::Include)
        || second.dependency_kind != Some(StylesheetDependencyKind::Include)
        || first_import.dependency_kind != Some(StylesheetDependencyKind::Import)
        || second_import.dependency_kind != Some(StylesheetDependencyKind::Import)
        || !first_import.dependencies.is_empty()
        || !second_import.dependencies.is_empty()
    {
        return None;
    }
    Some((|| {
        let first_program = compile_stylesheet_with_imports(
            &first.document,
            &[(&first_import.document, first_import.root)],
        )?;
        let second_program = compile_stylesheet_with_imports(
            &second.document,
            &[(&second_import.document, second_import.root)],
        )?;
        compile_stylesheet_with_two_included_programs_at(
            &graph.document,
            graph.root,
            [first_program, second_program],
        )
    })())
}

fn compile_nested_include_chain(
    graph: &LoadedStylesheetModule,
) -> Option<Result<StylesheetProgram, CompileFailure>> {
    let [dependency] = graph.dependencies.as_slice() else {
        return None;
    };
    let [nested] = dependency.dependencies.as_slice() else {
        return None;
    };
    if dependency.dependency_kind != Some(StylesheetDependencyKind::Include)
        || nested.dependency_kind != Some(StylesheetDependencyKind::Include)
        || !nested.dependencies.is_empty()
    {
        return None;
    }
    Some((|| {
        let nested_program = compile_stylesheet_module_at_unlinked(&nested.document, nested.root)?;
        let dependency_program = compile_stylesheet_with_single_include_program_at(
            &dependency.document,
            dependency.root,
            nested_program,
        )?;
        compile_stylesheet_with_single_include_program_at(
            &graph.document,
            graph.root,
            dependency_program,
        )
    })())
}

fn dependency_failure(error: DependencyFailure) -> ExecutionFailure {
    match error {
        DependencyFailure::Resolution { error, location } => resolution_failure(error, location),
        DependencyFailure::UnsupportedFragment {
            identity,
            fragment,
            location,
        } => dependency_failure_at(
            "FXRS1001",
            FailureCategory::Unsupported,
            location,
            format!("unsupported stylesheet fragment syntax: {identity}#{fragment}"),
        ),
        DependencyFailure::FragmentSelection {
            identity,
            fragment,
            matches,
            location,
        } => dependency_failure_at(
            "XTSE0165",
            FailureCategory::Invalid,
            location,
            format!(
                "stylesheet fragment must select exactly one element: {identity}#{fragment} selected {matches}"
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
            format!("stylesheet dependency cycle reaches {identity}"),
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
        DependencyFailure::XmlLimit {
            identity,
            detail,
            location,
        } => dependency_failure_at(
            "FXRS0006",
            FailureCategory::Limit,
            location,
            format!("stylesheet XML limit at {identity}: {detail}"),
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

    use super::{
        StylesheetCompileLimits, compile_resource, compile_resource_with_denied_and_limits,
        compile_resource_with_resolver,
    };
    use crate::runtime::golden_runtime_experiment::{FailureCategory, XML_LIMITS};

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

    #[test]
    fn xslt30_compilation_allows_import_after_another_top_level_declaration() {
        const PRINCIPAL: &str = "https://example.invalid/styles/main.xsl";
        const IMPORTED: &str = "https://example.invalid/styles/imported.xsl";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 1_024, 2_048));
        resources
            .admit(
                PRINCIPAL,
                br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc"><out/></xsl:template><xsl:import href="imported.xsl"/></xsl:stylesheet>"#
                    .to_vec(),
            )
            .expect("admit principal stylesheet");
        resources
            .admit(
                IMPORTED,
                br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="doc"><in/></xsl:template></xsl:stylesheet>"#
                    .to_vec(),
            )
            .expect("admit imported stylesheet");

        let program = compile_resource(&resources.seal(), PRINCIPAL)
            .expect("XSLT 3.0 permits imports after other declarations");
        assert_eq!(program.matched_templates.len(), 2);
    }

    #[test]
    fn repeated_includes_preserve_every_bounded_declaration_occurrence_in_source_order() {
        const PRINCIPAL: &str = "https://example.invalid/styles/main.xsl";
        const FIRST: &str = "https://example.invalid/styles/first.xsl";
        const SECOND: &str = "https://example.invalid/styles/second.xsl";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 4_096, 8_192));
        resources
            .admit(
                PRINCIPAL,
                br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="first.xsl"/><xsl:include href="second.xsl"/><xsl:include href="first.xsl"/></xsl:stylesheet>"#.to_vec(),
            )
            .expect("admit principal stylesheet");
        resources
            .admit(
                FIRST,
                br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item"><first/></xsl:template></xsl:stylesheet>"#.to_vec(),
            )
            .expect("admit first included stylesheet");
        resources
            .admit(
                SECOND,
                br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="item"><second/></xsl:template></xsl:stylesheet>"#.to_vec(),
            )
            .expect("admit second included stylesheet");
        let snapshot = resources.seal();

        let program = compile_resource_with_denied_and_limits(
            &snapshot,
            PRINCIPAL,
            std::iter::empty(),
            StylesheetCompileLimits::new(2, 4, 8_192, 4, XML_LIMITS),
        )
        .expect("compile three ordered include occurrences");

        assert_eq!(program.matched_templates.len(), 3);
        assert_eq!(
            program
                .matched_templates
                .iter()
                .map(|matched| matched.template.location.resource.as_str())
                .collect::<Vec<_>>(),
            [FIRST, SECOND, FIRST]
        );

        let failure = compile_resource_with_denied_and_limits(
            &snapshot,
            PRINCIPAL,
            std::iter::empty(),
            StylesheetCompileLimits::new(2, 3, 8_192, 4, XML_LIMITS),
        )
        .expect_err("repeated references must each consume the module budget");
        assert_eq!(failure.code, "FXRS0006");
        assert_eq!(failure.category, FailureCategory::Limit);
    }
}
