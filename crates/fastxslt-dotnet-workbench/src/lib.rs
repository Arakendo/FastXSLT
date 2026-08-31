//! Unstable native ABI used only by the ASP.NET boundary workbench.

use std::{
    collections::HashMap,
    mem::size_of,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
    sync::{
        Arc, Mutex, MutexGuard, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use fastxslt::workbench::{
    ExperimentalEngine, WorkbenchCancellation, WorkbenchFailure, WorkbenchLimits,
    WorkbenchLocation, WorkbenchResource, WorkbenchStylesheetResources,
};

mod registry_admission;

use registry_admission::{
    ADMISSION_CONTROL_COUNT_EXHAUSTED, ADMISSION_ENGINE_BYTES_EXHAUSTED,
    ADMISSION_ENGINE_COUNT_EXHAUSTED, ADMISSION_OUTCOME_BYTES_EXHAUSTED,
    ADMISSION_OUTCOME_COUNT_EXHAUSTED, ADMISSION_POLICY_REQUIRED, ADMISSION_TOTAL_BYTES_EXHAUSTED,
    MAX_HANDLE, RegistryAccounting, RegistryPolicy, admission_status, decode_policy_limit,
};

const ABI_VERSION: u32 = 3;
const MAX_IDENTITY_BYTES: usize = 4_096;
const MAX_RESOURCE_BYTES: usize = 1_048_576;
const MAX_OUTCOME_BYTES: usize = 1_048_576;
const OUTCOME_ENGINE: u32 = 1;
const OUTCOME_RESULT: u32 = 2;
const OUTCOME_FAILURE: u32 = 3;

static STATE: OnceLock<State> = OnceLock::new();

struct State {
    next_handle: AtomicU64,
    quarantined: AtomicBool,
    policy: OnceLock<RegistryPolicy>,
    accounting: Mutex<RegistryAccounting>,
    engines: Mutex<HashMap<u64, EngineEntry>>,
    controls: Mutex<HashMap<u64, WorkbenchCancellation>>,
    outcomes: Mutex<HashMap<u64, Outcome>>,
}

struct EngineEntry {
    engine: Arc<ExperimentalEngine>,
    known_capacity_bytes: usize,
}

enum Outcome {
    Engine(u64),
    Bytes { kind: u32, value: Vec<u8> },
}

impl Outcome {
    fn payload_bytes(&self) -> usize {
        match self {
            Self::Engine(_) => 0,
            Self::Bytes { value, .. } => value.len(),
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(1),
            quarantined: AtomicBool::new(false),
            policy: OnceLock::new(),
            accounting: Mutex::new(RegistryAccounting::default()),
            engines: Mutex::new(HashMap::new()),
            controls: Mutex::new(HashMap::new()),
            outcomes: Mutex::new(HashMap::new()),
        }
    }

    fn next_handle(&self) -> Result<u64, BoundaryFailure> {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        if handle == 0 || handle > MAX_HANDLE {
            self.quarantine();
            return Err(BoundaryFailure::new(
                "FXFFI0007",
                "native handle space exhausted",
            ));
        }
        Ok(handle)
    }

    fn configure_policy(&self, policy: RegistryPolicy) -> u32 {
        match self.policy.set(policy) {
            Ok(()) => 0,
            Err(conflicting) if self.policy.get() == Some(&conflicting) => 0,
            Err(_) => 1,
        }
    }

    fn insert_control(&self, control: WorkbenchCancellation) -> u64 {
        let Some(policy) = self.policy.get() else {
            return admission_status(ADMISSION_POLICY_REQUIRED);
        };
        let Ok(mut controls) = self.controls() else {
            return 0;
        };
        if controls.len() >= policy.control_limit {
            return admission_status(ADMISSION_CONTROL_COUNT_EXHAUSTED);
        }
        let Ok(handle) = self.next_handle() else {
            return 0;
        };
        controls.insert(handle, control);
        handle
    }

    fn engines(&self) -> Result<MutexGuard<'_, HashMap<u64, EngineEntry>>, BoundaryFailure> {
        self.engines.lock().map_err(|_| {
            self.quarantine();
            BoundaryFailure::new("FXFFI0008", "native engine registry is poisoned")
        })
    }

    fn accounting(&self) -> Result<MutexGuard<'_, RegistryAccounting>, BoundaryFailure> {
        self.accounting.lock().map_err(|_| {
            self.quarantine();
            BoundaryFailure::new("FXFFI0008", "native registry accounting is poisoned")
        })
    }

    fn outcomes(&self) -> Result<MutexGuard<'_, HashMap<u64, Outcome>>, BoundaryFailure> {
        self.outcomes.lock().map_err(|_| {
            self.quarantine();
            BoundaryFailure::new("FXFFI0008", "native outcome registry is poisoned")
        })
    }

    fn controls(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<u64, WorkbenchCancellation>>, BoundaryFailure> {
        self.controls.lock().map_err(|_| {
            self.quarantine();
            BoundaryFailure::new("FXFFI0008", "native control registry is poisoned")
        })
    }

    fn insert_outcome(&self, outcome: Outcome) -> u64 {
        let outcome = match outcome {
            Outcome::Bytes { value, .. } if value.len() > MAX_OUTCOME_BYTES => failure_outcome(
                "FXFFI0014",
                "boundary",
                None,
                None,
                "native outcome envelope exceeds the configured byte limit",
            ),
            outcome => outcome,
        };
        let Some(policy) = self.policy.get().copied() else {
            return admission_status(ADMISSION_POLICY_REQUIRED);
        };
        let payload_bytes = outcome.payload_bytes();
        let Ok(mut accounting) = self.accounting() else {
            return 0;
        };
        let Ok(mut outcomes) = self.outcomes() else {
            return 0;
        };
        if outcomes.len() >= policy.outcome_limit {
            return admission_status(ADMISSION_OUTCOME_COUNT_EXHAUSTED);
        }
        let Some(next_outcome_bytes) = accounting.outcome_payload_bytes.checked_add(payload_bytes)
        else {
            return admission_status(ADMISSION_OUTCOME_BYTES_EXHAUSTED);
        };
        if next_outcome_bytes > policy.outcome_payload_byte_limit {
            return admission_status(ADMISSION_OUTCOME_BYTES_EXHAUSTED);
        }
        let Some(next_accounted_bytes) = accounting
            .engine_known_capacity_bytes
            .checked_add(next_outcome_bytes)
        else {
            return admission_status(ADMISSION_TOTAL_BYTES_EXHAUSTED);
        };
        if next_accounted_bytes > policy.accounted_byte_limit {
            return admission_status(ADMISSION_TOTAL_BYTES_EXHAUSTED);
        }
        let Ok(handle) = self.next_handle() else {
            return 0;
        };
        outcomes.insert(handle, outcome);
        accounting.outcome_payload_bytes = next_outcome_bytes;
        handle
    }

    fn insert_boundary_failure(&self, failure: &BoundaryFailure) -> u64 {
        self.insert_outcome(failure_outcome(
            failure.code,
            "boundary",
            None,
            None,
            &failure.detail,
        ))
    }

    fn insert_created_engine(&self, engine: ExperimentalEngine) -> u64 {
        let Some(policy) = self.policy.get().copied() else {
            return admission_status(ADMISSION_POLICY_REQUIRED);
        };
        let known_capacity_bytes = engine.retention_estimate().known_retained_capacity_bytes;
        let Ok(mut accounting) = self.accounting() else {
            return 0;
        };
        let Ok(mut engines) = self.engines() else {
            return 0;
        };
        let Ok(mut outcomes) = self.outcomes() else {
            return 0;
        };
        if engines.len() >= policy.engine_limit {
            return admission_status(ADMISSION_ENGINE_COUNT_EXHAUSTED);
        }
        if outcomes.len() >= policy.outcome_limit {
            return admission_status(ADMISSION_OUTCOME_COUNT_EXHAUSTED);
        }
        let Some(next_engine_bytes) = accounting
            .engine_known_capacity_bytes
            .checked_add(known_capacity_bytes)
        else {
            return admission_status(ADMISSION_ENGINE_BYTES_EXHAUSTED);
        };
        if next_engine_bytes > policy.engine_known_capacity_byte_limit {
            return admission_status(ADMISSION_ENGINE_BYTES_EXHAUSTED);
        }
        let Some(next_accounted_bytes) =
            next_engine_bytes.checked_add(accounting.outcome_payload_bytes)
        else {
            return admission_status(ADMISSION_TOTAL_BYTES_EXHAUSTED);
        };
        if next_accounted_bytes > policy.accounted_byte_limit {
            return admission_status(ADMISSION_TOTAL_BYTES_EXHAUSTED);
        }
        let Ok(engine_handle) = self.next_handle() else {
            return 0;
        };
        let Ok(outcome_handle) = self.next_handle() else {
            return 0;
        };
        engines.insert(
            engine_handle,
            EngineEntry {
                engine: Arc::new(engine),
                known_capacity_bytes,
            },
        );
        outcomes.insert(outcome_handle, Outcome::Engine(engine_handle));
        accounting.engine_known_capacity_bytes = next_engine_bytes;
        outcome_handle
    }

    fn release_outcome(&self, outcome_handle: u64) -> bool {
        let Ok(mut accounting) = self.accounting() else {
            return false;
        };
        let Ok(mut outcomes) = self.outcomes() else {
            return false;
        };
        let Some(outcome) = outcomes.remove(&outcome_handle) else {
            return false;
        };
        accounting.outcome_payload_bytes = accounting
            .outcome_payload_bytes
            .checked_sub(outcome.payload_bytes())
            .expect("registered outcome charge must be conserved");
        true
    }

    fn release_engine(&self, engine_handle: u64) -> bool {
        let Ok(mut accounting) = self.accounting() else {
            return false;
        };
        let Ok(mut engines) = self.engines() else {
            return false;
        };
        let Some(engine) = engines.remove(&engine_handle) else {
            return false;
        };
        accounting.engine_known_capacity_bytes = accounting
            .engine_known_capacity_bytes
            .checked_sub(engine.known_capacity_bytes)
            .expect("registered engine charge must be conserved");
        true
    }

    fn quarantine(&self) {
        self.quarantined.store(true, Ordering::Release);
    }

    fn is_quarantined(&self) -> bool {
        self.quarantined.load(Ordering::Acquire)
    }

    #[cfg(test)]
    fn registry_observation(&self) -> RegistryObservation {
        let engines = self.engines.lock().expect("observe engine registry");
        let controls = self.controls.lock().expect("observe control registry");
        let outcomes = self.outcomes.lock().expect("observe outcome registry");
        RegistryObservation {
            engine_count: engines.len(),
            engine_capacity: engines.capacity(),
            control_count: controls.len(),
            control_capacity: controls.capacity(),
            outcome_count: outcomes.len(),
            outcome_capacity: outcomes.capacity(),
            outcome_payload_bytes: outcomes
                .values()
                .map(|outcome| match outcome {
                    Outcome::Engine(_) => 0,
                    Outcome::Bytes { value, .. } => value.len(),
                })
                .sum(),
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
struct RegistryObservation {
    engine_count: usize,
    engine_capacity: usize,
    control_count: usize,
    control_capacity: usize,
    outcome_count: usize,
    outcome_capacity: usize,
    outcome_payload_bytes: usize,
}

#[derive(Debug)]
struct BoundaryFailure {
    code: &'static str,
    detail: String,
}

impl BoundaryFailure {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

fn state() -> &'static State {
    STATE.get_or_init(State::new)
}

#[cfg(test)]
fn unlimited_test_policy() -> RegistryPolicy {
    RegistryPolicy {
        engine_limit: usize::MAX,
        control_limit: usize::MAX,
        outcome_limit: usize::MAX,
        outcome_payload_byte_limit: usize::MAX,
        engine_known_capacity_byte_limit: usize::MAX,
        accounted_byte_limit: usize::MAX,
    }
}

#[cfg(test)]
fn configured_test_state() -> State {
    let state = State::new();
    state
        .policy
        .set(unlimited_test_policy())
        .expect("fresh test state accepts policy");
    state
}

#[cfg(test)]
fn configure_global_test_policy() {
    let configured = state().policy.set(unlimited_test_policy());
    assert!(configured.is_ok() || state().policy.get() == Some(&unlimited_test_policy()));
}

fn guarded<T>(fallback: T, operation: impl FnOnce(&State) -> T) -> T {
    guarded_on(state(), fallback, operation)
}

fn guarded_on<T>(state: &State, fallback: T, operation: impl FnOnce(&State) -> T) -> T {
    if state.is_quarantined() {
        return fallback;
    }
    if let Ok(value) = catch_unwind(AssertUnwindSafe(|| operation(state))) {
        value
    } else {
        state.quarantine();
        fallback
    }
}

#[allow(unsafe_code)]
fn copy_input(
    pointer: *const u8,
    length: usize,
    maximum: usize,
) -> Result<Vec<u8>, BoundaryFailure> {
    if length > maximum || length > isize::MAX as usize {
        return Err(BoundaryFailure::new(
            "FXFFI0002",
            format!("input length {length} exceeds {maximum}"),
        ));
    }
    if length == 0 {
        return Ok(Vec::new());
    }
    if pointer.is_null() {
        return Err(BoundaryFailure::new(
            "FXFFI0001",
            "non-empty input has a null pointer",
        ));
    }
    // SAFETY: ADR-0008 requires the caller to provide a readable allocation of
    // `length` initialized bytes for this synchronous call. Length bounds and
    // nullability were validated above; the slice is copied immediately.
    Ok(unsafe { slice::from_raw_parts(pointer, length) }.to_vec())
}

#[allow(unsafe_code)]
fn copy_output(value: &[u8], pointer: *mut u8, capacity: usize) -> Result<(), BoundaryFailure> {
    if capacity < value.len() {
        return Err(BoundaryFailure::new(
            "FXFFI0006",
            format!("output capacity {capacity} is smaller than {}", value.len()),
        ));
    }
    if value.is_empty() {
        return Ok(());
    }
    if pointer.is_null() {
        return Err(BoundaryFailure::new(
            "FXFFI0001",
            "non-empty output has a null pointer",
        ));
    }
    // SAFETY: ADR-0008 requires a writable allocation of `capacity` bytes for
    // this call. Capacity and nullability were validated, and the regions cannot
    // overlap because `value` is Rust-owned while `pointer` is caller-owned.
    unsafe { ptr::copy_nonoverlapping(value.as_ptr(), pointer, value.len()) };
    Ok(())
}

fn decode_identity(value: Vec<u8>, field: &str) -> Result<String, BoundaryFailure> {
    String::from_utf8(value)
        .map_err(|_| BoundaryFailure::new("FXFFI0003", format!("{field} is not valid UTF-8")))
}

fn try_encode_failure(
    code: &str,
    category: &str,
    request_id: Option<&str>,
    location: Option<&WorkbenchLocation>,
    detail: &str,
) -> Option<Vec<u8>> {
    let start = location
        .map(|value| value.start.to_string())
        .unwrap_or_default();
    let end = location
        .map(|value| value.end.to_string())
        .unwrap_or_default();
    let fields = [
        code,
        category,
        request_id.unwrap_or_default(),
        location.map_or("", |value| &value.resource),
        &start,
        &end,
        detail,
    ];
    let encoded_length = fields.iter().try_fold(0_usize, |total, field| {
        u32::try_from(field.len()).ok()?;
        total
            .checked_add(size_of::<u32>())?
            .checked_add(field.len())
    })?;
    if encoded_length > MAX_OUTCOME_BYTES {
        return None;
    }
    let mut encoded = Vec::with_capacity(encoded_length);
    for field in fields {
        let length = u32::try_from(field.len()).ok()?;
        encoded.extend_from_slice(&length.to_le_bytes());
        encoded.extend_from_slice(field.as_bytes());
    }
    Some(encoded)
}

fn failure_outcome(
    code: &str,
    category: &str,
    request_id: Option<&str>,
    location: Option<&WorkbenchLocation>,
    detail: &str,
) -> Outcome {
    let value =
        try_encode_failure(code, category, request_id, location, detail).unwrap_or_else(|| {
            try_encode_failure(
                "FXFFI0014",
                "boundary",
                None,
                None,
                "native failure envelope exceeds the configured byte limit",
            )
            .expect("static bounded-envelope failure must fit")
        });
    Outcome::Bytes {
        kind: OUTCOME_FAILURE,
        value,
    }
}

fn engine_failure(failure: &WorkbenchFailure) -> Outcome {
    failure_outcome(
        &failure.code,
        &failure.category,
        failure.request_id.as_deref(),
        failure.location.as_deref(),
        &failure.detail,
    )
}

fn decode_flag(value: u32, field: &str) -> Result<bool, BoundaryFailure> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(BoundaryFailure::new(
            "FXFFI0012",
            format!("{field} flag must be zero or one"),
        )),
    }
}

fn insert_created_engine(state: &State, result: Result<ExperimentalEngine, CreateFailure>) -> u64 {
    match result {
        Ok(engine) => state.insert_created_engine(engine),
        Err(CreateFailure::Boundary(failure)) => state.insert_boundary_failure(&failure),
        Err(CreateFailure::Engine(failure)) => state.insert_outcome(engine_failure(&failure)),
    }
}

/// Returns the explicitly unstable native workbench ABI version.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_abi_version() -> u32 {
    guarded(u32::MAX, |_| ABI_VERSION)
}

/// Configures the immutable process-wide registry admission policy.
///
/// Zero means success, one means a conflicting policy was already configured,
/// and two means at least one limit cannot be represented on this platform.
#[allow(unsafe_code, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_configure_registry_policy(
    max_engines: u64,
    max_controls: u64,
    max_outcomes: u64,
    max_outcome_payload_bytes: u64,
    max_engine_known_capacity_bytes: u64,
    max_accounted_bytes: u64,
) -> u32 {
    guarded(3, |state| {
        let Some(policy) = (|| {
            Some(RegistryPolicy {
                engine_limit: decode_policy_limit(max_engines)?,
                control_limit: decode_policy_limit(max_controls)?,
                outcome_limit: decode_policy_limit(max_outcomes)?,
                outcome_payload_byte_limit: decode_policy_limit(max_outcome_payload_bytes)?,
                engine_known_capacity_byte_limit: decode_policy_limit(
                    max_engine_known_capacity_bytes,
                )?,
                accounted_byte_limit: decode_policy_limit(max_accounted_bytes)?,
            })
        })() else {
            return 2;
        };
        state.configure_policy(policy)
    })
}

/// Copies resources and creates one retained experimental engine.
#[allow(unsafe_code, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_create(
    source_identity_pointer: *const u8,
    source_identity_length: usize,
    source_pointer: *const u8,
    source_length: usize,
    stylesheet_identity_pointer: *const u8,
    stylesheet_identity_length: usize,
    stylesheet_pointer: *const u8,
    stylesheet_length: usize,
) -> u64 {
    guarded(0, |state| {
        if state.policy.get().is_none() {
            return admission_status(ADMISSION_POLICY_REQUIRED);
        }
        let result = (|| {
            let source_identity = decode_identity(
                copy_input(
                    source_identity_pointer,
                    source_identity_length,
                    MAX_IDENTITY_BYTES,
                )?,
                "source identity",
            )?;
            let source = copy_input(source_pointer, source_length, MAX_RESOURCE_BYTES)?;
            let stylesheet_identity = decode_identity(
                copy_input(
                    stylesheet_identity_pointer,
                    stylesheet_identity_length,
                    MAX_IDENTITY_BYTES,
                )?,
                "stylesheet identity",
            )?;
            let stylesheet = copy_input(stylesheet_pointer, stylesheet_length, MAX_RESOURCE_BYTES)?;
            ExperimentalEngine::new(
                source_identity,
                source,
                stylesheet_identity,
                stylesheet,
                WorkbenchLimits::default(),
            )
            .map_err(CreateFailure::Engine)
        })();
        insert_created_engine(state, result)
    })
}

/// Copies one optional stylesheet dependency and explicit denial policy before
/// creating a retained experimental engine.
#[allow(unsafe_code, clippy::too_many_arguments)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_create_with_stylesheet_dependency(
    source_identity_pointer: *const u8,
    source_identity_length: usize,
    source_pointer: *const u8,
    source_length: usize,
    stylesheet_identity_pointer: *const u8,
    stylesheet_identity_length: usize,
    stylesheet_pointer: *const u8,
    stylesheet_length: usize,
    dependency_identity_pointer: *const u8,
    dependency_identity_length: usize,
    dependency_pointer: *const u8,
    dependency_length: usize,
    admit_dependency: u32,
    deny_dependency: u32,
) -> u64 {
    guarded(0, |state| {
        if state.policy.get().is_none() {
            return admission_status(ADMISSION_POLICY_REQUIRED);
        }
        let result = (|| {
            let admitted = decode_flag(admit_dependency, "dependency admission")?;
            let denied = decode_flag(deny_dependency, "dependency denial")?;
            if !admitted && dependency_length != 0 {
                return Err(BoundaryFailure::new(
                    "FXFFI0013",
                    "unadmitted dependency must not carry resource bytes",
                )
                .into());
            }
            let source_identity = decode_identity(
                copy_input(
                    source_identity_pointer,
                    source_identity_length,
                    MAX_IDENTITY_BYTES,
                )?,
                "source identity",
            )?;
            let source = copy_input(source_pointer, source_length, MAX_RESOURCE_BYTES)?;
            let stylesheet_identity = decode_identity(
                copy_input(
                    stylesheet_identity_pointer,
                    stylesheet_identity_length,
                    MAX_IDENTITY_BYTES,
                )?,
                "stylesheet identity",
            )?;
            let stylesheet = copy_input(stylesheet_pointer, stylesheet_length, MAX_RESOURCE_BYTES)?;
            let dependency_identity = decode_identity(
                copy_input(
                    dependency_identity_pointer,
                    dependency_identity_length,
                    MAX_IDENTITY_BYTES,
                )?,
                "dependency identity",
            )?;
            let dependency = copy_input(dependency_pointer, dependency_length, MAX_RESOURCE_BYTES)?;
            ExperimentalEngine::new_with_stylesheet_resources(
                source_identity,
                source,
                stylesheet_identity,
                stylesheet,
                WorkbenchStylesheetResources {
                    dependencies: admitted
                        .then_some(WorkbenchResource {
                            identity: dependency_identity.clone(),
                            bytes: dependency,
                        })
                        .into_iter()
                        .collect(),
                    denied_identities: denied.then_some(dependency_identity).into_iter().collect(),
                },
                WorkbenchLimits::default(),
            )
            .map_err(CreateFailure::Engine)
        })();
        insert_created_engine(state, result)
    })
}

enum CreateFailure {
    Boundary(BoundaryFailure),
    Engine(WorkbenchFailure),
}

impl From<BoundaryFailure> for CreateFailure {
    fn from(value: BoundaryFailure) -> Self {
        Self::Boundary(value)
    }
}

/// Executes one request using a retained engine handle.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_transform(
    engine_handle: u64,
    request_identity_pointer: *const u8,
    request_identity_length: usize,
) -> u64 {
    guarded(0, |state| {
        let request_identity = match copy_input(
            request_identity_pointer,
            request_identity_length,
            MAX_IDENTITY_BYTES,
        )
        .and_then(|value| decode_identity(value, "request identity"))
        {
            Ok(value) => value,
            Err(failure) => return state.insert_boundary_failure(&failure),
        };
        let engine = {
            let Ok(engines) = state.engines() else {
                return 0;
            };
            let Some(engine) = engines
                .get(&engine_handle)
                .map(|entry| Arc::clone(&entry.engine))
            else {
                drop(engines);
                return state.insert_boundary_failure(&BoundaryFailure::new(
                    "FXFFI0004",
                    "unknown engine handle",
                ));
            };
            engine
        };
        insert_transform_outcome(state, engine.transform(&request_identity))
    })
}

/// Executes one request with scalar invocation-local controls.
///
/// `cancellation_requested` must be zero or one. Cancellation is already
/// signalled before execution begins; this is not an active callback or a hard
/// termination mechanism.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_transform_controlled(
    engine_handle: u64,
    request_identity_pointer: *const u8,
    request_identity_length: usize,
    cancellation_requested: u32,
    maximum_xslt_instructions: u64,
) -> u64 {
    guarded(0, |state| {
        let request_identity = match copy_input(
            request_identity_pointer,
            request_identity_length,
            MAX_IDENTITY_BYTES,
        )
        .and_then(|value| decode_identity(value, "request identity"))
        {
            Ok(value) => value,
            Err(failure) => return state.insert_boundary_failure(&failure),
        };
        if cancellation_requested > 1 {
            return state.insert_boundary_failure(&BoundaryFailure::new(
                "FXFFI0009",
                "cancellation flag must be zero or one",
            ));
        }
        let Ok(maximum_xslt_instructions) = usize::try_from(maximum_xslt_instructions) else {
            return state.insert_boundary_failure(&BoundaryFailure::new(
                "FXFFI0010",
                "XSLT instruction limit does not fit this platform",
            ));
        };
        let engine = {
            let Ok(engines) = state.engines() else {
                return 0;
            };
            let Some(engine) = engines
                .get(&engine_handle)
                .map(|entry| Arc::clone(&entry.engine))
            else {
                drop(engines);
                return state.insert_boundary_failure(&BoundaryFailure::new(
                    "FXFFI0004",
                    "unknown engine handle",
                ));
            };
            engine
        };
        let cancellation = WorkbenchCancellation::new();
        if cancellation_requested == 1 {
            cancellation.cancel();
        }
        insert_transform_outcome(
            state,
            engine.transform_with_invocation_policy(
                &request_identity,
                cancellation,
                maximum_xslt_instructions,
            ),
        )
    })
}

/// Creates a Rust-owned active cancellation control handle.
///
/// `first_charge_barrier` must be zero for an ordinary control or one for the
/// deterministic workbench-only barrier.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_control_create(first_charge_barrier: u32) -> u64 {
    guarded(0, |state| {
        let control = match first_charge_barrier {
            0 => WorkbenchCancellation::new(),
            1 => WorkbenchCancellation::with_first_charge_barrier(),
            _ => return 0,
        };
        state.insert_control(control)
    })
}

/// Executes one request with a retained active control handle.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_transform_with_control(
    engine_handle: u64,
    request_identity_pointer: *const u8,
    request_identity_length: usize,
    control_handle: u64,
    maximum_xslt_instructions: u64,
) -> u64 {
    guarded(0, |state| {
        let request_identity = match copy_input(
            request_identity_pointer,
            request_identity_length,
            MAX_IDENTITY_BYTES,
        )
        .and_then(|value| decode_identity(value, "request identity"))
        {
            Ok(value) => value,
            Err(failure) => return state.insert_boundary_failure(&failure),
        };
        let Ok(maximum_xslt_instructions) = usize::try_from(maximum_xslt_instructions) else {
            return state.insert_boundary_failure(&BoundaryFailure::new(
                "FXFFI0010",
                "XSLT instruction limit does not fit this platform",
            ));
        };
        let engine = {
            let Ok(engines) = state.engines() else {
                return 0;
            };
            let Some(engine) = engines
                .get(&engine_handle)
                .map(|entry| Arc::clone(&entry.engine))
            else {
                drop(engines);
                return state.insert_boundary_failure(&BoundaryFailure::new(
                    "FXFFI0004",
                    "unknown engine handle",
                ));
            };
            engine
        };
        let cancellation = {
            let Ok(controls) = state.controls() else {
                return 0;
            };
            let Some(control) = controls.get(&control_handle).cloned() else {
                drop(controls);
                return state.insert_boundary_failure(&BoundaryFailure::new(
                    "FXFFI0011",
                    "unknown control handle",
                ));
            };
            control
        };
        insert_transform_outcome(
            state,
            engine.transform_with_invocation_policy(
                &request_identity,
                cancellation,
                maximum_xslt_instructions,
            ),
        )
    })
}

/// Signals an active control. Returns one only for a live handle.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_control_cancel(control_handle: u64) -> u32 {
    guarded(0, |state| {
        let Ok(controls) = state.controls() else {
            return 0;
        };
        let Some(control) = controls.get(&control_handle) else {
            return 0;
        };
        control.cancel();
        1
    })
}

/// Reports whether the workbench-only first charge was observed.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_control_first_charge_observed(control_handle: u64) -> u32 {
    guarded(0, |state| {
        state
            .controls()
            .ok()
            .and_then(|controls| {
                controls
                    .get(&control_handle)
                    .map(WorkbenchCancellation::first_charge_observed)
            })
            .map_or(0, u32::from)
    })
}

/// Releases an active control handle. Returns one only when removed.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_control_release(control_handle: u64) -> u32 {
    guarded(0, |state| {
        state
            .controls()
            .ok()
            .and_then(|mut controls| controls.remove(&control_handle))
            .map_or(0, |_| 1)
    })
}

fn insert_transform_outcome(state: &State, result: Result<String, WorkbenchFailure>) -> u64 {
    match result {
        Ok(result) if result.len() <= MAX_OUTCOME_BYTES => state.insert_outcome(Outcome::Bytes {
            kind: OUTCOME_RESULT,
            value: result.into_bytes(),
        }),
        Ok(result) => state.insert_boundary_failure(&BoundaryFailure::new(
            "FXFFI0005",
            format!("result length {} exceeds {MAX_OUTCOME_BYTES}", result.len()),
        )),
        Err(failure) => state.insert_outcome(engine_failure(&failure)),
    }
}

/// Reports an outcome kind, or zero for an invalid handle/quarantined lane.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_outcome_kind(outcome_handle: u64) -> u32 {
    guarded(0, |state| {
        let Ok(outcomes) = state.outcomes() else {
            return 0;
        };
        match outcomes.get(&outcome_handle) {
            Some(Outcome::Engine(_)) => OUTCOME_ENGINE,
            Some(Outcome::Bytes { kind, .. }) => *kind,
            None => 0,
        }
    })
}

/// Reports an outcome byte length, or `usize::MAX` for an invalid handle.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_outcome_length(outcome_handle: u64) -> usize {
    guarded(usize::MAX, |state| {
        let Ok(outcomes) = state.outcomes() else {
            return usize::MAX;
        };
        match outcomes.get(&outcome_handle) {
            Some(Outcome::Bytes { value, .. }) => value.len(),
            _ => usize::MAX,
        }
    })
}

/// Copies outcome bytes. Zero means success; nonzero means boundary failure.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_outcome_copy(
    outcome_handle: u64,
    output_pointer: *mut u8,
    output_capacity: usize,
) -> u32 {
    guarded(4, |state| {
        let Ok(outcomes) = state.outcomes() else {
            return 4;
        };
        let Some(Outcome::Bytes { value, .. }) = outcomes.get(&outcome_handle) else {
            return 1;
        };
        match copy_output(value, output_pointer, output_capacity) {
            Ok(()) => 0,
            Err(failure) if failure.code == "FXFFI0006" => 2,
            Err(_) => 3,
        }
    })
}

/// Consumes a successful creation outcome and returns its engine handle.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_outcome_take_engine(outcome_handle: u64) -> u64 {
    guarded(0, |state| {
        let Ok(mut outcomes) = state.outcomes() else {
            return 0;
        };
        match outcomes.remove(&outcome_handle) {
            Some(Outcome::Engine(engine_handle)) => engine_handle,
            Some(other) => {
                outcomes.insert(outcome_handle, other);
                0
            }
            None => 0,
        }
    })
}

/// Releases an outcome handle. Returns one only when a handle was removed.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_outcome_release(outcome_handle: u64) -> u32 {
    guarded(0, |state| u32::from(state.release_outcome(outcome_handle)))
}

/// Releases an engine handle. Returns one only when a handle was removed.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_engine_release(engine_handle: u64) -> u32 {
    guarded(0, |state| u32::from(state.release_engine(engine_handle)))
}

/// Observes the current engine-handle cardinality for host-pressure experiments.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_registry_engine_count() -> usize {
    guarded(usize::MAX, |state| {
        state.engines().map_or(usize::MAX, |engines| engines.len())
    })
}

/// Observes the current control-handle cardinality for host-pressure experiments.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_registry_control_count() -> usize {
    guarded(usize::MAX, |state| {
        state
            .controls()
            .map_or(usize::MAX, |controls| controls.len())
    })
}

/// Observes the current outcome-handle cardinality for host-pressure experiments.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_registry_outcome_count() -> usize {
    guarded(usize::MAX, |state| {
        state
            .outcomes()
            .map_or(usize::MAX, |outcomes| outcomes.len())
    })
}

/// Observes exact bytes owned by byte-valued outcomes currently in the registry.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_registry_outcome_payload_bytes() -> usize {
    guarded(usize::MAX, |state| {
        let Ok(outcomes) = state.outcomes() else {
            return usize::MAX;
        };
        outcomes
            .values()
            .try_fold(0_usize, |total, outcome| match outcome {
                Outcome::Engine(_) => Some(total),
                Outcome::Bytes { value, .. } => total.checked_add(value.len()),
            })
            .unwrap_or(usize::MAX)
    })
}

#[cfg(test)]
#[path = "diagnostic_tests.rs"]
mod diagnostic_tests;

#[cfg(test)]
mod tests {
    use std::{
        process::Command,
        sync::{Arc, Barrier},
        thread,
    };

    use super::{
        ADMISSION_CONTROL_COUNT_EXHAUSTED, ADMISSION_ENGINE_BYTES_EXHAUSTED,
        ADMISSION_ENGINE_COUNT_EXHAUSTED, ADMISSION_OUTCOME_BYTES_EXHAUSTED,
        ADMISSION_OUTCOME_COUNT_EXHAUSTED, ADMISSION_POLICY_REQUIRED,
        ADMISSION_TOTAL_BYTES_EXHAUSTED, BoundaryFailure, ExperimentalEngine, MAX_OUTCOME_BYTES,
        OUTCOME_FAILURE, OUTCOME_RESULT, Outcome, RegistryPolicy, State, WorkbenchCancellation,
        WorkbenchLimits, admission_status, configure_global_test_policy, configured_test_state,
        copy_input, copy_output, failure_outcome, fastxslt_workbench_v0_control_cancel,
        fastxslt_workbench_v0_control_create, fastxslt_workbench_v0_control_first_charge_observed,
        fastxslt_workbench_v0_control_release, fastxslt_workbench_v0_create,
        fastxslt_workbench_v0_engine_release, fastxslt_workbench_v0_outcome_copy,
        fastxslt_workbench_v0_outcome_kind, fastxslt_workbench_v0_outcome_length,
        fastxslt_workbench_v0_outcome_release, fastxslt_workbench_v0_outcome_take_engine,
        fastxslt_workbench_v0_transform, fastxslt_workbench_v0_transform_controlled,
        fastxslt_workbench_v0_transform_with_control, guarded_on, insert_transform_outcome,
        try_encode_failure,
    };

    fn outcome_bytes(outcome: u64) -> Vec<u8> {
        let length = fastxslt_workbench_v0_outcome_length(outcome);
        let mut value = vec![0_u8; length];
        assert_eq!(
            fastxslt_workbench_v0_outcome_copy(outcome, value.as_mut_ptr(), value.len()),
            0
        );
        value
    }

    fn failure_fields(outcome: u64) -> Vec<String> {
        decode_failure_fields(&outcome_bytes(outcome))
    }

    fn decode_failure_fields(bytes: &[u8]) -> Vec<String> {
        let mut offset = 0;
        let mut fields = Vec::new();
        for _ in 0..7 {
            let length = u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("failure length field"),
            ) as usize;
            offset += 4;
            fields.push(
                String::from_utf8(bytes[offset..offset + length].to_vec())
                    .expect("UTF-8 failure field"),
            );
            offset += length;
        }
        assert_eq!(offset, bytes.len());
        fields
    }

    fn create_reference_engine() -> u64 {
        configure_global_test_policy();
        let source_identity = b"urn:w3c:xslt30:for-004:source";
        let source = include_bytes!("../../../vendor/xslt30-test/tests/expr/for/for03.xml");
        let stylesheet_identity = b"urn:w3c:xslt30:for-004:stylesheet";
        let stylesheet = include_bytes!("../../../vendor/xslt30-test/tests/expr/for/for-004.xsl");
        let creation = fastxslt_workbench_v0_create(
            source_identity.as_ptr(),
            source_identity.len(),
            source.as_ptr(),
            source.len(),
            stylesheet_identity.as_ptr(),
            stylesheet_identity.len(),
            stylesheet.as_ptr(),
            stylesheet.len(),
        );
        assert_eq!(fastxslt_workbench_v0_outcome_kind(creation), 1);
        let engine = fastxslt_workbench_v0_outcome_take_engine(creation);
        assert_ne!(engine, 0);
        assert_eq!(fastxslt_workbench_v0_outcome_kind(creation), 0);
        engine
    }

    fn reference_engine_value() -> ExperimentalEngine {
        ExperimentalEngine::new(
            "urn:w3c:xslt30:for-004:source".to_owned(),
            include_bytes!("../../../vendor/xslt30-test/tests/expr/for/for03.xml").to_vec(),
            "urn:w3c:xslt30:for-004:stylesheet".to_owned(),
            include_bytes!("../../../vendor/xslt30-test/tests/expr/for/for-004.xsl").to_vec(),
            WorkbenchLimits::default(),
        )
        .expect("create local reference engine")
    }

    fn process_working_set_bytes() -> Option<u64> {
        if cfg!(windows) {
            let script = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
            let output = Command::new("powershell")
                .args(["-NoProfile", "-Command", &script])
                .output()
                .ok()?;
            return String::from_utf8(output.stdout).ok()?.trim().parse().ok();
        }
        None
    }

    fn take_local_engine(state: &State, creation_outcome: u64) -> u64 {
        let Some(Outcome::Engine(engine_handle)) = state
            .outcomes()
            .expect("take local creation outcome")
            .remove(&creation_outcome)
        else {
            panic!("creation outcome must carry an engine handle");
        };
        engine_handle
    }

    #[test]
    fn pointer_copy_helpers_validate_before_their_exact_unsafe_operations() {
        assert_eq!(
            copy_input(std::ptr::null(), 0, 4).unwrap(),
            Vec::<u8>::new()
        );
        assert_eq!(
            copy_input(std::ptr::null(), 1, 4).unwrap_err().code,
            "FXFFI0001"
        );
        assert_eq!(
            copy_input(std::ptr::null(), 5, 4).unwrap_err().code,
            "FXFFI0002"
        );
        let input = [1_u8, 2, 3];
        assert_eq!(copy_input(input.as_ptr(), input.len(), 4).unwrap(), input);

        let mut output = [0_u8; 3];
        copy_output(&input, output.as_mut_ptr(), output.len()).unwrap();
        assert_eq!(output, input);
        assert_eq!(
            copy_output(&input, std::ptr::null_mut(), 2)
                .unwrap_err()
                .code,
            "FXFFI0006"
        );
        assert_eq!(
            copy_output(&input, std::ptr::null_mut(), input.len())
                .unwrap_err()
                .code,
            "FXFFI0001"
        );
    }

    fn policy_with(
        max_engines: usize,
        max_controls: usize,
        max_outcomes: usize,
        max_outcome_payload_bytes: usize,
        max_engine_known_capacity_bytes: usize,
        max_accounted_bytes: usize,
    ) -> RegistryPolicy {
        RegistryPolicy {
            engine_limit: max_engines,
            control_limit: max_controls,
            outcome_limit: max_outcomes,
            outcome_payload_byte_limit: max_outcome_payload_bytes,
            engine_known_capacity_byte_limit: max_engine_known_capacity_bytes,
            accounted_byte_limit: max_accounted_bytes,
        }
    }

    #[test]
    fn registry_policy_is_required_one_shot_and_idempotent_only_when_identical() {
        let state = State::new();
        let missing_policy = state.insert_outcome(Outcome::Bytes {
            kind: OUTCOME_RESULT,
            value: Vec::new(),
        });
        assert_eq!(missing_policy, admission_status(ADMISSION_POLICY_REQUIRED));
        assert!(!state.release_outcome(missing_policy));
        assert!(!state.release_engine(missing_policy));
        assert!(state.outcomes().expect("inspect outcomes").is_empty());

        let policy = policy_with(1, 1, 1, 4, 8, 12);
        assert_eq!(state.configure_policy(policy), 0);
        assert_eq!(state.configure_policy(policy), 0);
        assert_eq!(state.configure_policy(policy_with(2, 1, 1, 4, 8, 12)), 1);
        assert_eq!(state.policy.get(), Some(&policy));
    }

    #[test]
    fn count_and_byte_exhaustion_are_tagged_and_release_restores_capacity() {
        let state = State::new();
        assert_eq!(
            state.configure_policy(policy_with(usize::MAX, 1, 2, 4, usize::MAX, usize::MAX)),
            0
        );

        let control = state.insert_control(WorkbenchCancellation::new());
        assert_ne!(
            control & super::registry_admission::ADMISSION_STATUS_TAG,
            super::registry_admission::ADMISSION_STATUS_TAG
        );
        assert_eq!(
            state.insert_control(WorkbenchCancellation::new()),
            admission_status(ADMISSION_CONTROL_COUNT_EXHAUSTED)
        );

        let exact = state.insert_outcome(Outcome::Bytes {
            kind: OUTCOME_RESULT,
            value: vec![1; 4],
        });
        assert_eq!(
            state.insert_outcome(Outcome::Bytes {
                kind: OUTCOME_RESULT,
                value: vec![2],
            }),
            admission_status(ADMISSION_OUTCOME_BYTES_EXHAUSTED)
        );
        assert!(state.release_outcome(exact));
        let recovered = state.insert_outcome(Outcome::Bytes {
            kind: OUTCOME_RESULT,
            value: vec![3],
        });
        assert_ne!(
            recovered & super::registry_admission::ADMISSION_STATUS_TAG,
            super::registry_admission::ADMISSION_STATUS_TAG
        );

        let count_state = State::new();
        assert_eq!(
            count_state.configure_policy(policy_with(
                usize::MAX,
                usize::MAX,
                1,
                usize::MAX,
                usize::MAX,
                usize::MAX,
            )),
            0
        );
        let retained = count_state.insert_outcome(Outcome::Bytes {
            kind: OUTCOME_RESULT,
            value: Vec::new(),
        });
        assert_eq!(
            count_state.insert_outcome(Outcome::Bytes {
                kind: OUTCOME_RESULT,
                value: Vec::new(),
            }),
            admission_status(ADMISSION_OUTCOME_COUNT_EXHAUSTED)
        );
        assert!(count_state.release_outcome(retained));
        assert_ne!(
            count_state.insert_outcome(Outcome::Bytes {
                kind: OUTCOME_RESULT,
                value: Vec::new(),
            }),
            admission_status(ADMISSION_OUTCOME_COUNT_EXHAUSTED)
        );
    }

    #[test]
    fn engine_known_capacity_and_total_accounted_bytes_have_distinct_statuses() {
        let reference = reference_engine_value();
        let charge = reference.retention_estimate().known_retained_capacity_bytes;

        let engine_bytes = State::new();
        assert_eq!(
            engine_bytes.configure_policy(policy_with(
                1,
                usize::MAX,
                1,
                usize::MAX,
                charge - 1,
                usize::MAX,
            )),
            0
        );
        assert_eq!(
            engine_bytes.insert_created_engine(reference),
            admission_status(ADMISSION_ENGINE_BYTES_EXHAUSTED)
        );
        assert!(engine_bytes.engines().expect("inspect engines").is_empty());
        assert!(
            engine_bytes
                .outcomes()
                .expect("inspect outcomes")
                .is_empty()
        );

        let total = State::new();
        assert_eq!(
            total.configure_policy(policy_with(
                1,
                usize::MAX,
                1,
                usize::MAX,
                usize::MAX,
                charge - 1,
            )),
            0
        );
        assert_eq!(
            total.insert_created_engine(reference_engine_value()),
            admission_status(ADMISSION_TOTAL_BYTES_EXHAUSTED)
        );

        let count = State::new();
        assert_eq!(
            count.configure_policy(policy_with(
                0,
                usize::MAX,
                1,
                usize::MAX,
                usize::MAX,
                usize::MAX,
            )),
            0
        );
        assert_eq!(
            count.insert_created_engine(reference_engine_value()),
            admission_status(ADMISSION_ENGINE_COUNT_EXHAUSTED)
        );
    }

    #[test]
    fn concurrent_last_outcome_slot_has_exactly_one_winner() {
        let state = Arc::new(State::new());
        assert_eq!(
            state.configure_policy(policy_with(
                usize::MAX,
                usize::MAX,
                1,
                usize::MAX,
                usize::MAX,
                usize::MAX,
            )),
            0
        );
        let barrier = Arc::new(Barrier::new(3));
        let workers = (0..2)
            .map(|_| {
                let state = Arc::clone(&state);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    state.insert_outcome(Outcome::Bytes {
                        kind: OUTCOME_RESULT,
                        value: Vec::new(),
                    })
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        let results = workers
            .into_iter()
            .map(|worker| worker.join().expect("quota worker"))
            .collect::<Vec<_>>();
        assert_eq!(
            results
                .iter()
                .filter(|result| **result == admission_status(ADMISSION_OUTCOME_COUNT_EXHAUSTED))
                .count(),
            1
        );
        assert_eq!(state.outcomes().expect("inspect winner").len(), 1);
    }

    #[test]
    fn panic_quarantine_is_permanent_for_the_state() {
        let state = configured_test_state();
        assert_eq!(
            guarded_on(&state, 41, |_| panic!(
                "deliberate native workbench panic probe"
            )),
            41
        );
        assert!(state.is_quarantined());
        assert_eq!(guarded_on(&state, 42, |_| 99), 42);
        assert!(state.is_quarantined());
    }

    #[test]
    fn boundary_failure_keeps_static_code_and_owned_detail() {
        let failure = BoundaryFailure::new("FXFFI0004", "unknown engine handle");
        assert_eq!(failure.code, "FXFFI0004");
        assert_eq!(failure.detail, "unknown engine handle");
    }

    #[test]
    fn every_failure_envelope_is_preflighted_and_defensively_bounded() {
        let fixed_field_bytes = 7 * size_of::<u32>() + "C".len() + "c".len();
        let exact_detail = "x".repeat(MAX_OUTCOME_BYTES - fixed_field_bytes);
        let exact = try_encode_failure("C", "c", None, None, &exact_detail)
            .expect("exactly bounded failure envelope");
        assert_eq!(exact.len(), MAX_OUTCOME_BYTES);
        assert!(try_encode_failure("C", "c", None, None, &format!("{exact_detail}x")).is_none());

        let Outcome::Bytes { kind, value } = failure_outcome(
            "ENGINE",
            "invalid",
            Some("request"),
            None,
            &"y".repeat(MAX_OUTCOME_BYTES),
        ) else {
            panic!("failure must remain a byte outcome");
        };
        assert_eq!(kind, OUTCOME_FAILURE);
        assert!(value.len() <= MAX_OUTCOME_BYTES);
        let fields = decode_failure_fields(&value);
        assert_eq!(fields[0], "FXFFI0014");
        assert_eq!(fields[1], "boundary");

        let state = configured_test_state();
        let handle = state.insert_outcome(Outcome::Bytes {
            kind: OUTCOME_RESULT,
            value: vec![0; MAX_OUTCOME_BYTES + 1],
        });
        let outcomes = state.outcomes().expect("inspect local outcomes");
        let Outcome::Bytes { kind, value } = outcomes.get(&handle).expect("bounded replacement")
        else {
            panic!("oversized outcome must become a failure");
        };
        assert_eq!(*kind, OUTCOME_FAILURE);
        assert!(value.len() <= MAX_OUTCOME_BYTES);
        assert_eq!(decode_failure_fields(value)[0], "FXFFI0014");
    }

    #[test]
    fn creation_does_not_publish_an_engine_when_its_outcome_cannot_be_inserted() {
        let state = configured_test_state();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _outcomes = state.outcomes.lock().expect("lock local outcomes");
            panic!("poison outcome registry for atomic-publication probe");
        }));

        assert_eq!(state.insert_created_engine(reference_engine_value()), 0);
        assert!(state.engines.lock().expect("inspect engines").is_empty());
        assert!(state.is_quarantined());
    }

    #[test]
    #[ignore = "release-mode sacrificial registry-abandonment measurement"]
    fn measure_control_and_outcome_registry_abandonment() {
        const OPERATIONS: usize = 100_000;
        let baseline_rss = process_working_set_bytes();

        let controls_state = configured_test_state();
        let mut control_handles = Vec::with_capacity(OPERATIONS);
        for _ in 0..OPERATIONS {
            let handle = controls_state.next_handle().expect("control handle");
            controls_state
                .controls()
                .expect("control registry")
                .insert(handle, WorkbenchCancellation::new());
            control_handles.push(handle);
        }
        let controls_retained = controls_state.registry_observation();
        let controls_rss = process_working_set_bytes();
        {
            let mut controls = controls_state.controls().expect("release controls");
            for handle in control_handles {
                assert!(controls.remove(&handle).is_some());
            }
        }
        let controls_released = controls_state.registry_observation();
        let controls_released_rss = process_working_set_bytes();

        let outcomes_state = configured_test_state();
        let mut outcome_handles = Vec::with_capacity(OPERATIONS);
        for _ in 0..OPERATIONS {
            outcome_handles.push(
                outcomes_state.insert_boundary_failure(&BoundaryFailure::new(
                    "FXFFI-MEASURE",
                    "bounded abandonment measurement",
                )),
            );
        }
        let outcomes_retained = outcomes_state.registry_observation();
        let outcomes_rss = process_working_set_bytes();
        for handle in outcome_handles {
            assert!(outcomes_state.release_outcome(handle));
        }
        let outcomes_released = outcomes_state.registry_observation();
        let released_rss = process_working_set_bytes();

        assert_eq!(controls_retained.control_count, OPERATIONS);
        assert!(controls_retained.control_capacity >= OPERATIONS);
        assert_eq!(controls_retained.engine_count, 0);
        assert_eq!(controls_retained.engine_capacity, 0);
        assert_eq!(controls_retained.outcome_count, 0);
        assert_eq!(controls_retained.outcome_capacity, 0);
        assert_eq!(controls_retained.outcome_payload_bytes, 0);
        assert_eq!(controls_released.control_count, 0);
        assert!(controls_released.control_capacity > 0);
        assert_eq!(outcomes_retained.outcome_count, OPERATIONS);
        assert!(outcomes_retained.outcome_capacity >= OPERATIONS);
        assert_eq!(outcomes_retained.engine_count, 0);
        assert_eq!(outcomes_retained.control_count, 0);
        assert!(outcomes_retained.outcome_payload_bytes > 0);
        assert_eq!(outcomes_released.outcome_count, 0);
        assert!(outcomes_released.outcome_capacity > 0);
        println!(
            "operations={OPERATIONS} baseline_rss={baseline_rss:?} controls_retained={controls_retained:?} controls_rss={controls_rss:?} controls_released={controls_released:?} controls_released_rss={controls_released_rss:?} outcomes_retained={outcomes_retained:?} outcomes_rss={outcomes_rss:?} outcomes_released={outcomes_released:?} released_rss={released_rss:?}"
        );
    }

    #[test]
    #[ignore = "release-mode host-shaped native registry high-water measurement"]
    fn measure_host_shaped_registry_high_water() {
        const ENGINES_PER_GENERATION: usize = 4;
        const ACTIVE_CONTROLS: usize = 8;
        const RESULT_BURST: usize = 64;
        const DIAGNOSTIC_BURST: usize = 64;

        let state = configured_test_state();
        let baseline_rss = process_working_set_bytes();
        let mut old_generation = Vec::with_capacity(ENGINES_PER_GENERATION);
        let mut current_generation = Vec::with_capacity(ENGINES_PER_GENERATION);
        for generation in [&mut old_generation, &mut current_generation] {
            for _ in 0..ENGINES_PER_GENERATION {
                let creation = state.insert_created_engine(reference_engine_value());
                generation.push(take_local_engine(&state, creation));
            }
        }
        let engines_live = state.registry_observation();
        let engines_rss = process_working_set_bytes();

        let mut controls = Vec::with_capacity(ACTIVE_CONTROLS);
        for _ in 0..ACTIVE_CONTROLS {
            let handle = state.next_handle().expect("live control handle");
            state
                .controls()
                .expect("live control registry")
                .insert(handle, WorkbenchCancellation::new());
            controls.push(handle);
        }
        let controls_live = state.registry_observation();

        let mut outcomes = Vec::with_capacity(RESULT_BURST + DIAGNOSTIC_BURST);
        for _ in 0..RESULT_BURST {
            outcomes.push(insert_transform_outcome(
                &state,
                Ok("<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>".to_owned()),
            ));
        }
        for _ in 0..DIAGNOSTIC_BURST {
            outcomes.push(state.insert_boundary_failure(&BoundaryFailure::new(
                "FXFFI-LIVE",
                "bounded delayed-disposal diagnostic",
            )));
        }
        let burst_live = state.registry_observation();
        let burst_rss = process_working_set_bytes();

        for handle in outcomes {
            assert!(state.release_outcome(handle));
        }
        {
            let mut registry = state.controls().expect("release live controls");
            for handle in controls {
                assert!(registry.remove(&handle).is_some());
            }
        }
        for handle in old_generation {
            assert!(state.release_engine(handle));
        }
        let current_only = state.registry_observation();
        let current_only_rss = process_working_set_bytes();
        for handle in current_generation {
            assert!(state.release_engine(handle));
        }
        let released = state.registry_observation();
        let released_rss = process_working_set_bytes();

        assert_eq!(engines_live.engine_count, ENGINES_PER_GENERATION * 2);
        assert_eq!(controls_live.control_count, ACTIVE_CONTROLS);
        assert_eq!(burst_live.outcome_count, RESULT_BURST + DIAGNOSTIC_BURST);
        assert!(burst_live.outcome_payload_bytes > 0);
        assert_eq!(current_only.engine_count, ENGINES_PER_GENERATION);
        assert_eq!(current_only.control_count, 0);
        assert_eq!(current_only.outcome_count, 0);
        assert_eq!(released.engine_count, 0);
        println!(
            "engines_per_generation={ENGINES_PER_GENERATION} active_controls={ACTIVE_CONTROLS} result_burst={RESULT_BURST} diagnostic_burst={DIAGNOSTIC_BURST} baseline_rss={baseline_rss:?} engines_live={engines_live:?} engines_rss={engines_rss:?} controls_live={controls_live:?} burst_live={burst_live:?} burst_rss={burst_rss:?} current_only={current_only:?} current_only_rss={current_only_rss:?} released={released:?} released_rss={released_rss:?}"
        );
    }

    #[test]
    fn native_handles_execute_copy_and_release_the_safe_reference_lifecycle() {
        let engine = create_reference_engine();

        let request = b"native-reference";
        let outcome = fastxslt_workbench_v0_transform(engine, request.as_ptr(), request.len());
        assert_eq!(fastxslt_workbench_v0_outcome_kind(outcome), OUTCOME_RESULT);
        let length = fastxslt_workbench_v0_outcome_length(outcome);
        let mut result = vec![0_u8; length];
        assert_eq!(
            fastxslt_workbench_v0_outcome_copy(outcome, result.as_mut_ptr(), result.len()),
            0
        );
        assert_eq!(
            result,
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>36.02</out>"
        );
        assert_eq!(fastxslt_workbench_v0_outcome_release(outcome), 1);
        assert_eq!(fastxslt_workbench_v0_outcome_release(outcome), 0);

        let invalid_request = b"";
        let failure = fastxslt_workbench_v0_transform(
            engine,
            invalid_request.as_ptr(),
            invalid_request.len(),
        );
        assert_eq!(fastxslt_workbench_v0_outcome_kind(failure), OUTCOME_FAILURE);
        assert_eq!(fastxslt_workbench_v0_outcome_release(failure), 1);
        assert_eq!(fastxslt_workbench_v0_engine_release(engine), 1);
        assert_eq!(fastxslt_workbench_v0_engine_release(engine), 0);

        let unknown = fastxslt_workbench_v0_transform(engine, request.as_ptr(), request.len());
        assert_eq!(fastxslt_workbench_v0_outcome_kind(unknown), OUTCOME_FAILURE);
        assert_eq!(fastxslt_workbench_v0_outcome_release(unknown), 1);
    }

    #[test]
    fn scalar_controls_preserve_diagnostics_and_engine_reuse() {
        let engine = create_reference_engine();
        let cancellation_request = b"native-controlled-cancelled";
        let cancellation = fastxslt_workbench_v0_transform_controlled(
            engine,
            cancellation_request.as_ptr(),
            cancellation_request.len(),
            1,
            1_000_000,
        );
        assert_eq!(
            fastxslt_workbench_v0_outcome_kind(cancellation),
            OUTCOME_FAILURE
        );
        assert_eq!(
            failure_fields(cancellation),
            [
                "FXCT0001",
                "cancelled",
                "native-controlled-cancelled",
                "",
                "",
                "",
                "host cancellation observed while charging xslt-instruction work",
            ]
        );
        assert_eq!(fastxslt_workbench_v0_outcome_release(cancellation), 1);

        let budget_request = b"native-controlled-budget";
        let budget = fastxslt_workbench_v0_transform_controlled(
            engine,
            budget_request.as_ptr(),
            budget_request.len(),
            0,
            0,
        );
        assert_eq!(fastxslt_workbench_v0_outcome_kind(budget), OUTCOME_FAILURE);
        assert_eq!(
            failure_fields(budget),
            [
                "FXCT0002",
                "limit",
                "native-controlled-budget",
                "",
                "",
                "",
                "xslt-instruction work budget exhausted: limit 0, consumed 0, next charge 1",
            ]
        );
        assert_eq!(fastxslt_workbench_v0_outcome_release(budget), 1);

        let invalid_control = fastxslt_workbench_v0_transform_controlled(
            engine,
            budget_request.as_ptr(),
            budget_request.len(),
            2,
            1_000_000,
        );
        assert_eq!(
            failure_fields(invalid_control),
            [
                "FXFFI0009",
                "boundary",
                "",
                "",
                "",
                "",
                "cancellation flag must be zero or one",
            ]
        );
        assert_eq!(fastxslt_workbench_v0_outcome_release(invalid_control), 1);

        let recovery_request = b"native-controlled-recovery";
        let controlled_recovery = fastxslt_workbench_v0_transform(
            engine,
            recovery_request.as_ptr(),
            recovery_request.len(),
        );
        assert_eq!(
            fastxslt_workbench_v0_outcome_kind(controlled_recovery),
            OUTCOME_RESULT
        );
        assert_eq!(
            fastxslt_workbench_v0_outcome_release(controlled_recovery),
            1
        );
        assert_eq!(fastxslt_workbench_v0_engine_release(engine), 1);
    }

    #[test]
    fn active_control_cancels_after_a_real_charge_and_linearizes_release() {
        let engine = create_reference_engine();
        let target_control = fastxslt_workbench_v0_control_create(1);
        let unrelated_control = fastxslt_workbench_v0_control_create(0);
        assert_ne!(target_control, 0);
        assert_ne!(unrelated_control, 0);
        assert_eq!(fastxslt_workbench_v0_control_create(2), 0);

        let invocation = std::thread::spawn(move || {
            let request = b"native-active-cancelled";
            fastxslt_workbench_v0_transform_with_control(
                engine,
                request.as_ptr(),
                request.len(),
                target_control,
                1_000_000,
            )
        });
        let wait_started = std::time::Instant::now();
        while fastxslt_workbench_v0_control_first_charge_observed(target_control) == 0
            && wait_started.elapsed() < std::time::Duration::from_secs(5)
        {
            std::thread::yield_now();
        }
        assert_eq!(
            fastxslt_workbench_v0_control_first_charge_observed(target_control),
            1
        );
        assert_eq!(fastxslt_workbench_v0_control_cancel(unrelated_control), 1);
        assert!(!invocation.is_finished());
        assert_eq!(fastxslt_workbench_v0_control_cancel(target_control), 1);

        let cancellation = invocation.join().expect("native invocation joins");
        assert_eq!(
            failure_fields(cancellation),
            [
                "FXCT0001",
                "cancelled",
                "native-active-cancelled",
                "",
                "",
                "",
                "host cancellation observed while charging xslt-instruction work",
            ]
        );
        assert_eq!(fastxslt_workbench_v0_outcome_release(cancellation), 1);
        assert_eq!(fastxslt_workbench_v0_control_release(target_control), 1);
        assert_eq!(fastxslt_workbench_v0_control_cancel(target_control), 0);
        assert_eq!(fastxslt_workbench_v0_control_release(target_control), 0);
        assert_eq!(fastxslt_workbench_v0_control_release(unrelated_control), 1);

        let recovery_request = b"native-active-recovery";
        let recovery = fastxslt_workbench_v0_transform(
            engine,
            recovery_request.as_ptr(),
            recovery_request.len(),
        );
        assert_eq!(fastxslt_workbench_v0_outcome_kind(recovery), OUTCOME_RESULT);
        assert_eq!(fastxslt_workbench_v0_outcome_release(recovery), 1);
        assert_eq!(fastxslt_workbench_v0_engine_release(engine), 1);
    }
}
