# AR-0020 Immutable Execution-Packet Reference

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Status | Complete first reference tranche |
| Scope | Private test-only packet preparation and bounded ready-state admission |
| Governing review | [AR-0020](../Architectural%20Reviews/AR-0020-bounded-pre-execution-preparation-pipeline.md) |
| Claim | Ownership, boundedness, and topology-free phase-observation reference; no producer thread or performance result |

## Implemented seam

The existing transform-set reference prepared each request's source inside the
same loop that executed it. The prepared-input owner now exposes one private
`prepare_document` operation containing the unchanged controlled XML parse,
parsed-capacity observation, controlled XDM construction, failure mapping, and
immutable `Arc<Document>` publication. `PreparedInputBuilder` uses that same
operation, so the new experiment does not create another parser/XDM path.

The transform-set executor now separates source preparation from
`execute_prepared_request`. The ordinary reference path still creates one
invocation control, prepares the source, executes semantics, and serializes in
the same order. The private AR-0020 packet uses the same functions but retains
the control across the stage boundary; preparation work and cancellation state
are not reset before execution.

One test-only immutable packet owns:

- the request and result identities;
- the original resource-snapshot generation;
- a shared compiled stylesheet reference;
- the execution policy and initial multiple-match policy;
- the request's parameters and cancellation state;
- an optional immutable prepared source; and
- the same invocation control that paid preparation work.

The packet does not copy XML/XDM, serialize private representation, create a
public type, or introduce a thread. Its initial prepared-capacity charge is the
current XDM owner's `owned_capacity_bytes` observation. Shared snapshot and
compiled-program capacity is not multiplied into every packet charge.

## Bounded ready-state reference

The test-only FIFO checks packet count before known prepared-XDM capacity and
publishes only after both checked additions succeed. Rejection is represented
by typed count or known-capacity failures and leaves queue count and bytes
unchanged. Pop removes the exact queue charge before returning ownership to the
caller.

This is ready-queue accounting only. A popped packet still retains its prepared
document while executing. The second tranche therefore moves the packet's known
prepared-capacity charge from ready state to an executing-state lease during
assignment. The lease releases that charge on successful execution, execution
failure, or abandonment before execution. Current and high-water observations
distinguish ready packet count/capacity, executing packet count/capacity, and
aggregate prepared capacity. Per-worker attribution remains unavailable until
an actual worker topology exists.

Successful packets also retain monotonic preparation service time. Queue
assignment records ready-queue residence, and observed execution records
semantic-execution-plus-serialization service time. These private durations are
measurement fields, not deadlines, budget clocks, public telemetry, or evidence
of a performance benefit.

## Focused results

Eight focused tests establish:

1. a prepared packet produces the same semantic tree and serialized bytes as
   the existing worker-local preparation path;
2. two requests for one source currently perform independent preparation and
   retain distinct `Arc<Document>` allocations with equal shape and capacity;
3. count admission precedes byte admission and rejection does not mutate the
   ready queue;
4. pop restores exact ready-queue count and capacity immediately;
5. cancellation during preparation publishes no packet, and a new attempt can
   prepare cleanly;
6. cancellation signalled while a packet waits is observed through its retained
   invocation control after dequeue; and
7. a packet remains bound to and can execute from its original snapshot after
   the external old-generation handle is dropped, without belonging to an
   equal-byte replacement generation; and
8. prepared-capacity ownership moves exactly from ready to executing state,
   high-water observations retain peak pressure, phase durations are recorded,
   and success or abandoned assignment releases executing pressure; and
9. denied, missing, and malformed sources fail before an execution packet is
   published and retain their distinct failure classifications.

The numbered behaviors use eight tests because queue admission/release and
preparation cancellation/retry each share one focused case.

## Disposition

Retain this as the safe, topology-free AR-0020 reference. It proves that an
immutable unit of ready work and bounded ready-state accounting can exist
without changing the ordinary transform result or preparation semantics.

No overlap or throughput benefit is established. The topology-free reference
now supplies preparation, queue-residence, execution, ready-pressure, and
aggregate executing-pressure observations. The next tranche must add a real
raw-document workload and worker attribution before comparing worker-local,
one-producer, and bounded-producer-pool topologies. Plain duplicate preparation
remains the initial shared-source policy; single-flight is unadmitted unless
duplication becomes a measured problem.

The first threaded mechanics test now feeds exactly one transform worker from
either one or four preparation workers across a FIFO bounded to two packets and
two packets' known prepared capacity. Thirty-two request-correlated transforms
per topology retain the `for-004` semantic sentinel. Observed ready pressure
never exceeds either bound, executing packet high-water remains exactly one,
and total prepared pressure remains within ready capacity plus the one active
packet. This proves the intended distinction between direct sequential work and
single-transform-worker pipelining; it is not a throughput comparison.
