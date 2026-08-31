# AR-0010: Execution Supervision, Cooperative Control, and Hard Isolation

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-26 |
| Scope | Dispatch supervision, execution control, worker health, and security containment |
| Trigger | A dispatcher was proposed as a security layer capable of detecting and recovering a rogue parser worker |
| Related ADRs | ADR-0002, ADR-0005 |
| Related reviews | AR-0002, AR-0003, AR-0004, AR-0008, AR-0009 |
| Related evidence | `docs/Evidence/thread-pool-design-review-2026-08-25.md`; `docs/Evidence/peer-ar-0010-review-monday-2026-08-25.md`; `docs/Evidence/private-invocation-control-charge-points-2026-08-25.md`; `docs/Evidence/aspnet-worker-recovery-and-generation-replacement-2026-08-26.md`; `docs/Evidence/aspnet-predispatch-cooperative-cancellation-2026-08-26.md`; `docs/Evidence/aspnet-active-cooperative-cancellation-2026-08-26.md`; `docs/Evidence/aspnet-natural-cancellation-races-2026-08-26.md`; `docs/Evidence/aspnet-deterministic-instruction-budget-2026-08-26.md`; `docs/Evidence/aspnet-worker-control-frame-serialization-2026-08-31.md`; `docs/Evidence/template-candidate-fanout-and-cancellation-gap-2026-08-31.md`; future fault-injection and host-boundary measurements |

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
- A private invocation-control experiment now separates an atomic cooperative
  cancellation signal from eight independent work counters. XML events,
  allocated XDM nodes, XDM string-value visits, XPath candidate-child visits,
  XSLT instructions, semantic result nodes, result text bytes, and serialized
  bytes charge where their work occurs.
- XPath child-axis accounting is data-dependent rather than one unit per
  expression. Broader operation weights, composed budgets, accounting overhead,
  and defaults remain unmeasured.
- Cancellation observation occurs at charge points. Work inside one parser or
  dependency call remains non-interruptible until that call returns, so no
  wall-clock observation bound follows from the current experiment.
- Deterministic test faults now signal cancellation after earlier work in each
  implemented phase. All eight paths retain request/domain identity, and the
  private transform-set reference returns no partial result set even when a
  sibling completed first. This does not select future batch failure collection.
- Dynamic XDM string value reaches result construction as ordered borrowed
  fragments. The runtime meters and retains each fragment directly rather than
  allocating an aggregate temporary string before the result-text limit.
- The golden path now asserts an exact eight-domain charge profile. Structural
  observation gaps are one named semantic unit, but work inside a dependency
  call, allocation, or fragment append remains variable in wall time.
- Template selection now charges a distinct `xslt-template-candidate` unit
  before every source or temporary-tree candidate test. Zero-limit and
  deterministic-signal regressions retain structured identity and bound the
  candidate observation interval at one. The largest local paired probe measured
  240.9 us uncharged versus 284.2 us charged for 33,024 candidates; an index and
  supported public default remain unselected.
- A local optimized microprobe observed 1.215–1.249 ns per successful charge
  versus 0.205–0.207 ns for its black-box loop baseline on the recorded machine.
  It is not an end-to-end overhead or host-performance result.
- The ASP.NET workbench now acknowledges a correlated non-cooperating request,
  forcibly terminates only its isolated process, declines to retry the ambiguous
  invocation, replaces the slot from the sealed generation, and preserves a
  sibling plus later execution. This is hard-isolation evidence, not
  cooperative cancellation or a production restart policy.
- An already-signalled host cancellation now reaches the isolated semantic path,
  produces the same exact structured diagnostic as the direct facade, and
  leaves the process and prepared generation reusable. The serial protocol does
  not yet support signalling after execution begins, so observation latency and
  cancellation/completion races remain unmeasured.
- The isolated worker now has a private one-active-invocation supervisor: a
  reader routes correlated control while execution runs separately. A
  first-charge barrier proved matching active cancellation, unrelated-signal
  rejection, exact failure identity, and same-process reuse. The barrier makes
  its 0.5392–1.2906 ms observations unsuitable as natural wall-clock evidence.
- A 25-trial unpaused 20,000-item sample produced 25 structured cancellations,
  a 0.1309 ms median signal-to-response, and same-worker recovery. The earlier
  500-item attempt committed completion first. This covers both race outcomes
  but not representative/adversarial observation bounds.
- A zero-instruction isolated invocation returned `FXCT0002 / limit`, was not
  retried, and left its compiled/prepared worker reusable. The boundary now has
  executable distinctions among deterministic exhaustion, cooperative
  cancellation, and hard process termination.
- The first ADR-0008 in-process candidate deliberately provides no forced-stop
  claim. Its native panic policy is whole-lane quarantine, while the isolated
  candidate retains process termination/replacement. Exported panic injection
  and managed quarantine recovery policy remain untested.

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

- [x] Define a private invocation-control experiment separating cancellation
  from deterministic per-domain work counters and structural limits; deadline
  remains deliberately absent.
- [x] Inventory the implemented XML, XDM, XPath, instruction, and serialization
  charge/check points.
- [x] Establish maximum observation gaps in current named semantic units and
  identify work hidden inside dependency calls and append chunks.
- [ ] Measure wall-clock cancellation observation under adversarial dependency
  calls and representative end-to-end workloads; do not infer it from units.
- [x] Extend accounting to semantic result-node creation and retained UTF-8 text
  bytes independently of serialization.
- [x] Avoid aggregate temporary dynamic string-value allocation by writing
  XDM-owned fragments through the result meter in semantic order.
- [ ] Extend accounting to messages and diagnostics.
- [x] Retain a reproducible release-mode microprobe for successful local charge
  cost, with environment and interpretation limits.
- [ ] Measure accounting enabled/disabled through representative complete
  transforms and the ASP.NET host boundary before selecting check frequency or
  defaults.
- [x] Force exhaustion in every currently implemented work domain and preserve
  domain plus request identity in the private structured failure.
- [x] Attribute matched-template candidate work separately from entered XSLT
  instructions and observe cancellation at every candidate boundary.
- [ ] Add adversarial cases for excessive input bytes, names/attributes, nodes,
  sequence growth, recursion, expression work, diagnostics, and output.
- [x] Fault-inject cancellation at multiple execution phases and prove request
  identity and partial-result policy remain structured and deterministic.
- [ ] Evaluate panic containment under the selected Rust panic strategy,
  including whether compiled, snapshot, worker, and shared dependency state may
  be reused after a caught unwind.
- [ ] Complete the non-cooperating-worker boundary comparison.
  - [ ] Demonstrate through an in-process helper that a non-cooperating worker
    cannot receive a safe thread-level hard-kill guarantee.
  - [x] Demonstrate through an isolated helper that a non-cooperating worker
    process can be terminated without poisoning the supervisor.
- [ ] Define distinct boundary categories for deadline, panic/internal failure,
  and supervisor/transport failure through AR-0004; limit exhaustion,
  cancellation, and worker termination now have private executable evidence.
- Concurrent cancellation producers now pass through a dedicated outbound
  frame serializer. A byte-fragmenting 10,000-pair stress recovered all 20,000
  bounded cancel frames exactly once; existing live-worker evidence separately
  preserves unrelated-signal rejection, correlated cancellation, and reuse.
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
- 2026-08-25 -- Peer review confirmed the guarantee classes, direct semantic
  reference path, layer-owned accounting, and caution around panic recovery;
  retained Incubating pending charge-point and fault-injection evidence.
- 2026-08-25 -- Added the first private invocation-control experiment with an
  atomic cancellation token and six layer-owned work domains. Retained
  Incubating because observation latency, weights, overhead, deadlines, panic,
  dispatch, and hard isolation remain untested.
- 2026-08-25 -- Injected deterministic cancellation after partial work in every
  implemented charge domain and retained request/domain identity. The private
  set returns no partial results, but public batch collection remains open.
- 2026-08-25 -- Added separate semantic result-node and UTF-8 text-byte work
  domains before serialization. Temporary dynamic string-value allocation,
  messages, diagnostics, observation gaps, and overhead remain unresolved.
- 2026-08-25 -- Replaced aggregate runtime string-value materialization with an
  XDM-owned controlled fragment sink feeding result construction. This is a
  tree-evaluation memory improvement, not streaming conformance.
- 2026-08-25 -- Asserted the golden eight-domain charge profile, inventoried
  semantic-unit observation gaps, and retained a three-run local charge-cost
  microprobe. Wall-clock and end-to-end overhead remain open.
- 2026-08-26 -- Added an isolated ASP.NET fault probe that terminates an
  acknowledged non-cooperating worker request, preserves sibling execution, and
  reinitializes only the affected slot without retrying the failed invocation.
- 2026-08-26 -- Added pre-dispatch cooperative cancellation through the isolated
  boundary with direct diagnostic parity and same-worker reuse. Retained the
  active-signal and wall-clock follow-ups.
- 2026-08-26 -- Added correlated control-plane multiplexing and a deterministic
  active first-charge cancellation probe. Retained natural observation latency,
  race sampling, and public adapter work as follow-ups.
- 2026-08-26 -- Added an unpaused 25-trial larger-workload race sample after
  propagating explicit XML limits into preparation. Retained representative and
  adversarial wall-clock bounds as open.
- 2026-08-26 -- Adapted a managed `CancellationToken` to correlated cooperative
  cancellation without changing the completion-wins rule or claiming hard
  termination. A four-case direct/isolated matrix retained structured failure
  fields; dispatched and in-process parity remain open.
- 2026-08-26 -- Carried a zero XSLT-instruction budget through the isolated
  boundary as deterministic `FXCT0002 / limit`, declined retry/replacement, and
  reused the same compiled/prepared process afterward.
- 2026-08-31 -- Added a distinct template-candidate work domain and one local
  charge/check per implemented candidate scan. Paired release evidence retained
  the uncharged reference for overhead attribution without selecting an index.
- 2026-08-26 -- Added the first native in-process ASP.NET candidate under
  ADR-0008. It strengthens the mechanism comparison but retains no thread-level
  hard-kill claim; broader supervision and panic evidence remain open.
- 2026-08-31 -- Serialized private worker cancellation frames and retained a
  20,000-frame concurrent byte-fragmenting stress probe. This closes transport
  write atomicity evidence without changing cancellation ordering or making a
  public concurrency guarantee.
- 2026-08-31 -- Measured matched-template scan fanout and deterministically
  exposed a 128-candidate post-signal cancellation gap. Retained the choice of
  work domain, check frequency, and any dispatch index as open.
