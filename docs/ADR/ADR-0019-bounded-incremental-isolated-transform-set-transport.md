# ADR-0019: Bounded Incremental Isolated Transform-Set Transport

- Status: Accepted
- Date: 2026-09-05
- Related reviews: AR-0002, AR-0018, AR-0021
- Related ADRs: ADR-0002, ADR-0005, ADR-0016
- Related evidence: `docs/Evidence/ar-0021-versioned-incremental-protocol-2026-09-05.md`
- Supersedes: None

## Context

FastXSLT's isolated ASP.NET workbench reuses worker processes, compiled
stylesheets, and prepared inputs, but an ordinary warm transform still pays one
command/response transaction. Measurements attributed a largely fixed isolated
boundary premium of about 42-53 microseconds per invocation on the tested
workloads.

AR-0021 tested whether one bounded transport transaction could carry multiple
independent transformations without turning them into one semantic operation.
Private batch sizes up to 128 materially improved throughput: the first rotated
comparison measured batch-128 gains of about 5.22x, 3.69x, and 2.01x over
single-request isolated transport for 5-, 50-, and 500-item sources. Later
worker-count and result-pressure sweeps showed that the best batch size is not
monotonic and that large results can eliminate the throughput benefit.

Aggregate input-order responses supplied a simple reference implementation,
but delayed every result until the batch finished. A private incremental
input-order implementation moved first observation close to one-member latency
without a repeatable final-throughput penalty. A non-retaining consumer
received and discarded each outcome independently; a slow consumer naturally
backpressured the worker rather than causing an unbounded completed-result
collection.

Fault injection covered cancellation, budget failure, malformed and truncated
frames, worker loss at multiple execution and transfer positions, consumer
abandonment, and replacement-worker recovery. The resulting evidence supports
a transport decision. It does not support a universal batch size, public
callback type, completion-order protocol, or multiple simultaneous execution
lanes inside one worker.

## Decision

FastXSLT isolated-host adapters may amortize their process boundary by carrying
a bounded transform set in one transport transaction. This is **transport
batching**, not semantic batching. ADR-0005 remains authoritative: every member
is an independently executable request, sibling results are invisible, and
transport position or delivery order has no transformation meaning.

This ADR stabilizes transport semantics and failure behavior, not a concrete
wire protocol or language binding. The accepted behavioral contract uses
incremental input-order correlation and delivery over one sequential execution
lane per worker:

- every member carries stable logical request identity independent of its
  position, worker, or outcome order;
- parameters, dynamic context, cancellation, work budgets, messages,
  diagnostics, result construction, and serialization remain member-local;
- one member's ordinary success or structured failure does not prevent later
  independent members from executing;
- each fully decoded outcome is correlated and made observable separately;
- input order is only the correlation and observation order for outcomes; it
  does not require members to start or execute in that order outside the
  current one-lane implementation, constrain scheduling outside the worker
  transaction, or create workflow order;
- consumer demand provides backpressure; the adapter does not accumulate an
  unbounded completed-outcome collection; and
- batch-of-one and the existing single-request operation retain equivalent
  engine semantics and diagnostic identity.

An aggregate input-order implementation remains private as the simpler
semantic, diagnostic, and differential oracle. It is not the preferred
large-result delivery contract and need not be exposed publicly.

### Bounds and accounting

The transport must independently bound:

1. member count;
2. encoded command bytes;
3. admitted but terminally unresolved members;
4. cumulative encoded response bytes;
5. any completed outcomes retained by an adapter; and
6. prepared or compiled state retained through the transaction under the
   existing worker and engine policies.

Admission and response accounting occur before the corresponding bounded state
is retained. Exceeding one dimension produces a machine-readable operational
outcome, stops admitting or executing an unsafe suffix as appropriate, and
does not spill to disk.

Wire-equivalent command, response, and completed-outcome retention charges are
deterministic transport accounting. They are not claims about CLR heap size,
allocator footprint, pipe buffers, resident set, private bytes, or total
process memory.

ADR-0016 applies. Hosts own externally meaningful concurrency, memory, timeout,
and containment ceilings. FastXSLT owns safe wire maxima, accounting
definitions, enforcement, internal batch formation, and worker assignment. A
host does not configure preparation/execution worker ratios or depend on a
particular internal batch size.

### Cancellation and failure

Each member retains independent cooperative cancellation and work budgets.
Cancellation wins only when observed before committed completion; otherwise
the completed result stands. Cancelling one member does not implicitly cancel
siblings. An enclosing host cancellation may explicitly signal every still-live
member.

The transport reports only what the host boundary can prove:

- a fully decoded and correlated outcome observed by the consumer is complete;
- an admitted member whose outcome was not fully observed is ambiguous after
  worker or transport loss, unless direct bounded worker-phase evidence proves
  that it never started; and
- a command that never reached worker admission leaves its members unstarted.

Sequence position alone must not be used to invent an unstarted suffix after
loss. An already observed prefix remains complete. A partial frame contributes
no completed-outcome accounting and cannot establish completion.

FastXSLT and its adapters never retry an affected member implicitly. Retry,
quarantine, escalation, and publication reconciliation remain host-owned under
ADR-0005 and the execution-loss work tracked by AR-0018.

Consumer abandonment or disposal before terminal framing desynchronizes that
worker transaction. The adapter retires the affected worker generation,
reports the conservative member dispositions it can prove, performs no retry,
and may recover through a fresh worker. A controlled member cancellation does
not by itself require worker retirement.

### Protocol lifecycle

A supported worker protocol must carry an explicit protocol version and reject
unknown versions or operations before admission. Frame lengths, member counts,
indexes, and aggregate byte arithmetic are checked before allocation or
retention. Malformed, truncated, duplicate, missing, or out-of-range framing
fails closed with bounded diagnostics.

Protocol opcodes, field widths, concrete frame layout, and language-specific
consumer types remain private implementation details unless a later
compatibility decision deliberately stabilizes them. Accepting the behavioral
contract and requiring internal protocol versioning do not authorize third-
party protocol implementations or make the unpublished workbench ABI a
supported product surface.

## Consequences

Isolated hosts can amortize fixed transport work across independent transforms
while observing results early and retaining per-member semantics. Large
transform sets need not wait for aggregate completion or retain every completed
result in the adapter.

The failure radius can grow with the number of admitted unresolved members. A
lost worker can therefore make more attempts ambiguous than a single-request
transaction. Bounds, conservative truth, no hidden retry, and host-owned
attempt policy make that cost explicit rather than eliminating it.

Incremental framing adds acknowledgement, correlation, terminal-state, flush,
and disposal complexity. Slow consumers intentionally slow the transaction.
Abandonment sacrifices the current worker generation because continuing on a
desynchronized stream is less defensible than replacement.

No universal batch size follows from this decision. Small-result workloads may
benefit substantially from large batches, while larger semantic work, more
workers, large results, latency objectives, or failure-radius constraints may
favor smaller transactions. Combined/local worker execution remains the
baseline; AR-0020's rejected adaptive preparation staging is not revived.

## Non-decisions

This ADR does not:

- select a default or public batch size;
- expose batch size, preparation/execution ratios, queue structure, or worker
  roles as host configuration;
- select a concrete Rust, C#, callback, or async-enumeration API;
- select completion-order delivery;
- admit more than one simultaneous transform lane in a worker;
- promise exactly-once execution or publication across process loss;
- define durable attempt storage, retry, or quarantine policy;
- claim deterministic process-memory accounting; or
- replace the direct single-request path or aggregate private oracle.

## Alternatives considered

### Keep one transport transaction per transform

This is the simplest boundary and remains the batch-of-one oracle, but it pays
the measured fixed transport premium for every member and materially limits
tiny warm-transform throughput.

### Return one aggregate response

Aggregate framing is simple and useful as a differential oracle. It delays the
first observable result until terminal completion, retains completed outcomes,
and loses an observable prefix when the final aggregate cannot be delivered.
It is not selected as the practical transform-set delivery contract.

### Incremental input-order delivery

This is selected. It materially reduces time to first result, permits
non-retaining consumption, naturally propagates backpressure, and preserves a
simple deterministic correlation order without assigning semantic meaning to
that order.

### Incremental completion-order delivery

This might reduce head-of-line blocking for highly variable members, but no
current evidence shows a consequential deficiency in input-order delivery. It
would add scheduling and correlation states without a demonstrated need.

### Multiple execution lanes in one worker

This could combine transport amortization with intra-process concurrency, but
would change shared-state, cancellation, containment, accounting, and failure-
radius assumptions. Current evidence deliberately isolates transport batching
behind one sequential lane.

### Public host-selected batch and worker-role controls

Measured optima vary by source size, result size, worker count, and machine
behavior. Exposing internal ratios would leak private scheduling mechanics and
allow apparently reasonable settings to reduce throughput or tail latency.
Hosts supply the resource envelope; FastXSLT retains allocation strategy.

## Validation

- Differentially execute batch-of-one, aggregate, and incremental paths with
  exact result and structured-diagnostic parity.
- Verify stable member correlation, sibling-result invisibility, independent
  cancellation and budgets, and one active transform per worker.
- Exercise cancellation before work, during charged work, near completion, and
  after committed completion; permit only exact cancellation or completion.
- Fault-inject malformed and truncated commands, loss before admission and
  acknowledgement, loss at multiple member positions, partial result transfer,
  loss after a fully observed prefix, and consumer abandonment.
- Prove conservative complete, ambiguous, and unstarted classifications, no
  hidden retry, worker retirement when desynchronized, and fresh-worker
  recovery.
- Test exact count and byte boundaries, arithmetic overflow, charge-before-
  retention, suffix suppression on exhaustion, and release on every terminal
  path.
- Compare aggregate and incremental delivery across small and large results,
  recording time to first/final result, throughput, p50/p95/p99, managed
  allocation, worker CPU, observed process memory, exact wire bytes, retained
  completed outcomes, and ambiguity radius.
- Sweep batch sizes and worker counts in rotated, repeated runs. Treat the
  result as policy evidence, not a universal default.
- Version the supported protocol and prove unknown versions, operations, and
  malformed framing fail closed before member admission.
- Run the normal Rust, documentation, worker, and ASP.NET operational gates.

## Reopening triggers

Revisit this decision if representative workloads demonstrate consequential
input-order head-of-line blocking, one worker must execute multiple transforms
simultaneously, a supported host requires resumable transport after consumer
abandonment, exact process-memory enforcement becomes a product promise, or a
public cross-version worker protocol must be independently implemented.
