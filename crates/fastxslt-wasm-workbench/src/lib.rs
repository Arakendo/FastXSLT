//! Private WebAssembly feasibility adapter for AR-0015.
//!
//! This crate is unpublished and does not define a supported `FastXSLT` binding,
//! wire format, or JavaScript API.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use fastxslt::workbench::{
    ExperimentalEngine, WorkbenchFailure, WorkbenchLimits, WorkbenchResource,
    WorkbenchStylesheetResources,
};
use wasm_bindgen::prelude::*;

/// A private compile/prepare result that keeps structured failure fields.
#[wasm_bindgen]
pub struct WasmEngineCreation {
    engine: Option<WasmEngine>,
    failure: Option<WorkbenchFailure>,
}

#[wasm_bindgen]
impl WasmEngineCreation {
    /// Returns whether the engine was created successfully.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.engine.is_some()
    }

    /// Moves the successfully created engine out of this result once.
    #[wasm_bindgen(js_name = takeEngine)]
    pub fn take_engine(&mut self) -> Option<WasmEngine> {
        self.engine.take()
    }

    /// Returns the stable diagnostic code, or an empty string on success.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn code(&self) -> String {
        failure_field(self.failure.as_ref(), |failure| &failure.code)
    }

    /// Returns the machine-readable diagnostic category, or an empty string.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn category(&self) -> String {
        failure_field(self.failure.as_ref(), |failure| &failure.category)
    }

    /// Returns the bounded display detail, or an empty string on success.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn detail(&self) -> String {
        failure_field(self.failure.as_ref(), |failure| &failure.detail)
    }

    /// Returns the diagnostic resource identity, or an empty string.
    #[wasm_bindgen(getter, js_name = resourceIdentity)]
    #[must_use]
    pub fn resource_identity(&self) -> String {
        self.failure
            .as_ref()
            .and_then(|failure| failure.location.as_deref())
            .map_or_else(String::new, |location| location.resource.clone())
    }

    /// Returns the diagnostic source-span start, or zero when unavailable.
    #[wasm_bindgen(getter, js_name = locationStart)]
    #[must_use]
    pub fn location_start(&self) -> usize {
        self.failure
            .as_ref()
            .and_then(|failure| failure.location.as_deref())
            .map_or(0, |location| location.start)
    }

    /// Returns the diagnostic source-span end, or zero when unavailable.
    #[wasm_bindgen(getter, js_name = locationEnd)]
    #[must_use]
    pub fn location_end(&self) -> usize {
        self.failure
            .as_ref()
            .and_then(|failure| failure.location.as_deref())
            .map_or(0, |location| location.end)
    }
}

/// A private transform result with owned bytes and structured failure fields.
#[wasm_bindgen]
pub struct WasmTransformOutcome {
    bytes: Option<Vec<u8>>,
    failure: Option<WorkbenchFailure>,
}

#[wasm_bindgen]
impl WasmTransformOutcome {
    /// Returns whether transformation and serialization completed.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.bytes.is_some()
    }

    /// Moves serialized bytes across the host boundary once.
    #[wasm_bindgen(js_name = takeResultBytes)]
    pub fn take_result_bytes(&mut self) -> Vec<u8> {
        self.bytes.take().unwrap_or_default()
    }

    /// Returns the stable diagnostic code, or an empty string on success.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn code(&self) -> String {
        failure_field(self.failure.as_ref(), |failure| &failure.code)
    }

    /// Returns the machine-readable diagnostic category, or an empty string.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn category(&self) -> String {
        failure_field(self.failure.as_ref(), |failure| &failure.category)
    }

    /// Returns the bounded display detail, or an empty string on success.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn detail(&self) -> String {
        failure_field(self.failure.as_ref(), |failure| &failure.detail)
    }
}

/// One retained compile-once, prepare-once engine inside a WASM instance.
#[wasm_bindgen]
pub struct WasmEngine {
    inner: ExperimentalEngine,
}

/// Collects owned resources before creating one retained WASM engine.
#[wasm_bindgen]
pub struct WasmEngineBuilder {
    source_identity: String,
    source: Vec<u8>,
    stylesheet_identity: String,
    stylesheet: Vec<u8>,
    dependencies: Vec<WorkbenchResource>,
}

#[wasm_bindgen]
impl WasmEngineBuilder {
    /// Creates a builder by copying the principal source and stylesheet.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(
        source_identity: String,
        source: &[u8],
        stylesheet_identity: String,
        stylesheet: &[u8],
    ) -> Self {
        Self {
            source_identity,
            source: source.to_vec(),
            stylesheet_identity,
            stylesheet: stylesheet.to_vec(),
            dependencies: Vec::new(),
        }
    }

    /// Copies one additional stylesheet resource into the future snapshot.
    #[wasm_bindgen(js_name = addStylesheetResource)]
    pub fn add_stylesheet_resource(&mut self, identity: String, bytes: &[u8]) {
        self.dependencies.push(WorkbenchResource {
            identity,
            bytes: bytes.to_vec(),
        });
    }

    /// Seals resources, compiles the stylesheet, and prepares the source.
    #[wasm_bindgen]
    #[must_use]
    pub fn build(self) -> WasmEngineCreation {
        create_engine(
            self.source_identity,
            self.source,
            self.stylesheet_identity,
            self.stylesheet,
            self.dependencies,
        )
    }
}

#[wasm_bindgen]
impl WasmEngine {
    /// Copies two resources into a sealed snapshot, compiles, and prepares.
    #[wasm_bindgen(js_name = create)]
    #[must_use]
    pub fn create(
        source_identity: String,
        source: &[u8],
        stylesheet_identity: String,
        stylesheet: &[u8],
    ) -> WasmEngineCreation {
        create_engine(
            source_identity,
            source.to_vec(),
            stylesheet_identity,
            stylesheet.to_vec(),
            Vec::new(),
        )
    }

    /// Executes and serializes one request without retaining a host borrow.
    #[wasm_bindgen(js_name = transformBytes)]
    #[must_use]
    pub fn transform_bytes(&self, request_identity: &str) -> WasmTransformOutcome {
        transform_outcome(self.inner.transform_bytes(request_identity))
    }

    /// Executes with one invocation-local XSLT instruction ceiling.
    #[wasm_bindgen(js_name = transformWithXsltInstructionLimit)]
    #[must_use]
    pub fn transform_with_xslt_instruction_limit(
        &self,
        request_identity: &str,
        maximum_xslt_instructions: usize,
    ) -> WasmTransformOutcome {
        transform_outcome(
            self.inner
                .transform_with_xslt_instruction_limit(request_identity, maximum_xslt_instructions)
                .map(String::into_bytes),
        )
    }

    /// Returns the deterministic known-capacity estimate for the retained engine.
    #[wasm_bindgen(getter, js_name = knownRetainedCapacityBytes)]
    #[must_use]
    pub fn known_retained_capacity_bytes(&self) -> usize {
        self.inner
            .retention_estimate()
            .known_retained_capacity_bytes
    }

    /// Returns the retained prepared-XDM node count.
    #[wasm_bindgen(getter, js_name = preparedXdmNodeCount)]
    #[must_use]
    pub fn prepared_xdm_node_count(&self) -> usize {
        self.inner.retention_estimate().prepared_xdm_node_count
    }

    /// Returns the deterministic prepared-XDM capacity estimate.
    #[wasm_bindgen(getter, js_name = preparedXdmCapacityBytes)]
    #[must_use]
    pub fn prepared_xdm_capacity_bytes(&self) -> usize {
        self.inner.retention_estimate().prepared_xdm_capacity_bytes
    }
}

fn create_engine(
    source_identity: String,
    source: Vec<u8>,
    stylesheet_identity: String,
    stylesheet: Vec<u8>,
    dependencies: Vec<WorkbenchResource>,
) -> WasmEngineCreation {
    match ExperimentalEngine::new_with_stylesheet_resources(
        source_identity,
        source,
        stylesheet_identity,
        stylesheet,
        WorkbenchStylesheetResources {
            dependencies,
            denied_identities: Vec::new(),
        },
        WorkbenchLimits::default(),
    ) {
        Ok(inner) => WasmEngineCreation {
            engine: Some(WasmEngine { inner }),
            failure: None,
        },
        Err(failure) => WasmEngineCreation {
            engine: None,
            failure: Some(failure),
        },
    }
}

fn transform_outcome(result: Result<Vec<u8>, WorkbenchFailure>) -> WasmTransformOutcome {
    match result {
        Ok(bytes) => WasmTransformOutcome {
            bytes: Some(bytes),
            failure: None,
        },
        Err(failure) => WasmTransformOutcome {
            bytes: None,
            failure: Some(failure),
        },
    }
}

/// Exercises only host-to-WASM byte transfer for boundary-cost calibration.
#[wasm_bindgen(js_name = copyProbe)]
#[must_use]
pub fn copy_probe(bytes: &[u8]) -> usize {
    bytes.len()
}

/// Reports the current WASM linear-memory allocation in 64 KiB pages.
#[wasm_bindgen(js_name = linearMemoryPages)]
#[must_use]
pub fn linear_memory_pages() -> usize {
    linear_memory_pages_for_target()
}

#[cfg(target_arch = "wasm32")]
fn linear_memory_pages_for_target() -> usize {
    core::arch::wasm32::memory_size(0)
}

#[cfg(not(target_arch = "wasm32"))]
fn linear_memory_pages_for_target() -> usize {
    0
}

fn failure_field(
    failure: Option<&WorkbenchFailure>,
    select: impl FnOnce(&WorkbenchFailure) -> &String,
) -> String {
    failure.map_or_else(String::new, |value| select(value).clone())
}

#[cfg(test)]
mod tests {
    use super::{WasmEngine, WasmEngineBuilder};

    const IDENTITY_STYLESHEET: &[u8] = br#"<xsl:stylesheet version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output omit-xml-declaration="yes"/>
        <xsl:template match="/"><out><xsl:value-of select="root/value"/></out></xsl:template>
    </xsl:stylesheet>"#;

    #[test]
    fn retained_engine_reuses_prepared_input_after_budget_failure() {
        let mut creation = WasmEngine::create(
            "urn:test:source".into(),
            b"<root><value>kept</value></root>",
            "urn:test:stylesheet".into(),
            IDENTITY_STYLESHEET,
        );
        assert!(creation.succeeded());
        let engine = creation.take_engine().expect("successful engine");

        let limited = engine.transform_with_xslt_instruction_limit("limited", 0);
        assert!(!limited.succeeded());
        assert_eq!(limited.code(), "FXCT0002");
        assert_eq!(limited.category(), "limit");

        let mut completed = engine.transform_bytes("after-limit");
        assert!(completed.succeeded());
        assert_eq!(completed.take_result_bytes(), b"<out>kept</out>");
    }

    #[test]
    fn builder_resolves_include_only_from_admitted_memory() {
        let mut builder = WasmEngineBuilder::new(
            "https://example.invalid/source.xml".into(),
            b"<root/>",
            "https://example.invalid/main.xsl".into(),
            br#"<xsl:stylesheet version="1.0"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
                <xsl:include href="included.xsl"/>
                <xsl:output omit-xml-declaration="yes"/>
            </xsl:stylesheet>"#,
        );
        builder.add_stylesheet_resource(
            "https://example.invalid/included.xsl".into(),
            br#"<xsl:stylesheet version="1.0"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
                <xsl:template match="/"><out>sealed</out></xsl:template>
            </xsl:stylesheet>"#,
        );

        let mut creation = builder.build();
        assert!(creation.succeeded());
        let engine = creation.take_engine().expect("successful engine");
        let mut outcome = engine.transform_bytes("sealed-include");
        assert!(outcome.succeeded());
        assert_eq!(outcome.take_result_bytes(), b"<out>sealed</out>");
    }

    #[test]
    fn malformed_source_preserves_structured_classification() {
        let creation = WasmEngine::create(
            "urn:test:malformed-source".into(),
            b"<root>",
            "urn:test:stylesheet".into(),
            IDENTITY_STYLESHEET,
        );

        assert!(!creation.succeeded());
        assert_eq!(creation.code(), "FXXD0002");
        assert_eq!(creation.category(), "invalid");
    }
}
