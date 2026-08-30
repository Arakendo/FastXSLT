//! Private absolute-identity resolution experiment over one sealed snapshot.

use std::collections::BTreeSet;

use iri_string::{
    format::ToDedicatedString,
    types::{IriAbsoluteStr, IriReferenceStr},
};

use super::ResourceSnapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResolutionLimits {
    attempts: usize,
}

impl ResolutionLimits {
    pub(crate) const fn new(max_attempts: usize) -> Self {
        Self {
            attempts: max_attempts,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResolutionFailure {
    AttemptLimit { maximum: usize },
    InvalidBase { base: String },
    InvalidReference { reference: String },
    ResolutionFailed { base: String, reference: String },
    Denied { identity: String },
    Missing { identity: String },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ResolvedResource<'a> {
    pub(crate) identity: String,
    pub(crate) fragment: Option<String>,
    pub(crate) bytes: &'a [u8],
}

pub(crate) struct SnapshotResolver<'a> {
    snapshot: &'a ResourceSnapshot,
    denied: BTreeSet<String>,
    limits: ResolutionLimits,
    attempts: usize,
}

impl<'a> SnapshotResolver<'a> {
    pub(crate) fn new(
        snapshot: &'a ResourceSnapshot,
        denied: impl IntoIterator<Item = String>,
        limits: ResolutionLimits,
    ) -> Self {
        Self {
            snapshot,
            denied: denied.into_iter().collect(),
            limits,
            attempts: 0,
        }
    }

    pub(crate) fn resolve_from(
        &mut self,
        base: &str,
        reference: &str,
    ) -> Result<ResolvedResource<'a>, ResolutionFailure> {
        self.charge_attempt()?;

        let (identity, fragment) = resolve_reference(base, reference)?;
        if self.denied.contains(&identity) {
            return Err(ResolutionFailure::Denied { identity });
        }
        let bytes = self
            .snapshot
            .get(&identity)
            .ok_or_else(|| ResolutionFailure::Missing {
                identity: identity.clone(),
            })?;
        Ok(ResolvedResource {
            identity,
            fragment,
            bytes,
        })
    }

    fn charge_attempt(&mut self) -> Result<(), ResolutionFailure> {
        if self.attempts >= self.limits.attempts {
            return Err(ResolutionFailure::AttemptLimit {
                maximum: self.limits.attempts,
            });
        }
        self.attempts += 1;
        Ok(())
    }
}

pub(crate) fn resolve_reference(
    base: &str,
    reference: &str,
) -> Result<(String, Option<String>), ResolutionFailure> {
    let base_iri = IriAbsoluteStr::new(base).map_err(|_| ResolutionFailure::InvalidBase {
        base: base.to_owned(),
    })?;
    let reference_iri =
        IriReferenceStr::new(reference).map_err(|_| ResolutionFailure::InvalidReference {
            reference: reference.to_owned(),
        })?;
    let resolved = reference_iri.resolve_against(base_iri);
    resolved
        .ensure_rfc3986_normalizable()
        .map_err(|_| ResolutionFailure::ResolutionFailed {
            base: base.to_owned(),
            reference: reference.to_owned(),
        })?;
    let resolved = resolved.to_dedicated_string();
    Ok(match resolved.as_str().split_once('#') {
        Some((identity, fragment)) => (identity.to_owned(), Some(fragment.to_owned())),
        None => (resolved.as_str().to_owned(), None),
    })
}

#[cfg(test)]
mod tests {
    use super::{ResolutionFailure, ResolutionLimits, SnapshotResolver};
    use crate::resources::{ResourceLimits, ResourceSetBuilder};

    fn snapshot() -> crate::resources::ResourceSnapshot {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(6, 128, 512));
        resources
            .admit("urn:fastxslt:resource:document", b"<document/>".to_vec())
            .expect("admit URN resource");
        resources
            .admit(
                "https://example.invalid/admitted.xml",
                b"<admitted/>".to_vec(),
            )
            .expect("admit URL-shaped logical resource");
        resources
            .admit(
                "https://example.invalid/styles/include-0401a.xsl",
                b"<included/>".to_vec(),
            )
            .expect("admit sibling module");
        resources
            .admit(
                "https://example.invalid/shared/module.xsl",
                b"<shared/>".to_vec(),
            )
            .expect("admit parent-relative module");
        resources
            .admit(
                "https://example.invalid/styles/embedded.xml",
                b"<resource/>".to_vec(),
            )
            .expect("admit fragment-bearing reference resource");
        resources.seal()
    }

    #[test]
    fn resolves_only_exact_qualified_identities_from_the_sealed_snapshot() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(2));

        assert_eq!(
            resolver
                .resolve_from("urn:fastxslt:resource:document", "")
                .map(|resource| resource.bytes),
            Ok(&b"<document/>"[..]),
        );
        assert_eq!(
            resolver
                .resolve_from("https://example.invalid/admitted.xml", "")
                .map(|resource| resource.bytes),
            Ok(&b"<admitted/>"[..]),
        );
    }

    #[test]
    fn distinguishes_denied_from_missing_without_leaking_snapshot_presence() {
        let snapshot = snapshot();
        let denied = [
            "urn:fastxslt:resource:document".to_owned(),
            "urn:fastxslt:resource:not-admitted".to_owned(),
        ];
        let mut resolver = SnapshotResolver::new(&snapshot, denied, ResolutionLimits::new(3));

        for identity in [
            "urn:fastxslt:resource:document",
            "urn:fastxslt:resource:not-admitted",
        ] {
            assert_eq!(
                resolver.resolve_from(identity, ""),
                Err(ResolutionFailure::Denied {
                    identity: identity.to_owned()
                })
            );
        }
        assert_eq!(
            resolver.resolve_from("urn:fastxslt:resource:missing", ""),
            Err(ResolutionFailure::Missing {
                identity: "urn:fastxslt:resource:missing".to_owned()
            })
        );
    }

    #[test]
    fn charges_every_attempt_and_reports_the_fixed_limit_before_lookup() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(1));

        assert!(matches!(
            resolver.resolve_from("urn:fastxslt:resource:missing", ""),
            Err(ResolutionFailure::Missing { .. })
        ));
        assert_eq!(
            resolver.resolve_from("urn:fastxslt:resource:document", ""),
            Err(ResolutionFailure::AttemptLimit { maximum: 1 })
        );
    }

    #[test]
    fn resolves_sibling_and_parent_relative_iris_against_the_supplied_base() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(2));

        let sibling = resolver
            .resolve_from(
                "https://example.invalid/styles/include-0401.xsl",
                "include-0401a.xsl",
            )
            .expect("resolve sibling module");
        assert_eq!(
            sibling.identity,
            "https://example.invalid/styles/include-0401a.xsl"
        );
        assert_eq!(sibling.fragment, None);
        assert_eq!(sibling.bytes, b"<included/>");

        let parent = resolver
            .resolve_from(
                "https://example.invalid/styles/include-0401.xsl",
                "../shared/module.xsl",
            )
            .expect("resolve parent-relative module");
        assert_eq!(parent.identity, "https://example.invalid/shared/module.xsl");
        assert_eq!(parent.bytes, b"<shared/>");
    }

    #[test]
    fn separates_fragment_semantics_from_the_acquired_resource_identity() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(1));

        let resource = resolver
            .resolve_from("https://example.invalid/styles/embedded.xml", "#embedded")
            .expect("resolve same-document fragment reference");

        assert_eq!(
            resource.identity,
            "https://example.invalid/styles/embedded.xml"
        );
        assert_eq!(resource.fragment.as_deref(), Some("embedded"));
        assert_eq!(resource.bytes, b"<resource/>");
    }

    #[test]
    fn rejects_invalid_bases_references_and_unserializable_rfc3986_results() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(5));

        assert_eq!(
            resolver.resolve_from("relative/base.xsl", "module.xsl"),
            Err(ResolutionFailure::InvalidBase {
                base: "relative/base.xsl".to_owned()
            })
        );
        assert_eq!(
            resolver.resolve_from(r"C:\host\base.xsl", "module.xsl"),
            Err(ResolutionFailure::InvalidBase {
                base: r"C:\host\base.xsl".to_owned()
            })
        );
        assert_eq!(
            resolver.resolve_from(
                "https://example.invalid/styles/base.xsl#fragment",
                "module.xsl"
            ),
            Err(ResolutionFailure::InvalidBase {
                base: "https://example.invalid/styles/base.xsl#fragment".to_owned()
            })
        );
        assert_eq!(
            resolver.resolve_from(
                "https://example.invalid/styles/base.xsl",
                "bad reference.xsl"
            ),
            Err(ResolutionFailure::InvalidReference {
                reference: "bad reference.xsl".to_owned()
            })
        );
        assert_eq!(
            resolver.resolve_from("scheme:", ".///not-an-authority"),
            Err(ResolutionFailure::ResolutionFailed {
                base: "scheme:".to_owned(),
                reference: ".///not-an-authority".to_owned()
            })
        );
    }
}
