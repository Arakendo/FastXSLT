# Peer Review: Isolated Transport Batching

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Reviewer | Monday |
| Subject | Proposed bounded multi-request transport for isolated workers |
| Outcome | Retained as AR-0021 experiment pressure; attribution must precede protocol changes |

## Review result

The current measurements nominate isolated transport and supervision rather
than the semantic core. The roughly constant 42-53 microsecond sequential
premium across small, medium, and larger warm `for-004` tiers is consistent
with a per-invocation boundary cost, although it does not yet identify which
part of that boundary owns the time.

The proposed optimization is **transport batching, not semantic batching**.
One bounded command may carry multiple independent transformations, but every
member must retain its own request identity, parameters, diagnostics, result,
cancellation and budget behavior, and failure disposition. This fits ADR-0005
without giving submission order semantic meaning or making sibling results
visible.

Instrumentation should precede the batch operation. The useful attribution is
managed encoding and queueing, pipe write and flush, worker wake/read/decode,
engine execution, result encoding/write/flush, and managed receive/decode. If
the pipe and wakeup legs own most of the premium, batching has a strong case; if
supervision or worker bookkeeping owns a substantial part, the design target
changes.

Batch sizes of 8, 32, and 128 should be compared with batch-of-one to expose
where amortization saturates and where latency or retained memory becomes
unacceptable. The estimate that eight requests could divide a 50 microsecond
fixed cost to 6.25 microseconds per member is an opportunity illustration, not
a prediction.

Input-order outcome framing is the simpler initial reference. Completion-order
delivery should be considered only if variable-duration evidence shows that
head-of-line blocking matters. Batching must not revive AR-0020 staging or
optimize XPath to address a process-boundary cost.

Partial completion is the most consequential failure question. If a worker dies
after accepting 32 transformations, the host may know that some correlated
outcomes were completely transferred while other members were unstarted,
executing, or executed without an observed result. The transport must preserve
that per-member evidence, must not silently retry ambiguous attempts, and must
not let one ordinary member failure poison independent sibling outcomes. Worker
loss remains an operational boundary failure governed by AR-0018's request and
attempt provenance rather than proof that any input was defective.

The resulting architectural pressure is:

> Amortize the fixed isolated boundary across multiple independent transforms
> while preserving per-transform identity and boundedness.

The review therefore supports opening AR-0021 and retaining the current
persistent binary batch-of-one path as its semantic, diagnostic, and
containment oracle.

## Initial-result follow-up

The first bounded prototype makes the optimization case convincing. Its larger
relative gains on cheaper transforms and smaller gains on the 500-item tier
match transport-cost amortization rather than a change to XPath execution. The
aggregate-response latency increase also gives the mechanism a clear product
shape: high-throughput transform-set transport, not unconditional batching for
latency-sensitive calls.

The next admission pressure is containment rather than more throughput work.
Member cancellation and budgets must remain independent; partial worker loss
must distinguish fully transferred outcomes from ambiguous and unstarted
members; no retry may be hidden; and teardown must release retained state.
Incremental delivery should remain behind that trustworthy aggregate-response
oracle until fault behavior is proved.
