# Peer Documentation Review: Tosumu Database

| Field          | Value                                                                                    |
| -------------- | ---------------------------------------------------------------------------------------- |
| Date           | 2026-08-25                                                                               |
| Peer checkout  | `F:\LocalSource\Database`                                                                |
| Peer revision  | `a84a0c09c05de85751c1456151ac35afeb97da15`                                               |
| Peer worktree  | Clean at review time                                                                     |
| FastXSLT scope | Documentation authority, consumer boundaries, safety, diagnostics, inspection, and plans |

## Purpose

Review the Tosumu database project's documentation system for governance and
evidence practices that help an embedded XSLT engine before API stability.
Database storage semantics were not treated as FastXSLT architecture.

## Inputs reviewed

- `AGENTS.md`, `SECURITY.md`, `ERRORS.md`, and `INSPECT_API.md`;
- `docs/index.md`, `project-governance.md`, `safety-and-limits.md`,
  `error-model.md`, and `inspect-api.md`;
- ADR, Architectural Review, Plan, and Change Request guidance and templates;
- the Tokimu consumer request, provider baseline, implementation plan, and
  fixture records under `docs/CRs/Tokimu`; and
- selected `DESIGN.md` sections on testing evidence, compatibility, security,
  inspection, and explicit limitations.

## Findings adopted

### Consumer requests need their own authority class

FastXSLT is primarily a library used by other applications. A consumer need is
valuable architectural pressure, but placing it directly in a specification or
plan can accidentally make ASP.NET, a particular FFI layer, or one application's
types authoritative inside the engine.

FastXSLT therefore adds `docs/Change Requests/`. A CR preserves the consumer's
problem, requested contract, ownership boundary, and acceptance evidence. It is
not a commitment. Unresolved architecture still moves through an AR and an
accepted change still requires an executable plan.

### Public safety limits should be easy to find

The SDD already discusses security and budgets, but a potential adopter should
not need to read the entire architecture to learn that FastXSLT performs no
transforms, has no selected standards profile, has no resource-limit guarantee,
and is not production ready. `docs/safety-and-limits.md` now states those facts
and links back to owning contracts. `SECURITY.md` adds the current reporting
route and threat-scope limits for the public repository.

### Plans should conserve contracts across small vertical slices

The peer plan template usefully requires baseline evidence, ownership,
compiling slices, explicit exit states, validation matrices, failure semantics,
compatibility, security, performance bounds, and honest parking criteria.
FastXSLT's adapted template emphasizes standards cases, resource authority,
source provenance, semantic results versus serialization, consumer integration,
and correctness-gated performance.

### Structured findings and operation failures are different

The peer error design distinguishes a meaningful report containing findings
from a failure that prevents a trustworthy report. That maps well to stylesheet
compilation, independent batch results, and future conformance reports.
FastXSLT's SDD now records the distinction while deferring exact status and code
vocabularies until implemented failure owners exist.

### Inspection should expose meaning, not representation

An embedded host will need to understand admitted resources, compilation
dependencies, selected profile, diagnostics, capability requirements, and
bounded execution behavior without importing internal types. The SDD and
roadmap now name a read-only semantic inspection direction while explicitly
refusing to stabilize parser ASTs, XDM arenas, optimizer IR, cache layouts, or
module boundaries for diagnostic convenience.

## Concepts retained but deferred

### Stable error catalog

Tosumu has emitted failures and multiple tool consumers, so stable codes and
status mappings have owners. FastXSLT does not. A root error catalog now would
be speculative. M1 will establish the first boundary shapes from real negative
cases; M2 will stabilize diagnostic identifiers across the implemented XML and
XPath slice. AR-0004 now incubates that contract and names the evidence needed
before stabilization.

### Dedicated inspection API specification

No compiled stylesheet or runtime report exists yet. Exact envelope fields,
schema versions, and compatibility rules are deferred until the M2 semantic
inspection snapshot is exercised by a real caller. AR-0005 owns that incubation.

### Migration ledger and serialized artifact format

FastXSLT has no stable public API, ABI, persisted cache, or compiled-artifact
format. The plan template requires compatibility analysis and explicitly bars
an accidental persisted format. Versioning and migration documents should be
introduced only when a real durable boundary is proposed. AR-0006 separates the
compatibility domains and records the evidence required for that decision.

### Database-specific operational records

WAL recovery, page verification, backup consistency, protector management, and
physical format migration do not transfer to an in-memory XSLT engine. Their
general lessons—typed failure, bounded evidence, ownership, and honest limits—
were adopted without copying storage architecture.

## Result

The review adds documentation mechanisms that serve FastXSLT's actual product
shape: an embedded, pre-stability language engine with multiple future hosts.
It does not stabilize APIs or invent implementation claims ahead of executable
evidence.
