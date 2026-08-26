# Private Preparation Concurrency and Retry Baseline

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Implementation | Private `PreparedInputBuilder` experiment |
| Decision pressure | AR-0009 construction ownership, failure, retry, and single-flight policy |
| Claim | Current explicit-preparation baseline; no public cache or concurrency guarantee |

## Executable observations

Two builders may prepare the same logical identity from the same sealed
snapshot concurrently. Each builder owns its construction work and produces a
distinct `Arc<Document>`. The documents preserve the same source identity,
node count, and string-value semantics, but their allocations are not merged.

Cancellation before XML work and XDM-budget exhaustion both return structured
control failures without inserting a prepared entry. Reusing that builder with
fresh invocation control then prepares the identity successfully. A failed
attempt therefore does not poison the builder or expose a partial document.

## Interpretation

The private experiment currently has explicit construction, not lazy first
access. It consequently has no shared in-progress state, waiters, or
single-flight behavior. Concurrent independent builders duplicate work by
design. This is the reference behavior against which a future shared
preparation mechanism would need to demonstrate sufficient benefit.

Any single-flight proposal still needs to decide and test:

- whether one waiter's cancellation cancels only that wait or shared work;
- how construction failure is classified and delivered to all waiters;
- whether and when a later request retries;
- which owner accounts for in-flight bytes and work; and
- how snapshot generation and logical identity participate in the key.

These observations do not select lazy preparation, shared caching, waiter
semantics, eviction, or a public prepared-input handle.
