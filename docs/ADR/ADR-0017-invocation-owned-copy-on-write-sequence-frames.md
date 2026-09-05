# ADR-0017: Invocation-Owned Copy-on-Write Sequence Frames

- Status: Accepted
- Date: 2026-09-05
- Related reviews: AR-0013 and the 2026-09-04 performance review
- Related ADRs: ADR-0002, ADR-0003, ADR-0004, ADR-0014
- Related evidence: `docs/Evidence/runtime-frame-nested-copy-on-write-2026-09-05.md`
- Supersedes: None

## Context

ADR-0014 already permits immutable atomic bindings to be shared by runtime
frames within one invocation. Atomic sequences, source-node selections,
temporary trees, and local-shadow metadata still received complete deep clones
whenever `execute_sequence` established a nested lexical scope.

A field-attribution probe showed that a deliberately populated frame could make
this expensive, but did not establish that a compiled transform encountered the
mechanism. A subsequent compiled workload retained all three non-atomic value
kinds across nested literal-result sequences. With 16 bindings of each kind and
eight nested result levels, the complete reference requested 8,306 allocations,
481,676 total bytes, and 438,712 peak live bytes while taking 635.092 us. A safe
copy-on-write prototype requested 1,145 allocations, 108,228 total bytes, and
69,800 peak live bytes while taking 82.591 us.

The hostile comparison introduced every non-atomic value kind inside every
nested scope, forcing detachment instead of merely reading shared maps. At the
largest measured shape, the shared path was effectively neutral: 728.864 us
versus 730.396 us, with nine fewer allocation requests and 360 fewer requested
bytes. The smaller shape remained faster.

## Decision

Use private invocation-owned copy-on-write maps for runtime atomic sequences,
source-node selections, temporary trees, and local-shadow metadata.

The representation must:

- share maps only among lexical frames of one invocation;
- detach through safe `Arc::make_mut` before every mutation;
- avoid detaching an unrelated value-kind map merely to remove a name that is
  absent from that map;
- preserve cross-kind lexical shadowing, global fallback, node and temporary-
  tree identity, work accounting, cancellation, diagnostics, and cleanup;
- retain a test-only complete deep-clone path as the semantic, failure, and
  measurement oracle;
- remain private to the runtime and expose no map or `Arc` through a facade or
  host boundary; and
- use no unsafe code.

The values remain invocation state. Sharing their map ownership does not turn
them into compiled or prepared state.

## Non-decisions

This ADR does not admit:

- sharing across invocations, prepared inputs, snapshots, workers, or engine
  generations;
- concurrent mutation of one invocation;
- a parent-chain, general environment abstraction, persistent cache, or global
  interning scheme;
- mutation of a source document or temporary tree through a shared frame;
- any public representation, ABI, or host-lifecycle change; or
- unsafe access, custom allocation, or unchecked indexing.

## Consequences

Read-only nested scopes now pay fixed-size reference-count operations instead
of cloning every retained non-atomic value. A scope that mutates one value kind
copies only that kind and the shadow metadata when separation is required.
Mutation-heavy nesting can therefore repay the deferred copies, but the
measured worst case is neutral rather than materially worse.

The implementation gains additional `Arc` fields even though frames are not
shared across threads. Their purpose is safe copy-on-write ownership, not a
same-invocation concurrency guarantee.

## Validation

- Differentially execute read-only and mutation-at-every-level compiled
  workloads through shared and complete-clone frames.
- Verify that mutation detaches the affected value-kind map and cannot change
  the parent frame.
- Compare exact result, work-domain totals, cancellation category/domain, and
  cancellation charge boundary.
- Exercise normal cross-kind shadowing and temporary-tree behavior through the
  complete workspace suite.
- Measure allocation requests, requested bytes, peak live bytes, and latency
  over empty, read-only-populated, and mutation-heavy nested shapes.
- Run concurrent invocation, generation-overlap, corpus, workbench, and full
  workspace verification gates.

Revisit if representative mutation-heavy workloads regress, reference-count
traffic becomes material, a non-clone frame strategy is proposed, or any owner
beyond one invocation would share these maps.
