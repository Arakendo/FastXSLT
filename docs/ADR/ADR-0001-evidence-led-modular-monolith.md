# ADR-0001: Evidence-Led Modular Monolith

- Status: Accepted
- Date: 2026-08-25
- Related reviews: None
- Supersedes: None

## Context

FastXSLT needs distinct ownership for XML adaptation, XDM meaning, XPath, XSLT,
stylesheet compilation, execution, and diagnostics. At project creation there
are no real callers, dependency graphs, compile-time measurements, or independent
release requirements proving that these logical layers should be separate
crates.

The reviewed Tokimu project demonstrates the value of explicit ownership and
corpus-led admission. Its embedded Weaver XSLT peer also demonstrates that a
single package can preserve strong logical boundaries while a product is still
discovering its semantic and host-facing contracts.

## Decision

- Begin with one publish-disabled library crate, `fastxslt`.
- Represent the engine phases as private logical modules.
- Put golden transformation evidence in the repository-level `corpus/` tree so
  it is not owned by one implementation module.
- Add a crate only when at least one concrete pressure exists: required
  dependency direction, independent reuse or publication, materially different
  platform support, compile-time isolation, security containment, or measured
  build/runtime benefit.
- Require an Architectural Review before a split that changes ownership or
  creates a new stable cross-crate contract.

## Consequences

Early work can change internal representations without versioning artificial
crate APIs. Logical boundaries remain reviewable in code and documentation, but
Cargo cannot yet enforce every dependency direction. The project accepts that
tradeoff until evidence shows that structural enforcement is worth its public
and build-system cost.

The decision does not require a permanently monolithic crate and does not decide
the XML parser, XDM representation, standards profile, or public API.

## Alternatives considered

### One crate per conceptual layer immediately

This enforces dependency direction but creates cross-crate types and APIs before
the first vertical slice reveals which representations need to cross boundaries.

### One undifferentiated implementation module

This avoids premature packages but obscures semantic ownership and makes later
extraction needlessly difficult.

## Validation

- The workspace contains one library crate and compiles without dependencies.
- Private modules name each currently understood logical layer.
- The first vertical slice can cross the layers without exposing internal types
  publicly.
- Future extraction reviews must cite the pressure that this starting shape no
  longer satisfies.

