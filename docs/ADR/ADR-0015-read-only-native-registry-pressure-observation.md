# ADR-0015: Read-Only Native Registry Pressure Observation

- Status: Accepted
- Date: 2026-08-31
- Related reviews: AR-0002, AR-0017
- Related ADRs: ADR-0003, ADR-0008, ADR-0010
- Related evidence: `docs/Evidence/aspnet-native-registry-pressure-calibration-2026-08-31.md`
- Supersedes: None

## Context

AR-0017 must distinguish legitimate ASP.NET handle pressure from abandonment
before it can compare aggregate registry policies. Rust-only test observation
cannot establish what the real managed `SafeHandle`, generation, request, and
result lifecycle retains across the native boundary. Whole-process memory alone
also cannot distinguish live FastXSLT ownership from allocator or operating-
system page retention.

The existing unpublished ABI can mutate and release handles but cannot report
current registry ownership to the ASP.NET experiment. Exporting a Rust struct,
map, iterator, callback, or borrowed view would add layout or lifetime surface
far beyond the measurement need.

## Decision

Extend only the unpublished `fastxslt_workbench_v0_` experiment with four
read-only scalar operations reporting:

1. current engine-handle count;
2. current control-handle count;
3. current outcome-handle count; and
4. exact bytes owned by byte-valued outcomes currently in the registry.

Each operation returns `usize::MAX` when observation cannot be completed. It
uses the existing panic guard and synchronized registry access, performs no
semantic work, accepts no pointer or foreign memory, and returns no Rust layout.
Individual observations are not an atomic composite snapshot. Host experiments
must sample at quiescent checkpoints or document concurrent skew.

The scalar operations are measurement instrumentation, not a supported public
metrics API. They do not expose map capacity, allocator behavior, handle values,
engine/XDM representation, prepared-engine byte estimates, tenant identity, or
quota state.

## Unsafe-surface impact

Rust requires an unsafe export attribute for each C symbol. This decision adds
four reviewed export attributes and four function-local `unsafe_code`
allowances, taking the enforced native module totals from 16 to 20 exports and
from 18 to 22 allowances. It adds **zero** unsafe blocks. ADR-0008's only two
unchecked operations remain the validated input slice and bounded output copy.

No callback, borrowed memory, raw-pointer dereference, allocator transfer, or
new safety invariant is admitted. The normal engine and every other crate remain
unsafe-free under the existing workspace gate.

## Non-decisions

This ADR does not select or expose:

- a registry quota, rejection diagnostic, reserved failure handle, or scalar
  exhaustion status;
- map capacity, shrink policy, byte-exact engine accounting, or process-memory
  attribution;
- host-created domains, tenant ownership, or cross-process aggregation;
- a stable ABI version or supported observability contract; or
- any mutation, eviction, enumeration, or release operation.

AR-0017 remains responsible for the eventual policy decision. Removing this
instrumentation after that decision remains permitted.

## Validation

- Build and call all four scalars through the real managed P/Invoke boundary.
- Retain and release overlapping engine generations and delayed outcomes through
  managed ownership while checking exact count and payload deltas.
- Preserve the unchanged XSLT30 semantic result under observation.
- Assert logical registry ownership returns to its pre-experiment baseline.
- Keep `scripts/verify.ps1` enforcing exactly two unsafe blocks, 20 export
  attributes, and 22 scoped allowances in the sole native workbench module.
- Run normal Rust gates plus the ASP.NET registry-pressure harness.

Revisit if observation becomes concurrent and requires an atomic snapshot, an
engine-retained-byte estimate crosses the ABI, a supported metrics surface is
requested, or any additional unsafe operation is proposed.
