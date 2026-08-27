# ADR-0009: Scalar Native Invocation Controls

- Status: Accepted
- Date: 2026-08-26
- Related reviews: AR-0002, AR-0010
- Related ADRs: ADR-0003, ADR-0008
- Related evidence: `docs/Evidence/aspnet-predispatch-cooperative-cancellation-2026-08-26.md`, `docs/Evidence/aspnet-deterministic-instruction-budget-2026-08-26.md`, and `docs/Evidence/aspnet-native-invocation-controls-2026-08-26.md`
- Supersedes: None

## Context

ADR-0008 admitted an unpublished native .NET workbench ABI but deliberately
excluded cancellation and invocation controls from its first operation set.
The isolated ASP.NET path has since preserved direct-engine diagnostics for
already-signalled cooperative cancellation and deterministic XSLT-instruction
budget exhaustion. AR-0002 now needs equivalent in-process evidence before it
can compare lifecycle guarantees.

Active cancellation through a callback or shared control handle would add
cross-thread signalling, lifetime, disposal, and same-handle execution
invariants. Those mechanisms have not earned admission. Already-known
pre-dispatch cancellation and integer budgets need only invocation-local scalar
values; they do not require retained foreign memory, callbacks, or another
threading contract.

## Decision

Extend the explicitly unstable version-zero workbench ABI with one synchronous
controlled-transform operation. It carries:

- the existing engine handle and copied request-identity bytes;
- a fixed-width cancellation flag whose only valid values are zero and one;
  and
- a fixed-width unsigned maximum XSLT-instruction count that must fit the
  platform's Rust `usize`.

The native layer creates invocation-local safe Rust cancellation and budget
state after copying and validating all input. A true cancellation flag is
signalled before execution begins. The engine observes it cooperatively at its
ordinary owned charge points. The instruction limit uses the same safe
reference implementation and `FXCT0002 / limit` diagnostic as direct and
isolated execution. Cancellation retains precedence when both controls would
stop the first charge.

Keep the existing uncontrolled transform operation unchanged for the hot-path
comparison. The managed wrapper serializes each engine handle as before and
offers the controlled operation explicitly. Controlled failure must not poison
the retained compiled stylesheet or prepared source.

This extension does not admit:

- callbacks, polling foreign memory, borrowed cancellation state, or managed
  object pointers;
- active mid-execution signalling, deadlines, or a hard-termination claim;
- asynchronous native completion or concurrent use of one engine handle;
- resolver authority, generation replacement, or allocator transfer; or
- new unsafe pointer operations.

Active signalling requires another accepted ADR or a superseding revision that
defines control-handle ownership, transform/control races, release behavior,
and panic/disposal containment.

## Safety and ABI impact

The operation reuses ADR-0008's validated, immediately copied request-identity
pointer. Its new values cross the ABI by value and introduce no pointer,
borrowing, aliasing, allocation, or callback invariant. The exact first-party
unsafe surface therefore remains two unsafe blocks. One additional exported
symbol and its scoped export allowance increase the audited structural counts
to ten export attributes and twelve unsafe-code allowances.

Invalid cancellation flags become `FXFFI0009 / boundary`. An instruction limit
that cannot fit the target platform becomes `FXFFI0010 / boundary`. Ordinary
cancellation and budget outcomes retain their engine-owned diagnostic code,
category, request identity, and detail.

## Consequences

The workbench can compare two important lifecycle controls without pretending
that pre-dispatch cancellation is active cancellation or hard isolation. It
also establishes that scalar policy transfer does not require a callback-heavy
ABI.

The new operation is still private and unstable. It does not choose a public
.NET API, exception hierarchy, default limit, or deployment mode. Separate
operations may eventually prove clearer than one policy struct, but no struct
layout is stabilized by this experiment.

## Validation

- Differentially assert exact cancellation and instruction-budget diagnostic
  fields against the safe reference path.
- Reject cancellation values other than zero and one as boundary failures.
- Execute an ordinary transform after each controlled failure on the same
  engine handle.
- Preserve the ADR-0008 pointer-copy, panic-quarantine, disposal, independent
  handle, and unsafe-surface gates.
- Exercise the managed wrapper through the live ASP.NET operational harness.

Revisit this decision if active cancellation is required, policy fields expand
enough to pressure the call shape, platform integer widths differ in an
admitted target, or measurement shows controlled invocation changes the normal
hot path.
