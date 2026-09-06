# AR-0021: Bounded Isolated Transport Batching

| Field            | Value                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Status           | Accepted                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| Opened           | 2026-09-05                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Last reviewed    | 2026-09-05                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Scope            | Isolated-worker command framing, independent transform sets, result correlation, transport amortization, and containment accounting                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Trigger          | The .NET 10 host comparison measured a roughly constant 42-53 microsecond sequential isolated-boundary premium per warm transform, and a bounded multi-request transport frame was proposed to amortize it                                                                                                                                                                                                                                                                                                                                                                                                 |
| Related ADRs     | ADR-0002, ADR-0005, ADR-0016, ADR-0019                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Related reviews  | AR-0002, AR-0003, AR-0004, AR-0010, AR-0018, AR-0020                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Related evidence | `docs/Evidence/aspnet-native-vs-isolated-tiered-comparison-2026-08-26.md`; `docs/Evidence/aspnet-host-mode-guarantee-cost-matrix-2026-08-26.md`; `docs/Evidence/aspnet-fastxslt-saxon-microsoft-rerun-2026-09-05.md`; `docs/Evidence/peer-isolated-transport-batching-review-monday-2026-09-05.md`; `docs/Evidence/ar-0021-isolated-boundary-attribution-and-batch-prototype-2026-09-05.md`; `docs/Evidence/ar-0021-batch-worker-resource-sweep-2026-09-05.md`; `docs/Evidence/ar-0021-result-pressure-batch-sweep-2026-09-05.md`; `docs/Evidence/ar-0021-aggregate-vs-incremental-delivery-2026-09-05.md`; `docs/Evidence/ar-0021-versioned-incremental-protocol-2026-09-05.md` |

## Architectural question

Can FastXSLT materially improve warm isolated-worker throughput by carrying a
bounded set of independent transformations in one transport transaction,
without changing their individual semantics, identity, diagnostics, resource
authority, cancellation, budgets, or containment disposition?

If so, which count and byte dimensions bound the command and its outstanding
results, whether results initially return in input order, and how batch-level
transport failure remains distinct from per-invocation semantic failure must be
derived from evidence rather than from a convenient wire format.

The review must also determine what the host can know after partial completion.
If a worker disappears after accepting 32 members, the host may be able to prove
that some outcomes were transferred, while other members may be unstarted,
executing, or executed without a durably observed result. Transport design must
not turn those distinct states into a false all-or-nothing semantic result.

## Working terminology

**Transport batching** amortizes process-boundary framing, pipe operations,
flushes, wakeups, and managed decoding across multiple independent
transformations. It does not make sibling transformations semantically visible
to one another and does not create workflow order.

A **transport transaction** is one bounded command envelope and its bounded
response envelope or response sequence. It is not necessarily the eventual
public transform-set API or ABI representation.

A **member outcome** is correlated to one logical request identity and retains
that invocation's result or structured diagnostic. A worker-boundary failure
may instead leave some admitted attempts operationally ambiguous; it must not be
misreported as an ordinary member semantic failure.

## Trigger and current evidence

The current ASP.NET isolated experiment already uses persistent worker
processes, retained compiled/prepared state, and a private binary
length-prefixed protocol. It does not use JSON or Base64 on the warm path.
Each ordinary transform nevertheless acquires a client gate, writes and flushes
one command, waits for one response, and the worker writes and flushes that one
result. One worker permits one active invocation.

The 2026-09-05 five-process .NET 10 rerun measured these sequential medians:

| Tier      |    Native | Isolated | Reciprocal latency difference |
| --------- | --------: | -------: | ----------------------------: |
| 5 items   | 289,103/s | 18,101/s |                 about 51.8 us |
| 50 items  | 158,857/s | 16,924/s |                 about 52.8 us |
| 500 items |  25,818/s | 12,420/s |                 about 41.8 us |

That nearly fixed premium makes transport and supervision the leading
attribution target; it does not prove which operation owns the time. The same
run showed useful four-worker scaling, but multiplying worker processes also
duplicates compiled and prepared generations and therefore is not a substitute
for measuring per-transaction amortization.

Dividing an assumed 50 microsecond fixed premium across eight requests suggests
an optimistic 6.25 microseconds per member before batch overhead. This is only
an opportunity estimate. It is not a throughput prediction, default batch size,
or public performance claim.

AR-0020 rejected a pre-execution staging pipeline for the prototype because its
benefit was workload-dependent and its activation signal unstable. Transport
batching addresses a different measured boundary and must not revive staging,
preparation/execution thread ratios, or an adaptive scheduler under another
name.

## Ownership and constraints

- ADR-0005 remains binding. Every member is independently executable; request
  order, worker assignment, execution order, and completion order have no
  semantic meaning. Sibling results never become resources.
- Every request and member outcome retains logical identity independent of its
  frame position. A batch of one must have the same engine semantics and
  diagnostic identity as the existing convenience path.
- Compilation, prepared input, parameters, dynamic context, messages,
  diagnostics, cancellation, work budgets, result construction, and
  serialization retain their existing ownership. The transport may group
  envelopes; it may not merge invocation state.
- The snapshot and explicit capabilities remain the only resource authority.
  Frame membership, a URI-shaped identity, or a sibling result grants no file,
  network, publication, or resource-admission authority.
- Count, encoded request bytes, admitted in-flight members, outstanding result
  bytes, and any retained completed outcomes are separate bounded dimensions.
  No overflow may spill to disk.
- Under ADR-0016, FastXSLT defines and atomically enforces the supported quota
  dimensions while the host owns environment-dependent values. Wire-level
  safety maxima may remain implementation-owned where representation requires
  them.
- Each member retains its own cancellation and deterministic work budgets.
  Cancelling one member must not implicitly cancel siblings. Cancelling the
  enclosing host operation may signal every still-live member explicitly.
- An ordinary semantic, unsupported, denied-authority, budget, or cancellation
  outcome for one member must not poison independent siblings. This experiment
  must expose per-member outcomes without prematurely stabilizing the public
  batch failure-collection rule still owned by AR-0004.
- A process crash, hard timeout, malformed frame, broken pipe, or incomplete
  response is an operational boundary failure. AR-0018 owns the host's durable
  retry, reconciliation, and quarantine policy for affected attempts.
- Neither the worker adapter nor FastXSLT may silently retry a member after an
  operational boundary failure. A host may retry only through its explicit
  request/attempt policy and must retain the ambiguity observation.
- Host-observable completion, not worker-local execution, determines what can
  be classified as delivered after process loss. A fully transferred correlated
  outcome may be known complete; an accepted member without such an outcome may
  remain ambiguous even if the lost worker had finished executing it.
- Transport batching cannot weaken hard isolation. Terminating one worker may
  make more simultaneously admitted attempts ambiguous, and that increased
  failure radius must be measured and reported.
- The initial experiment keeps one sequential execution lane per worker. It
  isolates transport amortization from intra-worker concurrency, scheduling,
  and shared-state questions. Any later multi-lane worker requires separate
  evidence and renewed review.
- The existing protocol is already binary and length-prefixed. Replacing pipes,
  adding shared memory, or introducing an external serialization dependency is
  not admitted without phase attribution showing that it addresses remaining
  material cost.
- Result ordering is transport presentation, not semantic ordering. The first
  experiment may retain input-order response framing for simplicity. A move to
  completion-order delivery requires measured head-of-line pressure and stable
  request correlation.
- No transport representation, operation number, frame layout, batch size, or
  worker count becomes a supported public API or stable ABI through this review.

## Alternatives

### A. Retain one request-response transaction per transform

This is the current reference. It minimizes batch failure radius and naturally
returns each result immediately, but repeats the measured boundary cost for
every tiny warm transformation.

### B. One bounded frame, sequential member execution, one bounded response

The worker decodes a bounded member set, executes members independently on its
existing single lane, and returns correlated outcomes in input order. This is
the leading first experiment because it amortizes transport without adding
execution concurrency. It can delay an early small result behind a slow member
and retains more outcomes until the response is emitted.

### C. One bounded command with incrementally framed correlated outcomes

The command cost is amortized while each outcome can be returned as it becomes
available. With one sequential lane, this reduces response retention but still
has input-order execution. It adds framing and partial-transfer states and may
perform more writes or flushes than one aggregate response.

### D. Pipeline multiple outstanding single-member commands

The client writes several correlated commands without waiting for every prior
response. This may reduce caller/worker idle time but does not necessarily
amortize decode, result framing, or flush cost. The current one-active-invocation
worker and client gate deliberately prohibit it. It also expands cancellation,
backpressure, and worker-loss accounting.

### E. Execute batch members concurrently inside one worker process

One process could share compiled/prepared state across multiple execution lanes
and reduce process-count multiplication. This changes scheduling, shared-state
pressure, and the number of attempts lost to one process termination. It is
outside the initial transport experiment.

### F. Replace the binary pipe protocol or add shared-memory result transfer

Length-prefixed binary framing is already present. A different transport may
help large commands or results, but it is premature before phase attribution
and result-heavy measurements identify copies or pipe transfer as a material
remaining cost.

## Findings and uncertainties

- The current comparison establishes a large, nearly fixed isolated premium
  for this warm tiny-result workload. It does not yet assign that premium among
  managed encode/queue, pipe write/flush, worker wake/read/decode, supervision,
  result write/flush, or managed receive/decode.
- Persistent workers and retained compiled/prepared state already exclude
  process startup, stylesheet compilation, source parsing, and preparation from
  the timed warm path.
- Transport batching aligns with ADR-0005's independent transform sets, but the
  existing host workbench exposes only a batch-of-one worker command.
- Larger batches should amortize fixed work but can increase queue residence,
  time-to-first-result, retained result bytes, cancellation delay, and the
  number of ambiguous attempts after worker loss.
- One aggregate response maximizes amortization but may withhold all otherwise
  complete member outcomes until the final member finishes. Incremental
  correlated outcomes can reduce retention and preserve a host-observed
  completed prefix, at the cost of more framing and partial-transfer states.
- The optimal envelope is likely workload- and service-objective-dependent.
  Batch size must remain bounded and engine/adapter-owned unless a meaningful
  host-facing latency or byte policy is later evidenced.
- The current managed client admits only one batch transaction per worker: its
  gate is held from command emission through the terminal response. There is no
  second host batch waiting inside the worker. Later members do reside in the
  bounded decoded command while earlier members execute, so their residence is
  member-position/service-time pressure rather than a separate unbounded batch
  queue. Exact per-member phase timestamps would require another private
  measured operation and are not inferred from end-to-end latency.
- Worker/process scaling has now been swept at 1/2/4/8 lanes with CPU, observed
  working set, and duplicated-generation pressure recorded. Scaling plateaus
  on several larger-work cells and is not free throughput.
- The approximately 3.1 KiB managed allocation per isolated call nominates
  consolidated or pooled frame buffers only after phase and allocation
  attribution. It does not prove allocation is the latency bottleneck.
- Input-order results remain the simplest reference. Result-heavy evidence now
  justifies private incremental input-order delivery for early visibility but
  does not justify completion-order delivery.
- The first measured-command attribution records roughly 7-12 microseconds in
  request emission, 8-11 microseconds in worker decode/queue, and 21-33
  microseconds of unattributed response/pipe/wakeup work for the small and
  medium tiers. Worker execution is only about 4.5-8.8 microseconds there.
- A private sequential input-order prototype with exact 1 MiB command/response
  ceilings improved three-run rotated median throughput at batch 128 by 5.22x,
  3.69x, and 2.01x over ordinary requests at 5, 50, and 500 items. Batch 8
  already improved those medians materially.
- Aggregate response latency grows with batch size: median per-run p50 was
  about 0.95, 1.35, and 4.36 milliseconds at batch 128 across those tiers.
  No default size or automatic admission rule follows from the throughput win.
- A focused success/failure/success frame preserves the ordinary invalid-
  identity diagnostic for the bad member and exact later sibling completion.
  Later controls extend parity through cancellation, budgets, and worker loss.
- A second private controlled frame now proves pre-cancelled and zero-XSLT-
  instruction members return `FXCT0001` and `FXCT0002` independently between
  successful siblings, followed by worker reuse. Later probes add active
  cancellation and loss classification; broader composed budgets remain open.
- A private fault-only probe acknowledges a bounded five-member batch and emits
  start/finish observations before parking at member positions 0, 2, or 4. The
  ASP.NET supervisor then kills the worker without retrying any member and
  successfully starts a separately identified recovery worker.
- Under the aggregate-response oracle, even a worker-finished member remains
  operationally ambiguous when its correlated outcome was not transferred.
  The parked member is likewise ambiguous; the later sequential suffix is
  provably unstarted from the bounded observation prefix. No member is complete
  in these trials because the aggregate response had not begun.
- A second fault-only probe fully transferred two correlated member outcomes,
  stopped after exactly half of the third result payload, and then lost the
  worker. The host retained the first two as complete, classified the partial
  member ambiguous, classified the untouched suffix unstarted, performed no
  implicit retry, and recovered with a fresh worker. Incremental framing is not
  selected by this correctness probe.
- Closing a command halfway through a declared identity exits the worker before
  the decoder publishes any batch to the supervisor, so every member is
  unstarted and unadmitted. Conversely, terminating after a valid command was
  flushed but before observing acknowledgement or any outcome leaves every
  member ambiguous. Neither case retries, and both are followed by fresh-worker
  recovery.
- A private supervisor-driven batch state machine now cancels a selected member
  after its first real work charge, returns exact `FXCT0001`, then executes later
  siblings successfully and reuses the worker. It preserves exactly one active
  transform lane; asynchronous execution only permits the supervisor to process
  the existing cancellation frame.
- Twenty-five unbarriered races over immediate, 1 ms, and 5 ms signal windows
  produced 20 exact cancellations and 5 valid completions, with no third target
  disposition, sibling failure, or worker contamination. Completion wins when
  committed before the cancellation frame is processed.
- An 8,192-member 1/2/4/8-worker by 1/8/32/128-batch sweep confirms that
  process scaling and transport amortization compound, but not monotonically.
  Batch 128 is not a universal winner, while its one-worker-loss ambiguity
  radius is always 128 versus 8 or 32 for the smaller alternatives.
- Observed working set scales approximately with worker count and prepared
  generation duplication. Tiny-result batch-128 response frames remain only
  about 10.5-11.0 KiB; the later result-heavy matrix supplies the missing
  transfer-pressure evidence.
- A three-run rotated 65,536-member follow-up made worker CPU observable in
  every cell and confirmed that batch-size/worker-count interaction is not
  monotonic. Batch 32 led batch 128 at 2/4/8 workers for 500-item work, while
  batch 128 retained most medium/tiny wins.
- The longer run still shows substantial throughput ranges in several
  8-worker cells. At 500 items, batch 32 rose only from 93,019/s with four
  workers to 94,483/s with eight while estimated busy worker cores rose from
  about 3.19 to 3.78. This nominates boundary or machine contention for later
  attribution and rejects worker count as free scaling.
- A three-run result-pressure sweep returns 3.3 KiB, 33 KiB, and 165 KiB per
  member while keeping exact aggregate frames below 1 MiB. Batch 32 improves
  the 3.3 KiB multi-worker lane, but batching barely changes 33 KiB throughput
  and provides no repeatable 165 KiB gain.
- Aggregate framing makes time to first result equal time to final result.
  Median transaction p50 reaches about 16-17 ms for 3.3 KiB batch 128, 22-26 ms
  for 33 KiB batch 16, and 27-30 ms for 165 KiB batch 4 at four/eight workers.
  Batching therefore cannot be selected from member count alone.
- Managed allocation becomes result-dominated as output grows: approximately
  100.7-102.2 KiB per 33 KiB result and 497-498 KiB per 165 KiB result. Frame
  amortization removes little of that materialization pressure.
- A private incremental input-order candidate flushes each indexed correlated
  outcome while retaining one sequential execution lane and the cumulative
  response ceiling. On representative multi-worker result-heavy cells it moves
  first-result p50 from whole-batch latency to near one-member latency: about
  4.425 to 0.219 ms, 21.373 to 1.274 ms, and 26.685 to 6.820 ms.
- Incremental final throughput remains broadly near aggregate throughput but is
  not uniformly better. It adds roughly 0.6-1.3 KiB managed allocation per
  member plus one flush per outcome. Its evidence is early visibility, not a
  throughput optimization.
- Focused Rust and ASP.NET controls preserve input-order correlation, exact
  cancellation/budget diagnostics, later successful siblings, and worker reuse
  through incremental framing.
- When eight 165 KiB results cross the cumulative response ceiling, six fully
  observed outcomes remain complete, the executed-but-untransferred member is
  ambiguous, and the untouched suffix is unstarted. The adapter returns exact
  `FXWB1004`, performs no retry, and reuses the worker. Consumer backpressure
  and a supported delivery API remain unresolved.
- Aggregate framing now applies its 1 MiB response-retention ceiling as each
  member outcome is produced, before admitting that outcome to the private
  vector. Six representative 165 KiB outcomes fit; the seventh is rejected,
  the suffix is not executed, and retained bytes do not change on rejection.
  The host still receives only the aggregate failure and no completed prefix.
- A four-member slow-consumer probe delayed host reads by 30 ms between 165 KiB
  outcomes. First/final observation occurred at about 7/97 ms, all results and
  recovery stayed exact, and the worker retained no completed-outcome vector.
  Process working set peaked at 11.96 MiB from a 5.82 MiB baseline; that single
  allocator observation is not deterministic memory attribution.
- A private callback consumer then observed and discarded four exact 165 KiB
  outcomes as they arrived. The adapter retained zero completed outcomes, first
  visibility preceded terminal completion, and the worker remained reusable.
  Throwing from a second callback trial after two outcomes retired the
  desynchronized worker, caused no hidden retry, and allowed an exact fresh-
  worker recovery. This proves a bounded consumer seam without selecting a
  supported callback or async-enumeration contract.
- Exact transport-owned accounting for that four-member unversioned case records
  a 109-byte command and 660,198-byte response sequence. Version-one framing
  records 125 and 660,202 bytes respectively. Non-retaining consumption peaks
  at one 165,047-byte framed outcome, versus four outcomes and 660,188 bytes for
  collection. These wire-equivalent charges deliberately do not claim CLR heap,
  allocator, pipe-buffer, or process-memory accuracy.
- Killing the worker from the first completed callback now returns a typed
  private transport-loss observation: the one host-observed prefix member stays
  complete and all three remaining admitted members stay ambiguous. The client
  deliberately does not infer an unstarted suffix without worker-side phase
  evidence. No result vector or hidden retry is introduced, and a replacement
  recovers exactly.

## Disposition

**Accepted through ADR-0019.** Bounded isolated transport batching materially
amortizes the fixed boundary cost while preserving independent member
semantics, exact controlled diagnostics, conservative transport-loss truth,
bounded retention, and fresh-worker recovery.

Incremental input-order delivery is the accepted behavioral path; aggregate
input-order framing remains the simpler semantic and diagnostic oracle. The
decision does not stabilize a wire representation, batch-size policy, public
transform-set API, callback/async-enumeration shape, or multi-lane worker.

ADR-0019 selects the smallest evidence-backed behavioral transport contract:
bounded incremental input-order correlation and delivery, one
sequential execution lane per worker, conservative loss truth, no hidden retry,
host-owned external ceilings, and private batch-of-one/aggregate oracles. It
does not stabilize the concrete wire representation or a language binding and
deliberately leaves host API types, numeric batch policy, completion-order
delivery, and multi-lane workers unresolved.

The accepted one-transaction/one-lane topology also resolves the apparent need
for another in-flight quota. At admission, unresolved member count equals the
already bounded member count and can only decrease as outcomes become visible;
the client gate permits no second transaction in that worker. Encoded envelope
bytes, cumulative response bytes, adapter-retained completed outcomes, and the
worker generation's compiled/prepared ownership remain separately bounded and
accounted. Introducing a queue or multiple execution lanes would invalidate
that equivalence and requires renewed review rather than a speculative duplicate
limit now.

## Required follow-up

- [x] Attribute one warm isolated call across managed encode/queue, pipe
      write/flush, worker receive/decode, engine execution, result encode/write/
      flush, and managed receive/decode. The first probe keeps clocks separate and
      leaves response write/flush, pipe/wakeup, and managed read as one residual;
      subdivide it only if later work needs that attribution.
- [x] Add a private versioned batch operation bounded by member count and
      encoded envelope bytes; retain the existing operations as private oracles.
      Version one uses a length-delimited payload, echoes its version in the
      acknowledgement, and rejects unknown versions as `FXWB1005` before member
      decoding. Same-worker recovery after rejection is executable through the
      ASP.NET operational gate.
- [x] Bound every current transport-retention dimension without inventing a
      duplicate queue quota. One gated transaction means admitted unresolved
      members are bounded by the 128-member ceiling; the length-delimited command
      and cumulative response are each bounded to 1 MiB; incremental worker
      framing retains only the current outcome; and collecting versus
      non-retaining adapter ownership is measured separately. Compiled/prepared
      state remains worker-generation-owned rather than copied per member.
- [x] Compare batch sizes 1, 8, 32, and 128 over the 5/50/500-item tiers in
      randomized or time-balanced lane order. Report per-member throughput and
      p50/p95/p99 end-to-end latency.
- [x] Record aggregate and per-member request bytes, result bytes, managed
      allocation, worker CPU, aggregate working/private memory, queue residence,
      time to first result, and time to final result.
      Exact incremental command, response, and adapter-retention charges are now
      executable for both collecting and non-retaining consumption. Existing
      sweeps cover the remaining timing, allocation, CPU, and observed working-set
      dimensions. Exact private-memory attribution remains unavailable and is
      explicitly outside ADR-0019's deterministic accounting claim.
- [x] Verify exact semantic and structured-diagnostic parity for every member,
      stable correlation independent of frame position, sibling-result
      invisibility, and batch-of-one parity. Aggregate and incremental controls
      preserve exact success/cancellation/budget outcomes and correlation; the
      engine snapshot prevents sibling result admission, and batch one remains
      compared against the ordinary command.
- [x] Fault-inject one member cancellation, semantic failure, budget failure,
      malformed command, truncated response, and worker loss. Report which member
      attempts are complete, controlled failures, unstarted, or operationally
      ambiguous without inventing host retry policy.
- [x] Kill the worker before admission, after batch acknowledgement, during
      selected member positions, during result transfer, and after a correlated
      outcome is fully received. Prove that only fully observed member outcomes are
      classified complete and that no affected member is retried implicitly.
- [x] Compare aggregate response framing with incrementally framed input-order
      outcomes if retained result bytes or time to first result become material.
      Test completion-order delivery only if variable-duration evidence shows
      consequential head-of-line blocking. The private input-order candidate
      materially reduces first-result latency with broadly similar final throughput;
      it remains behind the aggregate oracle until a supported delivery surface is
      selected. Controlled cancellation/budget, cumulative-limit prefix retention,
      slow-reader backpressure, non-retaining callback delivery, and callback-
      abandonment worker retirement now have executable evidence.
- [x] Sweep 1/2/4/8 persistent workers separately from batching and record
      throughput against aggregate compiled/prepared memory. Do not conflate more
      processes with lower per-transaction overhead.
- [x] Retain the single-item warm lane as a negative control: batch machinery
      must not replace or regress the direct isolated convenience path when no set
      exists to amortize. The ordinary command remains unchanged and measured next
      to batch one.
- [x] Accept a later ADR before stabilizing an isolated transform-set transport
      contract. ADR-0019 selects bounded incremental input-order delivery and
      conservative failure collection while leaving public host API types and
      multi-lane workers unselected. The supported protocol must still be
      versioned before release.

## Reopening triggers

Reopen or broaden the review if a representative consumer requires incremental
results, one worker process must execute multiple members concurrently, result
transfer rather than transaction frequency becomes the measured bottleneck, or
the public Rust/host lifecycle selects a transform-set contract whose transport
must be standardized.

## Review history

- 2026-09-05 -- Opened as Incubating from the measured warm isolated-boundary
  premium and the proposal to amortize it with bounded independent transport
  sets. The existing binary batch-of-one protocol remains the reference.
- 2026-09-05 -- Added elapsed-duration attribution without cross-process clock
  subtraction. The flat request/decode/queue/round-trip regions justify a
  private transport-amortization prototype rather than more evaluator work.
- 2026-09-05 -- Added a count-, command-byte-, and aggregate-response-byte-
  bounded sequential input-order prototype. Three rotated comparisons found
  material gains at every multi-member size and tier, while aggregate-response
  latency exposed the expected throughput/failure-radius tradeoff. The command
  remains private and unversioned; cancellation and loss experiments stay open.
- 2026-09-05 -- Added per-member pre-cancellation and optional instruction-
  budget framing. Focused Rust and ASP.NET operational tests preserve exact
  controlled failures, successful siblings, and worker reuse without claiming
  active mid-member cancellation.
- 2026-09-05 -- Added a private aggregate-response loss probe at the first,
  middle, and last member. Host-observed evidence keeps worker-finished but
  untransferred members and the active member ambiguous, classifies the later
  sequential suffix unstarted, performs no implicit retry, and proves a fresh
  worker can recover. Transfer-boundary cases remain open.
- 2026-09-05 -- Added a fault-only partial-transfer probe. A fully received
  correlated prefix remains complete after later worker death, a half-written
  result remains ambiguous, and the sequential suffix remains unstarted. This
  closes the first transfer-classification question without selecting
  incremental result delivery; pre-acknowledgement and malformed-command loss
  were still open at that checkpoint.
- 2026-09-05 -- Closed the remaining framing matrix: a deliberately incomplete
  command never reaches supervisor admission and leaves all members unstarted;
  a complete flushed command with no host-observed acknowledgement or outcome
  leaves every member ambiguous. Neither path retries, and fresh-worker
  recovery remains intact. Active mid-member cancellation is now the remaining
  semantic-control fault tranche.
- 2026-09-05 -- Added active mid-member cancellation through a private
  supervisor-driven state machine. Earlier and later siblings complete, the
  selected member returns exact cancellation, the worker remains reusable, and
  active-transform high-water remains one. Natural race measurements and a
  supported protocol remain open.
- 2026-09-05 -- Removed the deterministic barrier for 25 natural batch races.
  Only cancellation or already-committed completion occurred (20/5), all
  siblings remained exact, and the worker recovered. The semantic-control
  tranche is complete; resource pressure and transport-policy selection remain.
- 2026-09-05 -- Added the first equal-work 1/2/4/8-worker and 1/8/32/128-batch
  resource sweep. It records exact wire/failure-radius pressure and observed
  memory, finds non-monotonic throughput, and rejects the current CPU samples
  where Windows timer quantization dominates. No default is nominated.
- 2026-09-05 -- Repeated the full matrix three times with rotated order and
  65,536 members per cell. CPU became observable throughout; medians confirmed
  workload-dependent batch optima, material 8-worker variance, and a large-work
  scaling plateau. Batch 128 remains unjustified as a default, and result-heavy
  pressure remains open.
- 2026-09-05 -- Added a rotated result-pressure matrix. Transport amortization
  remains useful at 3.3 KiB, is marginal at 33 KiB, and disappears at 165 KiB;
  aggregate time-to-first-result, managed outcome materialization, and failure
  radius worsen with batch size. This supplies pressure to compare incremental
  input-order delivery without selecting it.
- 2026-09-05 -- Implemented and measured private incremental input-order
  framing. It preserves controlled member outcomes and worker reuse, reduces
  first-result latency materially, and generally retains aggregate throughput.
  The aggregate path remains the oracle while consumer backpressure and
  a supported delivery surface remain open; completion order is not nominated.
- 2026-09-05 -- Added cumulative response-exhaustion evidence to the real
  incremental operation. Six complete outcomes survive, the next attempt is
  ambiguous, the untouched suffix is unstarted, no retry occurs, and the same
  worker remains usable.
- 2026-09-05 -- Added a deliberately slow incremental consumer. Its delay
  propagates as transport backpressure while the one-lane worker retains only
  its current outcome, preserves exact results, and remains reusable. A public
  callback/async-enumeration and abandonment contract remains out of scope.
- 2026-09-05 -- Added a real non-retaining callback boundary. Four large
  outcomes become visible before terminal completion with no adapter-owned
  result collection. Consumer abandonment retires the desynchronized worker,
  performs no retry, and recovers through a replacement. The callback remains
  private evidence rather than a supported API selection.
- 2026-09-05 -- Moved aggregate response-byte enforcement to outcome retention
  rather than final serialization. An over-limit outcome no longer joins the
  vector or permits suffix execution; exact limit diagnostics and same-worker
  recovery remain intact.
- 2026-09-05 -- Extracted aggregate execution, retention accounting, and
  framing into a private typed worker module. The supervisor retains command
  orchestration, active cancellation, and lifecycle ownership; no protocol or
  host surface changed.
- 2026-09-05 -- Added typed incremental transport-loss evidence at the consumer
  seam. A fully observed callback prefix remains complete while the entire
  unobserved admitted remainder stays conservatively ambiguous; replacement
  recovery remains exact.
- 2026-09-05 -- Moved Under Review after the bounded aggregate and incremental
  paths answered the core technical question positively. Remaining work is
  stabilization and product-boundary selection, not feasibility discovery.
- 2026-09-05 -- Added exact wire-equivalent request, response, and adapter-
  retention accounting. It distinguishes collecting four large outcomes from
  retaining only the current callback outcome without presenting either value
  as managed or process memory.
- 2026-09-05 -- Drafted a proposed ADR for bounded incremental input-order
  isolated transform-set transport. The proposal stabilizes the evidenced
  semantic, accounting, loss, and ownership boundaries without selecting a
  concrete host API, batch size, completion-order mode, or multi-lane worker.
- 2026-09-05 -- Accepted through ADR-0019. The decision stabilizes the bounded
  incremental behavioral contract, conservative loss truth, and host/engine
  ownership split while keeping wire representation and language bindings
  private.
- 2026-09-05 -- Added the private version-one length-delimited incremental
  operation. Unknown versions fail as exact `FXWB1005` before member decoding,
  the acknowledgement echoes the admitted version, same-worker recovery passes,
  and the complete operational fault/control matrix remains green.
- 2026-09-05 -- Closed current transport-retention accounting. The one gated
  transaction makes unresolved members a decreasing subset of the bounded batch;
  command, response, adapter outcome, and worker-generation ownership retain
  separate enforcement or accounting. A future queue or multi-lane worker must
  reopen the equivalence.
