# ADR-0003: Unsafe Rust Exception Policy

- Status: Accepted
- Date: 2026-08-25
- Related reviews: None
- Supersedes: None

## Context

FastXSLT intends to be a high-throughput embedded engine and may eventually face
measured pressure involving FFI, SIMD intrinsics, compact representations,
allocation, or another capability that safe Rust cannot provide adequately.
Unsafe Rust can remove compiler checks required to uphold memory safety. Passing
tests, benchmarks, Miri, sanitizers, or fuzzing can find defects but cannot prove
that every unchecked invariant holds for all executions.

The most sensitive candidate areas include XDM node identity and storage,
arenas/indexes, strings and byte views, snapshot and compiled-artifact lifetimes,
caches, FFI ownership, and concurrent transform reuse. A defect in these shared
foundations could remain latent until a particular workload, optimizer decision,
host callback, or request interleaving triggers undefined behavior.

## Decision

First-party unsafe code is prohibited by default. The workspace retains
`unsafe_code = "forbid"`; this ADR does not admit any unsafe implementation.

A future exception requires a separate accepted ADR tied to a concrete code
boundary. That ADR and implementation must satisfy all applicable requirements:

1. **Necessity:** identify a required capability or measured performance budget
   that safe Rust cannot reasonably satisfy. Convenience and speculative speed
   are insufficient.
2. **Safe alternatives:** implement, prototype, or analyze viable safe designs
   and record why they fail the named requirement.
3. **Safety contract:** state every invariant required to avoid undefined
   behavior, who establishes it, who preserves it, and over what lifetime and
   concurrency domain it holds.
4. **Containment:** minimize unsafe functions, blocks, modules, features, and
   build targets. Expose a safe API wherever feasible and validate all
   caller-controlled inputs before they reach unchecked operations.
5. **Local explanation:** every unsafe block includes a nearby `SAFETY` comment
   connecting the operation to its invariant. Public unsafe functions include a
   `# Safety` contract.
6. **Explicit operations:** `unsafe_op_in_unsafe_fn` remains denied so an unsafe
   function body does not make unchecked operations visually implicit.
7. **Reference behavior:** retain a safe reference implementation whenever
   practical. Differential tests must compare semantic results and diagnostics,
   not only serialized happy-path output.
8. **Focused verification:** attack boundaries, aliasing, lifetime, invalid
   indexes, malformed input, resource exhaustion, cancellation, concurrency,
   panic, and host/FFI failure conditions applicable to the invariant.
9. **Specialized tools:** add Miri, sanitizers, fuzzing, property tests, model
   checking, ABI tests, or platform-specific stress tests where each can inspect
   the actual risk. Record unsupported coverage rather than implying it ran.
10. **Measured benefit:** benchmark the safe and unsafe implementations through
    representative workloads and consumer boundaries. The gain must justify the
    permanent audit and maintenance cost.
11. **Reviewability:** record the exact unsafe surface and prevent its expansion
    without another review. Generated unsafe code and unsafe hidden in macros
    count toward that surface.
12. **Removal criteria:** name the performance threshold, new safe primitive,
    compiler/library improvement, platform change, or observed defect that would
    remove or reopen the exception.

For FFI, the exception ADR must additionally define ABI/version compatibility,
ownership and allocator pairing, string/buffer encoding, nullability, callback
lifetime and threading, panic containment, cancellation, and error transfer.

## Consequences

FastXSLT can consider unsafe code when evidence demonstrates real value without
turning “tested” into a claim of soundness. Unsafe optimization remains a named,
auditable architectural exception rather than an ordinary local refactor.

The safe reference path may cost code and maintenance, but provides a semantic
oracle, fallback, and differential target. An exception may omit it only by
explaining why a safe reference is impractical and what substitutes for that
lost evidence.

This policy governs first-party code. Dependencies may contain unsafe code even
when FastXSLT forbids it locally. Dependency admission must inspect unsafe
surface, maintenance, provenance, security history, and replaceability in
proportion to its semantic and trust role.

When an exception is accepted, enforcement must change only as narrowly as Rust
tooling permits—for example, isolating the code in a reviewed module or crate
rather than weakening the entire workspace silently.

## Alternatives considered

### Permanently forbid all unsafe code

This offers the simplest local policy but may preclude a required native ABI or
a proven optimization with a small, defensible invariant surface.

### Permit unsafe code when tests pass

Tests sample behavior and cannot prove absence of undefined behavior. This would
let ordinary implementation work silently expand the trusted computing base.

### Permit unsafe code after maintainer review without an ADR

Review is necessary but an informal decision would lose the measured reason,
invariant ownership, tool coverage, rejected safe alternatives, and removal
triggers needed by future maintainers.

### Admit narrowly through an evidence-bearing ADR

This preserves a strong default while allowing a real capability or performance
requirement to justify a locally auditable exception.

## Validation

- CI continues rejecting first-party unsafe code under the workspace lint.
- `unsafe_op_in_unsafe_fn` remains denied in anticipation of any exception.
- Any exception ADR links its benchmark, safety contract, verification matrix,
  exact code surface, safe reference or substitute evidence, and removal
  criteria.
- Reviews reject unsafe expansion not covered by the accepted exception.
- Dependency audits distinguish first-party prohibition from dependency unsafe
  code instead of claiming the entire binary is unsafe-free.
