//! Explicitly unstable facade for non-Rust host-boundary experiments.

use crate::execution_control_experiment::{
    CancellationToken, ControlFailure, InvocationControl, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{
    ExecutionFailure, compile_resource, execute_program, serialize_xml,
};
use crate::runtime::prepared_input_experiment::{
    PreparationFailure, PreparedInputBuilder, PreparedInputSet,
};

/// Explicit bounds for the isolated ASP.NET workbench experiment.
#[derive(Debug, Clone, Copy)]
pub struct WorkbenchLimits {
    /// Maximum bytes admitted for either the source or stylesheet.
    pub max_resource_bytes: usize,
    /// Maximum serialized result bytes.
    pub max_result_bytes: usize,
    /// Maximum XML events charged during source preparation.
    pub max_xml_events: usize,
    /// Maximum XDM nodes charged during source preparation and execution.
    pub max_xdm_nodes: usize,
    /// Maximum `XPath` operations charged during one transformation.
    pub max_xpath_operations: usize,
    /// Maximum XSLT instructions charged during one transformation.
    pub max_xslt_instructions: usize,
    /// Maximum result nodes charged during one transformation.
    pub max_result_nodes: usize,
}

impl Default for WorkbenchLimits {
    fn default() -> Self {
        Self {
            max_resource_bytes: 1_048_576,
            max_result_bytes: 1_048_576,
            max_xml_events: 100_000,
            max_xdm_nodes: 100_000,
            max_xpath_operations: 1_000_000,
            max_xslt_instructions: 1_000_000,
            max_result_nodes: 100_000,
        }
    }
}

/// Structured failure projected across the experimental worker boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchFailure {
    /// Stable private diagnostic identity for this experiment.
    pub code: String,
    /// Machine-readable failure category.
    pub category: String,
    /// Optional logical request identity.
    pub request_id: Option<String>,
    /// Human-readable diagnostic detail.
    pub detail: String,
}

/// Compile-once, prepare-once engine retained by the isolated host workbench.
///
/// This type is feature-gated, documentation-hidden, and not a supported public
/// API. It exists solely to measure a real non-Rust host lifecycle.
pub struct ExperimentalEngine {
    prepared: PreparedInputSet,
    source_id: String,
    program: crate::xslt::golden_semantics_experiment::StylesheetProgram,
    limits: WorkbenchLimits,
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
        let source_id = source_id.into();
        let stylesheet_id = stylesheet_id.into();
        let total_limit = limits
            .max_resource_bytes
            .checked_mul(2)
            .ok_or_else(|| workbench_failure("FXWB0001", "limit", "resource limit overflow"))?;
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(
            2,
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
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, &stylesheet_id)
            .map_err(|failure| project_execution(&failure))?;
        let mut builder = PreparedInputBuilder::new(snapshot);
        let mut control = InvocationControl::new(CancellationToken::new(), work_limits(limits));
        builder
            .prepare(&source_id, &mut control)
            .map_err(|failure| project_preparation(&failure))?;
        Ok(Self {
            prepared: builder.seal(),
            source_id,
            program,
            limits,
        })
    }

    /// Executes one request against the retained compiled and prepared state.
    ///
    /// # Errors
    ///
    /// Returns a structured failure for invalid identity, exhausted limits,
    /// unsupported semantics, cancellation, or serialization failure.
    pub fn transform(&self, request_id: &str) -> Result<String, WorkbenchFailure> {
        if request_id.is_empty() {
            return Err(workbench_failure(
                "FXWB0003",
                "invalid",
                "request identity must not be empty",
            ));
        }
        let document = self.prepared.get(&self.source_id).ok_or_else(|| {
            workbench_failure("FXWB0004", "internal", "prepared source is unavailable")
        })?;
        let mut control =
            InvocationControl::new(CancellationToken::new(), work_limits(self.limits));
        let semantic = execute_program(&self.program, &document, request_id, &mut control)
            .map_err(|failure| project_execution(&failure))?;
        serialize_xml(
            &semantic,
            &self.program.output,
            request_id,
            self.limits.max_result_bytes,
            &mut control,
        )
        .map_err(|failure| project_execution(&failure))
    }
}

fn work_limits(limits: WorkbenchLimits) -> WorkLimits {
    WorkLimits {
        xml_events: limits.max_xml_events,
        xdm_nodes: limits.max_xdm_nodes,
        xdm_string_value_nodes: limits.max_xdm_nodes,
        xpath_node_visits: limits.max_xpath_operations,
        xpath_operations: limits.max_xpath_operations,
        xslt_instructions: limits.max_xslt_instructions,
        result_nodes: limits.max_result_nodes,
        result_text_bytes: limits.max_result_bytes,
        serialized_bytes: limits.max_result_bytes,
    }
}

fn project_execution(failure: &ExecutionFailure) -> WorkbenchFailure {
    let (code, category, request_id, detail) = failure.workbench_parts();
    WorkbenchFailure {
        code: code.to_owned(),
        category: category.to_owned(),
        request_id: request_id.map(str::to_owned),
        detail: detail.to_owned(),
    }
}

fn project_preparation(failure: &PreparationFailure) -> WorkbenchFailure {
    let (code, category) = match failure {
        PreparationFailure::MissingResource { .. } => ("FXWB0005", "missing-resource"),
        PreparationFailure::DuplicateResource { .. } => ("FXWB0006", "invalid"),
        PreparationFailure::InvalidXml { .. } => ("FXXM0002", "invalid"),
        PreparationFailure::InvalidXdm { .. } => ("FXXD0002", "invalid"),
        PreparationFailure::Control(ControlFailure::Cancelled { .. }) => ("FXCT0001", "cancelled"),
        PreparationFailure::Control(ControlFailure::BudgetExhausted { .. }) => {
            ("FXCT0002", "limit")
        }
    };
    workbench_failure(code, category, format!("{failure:?}"))
}

fn workbench_failure(
    code: impl Into<String>,
    category: impl Into<String>,
    detail: impl Into<String>,
) -> WorkbenchFailure {
    WorkbenchFailure {
        code: code.into(),
        category: category.into(),
        request_id: None,
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{ExperimentalEngine, WorkbenchLimits};

    #[test]
    fn compiles_prepares_and_reuses_one_native_workload() {
        let engine = ExperimentalEngine::new(
            "urn:w3c:xslt30:for-004:source",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for03.xml").to_vec(),
            "urn:w3c:xslt30:for-004:stylesheet",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("workbench engine should initialize");

        for request_id in ["first", "second"] {
            assert_eq!(
                engine.transform(request_id).expect("transform should run"),
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>"
            );
        }
    }
}
