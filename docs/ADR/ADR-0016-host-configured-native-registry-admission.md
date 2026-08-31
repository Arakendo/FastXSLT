# ADR-0016: Host-Owned Operational Policy and Native Registry Admission

- Status: Accepted
- Date: 2026-08-31
- Related reviews: AR-0002, AR-0010, AR-0017
- Related ADRs: ADR-0002, ADR-0008, ADR-0010, ADR-0015
- Related evidence: `docs/Evidence/native-registry-candidate-policy-replay-2026-08-31.md`; `docs/Evidence/native-registry-exhaustion-delivery-comparison-2026-08-31.md`; `docs/Evidence/prepared-engine-retention-estimator-calibration-2026-08-31.md`; `docs/Evidence/aspnet-native-registry-pressure-calibration-2026-08-31.md`; `docs/Evidence/aspnet-native-registry-burst-pressure-2026-08-31.md`
- Supersedes: None

## Context

The unpublished native .NET workbench retains engines, active controls, and
outcomes in process-wide registries until the foreign caller releases their
numeric handles. Managed `SafeHandle` ownership makes normal disposal reliable,
but cannot bound a buggy direct ABI caller, delayed finalization, or deliberate
abandonment.

AR-0017 established that count alone is not a useful memory model. Outcome
payload bytes are exactly attributable, while prepared-engine size varies
materially with source and stylesheet shape. A private compositional estimate
tracks known retained engine capacities but remains a representation-specific
lower bound rather than allocator-exact memory. Logical release and operating-
system memory settlement are also observably different events.

The mechanism is decision-ready even though no universal production threshold
is. Different ASP.NET services and batch hosts have different concurrency,
generation overlap, application headroom, and recovery requirements.

## Decision

Adopt this cross-cutting default:

> **Host-Owned Policy Principle:** When a limit, capacity, concurrency level,
> timeout, retention budget, retry policy, deployment threshold, or similar
> operational value depends on consumer workload or environment, FastXSLT
> provides the mechanism and enforces the supplied value while the host owns
> the number—unless a standard, safety contract, ABI constraint, or engine
> invariant requires FastXSLT to fix it.

FastXSLT owns transformation semantics, deterministic enforcement, accounting
definitions, machine-readable failure behavior, and the range of values the
implementation can represent safely. Hosts own deployment authority, workload
budgets, memory headroom, concurrency, retention, timeout selection, retry and
publication policy, and the choice between trusted native and hard-isolated
execution.

The principle is a default, not a transfer of semantic responsibility. Values
defined by the selected standards profile remain standard-owned. ABI widths,
encoded envelope bounds, unsafe preconditions, parser/representation safety
ceilings, and other correctness invariants remain implementation-owned. A host
cannot configure FastXSLT into violating its semantic or safety contract.

Documentation may later provide examples, calculators, or explicitly named
presets. Such guidance is consumer evidence, not one architectural number for
all deployments.

### Native registry application

The native adapter will support one **host-supplied, process-wide registry
admission policy**. FastXSLT defines the dimensions and deterministic admission
behavior; the embedding host supplies every numeric limit.

The policy contains:

1. maximum retained engine handles;
2. maximum retained active-control handles;
3. maximum retained outcome handles;
4. maximum exact bytes owned by byte-valued outcomes;
5. maximum aggregate known prepared-engine capacity charge; and
6. maximum aggregate accounted bytes, currently the checked sum of dimensions
   four and five.

The engine-capacity charge uses the private compositional observation already
calibrated by AR-0017. Its name and documentation must say **known prepared
capacity**, not actual, allocated, resident, private, or process memory. Registry
map allocation, allocator metadata, `Arc` headers not owned by the observation,
CLR objects, managed copies, and unrelated process memory are outside that byte
sum. Separate count limits bound controls and unaccounted per-entry pressure.

No production numeric defaults are selected. A host must configure the policy
explicitly before the first handle-producing operation. Choosing the maximum
representable value is the explicit experimental opt-out for a dimension; an
omitted policy is not silently interpreted as unlimited.

### Configuration lifecycle

Configuration is one-shot and process-wide because the current ABI owns one
anonymous registry set. The first successful configuration freezes the exact
policy. Repeating the identical configuration is idempotent; a different
configuration is rejected. Policy cannot be loosened, tightened, or replaced
while the native lane remains loaded, even when all handles have drained.
Changing it requires a fresh process or isolated worker.

Configuration and admission linearize through private safe-Rust synchronization.
No registry lock or policy lock may enclose compilation, preparation,
transformation, foreign-memory access, or result copying.

This decision does not introduce host-created registry domains. If independent
in-process consumers later need separate authority or attribution, that larger
lifecycle requires renewed review.

### Admission and release

Every registry insertion checks all applicable count and byte dimensions and
reserves its charge atomically with publication. Engine creation reserves the
engine and its creation outcome together or publishes neither. Release removes
the handle and its exact recorded charge in the same registry critical section.
No live handle is evicted, retired, or invalidated to make room.

Deterministic precedence is required when one operation would exceed multiple
dimensions: family count, family bytes/known capacity, then total accounted
bytes. Focused tests must prove the selected precedence, concurrent last-slot
admission, immediate capacity recovery after release, creation rollback, and
transform/cancel/release races.

Compilation and preparation necessarily occur before a new engine's retained
charge is known. Admission therefore bounds retained registry ownership, not
temporary construction peak. Existing per-resource and semantic work limits
remain necessary; a host requiring a hard peak-memory boundary must use an
externally limited isolated process.

### Capacity-independent exhaustion result

Admission exhaustion returns a versioned tagged scalar and does not allocate an
ordinary outcome. Ordinary handles remain positive values with the high bit
clear. Tagged admission statuses set the high bit, carry native ABI version zero
in the reserved version field, and carry a fixed status code identifying the
exhausted dimension. Handle allocation must stop before entering the tagged
namespace.

The initial status table distinguishes:

- policy not configured;
- engine-count exhaustion;
- control-count exhaustion;
- outcome-count exhaustion;
- exact outcome-payload-byte exhaustion;
- known prepared-engine-capacity exhaustion; and
- aggregate accounted-byte exhaustion.

The managed wrapper must recognize a tag before calling any outcome operation,
project it to a machine-readable managed admission exception, and never release
or reuse it as a handle. Unknown tag versions or codes fail closed. Zero remains
reserved for quarantine or an internal boundary failure that could not be
represented; it is not quota exhaustion.

### Guarantee boundary

Native admission budgets bound FastXSLT-accounted retained ownership. They do
not guarantee a maximum ASP.NET working set, private-byte count, CLR heap,
allocator footprint, construction peak, or total process memory.

Hosts choose limits with explicit application headroom. Hard memory ceilings,
forced abandonment reclamation, and kill-on-overrun behavior belong to the
isolated-worker profile combined with operating-system or container controls.

## Consequences

Future operational-number questions begin with host ownership rather than a
magic engine constant. Departures must identify the standard, correctness,
safety, ABI, or implementation invariant that makes host selection unsuitable.

Trusted native embeddings can select workload-appropriate limits without
baking one customer's memory envelope into the engine. The same semantics can
serve a small development host, a latency-sensitive ASP.NET service, and a
large batch worker under different admission policies.

The cost is a process-global initialization contract, additional safe registry
accounting, a tagged return namespace, managed failure mapping, and more race
tests. The known-capacity estimator becomes part of private admission mechanics
but does not become a stable public layout or allocator metric.

## Non-decisions

This ADR does not:

- select production thresholds or recommended percentages of host memory;
- promise allocator-exact or process-wide memory enforcement;
- admit live policy mutation, silent eviction, automatic generation retirement,
  or a memory-pressure callback;
- select native execution as a product default;
- expose prepared-engine estimates as a supported metrics API;
- create tenant or host registry domains; or
- replace invocation budgets, per-object envelope limits, process recycling,
  or isolated hard containment.

## Validation

- Test identical one-shot configuration and rejection of missing, conflicting,
  or post-freeze configuration.
- Test each quota dimension at its exact boundary and one unit beyond it.
- Prove concurrent last-slot admission has exactly one winner where applicable.
- Prove release restores the exact count/byte charge and never evicts another
  valid handle.
- Prove engine plus creation-outcome admission remains all-or-nothing.
- Prove tagged statuses consume no outcome capacity and cannot be inspected,
  copied, taken, or released as handles.
- Exercise the policy and managed mapping through the real ASP.NET/P/Invoke
  workbench with semantic and diagnostic parity sentinels.
- Preserve ADR-0008's two unsafe blocks. Any new export attributes or scoped
  allowances must be enumerated by the unsafe-surface gate and must not add a
  pointer operation.
- Run the normal Rust, documentation, native ABI, and ASP.NET workbench gates.

Revisit if multiple in-process consumers require independent authority, the
known-capacity estimator becomes materially inaccurate for admitted workloads,
hosts require live policy replacement, or native embedding is promoted from an
unpublished experiment to a supported product surface.

## Unsafe-surface impact

Policy configuration adds one scalar-only C export and one function-local
`unsafe_code` allowance solely for its export attribute. The enforced native
module totals advance from 20 to 21 exports and from 22 to 23 scoped
allowances. The operation accepts six `u64` values and returns one `u32`; it
does not accept a pointer, borrow foreign memory, expose Rust layout, or add an
unsafe block. ADR-0008's two validated copy operations remain the complete
unchecked surface.
