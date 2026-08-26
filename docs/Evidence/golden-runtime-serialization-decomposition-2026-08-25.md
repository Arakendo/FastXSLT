# Golden Runtime Serialization Decomposition

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Checkpoint | `b5d7869` with 38 passing tests |
| Structural revision | `7dbdb25` |
| Scope | ADR-0004 private source-unit cohesion checkpoint |
| Public guarantee | None |

## Trigger

The private golden runtime reached 988 physical lines while owning transform-set
composition, semantic result construction, failure translation, serialization,
and their tests. It had not crossed ADR-0004's 1,001-line numeric signal, but the
next result-accounting change would have crossed it and serialization was
already an independently named responsibility.

## Extraction

XML serialization, escaping, byte charging, and the bounded output buffer moved
to a private child module. The parent continues to own transform-set composition,
execution, semantic result construction, and shared private failures. The child
depends on the parent's semantic result and failure vocabulary; the parent calls
the child only after semantic execution succeeds.

No public item, crate boundary, dynamic dispatch, allocation, alternate semantic
path, or host authority was introduced. Prepared-input tests continue to call
the same parent re-export, so direct and prepared execution retain one serializer.

## Conservation evidence

Before and after the extraction:

- all 38 focused tests passed;
- formatting, warnings-as-errors clippy, documentation, Markdown links, and
  pinned-corpus integrity passed;
- semantic result assertions and exact golden serialization were unchanged;
- structured serialization failures, byte limits, cancellation, request
  identity, and work-domain identity were unchanged; and
- no filesystem, snapshot, concurrency, ABI, dependency, or unsafe-code boundary
  changed.

The parent unit became 859 lines at the structural checkpoint and the serializer
was 139 lines. Subsequent result-accounting work began only after commit
`7dbdb25`, preserving attribution between decomposition and semantic changes.

## Disposition

The extraction is retained. It reduces responsibility coupling and keeps the
semantic result distinct from its encoding while remaining entirely private.
Revisit only if the serializer begins importing broad execution state, if a
second output method demonstrates a different ownership seam, or if measured
hot-path costs show the private call boundary matters.
