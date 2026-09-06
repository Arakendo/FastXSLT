# AR-0022: Finite Transform-Set Rolling Refill and Worker Claims

| Field | Value |
| --- | --- |
| Status | No Change |
| Opened | 2026-09-06 |
| Last reviewed | 2026-09-06 |
| Scope | Finite independent transform-set dispatch, completion-driven refill, worker-local claim allotment, utilization, and tail drain |
| Trigger | A synchronized 5-versus-8-request wave probe confirmed that full pool occupancy helps, while mandatory wave barriers reduced native throughput far below the continuously supplied lane; a small per-worker queue claim was proposed to reduce dispatcher interaction without creating a semantic batch barrier |
| Related ADRs | ADR-0005, ADR-0016, ADR-0019 |
| Related reviews | AR-0010, AR-0018, AR-0020, AR-0021 |
| Related evidence | `docs/Evidence/aspnet-best-practice-deployment-comparison-2026-09-06.md`; `docs/Evidence/isolated-wave-occupancy-2026-09-06.md`; `docs/Evidence/ar-0022-native-finite-dispatch-reference-2026-09-06.md`; `docs/Evidence/ar-0022-native-mixed-duration-dispatch-2026-09-06.md`; `docs/Reviews/performance-optimization-review-2026-09-04.md` |

## Architectural question

Can FastXSLT drain a finite independent transform set through a bounded,
continuously replenished worker pool with materially less dispatcher, queue,
wake-up, and tail-drain overhead by allowing each worker to claim a small local
allotment of ready requests, without introducing global wave barriers, hidden
workflow order, unbounded prefetch, unfair hoarding, or a larger unreported
failure radius?

If a local claim helps, should FastXSLT retain single-request completion-driven
refill, use a fixed small allotment, refill at a low watermark, or derive a
private allotment from current queue depth and observed work variance? The host
must not be required to configure internal claim size.

## Terminology correction and trigger evidence

The benchmark dimensions that opened this discussion must remain distinct:

```text
items per transform       = semantic work inside one invocation
transport batch members  = independent invocations in one worker transaction
worker count             = simultaneous worker/process capacity
finite-set job count     = independent invocations waiting to be drained
```

The reported native x8 values of about 1.36 million transforms per second at
five items and 166 thousand per second at 500 items do **not** compare five jobs
with 500 jobs. The latter transform walks roughly one hundred times as many
`order-item` elements. Its lower transforms-per-second value does not establish
a dispatcher bubble, queue starvation, or finite-set scheduling defect.

The focused wave experiment asks a different question. With the same five-item
transform, synchronized five-request waves reached five of eight native handles
at about 133 thousand transforms per second, while eight-request waves reached
all eight at about 167 thousand per second. Full occupancy helped, but the
eight-request wave remained about 8.2 times below the continuously supplied
native lane because every tiny group paid start and completion barriers and
waited for its slowest member.

This proves that lockstep waves are not a throughput-capacity proxy. It does not
prove that the current production-shaped dispatcher creates such waves. The
first native finite-queue experiment has now measured 500 actual jobs without
lockstep waves; isolated transport and representative mixed work remain open.

## Ownership and constraints

The host owns:

- the externally meaningful maximum concurrency and FastXSLT memory envelope;
- the independent requests submitted to a transform set;
- cancellation of the enclosing operation;
- publication, durable retry, quarantine, and reconciliation; and
- any service objective such as throughput, latency, or containment preference.

FastXSLT owns:

- internal queue structure, completion-driven assignment, and claim size;
- keeping admitted execution and retained bytes inside host-supplied ceilings;
- request/result correlation independent of worker and completion position;
- distinguishing host-queued, worker-claimed, worker-admitted, executing, and
  completed work where that distinction affects cancellation or loss truth;
- preventing starvation and releasing claims during cancellation, worker loss,
  generation replacement, or dispatcher shutdown; and
- deciding that the simplest single-claim design remains preferable when a
  larger claim does not earn its complexity.

ADR-0005 remains authoritative. A worker claim is a scheduling optimization
over independent requests, not semantic batching or workflow order. Sibling
results remain invisible and completion position has no transformation meaning.

ADR-0019 remains authoritative for isolated transport. A worker currently owns
one gated transaction and one sequential execution lane. Sending or admitting
another transaction before terminal completion would invalidate its bounded
unresolved-member accounting and requires a later accepted decision. This
review may first form differently sized bounded transactions per idle worker;
it does not silently authorize multiple queued transactions inside a worker.

ADR-0016's host-policy split applies. Hosts may select total concurrency,
retained-memory limits, and operational objectives. They do not configure
`worker_grab = 4`, queue topology, refill watermark, or per-worker roles.

A claimed request is not necessarily worker-admitted. Accounting and loss
classification must state exactly when authority and ambiguity transfer. Host-
side claims that were never sent can remain provably unstarted; worker-admitted
members follow ADR-0019's conservative loss rules. No claim mechanism may hide
retry or silently expand the ambiguity radius.

This review does not reopen AR-0020's rejected preparation-stage topology.
Workers remain combined/local with respect to any preparation they perform.

## Alternatives

### Retain the current bounded assignment

Continue using the existing per-worker assignment and accepted transport
batching. This has the smallest state machine and remains the reference until
worker-idle-with-pending-work evidence demonstrates a deficiency.

### Completion-driven claim of one

Keep a global bounded ready queue. Whenever one worker becomes available, give
it exactly one request and immediately refill that worker on completion. This
maximizes load balancing and minimizes claim retention, but can pay queue,
locking, dispatch, framing, and wake-up work for every invocation.

### Fixed bounded claim of two, four, or eight

An idle worker atomically claims up to a small number of requests and processes
them sequentially. In isolated mode the claim may map to one bounded ADR-0019
transport transaction. This amortizes queue and transport interaction, but can
hoard expensive work, enlarge the admitted loss radius, and worsen final drain
time when request costs vary.

### Low-watermark refill

Refill a worker's local claim before it becomes empty. This might overlap
dispatcher work with execution, but in isolated mode it pressures the accepted
one-transaction accounting and may require multiple admitted transactions or a
new replenishable protocol. It is not eligible until simpler terminal refill
shows a measured deficiency and the accounting is deliberately reopened.

### Queue-depth-sensitive claim

Use a small private rule such as a larger claim for a deep uniform queue and a
claim of one near the tail. This could reduce contention while limiting final
hoarding, but source bytes alone do not predict transform cost and a hidden
adaptive rule can regress mixed workloads. It requires repeatable evidence,
hysteresis only if stateful adaptation is necessary, and cross-machine
falsification before becoming an internal default.

### Work stealing

Allow idle workers to take unstarted members from another worker's local claim.
This could repair imbalance but adds ownership transfer, cancellation races,
accounting, diagnostic provenance, and worker-loss states. It is excluded from
the first experiment.

### Synchronized waves

Submit exactly one request per selected worker and wait for the entire wave
before submitting the next. The occupancy experiment retains this as a negative
control. Its barriers and slowest-member dependency make it unsuitable as the
candidate throughput dispatcher.

## Findings and uncertainties

- Continuous supply is materially faster than synchronized waves on the tiny
  native workload, but that observation alone does not identify which queue,
  host, or barrier costs a real finite transform set pays.
- Eight-way native wave occupancy has useful capacity beyond five-way
  occupancy. The gain is not proportional and does not establish an optimal
  claim size.
- The current best-practice deployment benchmark already gives every worker a
  long assignment and therefore approximates continuous supply. It is not a
  measurement of a central finite queue with dynamic completion-driven claims.
- ADR-0019 transport batching already amortizes isolated framing without a
  global cross-worker completion barrier. Worker claim allocation and transport
  member count may coincide in an experiment, but they are conceptually
  separate.
- Uniform tiny requests favor amortization and are the easiest case for local
  claims. Mixed sizes, failures, cancellation, generation drain, and large
  outcomes may reverse the result through hoarding, tail latency, memory, or
  ambiguity pressure.
- At 1.36 million transforms per second, 500 tiny transforms contain only about
  368 microseconds of idealized steady-state service. Fill, drain, timer,
  scheduling, and result-observation costs can dominate such a short operation.
  Longer finite sets are required to distinguish fixed cost from steady-state
  deficiency.
- Across three fresh processes, completion-driven claim one reached about
  1.285 million transforms per second for 500 actual jobs, approximately 95% of
  the continuously supplied reference. It reduced measured dynamic tail drain
  to about 4 microseconds while keeping aggregate transform-call busy time near
  89%.
- Claims two, four, and eight reduced acquisition count but did not win
  consistently across 500, 4,096, and 50,000 jobs. Larger claims increased
  dynamic tail radius and hoarded very short sets; claim eight permitted only
  one active worker for an eight-job set.
- Claim one and claim two were nearly tied at 4,096 jobs, while static balanced
  assignment led the sampled 50,000-job cell. Fresh-process variance remains
  material, so the result selects no production dispatcher or claim size.
- There is no evidence for a public claim-size setting, automatic allotment
  rule, multiple in-flight transactions per worker, or a production dispatcher
  change. Claim one remains the private dynamic reference for further tests.
- In a 4,096-job equal mixture of 5-, 50-, and 500-item transforms, dynamic
  refill removed severe clustered static imbalance: static reached about 219
  thousand transforms per second while claims one through eight remained near
  375-378 thousand/s by cross-process median.
- Claim two led the interleaved mixed median by about 8.2% over claim one, but
  lost to claim one in one of three fresh processes. It also increased final
  tail drain. Mixed duration therefore does not establish a repeatable larger-
  claim rule.

## Experiment method and decision gates

Use the unchanged five-item W3C workload first so semantic work remains fixed.
Compare finite transform sets of at least 8, 32, 128, 500, 4,096, 50,000, and a
duration-qualified larger set across eight persistent workers:

1. synchronized waves as the negative control;
2. the current static per-worker continuously supplied assignment;
3. completion-driven global claims of one; and
4. terminal-refill worker claims of two, four, and eight.

Repeat on a mixed-duration corpus-backed set before retaining any allotment.
Keep transport batch size constant when measuring queue claims where practical;
when an isolated claim intentionally maps to a transport transaction, report
that coupling explicitly and compare equal admission/failure-radius envelopes.

Record:

- total drain time and transforms per second;
- fill, steady-state, and tail-drain duration separately;
- each worker's busy time and idle time while global ready work remains;
- global queue depth, local claimed count, and admitted unresolved count over
  time;
- queue acquisition count, lock/wait time, dispatch operations, and isolated
  command/response bytes;
- time from one completion to that worker's next admitted start;
- per-request p50/p95/p99 and final-set completion latency;
- claimed and admitted bytes, results retained, and worker/process memory;
- cancellation observation, released claims, and generation drain;
- worker-loss disposition and maximum ambiguous/unstarted radius; and
- semantic and structured-diagnostic parity by logical request identity.

A larger claim wins only if it produces a repeatable consumer-visible drain or
steady-state improvement over completion-driven claim-one, while keeping worker
idle-with-pending-work, tail latency, retained memory, cancellation, fairness,
loss truth, and generation drain inside the same explicit bounds. Reduction in
queue operations alone is not sufficient.

If utilization is already high under claim-one, optimize neither queue claims
nor the dispatcher without a different measured bottleneck. If throughput
approaches the continuous reference only as set size grows, record a fixed-cost
amortization curve rather than inventing scheduling machinery. If mixed work
reverses a uniform-work gain, retain claim-one unless a conservative private
rule survives representative and cross-machine falsification.

## Disposition

**No Change.** Keep completion-driven claim-one as the private scheduling
reference. Do not add worker-local multi-request claims, prefetch,
low-watermark refill, work stealing, a public claim-size control, or another
dispatcher topology under the current evidence.

The initiating concern substantially arose from reading the 5/50/500 source-
item tiers as finite queue lengths. They instead measure semantic work inside
one transform. The separate finite-queue experiment then confirmed that
completion-driven claim one can drain 500 real tiny jobs near the continuous
capacity reference. Longer uniform queues did not reveal a queue-length
collapse, and mixed-duration work demonstrated the value of dynamic refill
without establishing a repeatable benefit for claims larger than one.

No ADR is required because this review confirms the simpler existing boundary.
ADR-0019 remains the accepted mechanism for amortizing isolated transport;
queue claims remain a separate concern. The workbench probes and evidence stay
available as regression and reopening oracles.

## Evidence checklist and reopening work

Checked items establish the no-change disposition. Unchecked items are dormant
requirements that apply only if a reopening trigger supplies new evidence for
a claim larger than one; they are not work remaining before this review is
concluded.

- [x] Instrument a native finite eight-worker claim-one reference with
      per-worker transform-call busy time, participation, job distribution, and
      final tail drain. Exact idle-with-global-ready intervals remain follow-up
      if later attribution needs them.
- [x] Record fill, aggregate busy fraction, final tail drain, and total drain
      over 8/32/128/500/4,096/50,000 jobs. A distinct steady-state interval
      remains inappropriate for the shortest sub-millisecond sets.
- [x] Compare static assignment, completion-driven claim-one, and terminal-
      refill claims 2/4/8 without a global wave barrier.
- [x] Keep source semantic work fixed and distinguish queue claim count from
      isolated transport batch-member count in every report.
- [ ] Repeat the comparison on mixed-duration, mixed-result-size, failure, and
      cancellation workloads to expose hoarding and tail imbalance. The native
      mixed-duration/order tranche is complete; result size and control/fault
      pressure remain open.
- [ ] Prove that cancellation and generation replacement return every claimed
      but unstarted request to an explicit terminal or host-visible disposition.
- [ ] Fault-inject worker loss before claim transfer, after worker admission,
      during a local claim, and near final drain; preserve ADR-0019 and AR-0018
      completion truth without hidden retry.
- [ ] Account claimed request bytes, admitted unresolved members, retained
      outcomes, and prepared/compiled ownership independently.
- [x] Reject a public claim-size/refill control. Hosts retain total concurrency,
      memory, timeout, and service objectives; FastXSLT owns internal allotment.
- [ ] Test low-watermark refill or work stealing only if terminal refill retains
      a measured idle gap that simpler claim sizing cannot remove.
- [ ] Falsify any retained private rule on at least two materially different
      machines and a named representative consumer distribution before an ADR.
- [x] Close with no change because claim-one already keeps workers busy and
      larger
      claims merely exchange queue work for tail, memory, or failure pressure.

## Reopening triggers

- A representative finite transform set leaves workers idle while bounded ready
  work exists.
- Queue acquisition or dispatcher wake-up is a measured material share of total
  drain time.
- The current static assignment creates measurable mixed-work stragglers.
- An isolated worker needs another transaction admitted before its current
  transaction terminates.
- A consumer requires a finite-set latency or throughput objective that the
  claim-one reference cannot meet within its concurrency and memory envelope.

## Review history

- 2026-09-06 -- Opened as Incubating from the 5-versus-8 synchronized-wave
  occupancy discussion and the proposal for small worker-local queue claims.
  Corrected the initiating terminology: 500 source items are not 500 queued
  jobs, and current evidence does not yet establish a dispatcher defect.
- 2026-09-06 -- Completed the native uniform-work reference across 8 to 50,000
  real transform jobs and claims 1/2/4/8. Claim one reached about 1.285 million
  transforms per second at 500 jobs, near the 1.36-million/s continuously
  supplied reference; larger claims reduced acquisitions but supplied no
  repeatable throughput win and increased tail/hoarding pressure. Kept the AR
  Incubating for mixed work, isolated transport, fault/control accounting, and
  cross-machine falsification.
- 2026-09-06 -- Completed the native mixed-duration/order reference over equal
  5/50/500-item populations. Dynamic refill fixed clustered static imbalance,
  but no claim larger than one won repeatably across order and fresh processes.
  Claim one remains the private reference; mixed result size and fault/control
  ownership remain open.
- 2026-09-06 -- Closed as No Change. The initiating benchmark interpretation
  conflated source items per transform with queued transform jobs. Dedicated
  finite-queue evidence found no length-driven throughput collapse, and neither
  uniform nor mixed-duration work gave larger local claims a repeatable benefit
  over claim one. Remaining fault/accounting work is dormant unless a reopening
  trigger first establishes a larger-claim candidate.
