//! Unstable native ABI used only by the ASP.NET boundary workbench.

use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
    sync::{
        Arc, Mutex, MutexGuard, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use fastxslt::workbench::{ExperimentalEngine, WorkbenchFailure, WorkbenchLimits};

const ABI_VERSION: u32 = 0;
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
    engines: Mutex<HashMap<u64, Arc<ExperimentalEngine>>>,
    outcomes: Mutex<HashMap<u64, Outcome>>,
}

enum Outcome {
    Engine(u64),
    Bytes { kind: u32, value: Vec<u8> },
}

impl State {
    fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(1),
            quarantined: AtomicBool::new(false),
            engines: Mutex::new(HashMap::new()),
            outcomes: Mutex::new(HashMap::new()),
        }
    }

    fn next_handle(&self) -> Result<u64, BoundaryFailure> {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        if handle == 0 || handle == u64::MAX {
            self.quarantine();
            return Err(BoundaryFailure::new(
                "FXFFI0007",
                "native handle space exhausted",
            ));
        }
        Ok(handle)
    }

    fn engines(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<u64, Arc<ExperimentalEngine>>>, BoundaryFailure> {
        self.engines.lock().map_err(|_| {
            self.quarantine();
            BoundaryFailure::new("FXFFI0008", "native engine registry is poisoned")
        })
    }

    fn outcomes(&self) -> Result<MutexGuard<'_, HashMap<u64, Outcome>>, BoundaryFailure> {
        self.outcomes.lock().map_err(|_| {
            self.quarantine();
            BoundaryFailure::new("FXFFI0008", "native outcome registry is poisoned")
        })
    }

    fn insert_outcome(&self, outcome: Outcome) -> u64 {
        let Ok(handle) = self.next_handle() else {
            return 0;
        };
        let Ok(mut outcomes) = self.outcomes() else {
            return 0;
        };
        outcomes.insert(handle, outcome);
        handle
    }

    fn insert_boundary_failure(&self, failure: &BoundaryFailure) -> u64 {
        self.insert_outcome(Outcome::Bytes {
            kind: OUTCOME_FAILURE,
            value: encode_failure(failure.code, "boundary", None, &failure.detail),
        })
    }

    fn quarantine(&self) {
        self.quarantined.store(true, Ordering::Release);
    }

    fn is_quarantined(&self) -> bool {
        self.quarantined.load(Ordering::Acquire)
    }
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

fn encode_failure(code: &str, category: &str, request_id: Option<&str>, detail: &str) -> Vec<u8> {
    let mut encoded = Vec::new();
    for field in [code, category, request_id.unwrap_or_default(), detail] {
        let length = u32::try_from(field.len()).expect("bounded failure field fits u32");
        encoded.extend_from_slice(&length.to_le_bytes());
        encoded.extend_from_slice(field.as_bytes());
    }
    encoded
}

fn engine_failure(failure: &WorkbenchFailure) -> Outcome {
    Outcome::Bytes {
        kind: OUTCOME_FAILURE,
        value: encode_failure(
            &failure.code,
            &failure.category,
            failure.request_id.as_deref(),
            &failure.detail,
        ),
    }
}

/// Returns the explicitly unstable native workbench ABI version.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_abi_version() -> u32 {
    guarded(u32::MAX, |_| ABI_VERSION)
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
        match result {
            Ok(engine) => {
                let Ok(engine_handle) = state.next_handle() else {
                    return 0;
                };
                let Ok(mut engines) = state.engines() else {
                    return 0;
                };
                engines.insert(engine_handle, Arc::new(engine));
                drop(engines);
                state.insert_outcome(Outcome::Engine(engine_handle))
            }
            Err(CreateFailure::Boundary(failure)) => state.insert_boundary_failure(&failure),
            Err(CreateFailure::Engine(failure)) => state.insert_outcome(engine_failure(&failure)),
        }
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
            let Some(engine) = engines.get(&engine_handle).cloned() else {
                drop(engines);
                return state.insert_boundary_failure(&BoundaryFailure::new(
                    "FXFFI0004",
                    "unknown engine handle",
                ));
            };
            engine
        };
        match engine.transform(&request_identity) {
            Ok(result) if result.len() <= MAX_OUTCOME_BYTES => {
                state.insert_outcome(Outcome::Bytes {
                    kind: OUTCOME_RESULT,
                    value: result.into_bytes(),
                })
            }
            Ok(result) => state.insert_boundary_failure(&BoundaryFailure::new(
                "FXFFI0005",
                format!("result length {} exceeds {MAX_OUTCOME_BYTES}", result.len()),
            )),
            Err(failure) => state.insert_outcome(engine_failure(&failure)),
        }
    })
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
    guarded(0, |state| {
        state
            .outcomes()
            .ok()
            .and_then(|mut outcomes| outcomes.remove(&outcome_handle))
            .map_or(0, |_| 1)
    })
}

/// Releases an engine handle. Returns one only when a handle was removed.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub extern "C" fn fastxslt_workbench_v0_engine_release(engine_handle: u64) -> u32 {
    guarded(0, |state| {
        state
            .engines()
            .ok()
            .and_then(|mut engines| engines.remove(&engine_handle))
            .map_or(0, |_| 1)
    })
}

#[cfg(test)]
mod tests {
    use super::{
        BoundaryFailure, OUTCOME_FAILURE, OUTCOME_RESULT, State, copy_input, copy_output,
        fastxslt_workbench_v0_create, fastxslt_workbench_v0_engine_release,
        fastxslt_workbench_v0_outcome_copy, fastxslt_workbench_v0_outcome_kind,
        fastxslt_workbench_v0_outcome_length, fastxslt_workbench_v0_outcome_release,
        fastxslt_workbench_v0_outcome_take_engine, fastxslt_workbench_v0_transform, guarded_on,
    };

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

    #[test]
    fn panic_quarantine_is_permanent_for_the_state() {
        let state = State::new();
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
    fn native_handles_execute_copy_and_release_the_safe_reference_lifecycle() {
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
}
