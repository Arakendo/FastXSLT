//! Explicitly unstable facade for non-Rust host-boundary experiments.

use crate::execution_control_experiment::{
    CancellationToken, ControlFailure, InvocationControl, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::{
    ExecutionFailure, compile_resource_with_denied, execute_program, serialize_xml,
};
use crate::runtime::prepared_input_experiment::{
    PreparationFailure, PreparedInputBuilder, PreparedInputSet,
};
use crate::xml::quick_xml_experiment::ParseLimits;

/// Explicit bounds for the isolated ASP.NET workbench experiment.
#[derive(Debug, Clone, Copy)]
pub struct WorkbenchLimits {
    /// Maximum bytes admitted for either the source or stylesheet.
    pub max_resource_bytes: usize,
    /// Maximum serialized result bytes.
    pub max_result_bytes: usize,
    /// Maximum XML events charged during source preparation.
    pub max_xml_events: usize,
    /// Maximum XML element nesting depth during source preparation.
    pub max_xml_depth: usize,
    /// Maximum XDM nodes charged during source preparation and execution.
    pub max_xdm_nodes: usize,
    /// Maximum `XPath` operations charged during one transformation.
    pub max_xpath_operations: usize,
    /// Maximum XSLT instructions charged during one transformation.
    pub max_xslt_instructions: usize,
    /// Maximum matched-template candidates considered during one transformation.
    pub max_xslt_template_candidates: usize,
    /// Maximum result nodes charged during one transformation.
    pub max_result_nodes: usize,
}

impl Default for WorkbenchLimits {
    fn default() -> Self {
        Self {
            max_resource_bytes: 1_048_576,
            max_result_bytes: 1_048_576,
            max_xml_events: 100_000,
            max_xml_depth: 64,
            max_xdm_nodes: 100_000,
            max_xpath_operations: 1_000_000,
            max_xslt_instructions: 1_000_000,
            max_xslt_template_candidates: 1_000_000,
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
    /// Optional owned logical resource and byte span.
    pub location: Option<Box<WorkbenchLocation>>,
    /// Human-readable diagnostic detail.
    pub detail: String,
}

/// Source provenance projected without exposing the private XDM location type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchLocation {
    /// Logical resource identity; never authority to reopen a file.
    pub resource: String,
    /// Inclusive byte offset where the relevant source span starts.
    pub start: usize,
    /// Exclusive byte offset where the relevant source span ends.
    pub end: usize,
}

/// One additional immutable stylesheet dependency supplied to the workbench.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchResource {
    /// Qualified logical identity used for resolution, not ambient authority.
    pub identity: String,
    /// Owned resource bytes copied into the sealed snapshot.
    pub bytes: Vec<u8>,
}

/// Explicit resource inputs and denial policy for workbench compilation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkbenchStylesheetResources {
    /// Additional stylesheet modules admitted before compilation.
    pub dependencies: Vec<WorkbenchResource>,
    /// Logical identities denied before snapshot membership is disclosed.
    pub denied_identities: Vec<String>,
}

/// Cooperative cancellation state supplied to one experimental invocation.
#[derive(Debug, Clone)]
pub struct WorkbenchCancellation(CancellationToken);

impl WorkbenchCancellation {
    /// Creates an unsignalled invocation-local cancellation state.
    #[must_use]
    pub fn new() -> Self {
        Self(CancellationToken::new())
    }

    /// Creates a workbench-only cancellation state paused at its first charge.
    #[must_use]
    pub fn with_first_charge_barrier() -> Self {
        Self(CancellationToken::with_first_charge_barrier())
    }

    /// Signals cooperative cancellation for invocations observing this state.
    pub fn cancel(&self) {
        self.0.cancel();
    }

    /// Reports whether an invocation reached the workbench-only first charge.
    #[must_use]
    pub fn first_charge_observed(&self) -> bool {
        self.0.first_charge_observed()
    }
}

impl Default for WorkbenchCancellation {
    fn default() -> Self {
        Self::new()
    }
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
        let source_id = source_id.into();
        let stylesheet_id = stylesheet_id.into();
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
        let program = compile_resource_with_denied(
            &snapshot,
            &stylesheet_id,
            stylesheet_resources.denied_identities,
        )
        .map_err(|failure| project_execution(&failure))?;
        let mut builder = PreparedInputBuilder::with_parse_limits(
            snapshot,
            ParseLimits {
                max_events: limits.max_xml_events,
                max_depth: limits.max_xml_depth,
            },
        );
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
        self.transform_with_cancellation(request_id, WorkbenchCancellation::new())
    }

    /// Executes one request with explicitly supplied cooperative cancellation.
    ///
    /// # Errors
    ///
    /// Returns `FXCT0001 / cancelled` when cancellation is observed at an
    /// engine-owned charge point, or another structured workbench failure.
    pub fn transform_with_cancellation(
        &self,
        request_id: &str,
        cancellation: WorkbenchCancellation,
    ) -> Result<String, WorkbenchFailure> {
        self.transform_with_control(request_id, cancellation, self.limits)
    }

    /// Executes one workbench request with an invocation-local XSLT instruction
    /// budget while retaining every other configured limit.
    ///
    /// # Errors
    ///
    /// Returns `FXCT0002 / limit` when the instruction budget is exhausted, or
    /// another structured workbench failure.
    pub fn transform_with_xslt_instruction_limit(
        &self,
        request_id: &str,
        maximum_xslt_instructions: usize,
    ) -> Result<String, WorkbenchFailure> {
        self.transform_with_invocation_policy(
            request_id,
            WorkbenchCancellation::new(),
            maximum_xslt_instructions,
        )
    }

    /// Executes one request with invocation-local cooperative cancellation and
    /// an XSLT instruction budget.
    ///
    /// This combined seam exists for host-boundary experiments that must carry
    /// both controls without changing the retained engine configuration.
    ///
    /// # Errors
    ///
    /// Returns the same structured cancellation, limit, or semantic failure as
    /// the corresponding direct engine controls.
    pub fn transform_with_invocation_policy(
        &self,
        request_id: &str,
        cancellation: WorkbenchCancellation,
        maximum_xslt_instructions: usize,
    ) -> Result<String, WorkbenchFailure> {
        let mut limits = self.limits;
        limits.max_xslt_instructions = maximum_xslt_instructions;
        self.transform_with_control(request_id, cancellation, limits)
    }

    fn transform_with_control(
        &self,
        request_id: &str,
        cancellation: WorkbenchCancellation,
        limits: WorkbenchLimits,
    ) -> Result<String, WorkbenchFailure> {
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
        let mut control = InvocationControl::new(cancellation.0, work_limits(limits));
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
        xslt_template_candidates: limits.max_xslt_template_candidates,
        result_nodes: limits.max_result_nodes,
        result_text_bytes: limits.max_result_bytes,
        serialized_bytes: limits.max_result_bytes,
    }
}

fn project_execution(failure: &ExecutionFailure) -> WorkbenchFailure {
    let (code, category, request_id, location, detail) = failure.workbench_parts();
    WorkbenchFailure {
        code: code.to_owned(),
        category: category.to_owned(),
        request_id: request_id.map(str::to_owned),
        location: location.map(|location| {
            Box::new(WorkbenchLocation {
                resource: location.resource.clone(),
                start: location.span.start,
                end: location.span.end,
            })
        }),
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
    let mut projected = workbench_failure(code, category, format!("{failure:?}"));
    if let PreparationFailure::InvalidXml { location, .. } = failure {
        projected.location = Some(Box::new(WorkbenchLocation {
            resource: location.resource.clone(),
            start: location.span.start,
            end: location.span.end,
        }));
    }
    projected
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
        location: None,
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExperimentalEngine, WorkbenchCancellation, WorkbenchLimits, WorkbenchResource,
        WorkbenchStylesheetResources,
    };

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

    #[test]
    fn cancelled_invocation_does_not_poison_reused_state() {
        let engine = ExperimentalEngine::new(
            "urn:w3c:xslt30:for-004:source",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for03.xml").to_vec(),
            "urn:w3c:xslt30:for-004:stylesheet",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("workbench engine should initialize");
        let cancellation = WorkbenchCancellation::new();
        cancellation.cancel();

        let failure = engine
            .transform_with_cancellation("cancelled", cancellation)
            .expect_err("signalled invocation should cancel");
        assert_eq!(failure.code, "FXCT0001");
        assert_eq!(failure.category, "cancelled");
        assert_eq!(failure.request_id.as_deref(), Some("cancelled"));
        assert_eq!(
            failure.detail,
            "host cancellation observed while charging xslt-instruction work"
        );
        assert!(engine.transform("after-cancel").is_ok());
    }

    #[test]
    fn explicit_xml_event_limit_reaches_prepared_input_parser() {
        let mut source = String::from("<order>");
        for _ in 0..600 {
            source.push_str("<order-item price='1.00' qty='1'/>");
        }
        source.push_str("</order>");
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:workbench:larger-source",
            source.into_bytes(),
            "urn:w3c:xslt30:for-004:stylesheet",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("explicit workbench XML limit should replace the private test default");

        assert_eq!(
            engine
                .transform("larger-source")
                .expect("transform should run"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>600.00</out>"
        );
    }

    #[test]
    fn preserves_machine_readable_diagnostics_across_workbench_phases() {
        let source = b"<order/>".to_vec();
        let stylesheet =
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec();
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:diagnostic:source",
            source.clone(),
            "urn:fastxslt:diagnostic:stylesheet",
            stylesheet.clone(),
            WorkbenchLimits::default(),
        )
        .expect("diagnostic engine should initialize");

        let invalid_identity = engine
            .transform("")
            .expect_err("empty request identity should fail");
        assert_eq!(invalid_identity.code, "FXWB0003");
        assert_eq!(invalid_identity.category, "invalid");
        assert_eq!(invalid_identity.request_id, None);
        assert_eq!(invalid_identity.location, None);
        assert_eq!(
            invalid_identity.detail,
            "request identity must not be empty"
        );

        let Err(malformed) = ExperimentalEngine::new(
            "urn:fastxslt:diagnostic:malformed-source",
            b"<order></other>".to_vec(),
            "urn:fastxslt:diagnostic:stylesheet",
            stylesheet,
            WorkbenchLimits::default(),
        ) else {
            panic!("malformed source should fail preparation");
        };
        assert_eq!(malformed.code, "FXXM0002");
        assert_eq!(malformed.category, "invalid");
        assert_eq!(malformed.request_id, None);
        let location = malformed
            .location
            .as_ref()
            .expect("XML failure must retain structured source provenance");
        assert_eq!(
            location.resource,
            "urn:fastxslt:diagnostic:malformed-source"
        );
        assert_eq!(location.start, 7);
        assert_eq!(location.end, 7);
        assert!(
            malformed
                .detail
                .contains("urn:fastxslt:diagnostic:malformed-source")
        );

        let Err(unsupported) = ExperimentalEngine::new(
            "urn:fastxslt:diagnostic:source",
            source,
            "urn:fastxslt:diagnostic:unsupported-stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:message/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        ) else {
            panic!("unsupported instruction should fail compilation");
        };
        assert_eq!(unsupported.code, "FXST1006");
        assert_eq!(unsupported.category, "unsupported");
        assert_eq!(unsupported.request_id, None);
        let location = unsupported
            .location
            .as_ref()
            .expect("compiler failure must retain structured source provenance");
        assert_eq!(
            location.resource,
            "urn:fastxslt:diagnostic:unsupported-stylesheet"
        );
        assert_eq!(location.start, 103);
        assert_eq!(location.end, 117);
        assert_eq!(
            unsupported.detail,
            "unsupported XSLT instruction: xsl:message at urn:fastxslt:diagnostic:unsupported-stylesheet:103..117"
        );
    }

    #[test]
    fn distinguishes_missing_and_denied_stylesheet_dependencies_without_string_parsing() {
        const SOURCE_ID: &str = "urn:fastxslt:resource-diagnostic:source";
        const STYLESHEET_ID: &str = "https://example.invalid/styles/main.xsl";
        const DEPENDENCY_ID: &str = "https://example.invalid/styles/dependency.xsl";
        let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="dependency.xsl"/></xsl:stylesheet>"#;

        let Err(missing) = ExperimentalEngine::new_with_stylesheet_resources(
            SOURCE_ID,
            b"<source/>".to_vec(),
            STYLESHEET_ID,
            stylesheet.to_vec(),
            WorkbenchStylesheetResources::default(),
            WorkbenchLimits::default(),
        ) else {
            panic!("unadmitted dependency must be missing");
        };
        let Err(denied) = ExperimentalEngine::new_with_stylesheet_resources(
            SOURCE_ID,
            b"<source/>".to_vec(),
            STYLESHEET_ID,
            stylesheet.to_vec(),
            WorkbenchStylesheetResources {
                dependencies: Vec::new(),
                denied_identities: vec![DEPENDENCY_ID.to_owned()],
            },
            WorkbenchLimits::default(),
        ) else {
            panic!("denial must precede membership disclosure");
        };

        assert_eq!(missing.code, "FXRS0002");
        assert_eq!(missing.category, "missing-resource");
        assert_eq!(denied.code, "FXRS0003");
        assert_eq!(denied.category, "denied");
        for failure in [&missing, &denied] {
            assert_eq!(failure.request_id, None);
            let location = failure
                .location
                .as_ref()
                .expect("dependency failure should retain the include location");
            assert_eq!(location.resource, STYLESHEET_ID);
            assert!(location.start < location.end);
            assert!(failure.detail.contains(DEPENDENCY_ID));
        }
    }

    #[test]
    fn compiles_one_explicit_workbench_stylesheet_dependency() {
        const DEPENDENCY_ID: &str = "https://example.invalid/styles/dependency.xsl";
        let engine = ExperimentalEngine::new_with_stylesheet_resources(
            "urn:fastxslt:workbench-dependency:source",
            b"<source/>".to_vec(),
            "https://example.invalid/styles/main.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="dependency.xsl"/><xsl:variable name="greeting">hello</xsl:variable></xsl:stylesheet>"#.to_vec(),
            WorkbenchStylesheetResources {
                dependencies: vec![WorkbenchResource {
                    identity: DEPENDENCY_ID.to_owned(),
                    bytes: br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#.to_vec(),
                }],
                denied_identities: Vec::new(),
            },
            WorkbenchLimits::default(),
        )
        .expect("explicit sealed dependency should compile");

        assert_eq!(
            engine.transform("dependency-transform").expect("transform"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>hello</out>"
        );
    }

    #[test]
    fn invocation_local_instruction_limit_does_not_poison_reused_state() {
        let engine = ExperimentalEngine::new(
            "urn:w3c:xslt30:for-004:source",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for03.xml").to_vec(),
            "urn:w3c:xslt30:for-004:stylesheet",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("workbench engine should initialize");

        let failure = engine
            .transform_with_xslt_instruction_limit("instruction-limited", 0)
            .expect_err("zero instruction budget should fail");
        assert_eq!(failure.code, "FXCT0002");
        assert_eq!(failure.category, "limit");
        assert_eq!(failure.request_id.as_deref(), Some("instruction-limited"));
        assert_eq!(
            failure.detail,
            "xslt-instruction work budget exhausted: limit 0, consumed 0, next charge 1"
        );
        assert!(engine.transform("after-instruction-limit").is_ok());
    }
}
