//! Safe process-wide registry admission policy vocabulary and accounting.

pub(super) const ADMISSION_STATUS_TAG: u64 = 1 << 63;
pub(super) const MAX_HANDLE: u64 = ADMISSION_STATUS_TAG - 1;

pub(super) const ADMISSION_POLICY_REQUIRED: u64 = 1;
pub(super) const ADMISSION_ENGINE_COUNT_EXHAUSTED: u64 = 2;
pub(super) const ADMISSION_CONTROL_COUNT_EXHAUSTED: u64 = 3;
pub(super) const ADMISSION_OUTCOME_COUNT_EXHAUSTED: u64 = 4;
pub(super) const ADMISSION_OUTCOME_BYTES_EXHAUSTED: u64 = 5;
pub(super) const ADMISSION_ENGINE_BYTES_EXHAUSTED: u64 = 6;
pub(super) const ADMISSION_TOTAL_BYTES_EXHAUSTED: u64 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_field_names)] // The suffix distinguishes ceilings from live accounting.
pub(super) struct RegistryPolicy {
    pub(super) engine_limit: usize,
    pub(super) control_limit: usize,
    pub(super) outcome_limit: usize,
    pub(super) outcome_payload_byte_limit: usize,
    pub(super) engine_known_capacity_byte_limit: usize,
    pub(super) accounted_byte_limit: usize,
}

#[derive(Debug, Default)]
pub(super) struct RegistryAccounting {
    pub(super) engine_known_capacity_bytes: usize,
    pub(super) outcome_payload_bytes: usize,
}

pub(super) const fn admission_status(code: u64) -> u64 {
    ADMISSION_STATUS_TAG | code
}

pub(super) fn decode_policy_limit(value: u64) -> Option<usize> {
    if value == u64::MAX {
        Some(usize::MAX)
    } else {
        usize::try_from(value).ok()
    }
}
