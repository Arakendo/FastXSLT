# AR-0017: Native Handle Registry Retention and Abandonment

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-31 |
| Last reviewed | 2026-08-31 |
| Scope | Unpublished native .NET workbench handle registries and process-memory ownership |
| Trigger | Adversarial review Finding 6 confirmed that foreign callers can retain engines, controls, and outcomes without an aggregate ceiling |
| Related ADRs | ADR-0002, ADR-0003, ADR-0008 |
| Related reviews | AR-0002, AR-0009, AR-0010, AR-0012 |
| Related evidence | `../Reviews/adversarial-engine-review-2026-08-30.md`; `../Evidence/native-outcome-bounds-and-atomic-creation-publication-2026-08-31.md`; `../Evidence/peer-ar-0017-review-monday-2026-08-31.md`; `../Evidence/native-registry-abandonment-measurement-2026-08-31.md`; future live-use and engine-retention measurements |

## Architectural question

How should an in-process native FastXSLT boundary bound, attribute, and reject
retained engine, control, and outcome handles when a foreign caller omits
release, without invalidating live handles or disguising process-wide policy as
an invocation budget?

## Trigger and evidence

The version-zero workbench ABI stores three process-global synchronized maps.
Handles remain until explicit release, and allocation is limited only by the
`u64` handle space and process memory. Managed `SafeHandle` wrappers provide
normal-path ownership but cannot bound a buggy direct ABI caller, delayed
finalization, or abandonment.

Two narrower defects do not depend on the aggregate policy: structured failure
encoding currently has no enforced `MAX_OUTCOME_BYTES` check, and successful
engine creation inserts the engine before it knows that the creation outcome
can be retained. Those must be corrected under ADR-0008's existing bounded
envelope and ownership contract. No measurement yet establishes a useful
process-wide count or byte threshold.

## Ownership and constraints

- The host owns deployment trust, tenant identity, process recycling, and which
  execution mode is appropriate. FastXSLT cannot infer a tenant from a C call.
- The native adapter owns numeric-handle validity, registry synchronization,
  release linearization, bounded boundary envelopes, and failure projection.
- Engines and outcomes have materially different retained sizes. A count-only
  limit must not be presented as deterministic memory accounting.
- Existing valid handles must not be evicted behind the caller's back. Silent
  least-recently-used eviction would turn later use into timing-dependent
  invalid-handle failures.
- Quota exhaustion must be machine-readable and must not require allocating an
  unbounded failure outcome in the already-exhausted registry.
- Registry locks must not enclose compilation, preparation, transformation, or
  foreign memory access. Panic quarantine remains whole-lane under ADR-0008.
- This review cannot make the unstable workbench ABI public or select native
  execution as a product default.

## Alternatives

### Rely only on managed ownership

Retain the current registries and treat omitted release as caller misuse. This
is simple but supplies no process-memory ceiling at the independently exported
ABI and does not answer the confirmed denial-of-service pressure.

### Fixed process-wide count ceilings

Set separate maximum counts for engines, controls, and outcomes and reject
before insertion. This is deterministic and cheap, but handles differ greatly
in retained size and unrelated consumers compete for one anonymous process
pool. The rejection path needs reserved or out-of-band failure capacity.

### Process-wide byte and count admission

Track bounded retained-byte estimates plus cardinality by registry and admit
atomically. This better represents engines and result envelopes but requires a
defined accounting model for prepared XDM, compiled state, map capacity, and
shared `Arc` ownership. An estimate must not be advertised as allocator-exact.

### Host-created registry domains

Require a host-owned context/domain handle with independent ceilings and place
engines, controls, and outcomes beneath it. This improves attribution and bulk
retirement but materially expands ABI lifecycle, failure, concurrency, and
managed-wrapper contracts. No current consumer has requested multi-tenant
in-process domains.

### Require process isolation for untrusted callers

Keep native handles conventionally owned for trusted callers and use the
isolated worker where abandonment must be reclaimed by terminating a process.
This is the strongest hard reclamation boundary, but it does not excuse
unbounded accidental growth in a long-lived trusted ASP.NET process.

## Findings and uncertainties

Per-object envelope bounds and atomic insertion rollback are existing ADR-0008
obligations, not optional quota policy. They are now executable: all failure
frames are preflighted against 1 MiB with a bounded replacement diagnostic, and
engine/outcome publication reserves both handles and locks before either map is
changed. Aggregate abandonment remains a product
decision. Unknowns include representative simultaneous engine generations,
normal control/outcome concurrency, prepared-engine retained bytes, failure
burst size, whether a native host needs multiple trust domains, and what
recovery action ASP.NET can take after process-wide exhaustion.

The leading experiment is separate count and retained-byte accounting with no
production ceiling. It should run 100,000 abandoned controls and outcomes in a
sacrificial process, then representative prepared engines, while recording RSS,
registry cardinality, payload bytes, map capacity, and release recovery.
Abandonment results must not be used as a proxy for legitimate live pressure.
A separate host-shaped probe must record high-water marks for overlapping
generations, active controls, result and diagnostic bursts, and delayed but
valid disposal. Any proposed policy must admit those observed live peaks while
still bounding abandoned state.

The first optimized abandonment run retained 100,000 controls at roughly
9.5 MiB above baseline. After releasing them all, cardinality returned to zero
but the map retained 69,777 effective capacity and working set remained about
5.3 MiB above baseline. A subsequent 100,000 bounded 80-byte failure outcomes
owned exactly 8 MiB of payload and added about 15.3 MiB working set above the
post-control-release point. Releasing them removed all logical entries and
payload but again retained map/allocator memory. This confirms abuse pressure
and release semantics; it does not select a ceiling or establish live-use need.

## Disposition

**Incubating.** Repair individual envelope bounds and insertion rollback under
ADR-0008, then measure abandonment. Do not select a fixed ceiling, eviction,
tenant model, or public failure contract until the measurements and a
representative host policy are reviewed.

## Required follow-up

- [x] Bound every encoded success and failure outcome before registry insertion.
- [x] Make engine plus creation-outcome insertion atomic or roll back the engine
  if the outcome cannot be delivered.
- [x] Add test-only registry cardinality, payload-byte, and capacity accounting.
- [x] Run 100,000-operation control and outcome abandonment/release probes in a
  sacrificial process and record whole-process memory.
- [ ] Record legitimate live-use high-water marks separately for overlapping
  generations, active controls, result/diagnostic bursts, and delayed valid
  disposal; do not calibrate policy from abandonment pressure alone.
- [ ] Measure representative prepared-engine retention separately from scalar
  controls and bounded outcome bytes.
- [ ] Compare fixed count, estimated-byte, host-domain, and isolated-process
  policies against an ASP.NET consumer's concurrency and recovery requirements.
- [ ] If a quota is selected, specify atomic admission, reserved failure
  delivery, concurrency races, release recovery, and host-visible diagnostics
  in an accepted ADR revision or superseding decision.

## Reopening triggers

Revisit when abandonment measurements exist, a host supplies simultaneous
generation and failure-burst requirements, native execution becomes a supported
profile candidate, or registry domains are needed for multiple in-process
consumers.

## Review history

- 2026-08-31 -- Opened as Incubating from adversarial review Finding 6;
  separated existing per-object bounds and rollback obligations from the open
  aggregate quota and attribution policy.
- 2026-08-31 -- Preflighted every structured envelope against the existing 1 MiB
  bound, added bounded `FXFFI0014` replacement failure, and made engine plus
  creation-outcome publication all-or-nothing before insertion. Aggregate
  abandonment policy remains open.
- 2026-08-31 -- Peer review retained Incubating and required separate live-use
  and abandonment high-water measurements before any ceiling is proposed.
- 2026-08-31 -- Measured 100,000 abandoned/released controls and outcomes with
  test-only cardinality, capacity, payload, and process working-set evidence.
  Empty maps retained capacity; live-use and prepared-engine measurements remain.
