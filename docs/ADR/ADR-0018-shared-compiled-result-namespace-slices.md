# ADR-0018: Shared Compiled Result-Namespace Slices

- Status: Accepted
- Date: 2026-09-05
- Related reviews: AR-0013 and the 2026-09-04 performance review
- Related ADRs: ADR-0002, ADR-0003, ADR-0004
- Related evidence: `docs/Evidence/namespace-scope-scaling-fixture-2026-09-05.md`
- Supersedes: None

## Context

Literal and statically named computed result elements compile their
stylesheet-derived namespace bindings into immutable instruction data. Runtime
construction previously deep-cloned every binding and both strings into each
semantic result element on every invocation.

The namespace-depth fixture contains 384 distinct declarations at depth 48,
but XSLT namespace-copy semantics produce 9,408 retained binding occurrences in
the compiled element plans. Those occurrences hold 350,544 bytes of string
capacity. One reference result construction requested 19,255 allocations and
863,823 bytes, peaking at 842,319 live requested bytes.

A safe prototype stores each compiled element's immutable bindings in an
`Arc<[NamespaceBinding]>` and lets the semantic result retain that slice. The
same construction requested 391 allocations and 60,927 bytes, peaking at 39,423
bytes, while improving median construction from 834.306 us to 19.850 us. The
ordinary shallow `for-004` result was neutral-to-positive.

## Decision

Store the namespace bindings of compiled literal and statically named computed
element instructions in immutable reference-counted slices. Semantic result
elements may retain the same slice rather than deep-copying its bindings.

The representation must:

- contain only stylesheet-derived immutable namespace values;
- preserve the exact per-element binding sequence, prefix preference,
  exclusions, undeclarations, namespace fixup, and serialized result;
- let a result safely outlive the compiled generation owner by owning an `Arc`,
  rather than borrowing from the generation;
- allow concurrent invocations to share only the immutable slice allocation;
- retain a test-only complete deep-copy result-construction path as the
  semantic, work-accounting, and measurement oracle;
- account for the slice allocation and strings in compiled retention estimates;
- remain private and use no unsafe code.

Namespaces acquired from source nodes, temporary trees, or other dynamic
construction remain owned by the resulting node. They do not enter the
compiled sharing path.

## Non-decisions

This ADR does not admit:

- interning namespace strings across compiled element plans or generations;
- sharing source-derived or invocation-derived namespace state;
- mutable namespace bindings or result-tree mutation through shared storage;
- a public namespace-node, result-tree, provider, or serializer API;
- a persisted representation or ABI layout; or
- unsafe reference counting, unchecked access, or custom allocation.

## Consequences

Repeated transformations no longer reproduce compiled namespace strings in
every semantic result. A result can retain part of a compiled generation's
namespace allocation until serialization or result disposal completes. This is
explicit ownership, not a hidden cache.

Reference counting adds one clone/drop per statically compiled result element.
The shallow control did not show a regression; revisit if a representative
namespace-light workload attributes material cost to that traffic.

## Validation

- Differentially construct the same result through shared and complete-copy
  paths and compare the full semantic result and work-domain totals.
- Serialize results after dropping the originating engine generation.
- Execute concurrent invocations over one compiled stylesheet and compare bytes.
- Measure ordinary shallow output and namespace depths 8, 24, and 48 for
  latency, allocation requests, total requested bytes, and peak live bytes.
- Exercise output methods, namespace shadowing, exclusions, fixup, copied
  source/temporary nodes, corpus cases, and host boundaries through the full
  workspace gate.

Revisit if result mutation is introduced, retained results materially delay
generation retirement, namespace-light workloads regress, or sharing beyond
one compiled element plan is proposed.
