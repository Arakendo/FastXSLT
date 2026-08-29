//! Builds one bounded, immutable stylesheet dependency graph from a sealed snapshot.

use crate::compile::golden_stylesheet_experiment::{
    CompileFailure, StylesheetDependencyKind, discovered_stylesheet_dependencies,
};
use crate::resources::{ResolutionFailure, SnapshotResolver};
use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
use crate::xml::quick_xml_experiment::parse_document;

use super::XML_LIMITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DependencyLimits {
    depth: usize,
    modules: usize,
    bytes: usize,
}

impl DependencyLimits {
    pub(super) const fn new(max_depth: usize, max_modules: usize, max_bytes: usize) -> Self {
        Self {
            depth: max_depth,
            modules: max_modules,
            bytes: max_bytes,
        }
    }
}

#[derive(Debug)]
pub(super) struct LoadedStylesheetModule {
    pub(super) identity: String,
    pub(super) document: Document,
    pub(super) dependency_kind: Option<StylesheetDependencyKind>,
    pub(super) dependencies: Vec<LoadedStylesheetModule>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum DependencyFailure {
    Resolution {
        error: ResolutionFailure,
        location: Option<SourceLocation>,
    },
    Fragment {
        identity: String,
        location: Option<SourceLocation>,
    },
    ModuleLimit {
        maximum: usize,
        location: Option<SourceLocation>,
    },
    DepthLimit {
        maximum: usize,
        location: Option<SourceLocation>,
    },
    ByteLimit {
        attempted: usize,
        maximum: usize,
        location: Option<SourceLocation>,
    },
    ByteCountOverflow {
        location: Option<SourceLocation>,
    },
    Cycle {
        identity: String,
        location: Option<SourceLocation>,
    },
    InvalidXml {
        identity: String,
        detail: String,
        location: Option<SourceLocation>,
    },
    InvalidXdm {
        identity: String,
        detail: String,
        location: Option<SourceLocation>,
    },
    InvalidDeclaration(CompileFailure),
}

pub(super) fn load_stylesheet_dependency_graph(
    resolver: &mut SnapshotResolver<'_>,
    principal_identity: &str,
    limits: DependencyLimits,
) -> Result<LoadedStylesheetModule, DependencyFailure> {
    let mut state = LoadState {
        limits,
        modules: 0,
        bytes: 0,
        active: Vec::new(),
    };
    load_module(resolver, principal_identity, "", None, 0, None, &mut state)
}

struct LoadState {
    limits: DependencyLimits,
    modules: usize,
    bytes: usize,
    active: Vec<String>,
}

fn load_module(
    resolver: &mut SnapshotResolver<'_>,
    base: &str,
    reference: &str,
    dependency_kind: Option<StylesheetDependencyKind>,
    depth: usize,
    location: Option<SourceLocation>,
    state: &mut LoadState,
) -> Result<LoadedStylesheetModule, DependencyFailure> {
    if depth > state.limits.depth {
        return Err(DependencyFailure::DepthLimit {
            maximum: state.limits.depth,
            location,
        });
    }
    let resource =
        resolver
            .resolve_from(base, reference)
            .map_err(|error| DependencyFailure::Resolution {
                error,
                location: location.clone(),
            })?;
    if resource.fragment.is_some() {
        return Err(DependencyFailure::Fragment {
            identity: resource.identity,
            location,
        });
    }
    if state.active.contains(&resource.identity) {
        return Err(DependencyFailure::Cycle {
            identity: resource.identity,
            location,
        });
    }
    if state.modules >= state.limits.modules {
        return Err(DependencyFailure::ModuleLimit {
            maximum: state.limits.modules,
            location,
        });
    }
    let attempted = state
        .bytes
        .checked_add(resource.bytes.len())
        .ok_or_else(|| DependencyFailure::ByteCountOverflow {
            location: location.clone(),
        })?;
    if attempted > state.limits.bytes {
        return Err(DependencyFailure::ByteLimit {
            attempted,
            maximum: state.limits.bytes,
            location,
        });
    }
    state.modules += 1;
    state.bytes = attempted;

    let parsed =
        parse_document(&resource.identity, resource.bytes, XML_LIMITS).map_err(|error| {
            DependencyFailure::InvalidXml {
                identity: resource.identity.clone(),
                detail: format!("{error:?}"),
                location: location.clone(),
            }
        })?;
    let document =
        Document::from_parsed(parsed).map_err(|error| DependencyFailure::InvalidXdm {
            identity: resource.identity.clone(),
            detail: format!("{error:?}"),
            location: location.clone(),
        })?;
    let references = discovered_stylesheet_dependencies(&document)
        .map_err(DependencyFailure::InvalidDeclaration)?;
    state.active.push(resource.identity.clone());
    let mut dependencies = Vec::with_capacity(references.len());
    for dependency in references {
        dependencies.push(load_module(
            resolver,
            &resource.identity,
            &dependency.href,
            Some(dependency.kind),
            depth + 1,
            Some(dependency.location),
            state,
        )?);
    }
    state.active.pop();
    Ok(LoadedStylesheetModule {
        identity: resource.identity,
        document,
        dependency_kind,
        dependencies,
    })
}

#[cfg(test)]
mod tests {
    use crate::resources::{
        ResolutionLimits, ResourceLimits, ResourceSetBuilder, SnapshotResolver,
    };

    use super::{DependencyFailure, DependencyLimits, load_stylesheet_dependency_graph};

    const ROOT: &str = "https://example.invalid/styles/root.xsl";
    const CHILD: &str = "https://example.invalid/styles/child.xsl";
    const LEAF: &str = "https://example.invalid/styles/leaf.xsl";

    fn module(include: Option<&str>) -> Vec<u8> {
        format!(
            "<xsl:stylesheet version=\"3.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\">{}</xsl:stylesheet>",
            include.map_or_else(String::new, |href| format!("<xsl:include href=\"{href}\"/>"))
        )
        .into_bytes()
    }

    fn snapshot(
        root_include: &str,
        child_include: Option<&str>,
    ) -> crate::resources::ResourceSnapshot {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 512, 1_536));
        resources
            .admit(ROOT, module(Some(root_include)))
            .expect("root");
        resources
            .admit(CHILD, module(child_include))
            .expect("child");
        resources.admit(LEAF, module(None)).expect("leaf");
        resources.seal()
    }

    #[test]
    fn builds_a_bounded_relative_include_chain_without_losing_identity() {
        let snapshot = snapshot("child.xsl", Some("leaf.xsl"));
        let mut resolver = SnapshotResolver::new(&snapshot, [], ResolutionLimits::new(3));

        let graph = load_stylesheet_dependency_graph(
            &mut resolver,
            ROOT,
            DependencyLimits::new(2, 3, 1_536),
        )
        .expect("bounded graph");

        assert_eq!(graph.identity, ROOT);
        assert_eq!(graph.dependencies[0].identity, CHILD);
        assert_eq!(graph.dependencies[0].dependencies[0].identity, LEAF);
    }

    #[test]
    fn distinguishes_depth_module_byte_and_cycle_limits() {
        let chain = snapshot("child.xsl", Some("leaf.xsl"));

        let mut resolver = SnapshotResolver::new(&chain, [], ResolutionLimits::new(3));
        assert!(matches!(
            load_stylesheet_dependency_graph(
                &mut resolver,
                ROOT,
                DependencyLimits::new(1, 3, 1_536)
            ),
            Err(DependencyFailure::DepthLimit {
                maximum: 1,
                location: Some(_)
            })
        ));

        let mut resolver = SnapshotResolver::new(&chain, [], ResolutionLimits::new(3));
        assert!(matches!(
            load_stylesheet_dependency_graph(
                &mut resolver,
                ROOT,
                DependencyLimits::new(2, 2, 1_536)
            ),
            Err(DependencyFailure::ModuleLimit {
                maximum: 2,
                location: Some(_)
            })
        ));

        let root_bytes = chain.get(ROOT).expect("root bytes").len();
        let mut resolver = SnapshotResolver::new(&chain, [], ResolutionLimits::new(3));
        assert!(matches!(
            load_stylesheet_dependency_graph(
                &mut resolver,
                ROOT,
                DependencyLimits::new(2, 3, root_bytes)
            ),
            Err(DependencyFailure::ByteLimit { maximum, .. }) if maximum == root_bytes
        ));

        let cycle = snapshot("child.xsl", Some("root.xsl"));
        let mut resolver = SnapshotResolver::new(&cycle, [], ResolutionLimits::new(3));
        assert!(matches!(
            load_stylesheet_dependency_graph(
                &mut resolver,
                ROOT,
                DependencyLimits::new(3, 3, 1_536)
            ),
            Err(DependencyFailure::Cycle {
                identity,
                location: Some(_)
            }) if identity == ROOT
        ));
    }
}
