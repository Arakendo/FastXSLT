# Peer SDD Review: Monday

| Field | Value |
| --- | --- |
| Received | 2026-08-25 |
| Reviewer | Monday, identified by the project owner as a peer |
| Scope | Initial FastXSLT Software Design Document |
| Disposition | Accepted with bounded documentation revisions |

## Summary

The reviewer found the initial SDD appropriately restrained and praised its
separation of replaceable XML mechanics from engine-owned semantics, its
compilation/runtime boundary, deny-by-default ambient resource policy, and its
requirement that performance claims carry evidence.

The central criticism was that the SDD defined semantic owners more precisely
than the contracts between those owners. The review recommended strengthening
constraints that can reject an illegal design without selecting concrete Rust
types prematurely.

## Accepted recommendations

The SDD now makes these contracts explicit:

- architectural invariants for semantic ownership, static and dynamic state,
  node identity/order, resource policy, provenance, results, optimization, and
  instrumentation;
- conceptual dependency direction and forbidden upward semantic ownership;
- minimum XDM assumptions independent of physical representation;
- a compilation pipeline separating syntax, static resolution, semantic
  normalization, and executable optimization;
- provenance preservation through lowering and optimization, including distinct
  stylesheet, source-document, and host/resource location roles;
- separate authority/capability and budget/limit concepts;
- a semantic transformation result boundary before serialization;
- observability that is explicitly supplied and semantically inert;
- compound thread-safety, reentrancy, and concurrency questions covering both
  compiled artifacts and host capabilities;
- immediate architectural decisions separated from deferred capabilities;
- a policy preventing open decisions from being settled implicitly by a
  convenient implementation.

The contributor instructions received matching static/dynamic, provenance,
result/serialization, observability, and decision-policy guardrails.

## Recommendations intentionally kept non-concrete

The review offered an illustrative dependency graph and semantic operation set.
FastXSLT adopted them as conceptual ownership constraints, not as fixed Rust
traits, module APIs, multiple IRs, or a particular tree design.

The review did not resolve and the revision does not resolve:

- the XSLT/XPath standards profile;
- parser, DOM wrapper, arena, or owned-tree choice;
- concrete XDM interfaces or typed-value scope;
- number, format, or stability of intermediate representations;
- sync/async API shape or result types;
- deterministic versus best-effort limits;
- observability library or event schema;
- thread-safety guarantees;
- streaming, schema awareness, packages, extensions, or alternate backends.

Those remain routed through AR-0001 or future Architectural Reviews and ADRs.

## Result

The revision strengthens the SDD as a constraint system while preserving its M0
restraint. It can now reject circular semantic ownership, leaked invocation
state, provenance-destroying lowering, ambient authority, serialization-coupled
semantics, and globally stateful instrumentation without prejudging the concrete
implementation.

