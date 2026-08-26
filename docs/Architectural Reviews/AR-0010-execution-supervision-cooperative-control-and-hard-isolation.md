# AR-0010: Execution Supervision, Cooperative Control, and Hard Isolation

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Dispatch supervision, execution control, worker health, and security containment |
| Trigger | A dispatcher was proposed as a security layer capable of detecting and recovering a rogue parser worker |
| Related ADRs | ADR-0002, ADR-0005 |
| Related reviews | AR-0002, AR-0003, AR-0004, AR-0008, AR-0009 |
| Related evidence | `docs/Evidence/thread-pool-design-review-2026-08-25.md`; private parser and output-limit tests; future fault-injection and host-boundary measurements |

## Architectural question

What security and availability guarantees may FastXSLT assign to a dispatcher
or execution supervisor, where must cooperative budget and cancellation checks
occur, and when does recovery from non-cooperating work require a process rather
than an in-process worker thread?

## Trigger and evidence

The project owner proposed a dispatch worker that hands jobs to a pool, monitors
them, and intervenes if a parser attack causes a worker to run rogue. The idea
has useful pressure: a volume executor needs bounded admission, queueing,
in-flight work, cancellation, deadlines, result correlation, and worker-health
observations.

An in-process supervisor cannot safely terminate an arbitrary Rust thread. The
thread may hold locks, mutate shared state, allocate, or be partway through an
invariant-preserving update. Rust's standard thread API supplies no safe forced
termination mechanism, and abandoning the thread does not reclaim its CPU,
locks, or address-space effects.

Current private evidence is narrower. XML parsing already enforces event and
depth limits and denies DTD/external-entity behavior. Resource admission,
transform-set request count, and serialized output are bounded. There is no
public cancellation contract, deadline, general work-unit budget, production
executor, panic-containment experiment, non-cooperating dependency case, or
isolated worker process.

## Ownership and constraints

- XML, XDM, XPath, XSLT execution, result construction, and serialization own
  deterministic counters at the points where their work is performed. A
  dispatcher cannot infer those costs accurately from outside the hot path.
- Invocation control owns the cancellation signal, deadline observation, and
  remaining per-invocation budgets. Checks must be cheap, bounded in latency,
  and independent of ambient global state.
- A dispatcher may own bounded admission, queue and in-flight policy, assignment,
  completion tracking, coarse deadline monitoring, cancellation signalling,
  and worker-health bookkeeping. It does not own XSLT semantics.
- A worker/job boundary may translate a caught unwind into an internal failure
  only where unwinding is enabled and containment is sound. Catching a panic
  does not prove shared engine or dependency state remains reusable.
- The host owns workload trust classification, service retry and availability
  policy, deployment topology, and whether hard isolation is required. An
  adapter must state the guarantees of the selected mode.
- Forced termination and reliable memory reclamation require a process or
  stronger isolation boundary. A Rust thread pool is not a hard security
  boundary and must never be advertised as one.
- Work counters and structural limits are the deterministic protection where
  practical. Wall-clock deadlines are operational safeguards affected by host
  load, scheduling, debugging, virtualization, and pauses.
- Any isolated mode must preserve ADR-0005 request/result semantics and use the
  same semantic engine. Transport and process lifecycle must not become a
  second execution backend.
- Supervision grants no new filesystem, network, entity, extension, or result
  publication authority. ADR-0002 memory-resident behavior remains the default
  inside each execution boundary.

## Failure and guarantee classes

| Condition | Primary control | Permitted recovery claim |
| --- | --- | --- |
| Hostile or expensive valid input | Structural and work budgets checked by the owning engine layer | Controlled per-request limit failure; worker may be reused if state invariants remain intact |
| Host cancellation | Cooperative signal checked at bounded work intervals | Controlled cancellation after the documented observation bound |
| Wall-clock deadline | Supervisor signal plus cooperative observation | Best-effort in process; hard deadline only with an isolation boundary that can be terminated |
| Rust/dependency panic | Sound job-boundary unwind containment where available | Invocation failure; worker/shared-state reuse requires explicit evidence |
| Deadlock, infinite loop without checks, blocked native call, or corrupted shared state | None sufficient inside the same process | No reliable thread-level recovery; terminate and replace an isolated process |
| Worker-process crash or hard timeout | Process supervisor | Discard worker state, correlate a structured operation failure, and replace according to host policy |

## Alternatives

### A. No dispatcher; callers invoke execution directly

This minimizes machinery and remains useful as a semantic reference path. It
does not provide shared queue limits, coarse health observations, or centralized
concurrency policy for volume work. Per-layer limits are still required.

### B. In-process dispatcher with cooperative workers

The dispatcher assigns independent requests to bounded workers and signals
cancellation or deadlines. Workers charge local budgets in engine hot paths.
This is the leading high-throughput baseline, but it cannot forcibly recover
from a non-cooperating thread and is defense in depth rather than hard isolation.

### C. In-process baseline plus an optional isolated worker-process mode

Trusted or already-bounded workloads use the cooperative path. Untrusted
workloads may use processes that can be killed and replaced after a hard
deadline or crash. This provides a real containment option while adding
transport, resource transfer, process lifecycle, deployment, and warm-state
costs that must be measured through AR-0002.

### D. Require all transformations to run out of process

This simplifies the public security story and maximizes fault isolation, but
imposes serialization, deployment, startup, and resource-sharing overhead on
all consumers. Current ASP.NET and workload evidence cannot justify it as the
only mode.

## Findings and uncertainties

- A dispatcher can be a meaningful defense-in-depth layer for policy around
  work; it cannot repair or safely kill an arbitrary thread.
- Parser attacks should normally become deterministic parser/XDM budget
  failures. Supervision complements those checks rather than replacing them.
- Cancellation, semantic work budget, wall-clock deadline, panic, and hard
  process termination are distinct controls and failure identities.
- Constant worker polling or a callback to the dispatcher for every parser
  event would put coordination in the hot path. Local counters with coarse
  external signalling are the leading shape, not an accepted implementation.
- Process isolation is the only current candidate for a forcible termination
  guarantee, but its ASP.NET deployment and performance costs are unmeasured.
- It is unknown which work counters provide useful deterministic bounds, how
  frequently each layer must check, how panic strategy affects containment, or
  whether third-party/native dependencies introduce non-cooperating regions.
- No evidence selects threads, async tasks, work stealing, queue technology,
  worker count, deadline defaults, retry policy, or a public supervisor API.

## Disposition

**Incubating.** Treat in-process supervision as bounded scheduling,
cooperative control, health observation, and failure correlation—not as hard
security isolation. Preserve an optional process-worker mode as the candidate
for workloads requiring forcible termination, but do not implement or promise
it before host and fault-injection evidence exists.

The immediate engine work is to make hostile input terminate through local,
structured budget checks. Do not add arbitrary thread termination, claim that a
timeout can safely reclaim an in-process worker, or allow a dispatcher to poll
or mutate semantic state.

## Required follow-up

- [ ] Define a private invocation-control experiment separating cancellation,
  deterministic work counters, structural limits, and wall-clock deadline.
- [ ] Inventory charge/check points and maximum observation gaps across XML,
  XDM construction, XPath evaluation, template dispatch, result construction,
  messages/diagnostics, and serialization.
- [ ] Add adversarial cases for excessive input bytes, names/attributes, nodes,
  sequence growth, recursion, expression work, diagnostics, and output.
- [ ] Fault-inject cancellation at multiple execution phases and prove request
  identity and partial-result policy remain structured and deterministic.
- [ ] Evaluate panic containment under the selected Rust panic strategy,
  including whether compiled, snapshot, worker, and shared dependency state may
  be reused after a caught unwind.
- [ ] Demonstrate with an isolated test helper that a non-cooperating worker
  cannot receive a safe in-process hard-kill guarantee and that a worker process
  can be terminated without poisoning the supervisor.
- [ ] Define distinct boundary categories for limit exhaustion, cancellation,
  deadline, worker crash, panic/internal failure, and supervisor/transport
  failure through AR-0004.
- [ ] Prototype the leading in-process and isolated modes through the ASP.NET
  workbench in AR-0002 and measure cold start, warm reuse, transfer, throughput,
  tail latency, and peak memory.
- [ ] Prove semantic and diagnostic parity across direct, in-process-dispatched,
  and isolated execution paths before presenting them as interchangeable modes.
- [ ] Threat-model tenant isolation, resource capability transfer, process
  identity, secret leakage, denial of service, and crash/restart loops before
  making a hardened-mode claim.

## Reopening triggers

Revisit this review when a representative host requires a hard termination
guarantee, cancellation observation is too slow, a dependency can block without
checking engine control, panic recovery proves unsafe, process-worker overhead
is measured, or a stronger sandbox such as WASM becomes a viable host boundary.

## Review history

- 2026-08-25 -- Opened as Incubating from the project-owner dispatcher/security
  discussion; separated cooperative supervision from hard process isolation.
