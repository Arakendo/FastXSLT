# Peer Unsafe-Rust Review: Monday

| Field | Value |
| --- | --- |
| Received | 2026-08-25 |
| Reviewer | Monday, identified by the project owner as a peer |
| Scope | Conditions for permitting first-party unsafe Rust |
| Disposition | Accepted and formalized in ADR-0003 |

## Summary

The reviewer supported FastXSLT's safe-Rust default and rejected “tested
properly” as sufficient permission for unsafe code. Tests and specialized tools
can find violations but cannot prove all invariants once compiler checks are
removed.

The proposed exception criteria were a measured need, written safety contract,
safe abstraction, minimal auditable surface, focused adversarial verification,
Miri/sanitizer/fuzz evidence where applicable, demonstrated benchmark benefit,
rejected safe alternatives, and ADR-governed reconsideration. The reviewer also
recommended a safe reference implementation for unsafe optimizations whenever
practical.

## FastXSLT disposition

ADR-0003 accepts the exception process while admitting no unsafe code. It adds
explicit invariant ownership, exact-surface control, FFI-specific requirements,
tool-coverage reporting, differential semantic/diagnostic parity, dependency
unsafe auditing, and removal criteria. The workspace retains
`unsafe_code = "forbid"` and denies implicit unsafe operations inside unsafe
functions.

XDM storage, arenas/indexes, strings, lifetimes, caches, snapshots, compiled
stylesheets, FFI, and concurrency remain highlighted as especially sensitive
areas rather than preapproved reasons to use unsafe code.
