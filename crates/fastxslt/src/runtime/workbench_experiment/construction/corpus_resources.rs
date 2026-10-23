//! Test-harness import: resource membership is distinct from catalog roles.

use super::*;
use crate::runtime::workbench_experiment::{PreprocessingProbeOptions, WorkbenchResource};
use std::collections::BTreeMap;

impl ExperimentalEngine {
    /// Coalesce repeated catalog roles, never different logical identities.
    /// The ordinary adapter constructor intentionally retains strict admission.
    pub(crate) fn new_with_corpus_resources(
        source_id: String,
        source: Vec<u8>,
        stylesheet_id: &str,
        stylesheet: Vec<u8>,
        resources: WorkbenchStylesheetResources,
        limits: WorkbenchLimits,
        probe: PreprocessingProbeOptions,
    ) -> Result<Self, WorkbenchFailure> {
        let entries = unique_resources(
            [
                WorkbenchResource {
                    identity: source_id.clone(),
                    bytes: source,
                },
                WorkbenchResource {
                    identity: stylesheet_id.to_owned(),
                    bytes: stylesheet,
                },
            ]
            .into_iter()
            .chain(resources.dependencies),
        )?;
        let total = limits
            .max_resource_bytes
            .checked_mul(entries.len())
            .ok_or_else(|| workbench_failure("FXWB0001", "limit", "resource limit overflow"))?;
        let mut builder = ResourceSetBuilder::new(ResourceLimits::new(
            entries.len(),
            limits.max_resource_bytes,
            total,
        ));
        for (identity, bytes) in entries {
            builder.admit(identity, bytes).map_err(|failure| {
                workbench_failure(
                    "FXWB0002",
                    "limit",
                    format!("catalog admission: {failure:?}"),
                )
            })?;
        }
        Self::from_admitted_resources(
            source_id,
            stylesheet_id,
            builder.seal(),
            resources.denied_identities,
            limits,
            &ConstructionOptions {
                source: Some(probe.subsets.declarations),
                stylesheet: Some(probe.subsets.declarations),
                source_external_subset: probe.subsets.source,
                stylesheet_external_subset: probe.subsets.stylesheet,
                capacity: PreparationCapacity::Growth,
                strict_preprocessing: probe.strict,
                selected_legacy: probe.selected_legacy,
            },
        )
    }
}

fn unique_resources(
    resources: impl IntoIterator<Item = WorkbenchResource>,
) -> Result<BTreeMap<String, Vec<u8>>, WorkbenchFailure> {
    let mut entries = BTreeMap::new();
    for resource in resources {
        match entries.entry(resource.identity) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(resource.bytes);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                if entry.get() != &resource.bytes {
                    return Err(workbench_failure(
                        "FXWB0002",
                        "invalid",
                        "conflicting bytes for catalog resource identity",
                    ));
                }
            }
        }
    }
    Ok(entries)
}

#[test]
fn corpus_roles_coalesce_only_equal_identity_and_bytes() {
    let resource = |identity: &str, bytes: &[u8]| WorkbenchResource {
        identity: identity.to_owned(),
        bytes: bytes.to_vec(),
    };
    let entries = unique_resources([
        resource("urn:a", b"same"),
        resource("urn:a", b"same"),
        resource("urn:b", b"same"),
    ])
    .unwrap();
    assert_eq!(entries.len(), 2);
    let failure =
        unique_resources([resource("urn:a", b"one"), resource("urn:a", b"two")]).unwrap_err();
    assert_eq!(failure.code, "FXWB0002");
}

#[test]
fn corpus_roles_share_one_resource_without_relaxing_adapter_admission() {
    use crate::xslt::golden_semantics_experiment::{
        ProcessorPolicyStamp, StylesheetPreprocessingStamp,
    };
    let stylesheet = br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out>same resource</out></xsl:template></xsl:stylesheet>"#;
    for (strict, selected_legacy) in [(false, false), (true, false), (true, true)] {
        let create = |limits| {
            ExperimentalEngine::new_with_corpus_resources(
                "urn:shared".to_owned(),
                stylesheet.to_vec(),
                "urn:shared",
                stylesheet.to_vec(),
                WorkbenchStylesheetResources {
                    dependencies: vec![WorkbenchResource {
                        identity: "urn:shared".to_owned(),
                        bytes: stylesheet.to_vec(),
                    }],
                    denied_identities: Vec::new(),
                },
                limits,
                PreprocessingProbeOptions {
                    strict,
                    selected_legacy,
                    subsets: WorkbenchExternalSubsetLimits {
                        declarations: InternalSubsetLimits {
                            declarations: 1,
                            nesting_depth: 1,
                            references: 1,
                            replacement_bytes: 1,
                        },
                        source: None,
                        stylesheet: None,
                    },
                },
            )
        };
        let engine = create(WorkbenchLimits::default()).unwrap();
        assert_eq!(
            engine.program.processor,
            if selected_legacy {
                ProcessorPolicyStamp::Xslt10Reference
            } else {
                ProcessorPolicyStamp::DeclarationDrivenBaseline
            }
        );
        if selected_legacy {
            assert_eq!(
                engine.program.preprocessing,
                StylesheetPreprocessingStamp::StrictXslt10Reference
            );
        }
        assert_eq!(
            engine.transform_bytes("roles").unwrap(),
            b"<out>same resource</out>"
        );
        let failure = create(WorkbenchLimits {
            max_resource_bytes: stylesheet.len() - 1,
            ..WorkbenchLimits::default()
        })
        .err()
        .unwrap();
        assert_eq!(failure.code, "FXWB0002");
    }
    let failure = ExperimentalEngine::new(
        "urn:shared",
        stylesheet.to_vec(),
        "urn:shared",
        stylesheet.to_vec(),
        WorkbenchLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(failure.code, "FXWB0002");
    assert!(failure.detail.contains("DuplicateIdentity"));
}
