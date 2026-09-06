# AR-0020: Bounded Pre-Execution Preparation Pipeline

| Field | Value |
| --- | --- |
| Status | Rejected |
| Opened | 2026-09-05 |
| Last reviewed | 2026-09-05 |
| Scope | Batch admission, preparation/execution overlap, immutable execution packets, ready-queue retention, and worker utilization |
| Trigger | A dedicated "packer thread" was proposed to prepare work for transform workers in future raw-document volume workloads |
| Related ADRs | ADR-0002, ADR-0005, ADR-0012, ADR-0016 |
| Related reviews | AR-0002, AR-0009, AR-0010, AR-0013 |
| Related evidence | `docs/Evidence/thread-pool-design-review-2026-08-25.md`; `docs/Evidence/private-prepared-input-reuse-2026-08-25.md`; `docs/Evidence/representative-standards-lifecycle-measurement-2026-08-26.md`; `docs/Evidence/peer-preparation-stage-review-monday-2026-09-05.md`; `docs/Evidence/ar-0020-immutable-execution-packet-reference-2026-09-05.md`; `docs/Evidence/ar-0020-raw-phase-ratio-probe-2026-09-05.md`; `docs/Evidence/ar-0020-preliminary-topology-comparison-2026-09-05.md`; `docs/Evidence/ar-0020-equal-thread-budget-comparison-2026-09-05.md`; `docs/Evidence/ar-0020-mixed-size-topology-comparison-2026-09-05.md`; `docs/Evidence/ar-0020-adaptive-controller-mechanics-2026-09-05.md`; `docs/Evidence/ar-0020-bounded-staged-trial-replay-2026-09-05.md`; `docs/Evidence/aspnet-post-performance-audit-tiered-rerun-2026-09-05.md`; future raw-document pipeline measurements |

## Architectural question

Can FastXSLT improve end-to-end throughput for large raw-document transform
sets by overlapping bounded pre-execution preparation with transformation,
without adding authority, copying source state, weakening invocation isolation,
or committing the executor to one dedicated packer thread?

If so, which work belongs before the execution queue, what constitutes an
immutable execution packet, how are count and retained-byte pressure bounded,
and should preparation use zero, one, or multiple workers?

## Working terminology

A **preparation stage** performs independently schedulable work required before
semantic transformation can begin. Candidate work includes locating already
admitted resources, obtaining an existing compiled program, parsing a selected
source into prepared XDM, validating invocation inputs, and constructing a
stylesheet-dependent invocation view where its accepted ownership permits that
work to move. The term does not imply a permanent thread.

An **execution packet** is an immutable, request-correlated ownership envelope
ready for one transform worker. Its candidate contents are logical request and
attempt identity, shared references to one compiled program and prepared input,
validated parameter values, host-supplied invocation controls, output policy,
and any invocation-owned prepared view. The name does not accept a public type
or representation.

A **ready queue** retains execution packets that completed preparation but have
not started transformation. Its count and attributable retained bytes are
separate pressure dimensions.

An **internal scheduling topology** is FastXSLT's private allocation of a
host-supplied execution envelope: combined prepare-then-execute workers,
separate preparation and execution stages, their ratio, queue structure, and
worker assignment. These mechanics are not host policy and are not a supported
configuration surface.

The candidate external concurrency contract is **a maximum total worker
budget**, conceptually "FastXSLT may use up to N workers." It does not promise N
simultaneous transformations, N execution workers, N preparation workers, or a
fixed role for any worker. No public API spelling is selected by this review.

**Direct sequential latency** means one caller prepares and then executes one
request with no cross-request overlap. **Single-transform-worker pipelined
throughput** means one preparation worker may prepare request N+1 while exactly
one execution worker transforms request N. The latter has multiple requests in
different stages but never has multiple transforms executing concurrently.
These lanes answer different questions and must not share the label
"sequential" without qualification.

## Trigger and evidence

ADR-0005 permits threads, work stealing, or another measured executor and
explicitly leaves worker count, queue depth, parsed-input retention, and
in-flight limits as bounded policy. Existing prepared-input evidence shows that
avoiding repeated XML parse and XDM construction can materially improve a warm
path, while the magnitude depends on the workload. The same evidence shows
that prepared representations can retain substantially more capacity than the
admitted source bytes.

The current `for-004` ASP.NET benchmark compiles and prepares before timing. Its
native 50-item lane has a p50 near 5.5 microseconds, so inserting another queue,
wake-up, and cross-thread ownership transfer into that already-ready path is
more likely to add fixed cost than eliminate work. It is a negative control for
unconditional staging, not a representative raw-document pipeline.

The proposed consumer pressure is different: a large publication batch may
start with many admitted but unprepared source documents. Parsing, prepared-XDM
construction, validation, and source/view setup may leave transform workers
idle or make their service time irregular. Those phases may be independent and
overlap-friendly, but FastXSLT has no representative trace showing worker
starvation, preparation-to-execution ratios, ready-queue memory, or an
end-to-end gain from overlap.

Initial peer review therefore reframed the proposed packer thread as a measured
preparation stage. It also identified prepared bytes outstanding per execution
worker as a required observation because packet count alone cannot distinguish
a 20 KiB packet from a 200 MiB packet.

## Ownership and constraints

- The host owns the transform set, resource generation, workflow stages,
  deployment trust, and environment-dependent limits under ADR-0016. FastXSLT
  defines and enforces supported count, byte, concurrency, and queue dimensions;
  the host owns their values unless a semantic or safety invariant fixes one.
- Host ownership stops at externally meaningful constraints and objectives,
  such as maximum total concurrency, attributable prepared capacity,
  cancellation/containment policy, and any later evidenced service objective.
  FastXSLT owns how those constraints are partitioned internally. Preparation-
  worker count, execution-worker count, their ratio, ready-queue topology, and
  worker assignment must not become supported host configuration from this
  experiment.
- The dispatcher may reassign capacity only within the host-supplied total
  worker and memory ceilings. Internal role changes must not create additional
  workers beyond that ceiling, reinterpret the ceiling as per-stage capacity,
  or hide queue-retained prepared state outside memory accounting.
- Admission and dispatch own request identity, bounded queueing, assignment,
  completion correlation, and backpressure. A preparation stage does not gain
  result-publication, retry, quarantine, or workflow authority.
- The resource snapshot owns immutable admitted bytes and logical identity.
  Preparation consumes only the sealed snapshot and explicit capabilities; a
  URL-shaped identity, diagnostic path, or packet boundary grants no file or
  network access.
- Compiled programs own stylesheet-derived static state. Prepared inputs own
  immutable source-derived XDM and admitted source-only indexes. Invocation-
  specific views and parameters must remain owned by exactly one invocation.
- ADR-0012 whitespace visibility views remain invocation-owned and may not be
  retained as a stylesheet-specific prepared-input cache. Moving their
  construction to a preparation worker would move work, not ownership.
- Execution workers own mutable runtime frames, focus, messages, diagnostics,
  result construction, and serialization state. An execution packet may carry
  validated immutable inputs and controls but must not become a mutable runtime
  shared between preparation and execution.
- Packet creation must share or transfer owned state. It must not copy the XML
  or XDM merely to fit a queue, serialize private Rust representation, expose
  arena/layout details, or create a second semantic engine.
- Every queue and in-flight stage requires bounded admission by count and by an
  attributable byte/capacity measure appropriate to its retained state.
  Prepared bytes outstanding per execution worker and in aggregate must be
  visible to the experiment. No overflow may spill to disk.
- Cancellation and deterministic work limits apply during preparation, ready-
  queue residence, and execution. Cancellation or failure before execution
  must not publish a partial packet or consume a transform worker.
- A queued packet may retain an old snapshot/compiled generation only through
  explicit ownership. Cancellation, rejection, or queue teardown must release
  it deterministically so generation replacement can drain.
- ADR-0005 remains binding: sibling results are not resources, packet order has
  no semantic meaning, and preparation or execution completion order cannot
  affect result meaning.
- A preparation thread is not a hard isolation boundary. AR-0010 process-
  isolation rules remain unchanged for non-cooperating work.

## Alternatives

### A. Workers prepare and execute

Each worker acquires an admitted request, performs any missing preparation, and
immediately transforms it. This has the fewest queues and ownership transfers,
preserves worker-local cache locality, and remains the reference topology. Long
or irregular preparation can leave execution capacity underused and mixes the
two phases in worker-utilization observations.

### B. One preparation worker feeds a bounded ready queue

A single producer prepares immutable packets while transform workers drain the
queue. This is simple and may overlap preparation with execution, but one
producer can become the throughput ceiling. It also adds a queue hop and can
amplify memory if ready packets accumulate.

### C. A bounded preparation pool feeds a bounded ready queue

Multiple preparation workers can scale parsing/XDM work and smooth source-size
variance. It adds scheduling, cancellation, memory admission, duplicate-work,
and generation-lifetime complexity. The optimal preparation-to-execution ratio
is workload-dependent, but that does not make it host-owned policy. It is a
private engine choice that must either be selected conservatively from validated
workload characteristics within host-supplied ceilings or remain disabled.

### D. Dispatch first, then prepare worker-locally

The dispatcher assigns a request to an execution worker or worker-local queue;
that worker constructs any destination-local or scratch-sensitive state before
transforming. This preserves locality and avoids a cross-thread packet handoff,
but does not overlap preparation and transformation on the same worker. It may
be preferable when cache/NUMA transfer costs dominate or preparation is small.

### E. Eagerly prepare every resource during snapshot sealing

This maximizes ready work but pays parse/XDM cost and retained memory for unused
resources. AR-0009 already declines to equate snapshot admission with eager
full-tree retention. This remains unavailable without new workload and memory
evidence.

### F. Adapt topology automatically

The executor could observe phase service times and vary preparation concurrency
or queue depth. This adds feedback stability, reproducibility, policy, and
explainability questions before a static comparison exists. It is not an
initial experiment. Any future classifier must be conservative, bounded,
falsifiable against mixed workloads, and unable to exceed the host's external
limits. An unexplained self-tuning scheduler is not an acceptable substitute
for such evidence.

The least expansive adaptive candidate keeps a fixed total worker budget and
makes workers capable of combined local prepare-then-execute work or temporary
preparation assistance. Combined/local execution remains the resting state.
Only sustained execution starvation may activate preparation ahead; growing
ready bytes, ready age, degraded small-request tail latency, or saturated
execution must stop it. This changes assignment inside an admitted envelope,
not the host-visible thread count, and does not yet admit an implementation.

The dispatch decision is therefore "what should the next available worker do?"
rather than "how many new operating-system threads should be created?" Raw work
waiting, ready work waiting, sustained executor starvation, phase-time moving
averages, oldest-request age, and prepared-byte pressure are candidate inputs.

Any experiment must use smoothed observation windows and hysteresis rather than
reacting to an instantaneous queue sample. It must record topology transitions
and prove that assignment does not oscillate, starve either phase, amplify
prepared retention, reorder semantics, or weaken cancellation. Dynamic thread
creation and destruction are a separate candidate and are excluded initially.

### G. Let the host configure the preparation/execution split

The host could select preparation workers, execution workers, their ratio, or
the queue topology directly. Current evidence rejects this as a supported
surface: the same split that helps one source size can reduce throughput or
materially worsen another size's tail latency, and workload ordering changes
the result. Exposing these mechanics would transfer an unstable engine-detail
decision to consumers and turn benchmark-specific tuning into production risk.
This alternative is unavailable from this review.

## Findings and uncertainties

- The accepted architecture permits a preparation stage, but does not require
  or select one.
- The likely opportunity is overlap in raw-document batches, not moving
  already-completed compilation/preparation through another thread.
- The preparation unit must be a cheap ownership envelope over immutable state,
  not a copied or serialized document.
- Count-only backpressure is insufficient because prepared inputs and views can
  vary greatly in retained size.
- A dedicated single packer is only one experiment point. Zero, one, and a
  bounded pool must be compared, but their topology and split remain private
  engine mechanics within host-owned total limits.
- It is unknown whether parsing, XDM construction, view construction, worker
  starvation, queue contention, cache migration, memory bandwidth, or execution
  currently limits a representative raw batch.
- It is unknown whether source preparation can be scheduled independently
  without duplicating work when several requests share one source, or whether
  single-flight coordination would cost more than duplication.
- It is unknown whether packets should be built in submission order, smallest-
  first, reuse groups, or another locality policy. No ordering policy may become
  semantic.
- The current warm benchmark should detect unconditional overhead, but cannot
  prove value for this proposal because it excludes the work being overlapped.
- The first topology-free reference now constructs an immutable packet from the
  same controlled preparation used by `PreparedInputBuilder`, carries one
  invocation control across preparation and execution, and differentially
  matches worker-local preparation through semantic and serialized results.
- Its bounded FIFO checks packet count before known prepared-XDM capacity,
  rejects without mutation, and restores the exact ready-queue charge on pop.
  Execution-owned outstanding bytes remain deliberately separate and open.
- Two packets for the same source currently retain distinct prepared documents.
  This establishes plain duplication as the reference before any single-flight
  coordination is considered.
- Cancellation before packet publication leaves no packet and permits a clean
  new attempt. Cancellation while queued reaches execution through the retained
  control. An old-generation packet remains independent of an equal-byte
  replacement generation.
- The topology-free observation layer records preparation, ready-queue
  residence, and execution service time separately. It moves known prepared
  capacity from ready state to an executing lease and retains current/high-water
  count and byte observations. The lease releases on success, failure, or
  assignment abandonment. Per-worker attribution and starvation cannot exist
  until a worker topology is introduced.
- Denied authority, missing resource, malformed XML, and cooperative
  cancellation all fail before packet publication with distinct typed
  classifications. Broader fault injection remains part of the topology
  comparison rather than being inferred from these reference cases.
- Two release-mode runs over pinned `for-004` semantics found preparation
  materially larger than execution: 7.95-8.03x at 50 generated items and
  6.47-6.71x at 500. The noisy 5-item tier ranged from 2.38x to 5.00x. This
  admits the topology comparison; it does not prove overlap, throughput, or
  consumer value.
- A threaded mechanics case now feeds exactly one transform worker with either
  one or four preparation workers. Its two-packet/two-packet-capacity queue
  stays within both bounds across 32 transforms, the active-transform high-water
  remains one, and every result retains the semantic sentinel. Timing and
  topology selection remain deliberately absent from this correctness case.
- Two release runs of the preliminary A/B/C comparison found one-preparer
  pipelining 1.06-1.10x faster at 50 items and 1.14-1.23x at 500; four preparers
  reached 2.69-2.70x and 3.64-4.14x respectively. Consumer wait fell sharply,
  while total prepared high-water remained at or below two ready packets plus
  one active packet. This is same-process generated-source evidence, not a
  topology selection.
- Equal-ten-thread comparison reverses by workload size. Staged 8/2 and 7/3
  beat ten combined workers at 50 items, while ten combined workers beat every
  split at 500 items and also led the closest 8/2 split on latency. A phase
  ratio cannot select topology by itself; locality, contention, handoff, and
  source shape remain material.
- A longer 4:3:1 small/medium/large mixed batch did not show a repeatable staged
  throughput win. Combined workers led both interleaved runs and one clustered
  run; 7/3 narrowly led the other clustered run. Interleaving improved every
  topology. Staged 7/3 sometimes improved large-request p95 while materially
  worsening small-request p95, so per-size latency and ordering cannot be
  collapsed into one aggregate.
- The reversals across uniform and mixed workloads are evidence against a
  public preparation/execution ratio. Combined workers remain the private
  reference and baseline. Staging stays disabled outside experiments unless
  FastXSLT can derive a conservative activation rule from validated workload
  characteristics and prove that rule does not create unacceptable throughput,
  latency, memory, cancellation, or generation-drain regressions. If it cannot,
  the staged implementation should be removed rather than exposed as a knob.
- The dispatcher could observe preparation/execution service time, ready count
  and bytes, oldest-ready age, worker busy/wait state, and prepared-capacity
  high-water without exposing those mechanics publicly. These signals are
  candidate classifier inputs, not proof that adaptive scheduling is stable or
  beneficial. A fixed pool with opportunistic role assignment is narrower than
  dynamically creating role-specific threads and is the first adaptive shape
  eligible for measurement.
- The combined baseline has no separate execution-worker starvation signal:
  every worker prepares and then executes its own request. Such starvation,
  ready-queue age, and ready-byte pressure become directly observable only
  after staging exists. The deterministic controller currently receives a
  synthetic starvation input and therefore proves control mechanics, not that
  the production dispatcher can derive its activation signal.
- Preparation/execution service-time ratio is observable in combined mode but
  is insufficient by itself. The equal-budget 50- and 500-item reversal showed
  that a preparation-dominant workload can still favor combined locality.
  Activation therefore needs either a separately validated workload classifier
  or a bounded trial with explicit rollback; neither is selected.
- A two-run bounded-trial replay rejected staged 7/3 for every tested workload
  under explicit 5% throughput, 25% worst-tail, and 1.5x prepared-high-water
  experimental gates. Uniform-500 and mixed workloads consistently lost
  throughput; mixed worst-size p95 regressed by 5.585-10.689x. Uniform-50 was
  ambiguous: its staged/combined throughput ratio changed from 1.032 to 1.545,
  while prepared high-water remained 2.4-2.6x. A single short trial is therefore
  too variable to act as the activation oracle, and rejected-trial exposure is
  a real cost rather than free discovery.
- The current provisional answer is negative: adaptive staging is not eligible
  for the prototype execution path. Combined/local execution is the only
  candidate baseline for an eventual supported executor. More synthetic tuning
  cannot reverse that presumption by itself; a named representative consumer
  workload must first demonstrate a repeatable combined-mode deficiency and a
  material, low-risk staged opportunity.
- A test-private deterministic controller now proves the narrow state-machine
  mechanics without connecting them to the runtime dispatcher. Three sustained
  starvation windows activate preparation ahead; ready execution keeps
  priority; minimum dwell plus sustained relief prevent immediate flapping; and
  ready-byte, oldest-ready-age, or small-request-p95 pressure retreats
  immediately. These example thresholds prove control flow only and are not
  product defaults or performance evidence.
- A ratio learned on one machine is not portable evidence. Cache size, core and
  NUMA topology, memory bandwidth, allocator behavior, operating-system
  scheduling, and the workload distribution can all change the break-even
  point. Any automatic default must survive at least two materially different
  systems or remain an experiment.

## Experiment method and win condition

Use one corpus-backed, semantically checked raw-document workload with multiple
source sizes and at least one real reuse relationship. Hold compiled semantics,
resource authority, results, diagnostics, and budgets constant while comparing:

1. worker-local prepare plus execute;
2. one preparation worker plus a bounded ready queue and exactly one transform
   worker, measuring cross-request pipelined throughput; and
3. a small bounded preparation pool plus the same ready queue.

Record source/admitted bytes, prepared capacity, packet count and bytes
outstanding, prepared bytes per execution worker, preparation and execution
service time, queue wait, worker starvation/utilization, end-to-end throughput,
p50/p95/p99, cancellation latency, peak/retained memory, generation drain, and
result-transfer cost. Include one already-warm control to expose pure queue-hop
overhead and one oversized-packet case to exercise byte backpressure.

Run a second comparison under an equal total thread budget. Ten combined
prepare-then-execute workers are the control against staged 9-preparer/1-
executor, 8/2, 7/3, and 5/5 allocations. Report total throughput, request
latency distribution, stage starvation, and prepared-capacity high-water. Do
not compare a split pool only against a smaller combined pool: that would
attribute extra threads to staging. The equal split is not privileged; measured
preparation/execution ratios make asymmetric allocations first-class cases.

The candidate wins only if preparation is material and overlap-friendly,
transform-worker utilization and end-to-end throughput improve, tail latency
and retained memory remain within host-supplied limits, and all semantic,
diagnostic, authority, budget, cancellation, and generation invariants remain
equal to the reference. Remove it if the gain is absent or the memory/queue cost
does not justify the additional lifecycle.

## Disposition

**Rejected.** Combined prepare-then-execute workers remain the private reference
topology and the only prototype baseline. Staged or adaptive preparation must
not enter the runtime or public surface under current evidence. The test-only
implementations and measurements remain as negative evidence and reopening
oracles, not dormant production features. Do not add a permanent packer thread,
public execution-packet type, public preparation/execution worker split, public
queue topology, eager snapshot preparation, automatic topology, or new cache
from this review.

Internal preparation/execution worker ratios are not part of the supported host
configuration surface. Any staged topology must be selected by FastXSLT from
host-supplied resource ceilings and validated workload characteristics, or
remain disabled. If no robust activation rule can be demonstrated, retain the
combined baseline and close or remove the staged experiment as a useful
negative result.

For the prototype path, staging and adaptive topology selection remain absent.
Further controller implementation is gated on representative consumer evidence
showing a stable problem in combined mode. A candidate must then survive
repeated windows, variance analysis, an explicit trial-exposure budget,
immediate rollback, workload phase changes, and cross-machine falsification
while materially outperforming the simpler baseline. Passing synthetic
workloads alone is insufficient.

ADR-0005 already permits the private topology comparison. A later accepted ADR
is required if the result selects a supported lifecycle, changes prepared-input
ownership, introduces single-flight/cache behavior, or stabilizes public queue
or packet semantics. A well-measured negative result closes the experiment
without an ADR.

## Evidence checklist and reopening work

Checked items establish the rejected disposition. Unchecked items are dormant
requirements that apply only if a reopening trigger is met; they are not work
remaining before this review can be considered concluded.

- [ ] Capture a representative raw-document publication workload with trusted
  semantic sentinels, source-size distribution, reuse relationships, and
  host-supplied memory/latency/concurrency limits.
- [x] Keep adaptive staging out of the prototype execution path. Combined/local
  execution remains the only current baseline; further controller wiring is
  gated on the representative workload above demonstrating a repeatable
  deficiency.
- [x] Inventory the exact current work performed before worker semantic
  execution and classify each item as snapshot-, compiled-, prepared-,
  invocation-, or worker-owned.
- [x] Add topology-free observations for preparation service time, execution
  service time, queue wait, ready-packet count/bytes, and aggregate executing
  prepared bytes, including current and high-water pressure.
- [ ] Add per-worker prepared-byte attribution and worker starvation/utilization
  when the comparison introduces actual execution workers.
- [x] Define a private immutable execution-packet prototype containing shared
  handles and invocation inputs without copying XML/XDM or exposing private
  layout.
- [x] Compare worker-local preparation, one preparation worker, and a bounded
  four-worker preparation pool under identical private semantics and queue
  limits on the generated-source mechanics workload.
- [ ] Repeat that comparison on representative consumer inputs and host limits.
- [x] Compare combined, 8/2, and 7/3 equal-budget lanes over a deterministic
  mixed-size batch in clustered and interleaved orders, retaining per-size tail
  latency and prepared high-water.
- [x] Exclude preparation/execution ratios, queue topology, and worker
  assignment from the supported host configuration surface; retain only
  externally meaningful total limits and objectives as host policy.
- [x] Define the candidate external concurrency meaning as one maximum total
  worker budget, without promising simultaneous-transform count or stable
  internal worker roles. Public API spelling remains unselected.
- [ ] Derive and falsify a conservative engine-owned staging activation rule
  against uniform, mixed-size, differently ordered, and representative consumer
  workloads while enforcing host-supplied concurrency and prepared-capacity
  ceilings.
- [ ] Compare a fixed-budget flexible-worker controller against the combined
  baseline and best static staged candidates. Keep combined/local execution as
  its resting state; permit preparation ahead only after sustained executor
  starvation, and use ready bytes/age, executor saturation, and per-size tail
  latency as deactivation or veto signals.
- [ ] Inventory which activation inputs are observable while the dispatcher is
  still in combined/local mode. Do not use separate executor-starvation,
  ready-age, or ready-byte signals to justify initial activation because those
  signals do not exist until staging has already begun.
- [ ] Compare two honest activation mechanisms: a conservative classifier over
  pre-existing workload characteristics, and a bounded staged trial with
  immediate rollback. Reject service-time ratio alone and retain the trial's
  temporary throughput, latency, and memory cost as part of the decision.
- [x] Add an offline bounded staged-trial replay with independent throughput,
  worst-size-tail, and prepared-high-water vetoes, reporting the full exposure
  duration of rejected trials. It rejected every 7/3 candidate in two runs and
  exposed material uniform-50 reference variance; it is not an online
  dispatcher implementation.
- [x] Prove the controller's deterministic mechanics separately from timing:
  sustained activation, ready-work priority, minimum dwell, relief hysteresis,
  immediate byte/age/small-tail retreat, and no dynamic worker creation.
- [ ] Exercise controller stability with bursty and phase-changing workloads;
  retain transition counts, dwell time, starvation, utilization, throughput,
  per-size p50/p95/p99, cancellation latency, and prepared-capacity pressure.
  Require smoothing and hysteresis and reject oscillating behavior. Do not
  create or destroy threads in the initial controller experiment.
- [ ] Falsify the same activation rule on at least two materially different
  processor/cache/memory/scheduler environments before treating it as a
  portable internal default. Machine-specific success is not sufficient.
- [x] Close the experiment with combined workers as the baseline and keep
  staging absent from runtime code rather than exposing manual topology
  controls. No robust activation rule was demonstrated by current evidence.
- [x] Compare combined workers with staged pools under the same total thread
  budget, including 10 combined, 9/1, 8/2, 7/3, and 5/5 allocations.
- [x] Prove one- and four-preparer mechanics can feed exactly one transform
  worker through simultaneous count and prepared-capacity bounds without
  changing results or implying concurrent transforms.
- [x] Enforce ready-queue count and byte/capacity bounds with deterministic
  backpressure and structured exhaustion before measuring throughput.
- [ ] Exercise cancellation and every preparation failure before publication,
  while queued, during generation replacement, and after worker assignment.
- [x] Prove shared sources are prepared according to an explicit duplicate or
  single-flight policy without poisoning retry or merging logical identity.
- [ ] Prove old snapshot and compiled generations drain after queued/executing
  packet release and remain independent of replacement generations.
- [ ] Run the comparison through the ASP.NET host boundary and retain both the
  raw-document case and the already-warm negative control.
- [ ] Record preparation cost, break-even volume/reuse, peak and retained
  memory, throughput, tail latency, cancellation, and negative results before
  accepting any topology.

## Reopening triggers

- A representative raw batch shows transform workers idle while independent
  preparation work is available.
- Preparation variance materially harms throughput or tail latency.
- Ready work cannot be retained within the host's memory budget.
- Cross-thread packet transfer or queueing regresses warm/tiny workloads.
- A consumer requires explicit preparation control or a stable batch lifecycle.
- Another physical source strategy changes which work can be prepared
  independently.

## Review history

- 2026-09-05 -- Opened as Incubating from the project-owner proposal for a
  packer thread; reframed it as a topology-neutral bounded preparation stage.
- 2026-09-05 -- Initial peer review retained immutable handle-based packets,
  zero/one/many preparation-worker comparison, raw-document focus, and explicit
  prepared-byte pressure per execution worker.
- 2026-09-05 -- Added the topology-free safe reference: shared controlled
  preparation, immutable packets, count/known-prepared-capacity FIFO admission,
  exact dequeue accounting, explicit duplicate shared-source preparation,
  queued cancellation, semantic parity, and generation isolation. No thread or
  performance claim was introduced.
- 2026-09-05 -- Added private phase durations plus ready/executing current and
  high-water pressure. Assignment now transfers its prepared-capacity charge to
  a drop-guarded execution lease; worker-specific observations remain deferred
  until a topology exists.
- 2026-09-05 -- Added pre-publication failure probes for denied, missing, and
  malformed sources alongside the existing cancellation/retry case.
- 2026-09-05 -- A two-run release probe over pinned `for-004` semantics found
  raw preparation materially larger than execution at 50 and 500 generated
  items. This opens the zero/one/many topology experiment while leaving host
  value and representative workload evidence unresolved.
- 2026-09-05 -- Distinguished direct sequential latency from single-transform-
  worker pipelined throughput. A preparation worker may overlap request N+1
  with execution of N without implying concurrent transforms.
- 2026-09-05 -- Added a threaded correctness baseline for one and four
  preparers feeding exactly one transform worker through a two-dimensional
  bounded queue. This does not yet compare throughput.
- 2026-09-05 -- Two preliminary release runs found a small repeatable gain from
  one-preparer overlap and a larger gain from four preparers, with bounded
  prepared high-water. Representative and ASP.NET comparisons remain open.
- 2026-09-05 -- Equal-ten-thread runs retained combined workers as a serious
  candidate: asymmetric staging won at 50 items, but combined workers won at
  500. No fixed split or adaptive policy was selected.
- 2026-09-05 -- Mixed-size runs retained combined workers as the more robust
  exploratory throughput candidate, exposed a large-tail versus small-tail
  tradeoff for 7/3, and made workload ordering an explicit selection input.
- 2026-09-05 -- Tightened the product boundary after the mixed-workload
  reversal: hosts own meaningful total resource limits and service policy;
  FastXSLT owns internal worker partitioning, queue topology, and assignment.
  Combined workers remain the baseline, while staging stays private and
  disabled unless a conservative engine-owned activation rule is proven.
- 2026-09-05 -- Added the narrow adaptive hypothesis: a dispatcher may use
  bounded semantic-free observations to reassign a fixed worker budget, with
  combined/local execution as the resting state and opportunistic preparation
  activated only under sustained starvation. Hysteresis, byte pressure, tail
  latency, and cancellation are mandatory controls; dynamic thread population
  remains excluded.
- 2026-09-05 -- Clarified the candidate host contract as one maximum total
  worker budget. Role-flexible workers may be reassigned only inside that fixed
  ceiling; the contract does not expose or imply per-stage worker counts.
- 2026-09-05 -- Added a five-test deterministic controller reference. It proves
  sustained activation, execute-ready priority, dwell/relief hysteresis, and
  immediate memory/age/small-tail retreat without threads or runtime wiring.
  Cross-machine and threaded performance evidence remain open.
- 2026-09-05 -- Added a two-run bounded staged-trial replay. Explicit
  throughput, worst-tail, and prepared-memory gates rejected every 7/3
  candidate; large and mixed workloads lost consistently, while uniform-50
  varied enough to reject a one-window activation oracle.
- 2026-09-05 -- Recorded the provisional negative disposition: adaptive
  staging stays out of the prototype. Further controller work now requires a
  representative consumer workload with a repeatable combined-mode deficiency;
  synthetic topology wins alone cannot admit it.
- 2026-09-05 -- Concluded the review as Rejected. Combined/local workers remain
  the prototype baseline; all incomplete experimental work is dormant reopening
  evidence rather than a current implementation plan. No ADR was required
  because no new architecture or public contract was selected.
