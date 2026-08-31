# AR-0017: Native Handle Registry Retention and Abandonment

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-08-31 |
| Last reviewed | 2026-08-31 |
| Scope | Unpublished native .NET workbench handle registries and process-memory ownership |
| Trigger | Adversarial review Finding 6 confirmed that foreign callers can retain engines, controls, and outcomes without an aggregate ceiling |
| Related ADRs | ADR-0002, ADR-0003, ADR-0008, ADR-0015, ADR-0016 |
| Related reviews | AR-0002, AR-0009, AR-0010, AR-0012 |
| Related evidence | `../Reviews/adversarial-engine-review-2026-08-30.md`; `../Evidence/native-outcome-bounds-and-atomic-creation-publication-2026-08-31.md`; `../Evidence/peer-ar-0017-review-monday-2026-08-31.md`; `../Evidence/native-registry-abandonment-measurement-2026-08-31.md`; `../Evidence/native-registry-live-use-high-water-2026-08-31.md`; `../Evidence/aspnet-native-registry-pressure-calibration-2026-08-31.md`; `../Evidence/aspnet-native-registry-burst-pressure-2026-08-31.md`; `../Evidence/native-registry-candidate-policy-replay-2026-08-31.md`; `../Evidence/aspnet-native-sustained-generation-replacement-2026-08-31.md`; `../Evidence/native-registry-exhaustion-delivery-comparison-2026-08-31.md`; `../Evidence/aspnet-native-large-prepared-engine-pressure-2026-08-31.md`; `../Evidence/prepared-engine-retention-estimator-calibration-2026-08-31.md`; `../Evidence/aspnet-native-extended-reclamation-observation-2026-08-31.md`; `../Evidence/native-host-configured-registry-admission-2026-08-31.md` |

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

At opening, two narrower defects did not depend on the aggregate policy:
structured failure encoding had no enforced `MAX_OUTCOME_BYTES` check, and
successful engine creation inserted the engine before knowing that its creation
outcome could be retained. Those defects were corrected under ADR-0008's
existing bounded-envelope and ownership contract before aggregate policy was
selected. The subsequent experiments deliberately did not infer one universal
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

The first separate live-use probe retained two overlapping four-engine
generations, eight controls, and a delayed 128-outcome burst. Its high-water
cardinality was 144 handles, outcome payload was 8,640 bytes, and working set
rose about 828 KiB over baseline. The engine-only checkpoint attributed about
784 KiB of that process delta to eight tiny compiled/prepared `for-004`
engines. This is one host-shaped calibration point, not a supported maximum;
additional prepared workloads and real consumer bursts remain pressure.

The first ASP.NET burst tranche then held eight real transforms at their first
charge, retained 128 decoded structured failures, and retained eight validated
900,049-byte results. Its component high-water was 17 engines, eight controls,
136 outcomes, and exactly 7,210,248 outcome bytes. Every native ownership
dimension returned immediately to baseline after release. Process memory did
not: managed strings decoded from the large results remained eligible for later
collection, independently of native registry ownership and allocator/OS page
retention. This completes the first active-control, diagnostic, and near-limit
result pressure slice without making RSS a quota oracle.

Arithmetic replay against the generation and burst traces shows why outcome
count is useful but insufficient as a memory boundary. A 256-outcome count
ceiling alone can admit up to 256 MiB of bounded envelopes, whereas the observed
mixed burst occupied about 7.2 MiB and the earlier 256-result trace occupied
only 14,080 bytes. Exact aggregate outcome bytes therefore remain part of the
leading hybrid candidate. No threshold, safety margin, engine-byte estimate, or
exhaustion response is selected by that replay.

## Planned decision experiment

Policy calibration and policy correctness are separate evidence jobs:

```text
corpus-backed ASP.NET soak
    -> legitimate live-use and abandonment traces
    -> candidate ceiling calibration and recovery expectations

focused Rust and ABI tests
    -> atomic admission and release recovery
    -> deterministic races and no valid-handle eviction
```

The next ASP.NET experiment will exercise native pools at concurrency 1, 4, 8,
16, and 32 with two and three overlapping prepared generations. Small, medium,
and large memory-resident inputs will include unchanged admitted XSLT30
stylesheets where the current engine surface can execute them. The corpus cases
provide realistic compiled/prepared/result/diagnostic shapes and semantic
sentinels; they do not turn registry policy into a conformance denominator.

Each trace will distinguish legitimate live pressure from deliberate
abandonment and record, per registry family:

- current and legitimate high-water handle counts;
- deliberately abandoned handles;
- exact retained outcome-payload bytes;
- conservative estimated prepared-engine bytes, with its accounting scope;
- released/reclaimed handles and rejected admissions;
- host working set and private bytes;
- p50, p95, and p99 request latency; and
- semantic or diagnostic parity against the same workload without pressure.

The workload will include generation promotion while old leases drain, delayed
but valid outcome disposal, cancellation and diagnostic bursts, and results
near the one-megabyte per-object ceiling. A bounded soak will periodically
replace generations rather than merely creating unrelated engines in a loop.

After each burst and release, the experiment will sample both logical registry
ownership and process memory over time. This **reclamation half-life** separates
memory still owned by FastXSLT from pages retained by the allocator or operating
system after logical release. Working set alone must not be used to declare a
handle leak or quota failure.

Captured traces will be replayed against these unselected candidates:

1. separate process-wide count ceilings;
2. count ceilings plus exact aggregate outcome bytes and conservative estimated
   engine retention;
3. host-created registry domains; and
4. isolated workers when hard reclamation is required.

The leading candidate remains a configurable hybrid count/byte policy for
trusted native embedding plus isolated execution for hard reclamation, but the
trace must earn that decision. Host domains remain unselected absent a real
multi-tenant in-process consumer.

Quota failure delivery will compare two narrow mechanisms before an ABI
decision:

- a reserved static/sentinel structured failure that consumes no ordinary
  registry slot and cannot be released or reused as a normal handle; and
- an out-of-band scalar admission status that creates no outcome object.

The first retains uniform structured outcomes at the cost of special handle
semantics. The second makes admission failure explicit but changes the otherwise
uniform outcome contract.

The completed comparison also considered writing a scalar status beside an
output handle. It nominates a versioned **tagged scalar admission result**
instead: valid handles occupy one nonzero range and fixed admission statuses
occupy a disjoint tagged range in the existing `u64` return. This retains call
correlation without an ordinary outcome slot, writable foreign pointer,
thread-local error, or sentinel that pretends to be releasable. The shape
remains unaccepted until a quota decision defines its exact encoding, status
table, atomic admission semantics, and wrapper behavior.

Once a candidate is calibrated, focused Rust/ABI tests must prove concurrent
atomic admission, immediate capacity recovery after release, bounded failure
delivery, creation rollback, deterministic transform/cancel/release races, and
that no valid handle is silently evicted. The host soak cannot prove those
properties.

The ASP.NET generation trace now covers the planned 1/4/8/16/32
concurrency points, two- and three-generation overlap, and 16 through 256
deliberately delayed valid outcomes over unchanged XSLT30 `for-004`. At the
largest point, 96
experiment engines plus the ordinary singleton and 256 outcomes returned
logically to baseline immediately after explicit release. Working set and
private bytes remained above fresh-process baseline and fluctuated during the
one-second settlement window, validating the need for separate ownership and
process-memory observations. A second trace now covers eight active first-charge
controls, 128 retained structured failures, and eight retained 900 KB results;
exact native ownership again returned to baseline while managed and process
memory followed independent reclamation timelines. The captured observations
have also been replayed against count-only, count-plus-exact-outcome-byte, and
full-hybrid candidates using the private known-capacity engine estimate.

The first sustained replacement trace performed 32 full eight-engine
promotions while retaining exactly two old-generation leases. Its engine
high-water reached the predicted 25 handles including the ordinary singleton,
then stayed flat through the remaining promotions and returned exactly to
baseline. All 512 current-generation requests and 32 retired-generation
sentinels preserved generation identity and results. Observed replacement
latency was 4.63/6.54/7.81 ms at p50/p95/p99; request latency was
151.7/237.4/935.1 us. These are one-host calibration observations, not supported
latency or overlap guarantees.

The first large prepared-engine trace raised `for-004` to a generated 5,000-item
source and retained three ×16 generations. Each added generation contributed a
stable approximately 90 MB working set and 96 MB private bytes, or about
5.6/6.0 MB per engine for this shape. The 48 engines admitted only 8.18 MB of
raw source/stylesheet bytes but added about 289.4 MB private memory at peak.
This falsifies raw admitted bytes as a conservative prepared-engine estimate and
shows why count alone cannot describe memory. Exact logical ownership returned
to baseline; process memory remained above baseline after the one-second
settlement window.

A private compositional estimator now attributes known capacities to the
engine, prepared map, immutable XDM, and recursively owned compiled stylesheet
state. Across eight exact, source-heavy, and stylesheet-heavy calibration shapes it
covered 90.94% through 99.97% of production-like live allocator-requested
bytes, stayed below that comparison in every row, and tracked the 5,000-item,
900 KB text, namespace/attribute, 128-template, and 256-global shapes. A shallow
compiled estimate initially covered only 12.37% of the template-heavy row and
was rejected before documentation. The corrected observation remains a private
representation-specific lower bound: it does not cross the ABI, select a
threshold, or become allocator-exact memory accounting.

The same three ×16, 5,000-item shape now has a 30-second natural reclamation
trace. More than 98% of both peak process-memory deltas disappeared during
explicit disposal before the first zero-delay process sample. Private bytes
were slightly below baseline at ten and 30 seconds; working set remained within
about 2 MiB at 30 seconds and was non-monotonic. This closes the extended-window
observation but cannot support a precise half-life or universal reclamation
guarantee: the peak-to-half transition is below the harness's sampling
resolution.

ADR-0015 admits the four scalar observation exports used by this trace. It adds
no unsafe block, quota behavior, registry mutation, layout exposure, or public
metrics contract.

## Disposition

**Accepted through ADR-0016.** The native adapter supplies one immutable,
host-configured process-wide hybrid admission policy: separate family counts,
exact aggregate outcome bytes, private known prepared-engine capacity, and an
aggregate accounted-byte ceiling. The host owns every numeric limit and must
configure explicitly before handle admission; no production default is inferred
from the calibration traces. Capacity-independent exhaustion uses a versioned
tagged scalar status. Existing handles are never evicted, and isolated workers
remain the profile for hard memory ceilings or abandonment reclamation.

The accepted mechanism bounds FastXSLT-accounted retained ownership. It does
not claim allocator-exact accounting, a construction-peak bound, or a total
ASP.NET process-memory ceiling.

## Required follow-up

- [x] Bound every encoded success and failure outcome before registry insertion.
- [x] Make engine plus creation-outcome insertion atomic or roll back the engine
  if the outcome cannot be delivered.
- [x] Add test-only registry cardinality, payload-byte, and capacity accounting.
- [x] Run 100,000-operation control and outcome abandonment/release probes in a
  sacrificial process and record whole-process memory.
- [x] Record an initial legitimate live-use high-water separately for overlapping
  generations, active controls, result/diagnostic bursts, and delayed valid
  disposal; do not calibrate policy from abandonment pressure alone.
- [x] Measure the current tiny representative prepared-engine retention
  separately from scalar controls and bounded outcome bytes.
- [x] Measure one materially larger prepared-input shape separately and test
  whether raw admitted bytes or engine count explain its process pressure.
- [x] Calibrate a private compositional prepared-engine retention estimate over
  source-heavy and stylesheet-heavy shapes without exporting or enforcing it.
- [x] Compare fixed count, estimated-byte, host-domain, and isolated-process
  policies against the current ASP.NET consumer evidence; select host-supplied
  hybrid admission without inventing production thresholds.
- [x] Run the corpus-backed ASP.NET pressure matrix with generation overlap,
  delayed disposal, failure/result bursts, semantic sentinels, and separate
  legitimate versus abandoned high-water accounting.
- [x] Establish the first ASP.NET registry-observation and valid-retention trace
  across concurrency 1/4/8/16/32, two/three generations, and exact outcome-byte
  accounting while preserving the unchanged `for-004` result.
- [x] Exercise and separately account for real active controls, retained
  structured failures, and retained near-limit semantic results through the
  ASP.NET/native boundary.
- [x] Run the first sustained bounded generation-replacement trace with
  promoted and retired semantic sentinels plus replacement/request latency
  distributions.
- [x] Sample logical release and process-memory settlement separately after
  bounded result and prepared-engine bursts.
- [x] Extend natural reclamation observation beyond the prior one-second window;
  record that the peak-to-half transition occurred before the first
  post-disposal sample rather than inventing a precise half-life.
- [x] Replay the captured trace against count-only and hybrid count/byte
  candidates without enforcing either in the production path.
- [x] Compare reserved static/sentinel failure delivery with out-of-band scalar
  admission status before changing the ABI.
- [x] If a quota is selected, specify atomic admission, reserved failure
  delivery, concurrency races, release recovery, and host-visible diagnostics
  in an accepted ADR revision or superseding decision.

## Reopening triggers

Revisit if one process must isolate quota ownership between mutually untrusted
in-process consumers, measured estimator error makes the admitted accounting
unsafe or unusably conservative, a supported native profile requires a
production preset, or a real host requires policy replacement without starting
a fresh process.

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
- 2026-08-31 -- Measured a separate 144-handle live-use high-water with two ×4
  generations, eight controls, and 128 delayed outcomes. The engine-only phase
  retained eight tiny prepared engines before scalar pressure was added.
- 2026-08-31 -- Peer review defined the next decision experiment: corpus-backed
  ASP.NET calibration separate from focused quota-correctness tests, explicit
  legitimate/abandoned accounting, generation-overlap pressure, reclamation
  half-life, candidate-policy replay, and comparison of static versus scalar
  exhaustion delivery.
- 2026-08-31 -- Added read-only scalar registry observation and ran the first
  ASP.NET matrix through concurrency 32, three generations, and 256 delayed
  valid outcomes. Every row returned logical ownership to baseline; process
  memory remained non-monotonic during the one-second settlement trace.
- 2026-08-31 -- Added the first active-control, structured-failure, and
  near-limit-result ASP.NET burst. Eight live controls, 128 failures, and eight
  900 KB results returned exact registry ownership to baseline. Candidate replay
  retained count ceilings as abuse protection and nominated exact aggregate
  outcome bytes as a necessary hybrid dimension without selecting thresholds.
- 2026-08-31 -- Ran 32 sustained eight-engine promotions with two retained old
  leases. Engine ownership remained at the predicted 25-handle high-water after
  overlap filled, all 544 semantic observations passed, and exact ownership
  returned to baseline. Latency remains evidence, not a guarantee.
- 2026-08-31 -- Compared capacity-independent exhaustion delivery. A versioned
  tagged scalar admission result is nominated over a structured sentinel or
  writable output pointer, but no encoding or ABI behavior is accepted before
  quota policy is selected.
- 2026-08-31 -- Measured three ×16 engines over a 5,000-item prepared source.
  Each generation added about 96 MB private memory while aggregate admitted
  bytes remained only 8.18 MB. Logical ownership returned exactly to baseline;
  no general engine-byte estimator was inferred.
- 2026-08-31 -- Added and allocator-calibrated a private compositional
  prepared-engine estimate across the exact workload and seven generated shapes. Recursive compiled
  ownership repaired a deliberately exposed template-heavy blind spot; the
  estimator remains a lower-bound experiment and does not select quota policy.
- 2026-08-31 -- Extended the largest prepared-engine settlement trace to 30
  seconds. More than 98% of the peak deltas were gone before the first sample;
  process memory was near baseline by ten seconds, without supporting a
  portable reclamation-time guarantee.
- 2026-08-31 -- Accepted through ADR-0016. The host now owns explicit immutable
  limits over count, exact outcome bytes, known prepared-engine capacity, and
  aggregate accounted bytes. Tagged scalar exhaustion is capacity-independent;
  isolated execution remains the hard-reclamation boundary. No production
  thresholds were selected.
