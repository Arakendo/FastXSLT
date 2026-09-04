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

/// Private lower-bound observation of capacities owned by one workbench engine.
///
/// This is admission-accounting evidence, not allocator-exact memory usage or a
/// supported public metric. B-tree node allocation, `Arc` allocation headers,
/// nested compiled-expression allocations, allocator metadata, and host copies
/// are deliberately excluded until their owners can account for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkbenchRetentionEstimate {
    /// Inline bytes occupied by the engine value, including collection headers.
    pub engine_inline_bytes: usize,
    /// Heap capacity of the retained source identity string.
    pub source_identity_capacity_bytes: usize,
    /// Known prepared-map header, entry payload, and identity capacities.
    pub prepared_map_known_capacity_bytes: usize,
    /// Capacity bytes owned by retained immutable XDM documents.
    pub prepared_xdm_capacity_bytes: usize,
    /// Known recursively owned compiled vector, box, and string capacities.
    pub compiled_known_capacity_bytes: usize,
    /// Number of immutable documents retained by the engine.
    pub prepared_document_count: usize,
    /// Number of XDM nodes retained across those documents.
    pub prepared_xdm_node_count: usize,
    /// Sum of the explicitly accounted fields above.
    pub known_retained_capacity_bytes: usize,
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

    /// Reports a private compositional lower bound over known retained capacity.
    ///
    /// The observation is deterministic for the current private representation,
    /// but intentionally excludes allocator and host memory. It exists only to
    /// calibrate AR-0017 experiments and must not be treated as a public layout
    /// or stable admission formula.
    ///
    /// # Panics
    ///
    /// Panics only if the sum of live capacity observations exceeds `usize`,
    /// which cannot represent a live allocation on the current process.
    #[must_use]
    pub fn retention_estimate(&self) -> WorkbenchRetentionEstimate {
        let prepared = self.prepared.retention_observation();
        let engine_inline_bytes = std::mem::size_of::<Self>();
        let source_identity_capacity_bytes = self.source_id.capacity();
        let compiled_known_capacity_bytes = self.program.known_owned_capacity_bytes();
        let known_retained_capacity_bytes = [
            engine_inline_bytes,
            source_identity_capacity_bytes,
            prepared.prepared_map_known_capacity_bytes,
            prepared.xdm_owned_capacity_bytes,
            compiled_known_capacity_bytes,
        ]
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
        .expect("live engine retained-capacity components must fit usize");
        WorkbenchRetentionEstimate {
            engine_inline_bytes,
            source_identity_capacity_bytes,
            prepared_map_known_capacity_bytes: prepared.prepared_map_known_capacity_bytes,
            prepared_xdm_capacity_bytes: prepared.xdm_owned_capacity_bytes,
            compiled_known_capacity_bytes,
            prepared_document_count: prepared.document_count,
            prepared_xdm_node_count: prepared.xdm_node_count,
            known_retained_capacity_bytes,
        }
    }

    #[cfg(test)]
    fn test_only_snapshot_known_capacity_bytes(&self) -> usize {
        self.prepared.test_only_snapshot_known_capacity_bytes()
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
    use std::fmt::Write as _;
    use std::mem::size_of;

    use super::{
        ExperimentalEngine, WorkbenchCancellation, WorkbenchLimits, WorkbenchResource,
        WorkbenchRetentionEstimate, WorkbenchStylesheetResources,
    };

    fn retention_source(items: usize) -> Vec<u8> {
        let mut source = String::from("<order>");
        for _ in 0..items {
            source.push_str("<order-item price='1.00' qty='1'/>");
        }
        source.push_str("</order>");
        source.into_bytes()
    }

    fn retention_engine(items: usize) -> ExperimentalEngine {
        ExperimentalEngine::new(
            format!("urn:fastxslt:retention:source:{items}"),
            retention_source(items),
            "urn:w3c:xslt30:for-004:stylesheet",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("retention-observed engine should initialize")
    }

    fn exact_for_004_engine() -> ExperimentalEngine {
        ExperimentalEngine::new(
            "urn:w3c:xslt30:for-004:source",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for03.xml").to_vec(),
            "urn:w3c:xslt30:for-004:stylesheet",
            include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("exact for-004 engine should initialize")
    }

    #[test]
    fn production_default_collation_expression_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:default-collation:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:default-collation:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="fn:default-collation()" xmlns:fn="http://www.w3.org/2005/xpath-functions"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production default-collation expression should compile");

        assert_eq!(
            engine
                .transform("default-collation-workbench")
                .expect("production default-collation expression should execute"),
            "http://www.w3.org/2005/xpath-functions/collation/codepoint"
        );
    }

    #[test]
    fn production_escape_html_uri_expression_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:escape-html-uri:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:escape-html-uri:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="escape-html-uri(codepoints-to-string((9, 65, 128)))"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production escape-html-uri expression should compile");

        assert_eq!(
            engine
                .transform("escape-html-uri-workbench")
                .expect("production escape-html-uri expression should execute"),
            "%09A%C2%80"
        );
    }

    #[test]
    fn production_encode_for_uri_expression_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:encode-for-uri:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:encode-for-uri:stylesheet",
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="concat('http://www.example.com/', encode-for-uri('~bébé')) eq 'http://www.example.com/~b%C3%A9b%C3%A9'"/></xsl:template></xsl:stylesheet>"#.as_bytes().to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production encode-for-uri expression should compile");

        assert_eq!(
            engine
                .transform("encode-for-uri-workbench")
                .expect("production encode-for-uri expression should execute"),
            "true"
        );
    }

    #[test]
    fn production_iri_to_uri_expression_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:iri-to-uri:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:iri-to-uri:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="iri-to-uri(codepoints-to-string(32 to 34))"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production iri-to-uri expression should compile");

        assert_eq!(
            engine
                .transform("iri-to-uri-workbench")
                .expect("production iri-to-uri expression should execute"),
            "%20!%22"
        );
    }

    #[test]
    fn production_case_conversion_expression_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:case-conversion:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:case-conversion:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="concat(lower-case('AB'), upper-case('cd')) eq 'abCD'"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production case-conversion expression should compile");

        assert_eq!(
            engine
                .transform("case-conversion-workbench")
                .expect("production case-conversion expression should execute"),
            "true"
        );
    }

    #[test]
    fn production_source_free_string_functions_reach_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:source-free-string:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:source-free-string:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="codepoint-equal(normalize-space('  A   B  '), 'A B')"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production source-free string functions should compile");

        assert_eq!(
            engine
                .transform("source-free-string-workbench")
                .expect("production source-free string functions should execute"),
            "true"
        );
    }

    #[test]
    fn production_sequence_cardinality_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:sequence-cardinality:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:sequence-cardinality:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="empty(())"/><xsl:value-of select="exists(reverse((1, 2, 3)))"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production sequence-cardinality expression should compile");

        assert_eq!(
            engine
                .transform("sequence-cardinality-workbench")
                .expect("production sequence-cardinality expression should execute"),
            "truetrue"
        );
    }

    #[test]
    fn production_deep_equal_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:deep-equal:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:deep-equal:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="deep-equal((1, 2), (1, 2))"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production deep-equal expression should compile");

        assert_eq!(
            engine
                .transform("deep-equal-workbench")
                .expect("production deep-equal expression should execute"),
            "true"
        );
    }

    #[test]
    fn production_source_free_boolean_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:boolean:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:boolean:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="not(false()) and boolean(xs:int('1'))"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production source-free boolean expression should compile");

        assert_eq!(
            engine
                .transform("boolean-workbench")
                .expect("production source-free boolean expression should execute"),
            "true"
        );
    }

    #[test]
    fn production_string_length_expression_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:string-length:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:string-length:stylesheet",
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="string-length('A😀') + string-length('bc')"/></xsl:template></xsl:stylesheet>"#.as_bytes().to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production string-length expression should compile");

        assert_eq!(
            engine
                .transform("string-length-workbench")
                .expect("production string-length expression should execute"),
            "4"
        );
    }

    #[test]
    fn production_duration_component_expression_reaches_the_workbench_host_path() {
        let engine = ExperimentalEngine::new(
            "urn:fastxslt:duration-component:source",
            b"<source/>".to_vec(),
            "urn:fastxslt:duration-component:stylesheet",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="minutes-from-duration(xs:duration('-P3Y4M8DT1H23M2.34S'))"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("production duration-component expression should compile");

        assert_eq!(
            engine
                .transform("duration-component-workbench")
                .expect("production duration-component expression should execute"),
            "-23"
        );
    }

    fn retention_custom_engine(
        label: &str,
        source: Vec<u8>,
        stylesheet: Vec<u8>,
    ) -> ExperimentalEngine {
        ExperimentalEngine::new(
            format!("urn:fastxslt:retention:shape:{label}"),
            source,
            format!("urn:fastxslt:retention:shape:{label}:stylesheet"),
            stylesheet,
            WorkbenchLimits::default(),
        )
        .expect("retention-observed shape should initialize")
    }

    fn retention_shape_engine(label: &str, source: Vec<u8>) -> ExperimentalEngine {
        retention_custom_engine(
            label,
            source,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
    }

    fn text_heavy_source() -> Vec<u8> {
        format!("<root><payload>{}</payload></root>", "x".repeat(900_000)).into_bytes()
    }

    fn namespace_attribute_source() -> Vec<u8> {
        let mut source = String::from("<root>");
        for _ in 0..2_000 {
            source.push_str(
                "<p:item xmlns:p='urn:fastxslt:retention:p' a='one' b='two'>text</p:item>",
            );
        }
        source.push_str("</root>");
        source.into_bytes()
    }

    fn template_heavy_stylesheet() -> Vec<u8> {
        let mut stylesheet = String::from(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><out/></xsl:template>"#,
        );
        for index in 0..128 {
            write!(
                stylesheet,
                "<xsl:template match='e{index}'><out><xsl:text>value-{index}</xsl:text></out></xsl:template>"
            )
            .expect("writing to a String cannot fail");
        }
        stylesheet.push_str("</xsl:stylesheet>");
        stylesheet.into_bytes()
    }

    fn global_heavy_stylesheet() -> Vec<u8> {
        let mut stylesheet = String::from(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">"#,
        );
        for index in 0..256 {
            write!(
                stylesheet,
                "<xsl:variable name='global-{index}'>value-{index}</xsl:variable>"
            )
            .expect("writing to a String cannot fail");
        }
        stylesheet.push_str("<xsl:template match='/'><out/></xsl:template></xsl:stylesheet>");
        stylesheet.into_bytes()
    }

    fn assert_estimate_conserves(estimate: WorkbenchRetentionEstimate) {
        assert_eq!(estimate.prepared_document_count, 1);
        assert!(estimate.prepared_xdm_node_count > 0);
        assert_eq!(
            estimate.engine_inline_bytes,
            size_of::<ExperimentalEngine>()
        );
        assert_eq!(
            estimate.known_retained_capacity_bytes,
            estimate.engine_inline_bytes
                + estimate.source_identity_capacity_bytes
                + estimate.prepared_map_known_capacity_bytes
                + estimate.prepared_xdm_capacity_bytes
                + estimate.compiled_known_capacity_bytes
        );
    }

    #[test]
    fn retention_estimate_is_compositional_and_scales_with_prepared_xdm() {
        let small = retention_engine(5).retention_estimate();
        let medium = retention_engine(500).retention_estimate();
        assert_estimate_conserves(small);
        assert_estimate_conserves(medium);
        assert!(medium.prepared_xdm_node_count > small.prepared_xdm_node_count);
        assert!(medium.prepared_xdm_capacity_bytes > small.prepared_xdm_capacity_bytes);
        assert!(medium.known_retained_capacity_bytes > small.known_retained_capacity_bytes);
        assert_eq!(
            medium.compiled_known_capacity_bytes,
            small.compiled_known_capacity_bytes
        );
    }

    #[cfg(feature = "allocation-observation")]
    #[test]
    #[ignore = "manual release-mode workbench engine retention-estimator calibration"]
    fn measures_retention_estimate_against_allocator_requested_bytes() {
        let mut exact_retained = None;
        let exact_allocations = allocation_counter::measure(|| {
            exact_retained = Some(Box::new(exact_for_004_engine()));
        });
        let exact_engine = exact_retained.as_ref().expect("retain exact engine");
        let exact_estimate = exact_engine.retention_estimate();
        let exact_test_snapshot = exact_engine.test_only_snapshot_known_capacity_bytes();
        let exact_denominator = usize::try_from(exact_allocations.bytes_current)
            .expect("positive allocator-retained bytes fit usize")
            .checked_sub(exact_test_snapshot)
            .expect("test-only snapshot must be part of measured retained allocation");
        assert!(exact_estimate.known_retained_capacity_bytes <= exact_denominator);
        println!(
            "shape=exact-for-004 estimate={exact_estimate:?} allocator_requested={exact_allocations:?} test_snapshot_known_bytes={exact_test_snapshot} estimator_numerator={} production_like_allocator_denominator={exact_denominator}",
            exact_estimate.known_retained_capacity_bytes,
        );

        for items in [5, 500, 5_000] {
            let mut retained = None;
            let allocations = allocation_counter::measure(|| {
                retained = Some(Box::new(retention_engine(items)));
            });
            let engine = retained.as_ref().expect("retain measured engine");
            let estimate = engine.retention_estimate();
            assert_estimate_conserves(estimate);
            let allocator_retained_with_test_snapshot = usize::try_from(allocations.bytes_current)
                .expect("positive allocator-retained bytes fit usize");
            let test_snapshot = engine.test_only_snapshot_known_capacity_bytes();
            let production_like_allocator_retained = allocator_retained_with_test_snapshot
                .checked_sub(test_snapshot)
                .expect("test-only snapshot must be part of measured retained allocation");
            assert!(estimate.known_retained_capacity_bytes <= production_like_allocator_retained);
            println!(
                "items={items} estimate={estimate:?} allocator_requested={allocations:?} test_snapshot_known_bytes={test_snapshot} estimator_numerator={} production_like_allocator_denominator={production_like_allocator_retained}",
                estimate.known_retained_capacity_bytes,
            );
        }

        for (label, build) in [
            ("text-heavy", text_heavy_source as fn() -> Vec<u8>),
            ("namespace-attribute", namespace_attribute_source),
        ] {
            let mut retained = None;
            let allocations = allocation_counter::measure(|| {
                retained = Some(Box::new(retention_shape_engine(label, build())));
            });
            let engine = retained.as_ref().expect("retain measured shape engine");
            let estimate = engine.retention_estimate();
            assert_estimate_conserves(estimate);
            let allocator_retained_with_test_snapshot = usize::try_from(allocations.bytes_current)
                .expect("positive allocator-retained bytes fit usize");
            let test_snapshot = engine.test_only_snapshot_known_capacity_bytes();
            let production_like_allocator_retained = allocator_retained_with_test_snapshot
                .checked_sub(test_snapshot)
                .expect("test-only snapshot must be part of measured retained allocation");
            assert!(estimate.known_retained_capacity_bytes <= production_like_allocator_retained);
            println!(
                "shape={label} estimate={estimate:?} allocator_requested={allocations:?} test_snapshot_known_bytes={test_snapshot} estimator_numerator={} production_like_allocator_denominator={production_like_allocator_retained}",
                estimate.known_retained_capacity_bytes,
            );
        }

        for (label, build_stylesheet) in [
            (
                "template-heavy",
                template_heavy_stylesheet as fn() -> Vec<u8>,
            ),
            ("global-heavy", global_heavy_stylesheet),
        ] {
            let mut retained = None;
            let allocations = allocation_counter::measure(|| {
                retained = Some(Box::new(retention_custom_engine(
                    label,
                    b"<root/>".to_vec(),
                    build_stylesheet(),
                )));
            });
            let engine = retained.as_ref().expect("retain measured compiled shape");
            let estimate = engine.retention_estimate();
            assert_estimate_conserves(estimate);
            let allocator_retained_with_test_snapshot = usize::try_from(allocations.bytes_current)
                .expect("positive allocator-retained bytes fit usize");
            let test_snapshot = engine.test_only_snapshot_known_capacity_bytes();
            let production_like_allocator_retained = allocator_retained_with_test_snapshot
                .checked_sub(test_snapshot)
                .expect("test-only snapshot must be part of measured retained allocation");
            assert!(estimate.known_retained_capacity_bytes <= production_like_allocator_retained);
            println!(
                "shape={label} estimate={estimate:?} allocator_requested={allocations:?} test_snapshot_known_bytes={test_snapshot} estimator_numerator={} production_like_allocator_denominator={production_like_allocator_retained}",
                estimate.known_retained_capacity_bytes,
            );
        }
    }

    #[test]
    fn compiles_prepares_and_reuses_one_native_workload() {
        let engine = exact_for_004_engine();

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
