# ADR-0024: Invocation Owned Global Resolution With Shared Execution

- Status: Accepted
- Date: 2026-10-14
- Opened: 2026-10-14
- Related review: [AR-0030](../Architectural%20Reviews/AR-0030-invocation-owned-global-constructor-evaluation.md)
- Related decisions: ADR-0004, ADR-0007, ADR-0012, ADR-0014, ADR-0017, ADR-0022, ADR-0023
- Supersedes: None

## Context

General global constructors can dispatch templates that discover global
dependencies during execution. The current completed-globals initialization
boundary cannot admit unchanged OASIS `variable40`. AR-0030 demonstrates safe
recursive resolution using the shared executor, without holding mutable binding
borrows across execution. Original declarations, supplied include/import modules,
qualified names, effective source owners, failures and controls have private
reference evidence.

The bounded static candidate orders straight-line named-template dependencies,
but refuses dynamic dispatch, conditional reads and local bindings. Extending
static reachability is possible work, not a demonstrated replacement for runtime
resolution. The demand reference passes unchanged `variable40` under both
preprocessing profiles; production remains unsupported.

Cost comparisons also limit the decision. The test driver's hooks and name scans
are slower than current production on ordinary all-used globals. Skipping unused
globals saves work, but does not by itself establish the permitted initialization
policy. The ownership decision must not freeze that test implementation.

## Decision

Use one invocation-owned authority for resolving admitted global bindings.
Compilation owns resolved declarations, precedence and immutable initializer
plans. The invocation owns supplied values, evaluation state, completed values,
active dependency information and failures. A read identifies the resolved
binding after module linking, not a lexical prefix or caller-local name.

An unresolved binding can execute its initializer; an active binding cannot be
re-entered without reporting circularity; a completed binding returns its
immutable value. Failed evaluation cannot publish a partial value or silently
restart on a later read. The physical cell, index, map, callback and reference-
counting layout remain private implementation choices.

General constructors execute through the ordinary sequence/template executor.
Existing specialized initializer paths may remain only as proven optimizations
of the same binding-initialization semantics. They must preserve the resolver's
binding identity, entry context, value ownership, parameter precedence,
failure/message behavior, controls and accounting for their eligible subset.
When eligibility is not proven, an optimization must fall back to shared
execution; it may not define an independent dependency, circularity, focus or
failure model. If shared execution does not yet admit the operation, retain an
explicit unsupported boundary rather than guessing semantics. No second global
value cache is introduced. Completed-value fast paths may bypass expensive
resolution plumbing without bypassing ownership, cancellation or accounting.

Static dependency analysis is an admission or scheduling optimization, not an
alternate dependency semantics. Refusal or inability to prove a dependency graph
is not a standards circularity or initialization failure. Static edges must not
fabricate executed dependencies, suppress runtime binding resolution, or report
a cycle that the authoritative resolver would not encounter under the selected
initialization policy. Circularity and binding-read semantics remain owned by
the invocation resolver.

### Initialization schedule and context

The first production integration retains eager initialization of admitted
globals before principal execution. Supplied global parameters seed their
bindings and suppress evaluation of their defaults. Requesting a binding during
initialization may recursively initialize dependencies through the same owner.
This selects recursive resolution, not lazy omission of all unused globals.

Preserve existing observable initialization behavior for already supported
programs. Do not introduce a new declaration-order side-effect guarantee. New
constructors require profile-specific error/message tests before admission;
changing unused-binding evaluation policy requires a separate documented
decision rather than a performance optimization hidden in this integration.

Global entry context is explicit and independent of the reading template's
focus, lexical locals, template parameters and current matched rule. The initial
constructor slice uses the proven principal effective-document root and singleton
focus for its admitted legacy context. This does not redefine modern global
context as the principal root. Broader modern absent/atomic/package context modes
for these constructors remain unsupported until their own parity evidence
exists; already admitted source-free initializers are not removed. ADR-0023 preprocessing
selection does not select the initialization schedule or global entry context.

### Ownership and failure

No mutable binding or map borrow may survive a recursive executor call. Active
markers clear on success, error and cancellation, and during panic unwinding;
this is not recovery from process abort. Work, depth, temporary
retention and diagnostic-context growth remain bounded; failed state and
dependent frames are released with the invocation. Values cannot leak into
compiled programs, prepared inputs, another invocation or a global registry.

Source and temporary values retain their actual semantic owners. Effective
whitespace relationships, document-origin identity, lexical COW isolation and
generation lifetime obey the existing accepted decisions. Global initialization
does not acquire new resource authority or reuse source-specific caches against
another effective document.

Preserve the originating diagnostic and source location. Dependency context is
bounded secondary information, not a replacement for the primary failure.
Repeated reads do not replay constructor messages or grow failure context.
Production circularity diagnostics must use the applicable engine/standards
contract; the reference's `FXAR0030` is not a selected production code.

## Consequences

Dynamic dependencies no longer require a second instruction interpreter or
exhaustive static prediction. Existing static plans and cheap completed reads
can coexist with the selected ownership model. New state transitions and
recursive entry still require careful integration and regression measurement.

Acceptance selects this boundary, not completed support. Keep unsupported
guards until each value kind, consumer and module path passes its integration
gates. No public global-value type, host callback, ABI, wire representation,
parallel global evaluation, cross-invocation cache, lazy-error policy or general
package support is selected. No production performance improvement is promised.

## Alternatives considered

- Keep current bounded specializations: lowest migration cost, but leaves
  general template-dispatched global constructors unsupported.
- Expand static transitive admission as the sole mechanism: useful for the
  proven straight-line subset; broader dynamic analysis adds complexity and
  must distinguish conservative refusal from actual semantic circularity.
- Ship the reference driver unchanged: rejected as the physical design because
  its ordinary-global measurements show overhead and its hooks are test seams.
- Separate eager and demand evaluators: rejected because dependency, value,
  failure and message behavior would have competing owners.

## Validation

Before production admission:

1. Checkpoint ADR-0004 decomposition around initialization and execution owners.
   Do not extract a broad mutable invocation context merely to satisfy borrowing.
2. Preserve existing initializer results, supplied-parameter precedence and
   observable behavior. Test eager initialization through the selected owner,
   recursive named/matched dispatch, modes, conditional reads and cycles.
3. Pin fresh global focus, no caller-frame leakage, resolved import winners,
   effective-view identity, generation lifetime and concurrent isolation.
4. Exercise exact/one-less work and retention limits, cancellation, failure
   cleanup, bounded dependency provenance and repeated-read single evaluation.
5. Admit unchanged `variable40` first, then classify related cases separately.
   Replay both full OASIS profiles and applicable pinned modern tests without
   modifying expectations or converting reference passes into production credit.
6. Compare ordinary no-global, single/repeated/all-used global workloads and
   constructor-heavy workloads against the controls in AR-0030. Measure host
   effects before any host-performance claim; do not weaken control charges to
   make the reference overhead disappear.
7. Run full repository and official WASM validation. Retain the safe reference
   and existing complete-clone/effective-view differential oracles.
8. Pin static-analysis refusal versus actual resolver circularity, and
   differential parity/fallback for each specialized initializer's eligible
   subset. Preserve unsupported classification when no shared fallback is yet
   admitted.
