# Architecture Decision Records

ADRs record accepted, binding architectural decisions. Implementation work must
follow them unless a later accepted ADR explicitly supersedes the decision.

Proposed, unnumbered drafts live in [`Proposed/`](Proposed/). A draft has no
architectural authority until maintainers accept, number, and move it here.

## Statuses

- **Accepted** -- current binding decision.
- **Superseded** -- replaced by another named ADR.
- **Deprecated** -- retained for compatibility while being removed.

ADRs are append-oriented historical records. Correct typos and links in place,
but supersede a materially changed decision instead of rewriting its history.

## Index

- [ADR-0001: Evidence-Led Modular Monolith](ADR-0001-evidence-led-modular-monolith.md)
  -- Accepted; keep logical engine layers in one crate until concrete pressure
  justifies a structural split.
- [ADR-0002: Memory-Resident Execution](ADR-0002-memory-resident-execution.md)
  -- Accepted; host adapters import owned bytes and close handles before sealed
  snapshots, while core compilation and transformation avoid implicit disk I/O.

