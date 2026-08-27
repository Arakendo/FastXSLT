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
  -- Accepted through ADR-0007; modern reference editions guide a staged,
  explicitly incomplete profile without creating a broad conformance claim.
- [AR-0002: ASP.NET Host Integration Boundary](AR-0002-aspnet-host-integration.md)
  -- Under Review; native and isolated candidates now share an executable
  lifecycle with measured cost/guarantee differences, while representative
  consumer requirements still block supported profile selection.
- [AR-0003: Memory Resource Snapshots and Batch Transforms](AR-0003-memory-resource-snapshots-and-batch-transforms.md)
  -- Accepted through ADR-0002 and ADR-0005; bounded memory-resident snapshots
  and unordered independent transform sets are binding, while cache,
  supervision, and host lifecycle questions remain in focused reviews.
- [AR-0004: Structured Diagnostics and Boundary Error Identity](AR-0004-structured-diagnostics-and-boundary-error-identity.md)
  -- Incubating; derive machine-readable findings and operation failures from
  implemented cases without introducing a speculative global error framework.
- [AR-0005: Semantic Inspection and Explainability Boundary](AR-0005-semantic-inspection-and-explainability-boundary.md)
  -- Incubating; determine how hosts inspect resources, compilation, and
  execution without stabilizing private engine representation.
- [AR-0006: Compatibility Domains, Versioning, and Persisted Artifacts](AR-0006-compatibility-domains-versioning-and-persisted-artifacts.md)
  -- Deferred; no current consumer or measurement justifies a persisted
  artifact, stable ABI, or umbrella compatibility mechanism.
- [AR-0007: Streaming Compatibility of Core Architecture](AR-0007-streaming-compatibility-of-core-architecture.md)
  -- Deferred; preserve the semantic/physical seam without implementing a
  streaming strategy or generalized provider until profile or measured
  workload pressure reopens the question.
- [AR-0008: XML Parser Mechanics Boundary](AR-0008-xml-parser-mechanics-boundary.md)
  -- Under Review; evaluate a private event parser without delegating XML
  policy, XDM ownership, resource authority, or public types.
- [AR-0009: Prepared Input Retention and Cache Lifecycle](AR-0009-prepared-input-retention-and-cache-lifecycle.md)
  -- Incubating; determine what immutable source-derived state may be reused and
  who owns its bounded lifetime without creating hidden semantics or authority.
- [AR-0010: Execution Supervision, Cooperative Control, and Hard Isolation](AR-0010-execution-supervision-cooperative-control-and-hard-isolation.md)
  -- Incubating; separate bounded cooperative dispatch from the process boundary
  required for forcible termination and hard recovery.
- [AR-0011: Corpus Verification Ledger, Classification, and Reporting](AR-0011-corpus-verification-ledger-classification-and-reporting.md)
  -- Accepted through ADR-0006; preserve native identity, explainable
  disposition, separate selection/execution axes, and denominator conservation
  while schema, storage, CI, and publication remain deferred.
