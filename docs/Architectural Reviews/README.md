# Architectural Review Records

Architectural Reviews preserve questions, triggers, evidence, ownership
analysis, alternatives, findings, dispositions, follow-up, and reopening
criteria. They occupy the space between informal research and binding ADRs.

```text
Question or implementation pressure
              |
              v
     Architectural Review
              |
     +--------+-------------------+
     |                            |
defer / reject / no change   accept a decision
                                  |
                                  v
                           proposed or revised ADR
```

An AR does not override an accepted ADR. If a review finds that a decision must
change, preserve the review as evidence and deliberately supersede the ADR.

## When to open a review

Open an AR for a new subsystem or crate, unclear semantic or host ownership, a
stable public contract, a parser/backend choice that shapes semantics, repeated
boundary friction, a potentially invalid ADR, or a deferred/rejected proposal
whose reasoning should remain durable.

Ordinary bug fixes, local refactors, and implementation choices that preserve
accepted contracts do not require an AR.

## Statuses

- **Proposed** -- question and initial evidence recorded.
- **Under Review** -- alternatives are actively being evaluated.
- **Incubating** -- plausible direction needs more cases or consumers.
- **Accepted** -- findings resulted in a named ADR or ADR revision.
- **Deferred** -- no decision is justified until named triggers occur.
- **Rejected** -- proposal should not proceed under current evidence.
- **No Change** -- existing architecture was confirmed.
- **Superseded** -- a later review replaced the active findings.
- **Reopened** -- new evidence started a new append-only review cycle.

## Naming and index

Copy [TEMPLATE.md](TEMPLATE.md), use the next unused independent sequence number,
and add the record to this index. Never reuse a retired number.

- [AR-0001: Initial Standards Profile and Conformance Baseline](AR-0001-initial-standards-profile.md)
  -- Under Review; required before FastXSLT claims a version or implements a
  public transform slice.
- [AR-0002: ASP.NET Host Integration Boundary](AR-0002-aspnet-host-integration.md)
  -- Proposed; determine how managed applications embed and reuse FastXSLT
  without confusing Rust-core speed with end-to-end host performance.
- [AR-0003: Memory Resource Snapshots and Batch Transforms](AR-0003-memory-resource-snapshots-and-batch-transforms.md)
  -- Under Review; study bounded preload, sealed resource identity, shared
  compilation, and batch/graph execution for volume consumers.
