# Peer ADR-0004 Review: Monday

| Field | Value |
| --- | --- |
| Received | 2026-08-25 |
| Reviewer | Monday, identified by the project owner as a peer |
| Scope | ADR-0004 source cohesion and decomposition policy |
| Disposition | Accepted decision confirmed; minor clarification applied |

## Confirmed findings

The reviewer endorsed ADR-0004's governing distinction: physical size triggers
review, while responsibility and ownership seams justify decomposition. The
review specifically confirmed the value of:

- responsibility triggers that apply even below numeric thresholds;
- post-extraction coupling analysis that rejects a fragmented monolith sharing
  one broad context;
- checkpointed separation of structural extraction from semantic repair;
- conservation requirements spanning standards semantics, node identity,
  authority, diagnostics, concurrency, ABI, performance, and unsafe invariants;
- private-module preference under ADR-0001; and
- a first calibration review that permits real FastXSLT evidence to revise the
  provisional numeric thresholds.

The reviewer recommended leaving ADR-0004 accepted.

## Clarification applied

Compile-time pressure was added as an explicit review observation. Generic,
macro, monomorphization, and code-generation coupling may reveal a useful seam
independently of line count or runtime cost. A review triggered by that pressure
records clean/incremental build and downstream recompilation effects.

This clarification does not make compile time an automatic crate-split rule,
line threshold, or CI gate. ADR-0001 still requires independent evidence before
creating a crate, and ADR-0004 still rejects decomposition that merely relocates
coupling or build cost.
