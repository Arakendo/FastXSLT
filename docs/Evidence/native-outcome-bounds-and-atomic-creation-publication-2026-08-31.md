# Native Outcome Bounds and Atomic Creation Publication

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Scope | Unpublished ADR-0008 native .NET workbench ABI |
| Review pressure | Adversarial review Finding 6; AR-0017 |
| Claim | Existing per-object bound and ownership obligations repaired; no aggregate registry quota selected |

## Bounded failure envelopes

Structured failure encoding now computes the complete seven-field frame length
with checked arithmetic before allocating its output vector. Every field must
fit the protocol's `u32` length and the aggregate must not exceed the existing
1 MiB `MAX_OUTCOME_BYTES` boundary.

If an engine or boundary failure cannot fit, the ABI retains a small static
`FXFFI0014 / boundary` failure explaining that the native failure envelope
exceeded its configured byte limit. `State::insert_outcome` also converts any
oversized byte outcome into that bounded failure before registry insertion, so
future call sites cannot bypass the invariant accidentally.

Focused tests establish:

- an envelope of exactly 1 MiB is accepted;
- a one-byte-larger envelope is rejected during length preflight;
- a huge engine diagnostic becomes the bounded replacement failure; and
- direct insertion of an oversized result is defensively converted to the same
  bounded failure.

This preserves machine-readable failure delivery without truncating semantic
diagnostics into plausible but incomplete content.

## Atomic creation publication

Successful creation now reserves both numeric handles and acquires both engine
and outcome registry locks before inserting either object. It then publishes
the engine and the creation outcome together while holding the fixed
engine-then-outcome lock order. Failure to reserve a handle or acquire either
registry leaves both maps unchanged.

A focused fault probe poisons the local outcome registry, attempts to publish a
real compiled/prepared engine, and verifies:

- the operation returns zero;
- the engine registry remains empty; and
- the local native lane enters its existing quarantine state.

This removes the path where an engine could be retained without any handle ever
being delivered to the caller. It does not claim transactional recovery from
allocation panic; ADR-0008 continues to quarantine the entire in-process lane
after a caught panic.

## Validation

- `cargo clippy -p fastxslt-dotnet-workbench --all-targets --all-features --
  -D warnings`: passed.
- `cargo test -p fastxslt-dotnet-workbench --all-features`: 12 passed.
- Complete `scripts/verify.ps1` workspace verification: passed.

## Remaining Finding 6 work

Engine, control, and outcome registries still have no aggregate count or
retained-byte ceiling. AR-0017 intentionally retains that decision until
test-only accounting and sacrificial-process abandonment measurements establish
normal concurrency, memory growth, and a useful rejection/recovery policy.
