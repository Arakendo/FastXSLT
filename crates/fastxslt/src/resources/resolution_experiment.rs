//! Private absolute-identity resolution experiment over one sealed snapshot.

use std::collections::BTreeSet;

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
    EmptyReference,
    InvalidReference { reference: String },
    RelativeReferenceUnsupported { reference: String },
    FragmentUnsupported { reference: String },
    Denied { identity: String },
    Missing { identity: String },
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

    pub(crate) fn resolve(&mut self, reference: &str) -> Result<&'a [u8], ResolutionFailure> {
        if self.attempts >= self.limits.attempts {
            return Err(ResolutionFailure::AttemptLimit {
                maximum: self.limits.attempts,
            });
        }
        self.attempts += 1;

        if reference.is_empty() {
            return Err(ResolutionFailure::EmptyReference);
        }
        if reference.chars().any(char::is_whitespace) {
            return Err(ResolutionFailure::InvalidReference {
                reference: reference.to_owned(),
            });
        }
        if reference.contains('#') {
            return Err(ResolutionFailure::FragmentUnsupported {
                reference: reference.to_owned(),
            });
        }
        if !has_absolute_uri_scheme(reference) {
            return Err(ResolutionFailure::RelativeReferenceUnsupported {
                reference: reference.to_owned(),
            });
        }
        if self.denied.contains(reference) {
            return Err(ResolutionFailure::Denied {
                identity: reference.to_owned(),
            });
        }
        self.snapshot
            .get(reference)
            .ok_or_else(|| ResolutionFailure::Missing {
                identity: reference.to_owned(),
            })
    }
}

fn has_absolute_uri_scheme(reference: &str) -> bool {
    let Some((scheme, remainder)) = reference.split_once(':') else {
        return false;
    };
    if scheme.len() == 1 && remainder.starts_with(['/', '\\']) {
        return false;
    }
    let mut characters = scheme.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

#[cfg(test)]
mod tests {
    use super::{ResolutionFailure, ResolutionLimits, SnapshotResolver};
    use crate::resources::{ResourceLimits, ResourceSetBuilder};

    fn snapshot() -> crate::resources::ResourceSnapshot {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 64, 128));
        resources
            .admit("urn:fastxslt:resource:document", b"<document/>".to_vec())
            .expect("admit URN resource");
        resources
            .admit(
                "https://example.invalid/admitted.xml",
                b"<admitted/>".to_vec(),
            )
            .expect("admit URL-shaped logical resource");
        resources.seal()
    }

    #[test]
    fn resolves_only_exact_qualified_identities_from_the_sealed_snapshot() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(2));

        assert_eq!(
            resolver.resolve("urn:fastxslt:resource:document"),
            Ok(&b"<document/>"[..])
        );
        assert_eq!(
            resolver.resolve("https://example.invalid/admitted.xml"),
            Ok(&b"<admitted/>"[..])
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
                resolver.resolve(identity),
                Err(ResolutionFailure::Denied {
                    identity: identity.to_owned()
                })
            );
        }
        assert_eq!(
            resolver.resolve("urn:fastxslt:resource:missing"),
            Err(ResolutionFailure::Missing {
                identity: "urn:fastxslt:resource:missing".to_owned()
            })
        );
    }

    #[test]
    fn rejects_relative_paths_fragments_and_invalid_references_without_fallback() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(5));

        assert_eq!(
            resolver.resolve("relative/document.xml"),
            Err(ResolutionFailure::RelativeReferenceUnsupported {
                reference: "relative/document.xml".to_owned()
            })
        );
        assert_eq!(
            resolver.resolve(r"C:\host\document.xml"),
            Err(ResolutionFailure::RelativeReferenceUnsupported {
                reference: r"C:\host\document.xml".to_owned()
            })
        );
        assert_eq!(
            resolver.resolve("urn:fastxslt:resource:document#fragment"),
            Err(ResolutionFailure::FragmentUnsupported {
                reference: "urn:fastxslt:resource:document#fragment".to_owned()
            })
        );
        assert_eq!(
            resolver.resolve("urn:fastxslt:bad reference"),
            Err(ResolutionFailure::InvalidReference {
                reference: "urn:fastxslt:bad reference".to_owned()
            })
        );
        assert_eq!(resolver.resolve(""), Err(ResolutionFailure::EmptyReference));
    }

    #[test]
    fn charges_every_attempt_and_reports_the_fixed_limit_before_lookup() {
        let snapshot = snapshot();
        let mut resolver = SnapshotResolver::new(&snapshot, Vec::new(), ResolutionLimits::new(1));

        assert!(matches!(
            resolver.resolve("urn:fastxslt:resource:missing"),
            Err(ResolutionFailure::Missing { .. })
        ));
        assert_eq!(
            resolver.resolve("urn:fastxslt:resource:document"),
            Err(ResolutionFailure::AttemptLimit { maximum: 1 })
        );
    }
}
