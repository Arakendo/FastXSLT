//! Shared memory-resident admission, compilation and preparation composition.

#[cfg(test)]
use super::WorkbenchExternalSubsetLimits;
use super::{
    ExperimentalEngine, WorkbenchFailure, WorkbenchLimits, WorkbenchStylesheetResources,
    project_execution, project_preparation, work_limits, workbench_failure,
    workbench_stylesheet_compile_limits,
};
use crate::execution_control_experiment::{CancellationToken, InvocationControl};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::compile_resource_with_denied_and_limits;
#[cfg(test)]
use crate::runtime::prepared_input_experiment::PreparationCapacity;
use crate::runtime::prepared_input_experiment::PreparedInputBuilder;
use crate::xml::internal_subset::InternalSubsetLimits;
use crate::xml::quick_xml_experiment::ParseLimits;

#[derive(Clone, Copy, Default)]
struct ConstructionOptions {
    source: Option<InternalSubsetLimits>,
    stylesheet: Option<InternalSubsetLimits>,
    source_external_subset: Option<(&'static str, usize)>,
    stylesheet_external_subset: Option<(&'static str, usize)>,
    #[cfg(test)]
    capacity: PreparationCapacity,
}

#[cfg(test)]
impl ConstructionOptions {
    fn configure_builder(&self, builder: PreparedInputBuilder) -> PreparedInputBuilder {
        let builder = builder.with_capacity(self.capacity);
        let builder = match self.source {
            Some(limits) => builder.with_internal_subset_limits(limits),
            None => builder,
        };
        match self.source_external_subset {
            Some((reference, max_bytes)) => {
                builder.with_single_external_subset(reference, max_bytes)
            }
            None => builder,
        }
    }
}

impl ExperimentalEngine {
    /// Imports bounded bytes, compiles the stylesheet, and prepares the source.
    ///
    /// # Errors
    ///
    /// Returns a structured failure when admission, compilation, or preparation
    /// rejects the supplied resources or limits.
    pub fn new(
        source_id: impl Into<String>,
        source: Vec<u8>,
        stylesheet_id: impl Into<String>,
        stylesheet: Vec<u8>,
        limits: WorkbenchLimits,
    ) -> Result<Self, WorkbenchFailure> {
        Self::new_with_stylesheet_resources(
            source_id,
            source,
            stylesheet_id,
            stylesheet,
            WorkbenchStylesheetResources::default(),
            limits,
        )
    }

    /// Imports an explicit stylesheet dependency set, applies denial policy,
    /// compiles once, and prepares the source.
    ///
    /// This workbench-only constructor exists to pressure resource diagnostics;
    /// it is not a supported resolver or resource API.
    ///
    /// # Errors
    ///
    /// Returns a structured failure when admission, resolution, compilation, or
    /// preparation rejects the supplied resources or limits.
    pub fn new_with_stylesheet_resources(
        source_id: impl Into<String>,
        source: Vec<u8>,
        stylesheet_id: impl Into<String>,
        stylesheet: Vec<u8>,
        stylesheet_resources: WorkbenchStylesheetResources,
        limits: WorkbenchLimits,
    ) -> Result<Self, WorkbenchFailure> {
        Self::new_with_construction_options(
            source_id,
            source,
            stylesheet_id,
            stylesheet,
            stylesheet_resources,
            limits,
            &ConstructionOptions::default(),
        )
    }

    #[cfg(test)]
    pub(crate) fn new_with_bounded_internal_source_subset(
        source_id: impl Into<String>,
        source: Vec<u8>,
        stylesheet_id: impl Into<String>,
        stylesheet: Vec<u8>,
        stylesheet_resources: WorkbenchStylesheetResources,
        limits: WorkbenchLimits,
        dtd_limits: InternalSubsetLimits,
    ) -> Result<Self, WorkbenchFailure> {
        Self::new_with_construction_options(
            source_id,
            source,
            stylesheet_id,
            stylesheet,
            stylesheet_resources,
            limits,
            &ConstructionOptions {
                source: Some(dtd_limits),
                stylesheet: None,
                source_external_subset: None,
                stylesheet_external_subset: None,
                capacity: PreparationCapacity::Growth,
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn new_with_bounded_internal_subsets(
        source_id: impl Into<String>,
        source: Vec<u8>,
        stylesheet_id: impl Into<String>,
        stylesheet: Vec<u8>,
        stylesheet_resources: WorkbenchStylesheetResources,
        limits: WorkbenchLimits,
        dtd_limits: InternalSubsetLimits,
    ) -> Result<Self, WorkbenchFailure> {
        Self::new_with_construction_options(
            source_id,
            source,
            stylesheet_id,
            stylesheet,
            stylesheet_resources,
            limits,
            &ConstructionOptions {
                source: Some(dtd_limits),
                stylesheet: Some(dtd_limits),
                source_external_subset: None,
                stylesheet_external_subset: None,
                capacity: PreparationCapacity::Growth,
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn new_with_bounded_external_subsets(
        source_id: impl Into<String>,
        source: Vec<u8>,
        stylesheet_id: impl Into<String>,
        stylesheet: Vec<u8>,
        stylesheet_resources: WorkbenchStylesheetResources,
        limits: WorkbenchLimits,
        external_subsets: WorkbenchExternalSubsetLimits,
    ) -> Result<Self, WorkbenchFailure> {
        Self::new_with_construction_options(
            source_id,
            source,
            stylesheet_id,
            stylesheet,
            stylesheet_resources,
            limits,
            &ConstructionOptions {
                source: Some(external_subsets.declarations),
                stylesheet: Some(external_subsets.declarations),
                source_external_subset: external_subsets.source,
                stylesheet_external_subset: external_subsets.stylesheet,
                capacity: PreparationCapacity::Growth,
            },
        )
    }

    fn new_with_construction_options(
        source_id: impl Into<String>,
        source: Vec<u8>,
        stylesheet_id: impl Into<String>,
        stylesheet: Vec<u8>,
        stylesheet_resources: WorkbenchStylesheetResources,
        limits: WorkbenchLimits,
        dtd_limits: &ConstructionOptions,
    ) -> Result<Self, WorkbenchFailure> {
        let source_id = source_id.into();
        let stylesheet_id = stylesheet_id.into();
        let denied_resources = stylesheet_resources
            .denied_identities
            .iter()
            .cloned()
            .collect();
        let entry_limit = stylesheet_resources
            .dependencies
            .len()
            .checked_add(2)
            .ok_or_else(|| workbench_failure("FXWB0001", "limit", "resource count overflow"))?;
        let total_limit = limits
            .max_resource_bytes
            .checked_mul(entry_limit)
            .ok_or_else(|| workbench_failure("FXWB0001", "limit", "resource limit overflow"))?;
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(
            entry_limit,
            limits.max_resource_bytes,
            total_limit,
        ));
        resources
            .admit(source_id.clone(), source)
            .map_err(|failure| {
                workbench_failure(
                    "FXWB0002",
                    "limit",
                    format!("source admission: {failure:?}"),
                )
            })?;
        resources
            .admit(stylesheet_id.clone(), stylesheet)
            .map_err(|failure| {
                workbench_failure(
                    "FXWB0002",
                    "limit",
                    format!("stylesheet admission: {failure:?}"),
                )
            })?;
        for dependency in stylesheet_resources.dependencies {
            resources
                .admit(dependency.identity, dependency.bytes)
                .map_err(|failure| {
                    workbench_failure(
                        "FXWB0002",
                        "limit",
                        format!("stylesheet dependency admission: {failure:?}"),
                    )
                })?;
        }
        let snapshot = resources.seal();
        let compile_limits = workbench_stylesheet_compile_limits(
            limits,
            dtd_limits.stylesheet,
            dtd_limits.stylesheet_external_subset,
        );
        let program = compile_resource_with_denied_and_limits(
            &snapshot,
            &stylesheet_id,
            stylesheet_resources.denied_identities,
            compile_limits,
        )
        .map_err(|failure| project_execution(&failure))?;
        let builder = PreparedInputBuilder::with_parse_limits(
            snapshot,
            ParseLimits {
                max_events: limits.max_xml_events,
                max_depth: limits.max_xml_depth,
            },
        );
        #[cfg(test)]
        let mut builder = dtd_limits.configure_builder(builder);
        #[cfg(not(test))]
        let mut builder = {
            debug_assert!(dtd_limits.source.is_none());
            debug_assert!(dtd_limits.source_external_subset.is_none());
            debug_assert!(dtd_limits.stylesheet_external_subset.is_none());
            builder
        };
        let mut control = InvocationControl::new(CancellationToken::new(), work_limits(limits));
        builder
            .prepare(&source_id, &mut control)
            .map_err(|failure| project_preparation(&failure))?;
        Ok(Self {
            prepared: builder.seal(),
            source_id,
            program,
            denied_resources,
            limits,
        })
    }
}

#[cfg(test)]
mod capacity_tests;
