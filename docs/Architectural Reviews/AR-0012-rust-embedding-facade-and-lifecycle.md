# AR-0012: Rust Embedding Facade and Lifecycle

| Field | Value |
| --- | --- |
| Status | Proposed |
| Opened | 2026-08-26 |
| Last reviewed | 2026-08-26 |
| Scope | Supported host-neutral Rust resource, compilation, invocation, and result boundary |
| Trigger | CR-0001 supplies the first concrete Rust-native consumer workflow |
| Related ADRs | ADR-0002, ADR-0005, ADR-0007 |
| Related evidence | CR-0001; AR-0003, AR-0004, AR-0005, AR-0009, and AR-0010 |

## Architectural question

What is the smallest supported Rust facade that lets embedded consumers admit
bounded resources, compile and reuse stylesheets, invoke transformations with
isolated dynamic state, and receive structured results without exposing private
engine representation or committing every consumer to one cache/executor?

## Trigger and evidence

CR-0001 records Tokimu's future X3D-to-VRML workflow and preference for direct
Rust embedding. FastXSLT already has private executable resource snapshots,
compiled stylesheets, prepared inputs, direct and batch execution, parameters
in narrow slices, diagnostics, budgets, cancellation, semantic inspection, and
in-memory serialization. ASP.NET workbench evidence exercises a similar
lifecycle through native and isolated adapters.

Those experiments establish useful lifecycle verbs and ownership pressure, but
their Rust types are private and deliberately unstable. Tokimu has not yet
provided a pinned redistributable stylesheet fixture, authoritative resource
graph, complete parameter set, representative distributions, or trusted result.
There is therefore insufficient evidence for exact public types, trait shapes,
thread-safety guarantees, cache defaults, or semantic-versioning promises.

## Ownership and constraints

- The host owns acquisition, ambient authority, generations, publication,
  deployment, domain validation, and application cache policy.
- FastXSLT owns standards semantics, qualified lookup within admitted authority,
  immutable compiled state, invocation isolation, diagnostics, and limits.
- ADR-0002 requires memory-resident core execution after bounded import and
  prohibits hidden file reopening, temporary artifacts, and disk caches.
- ADR-0005 keeps transforms independent and leaves dependent workflow stages
  and result promotion to the host.
- AR-0009 prevents prepared state from acquiring invocation state, global cache
  authority, or accidental content-hash identity semantics.
- AR-0004 and AR-0005 require structured outcomes and bounded semantic
  inspection without exposing AST, IR, arena, optimizer, or cache layout.
- Rust embedding must use the same semantic reference path as native, isolated,
  and future adapters; it is not another backend.

## Alternatives

### A. Small owned lifecycle facade

Expose concrete opaque Rust types for a bounded resource builder/snapshot,
compiled stylesheet, optional prepared input, invocation request/policy, and
owned result/outcome. This closely follows proven lifecycle evidence and keeps
implementation types private, but ownership, cloning, concurrency, and version
evolution must be designed carefully.

### B. Resolver-, source-, and sink-oriented traits first

Expose host-implemented traits for resource lookup and result transfer. This
can reduce copies and integrate diverse stores, but callbacks complicate
authority, reentrancy, lifetimes, cancellation, panic behavior, and deterministic
snapshot semantics before a second implementation proves the abstraction.

### C. Single convenience transform as the initial public API

Expose stylesheet bytes plus source bytes and return result bytes. This is easy
to learn but obscures compile-once reuse, resource graphs, preparation,
authority, and phase diagnostics. It may exist only as a batch-of-one adapter
over the same lifecycle, not as separate semantics.

### D. Publish current private engine types

This would produce an API quickly but stabilize experiments, arena/storage
choices, and cache/runtime details without consumer evidence. Reject as a
facade strategy.

## Findings and uncertainties

- Resource admission, sealed authority, compile once, invocation-local state,
  structured outcomes, and in-memory results are common requirements across
  Rust and ASP.NET consumers.
- A Rust consumer should not pay an interop tax or depend on the workbench ABI.
- Prepared-input visibility, borrowing versus ownership, clone/thread-safety,
  parameter value model, base-URI representation, resolver shape, result sink,
  and compatibility policy remain unresolved.
- The Web3D workload provides substantial future pressure but cannot select the
  facade until its authoritative invocation and legal fixture treatment exist.

## Disposition

Remain **Proposed**. Preserve the proven lifecycle internally and collect
consumer-shaped evidence before proposing a public contract or ADR. CR-0001 is
pressure to design the seam, not permission to publish private experimental
types.

## Required follow-up

- [ ] Reproduce and document CR-0001's authoritative Web3D invocation, pinned
  stylesheet revision/license, resources, parameters, sentinels, and output.
- [ ] Inventory the Web3D stylesheet to its first unsupported frontier without
  copying non-admitted bytes into the MIT crate.
- [ ] Build a narrow Tokimu-shaped adapter experiment over the private safe Rust
  lifecycle and record friction, copies, ownership, and diagnostics.
- [ ] Exercise at least one second Rust consumer shape or explain why the
  Tokimu/standards harness combination is sufficient to avoid overfitting.
- [ ] Decide whether parameters, URI/base identity, inspection, cancellation,
  budgets, and results use owned values, borrowing, traits, or layered adapters.
- [ ] Propose an ADR only after the smallest lifecycle survives those examples.

## Reopening triggers

Move Under Review when an authoritative CR-0001 fixture can compile far enough
to exercise the adapter, another Rust consumer supplies materially different
ownership needs, or a public preview requires a supported Rust API.

## Review history

- 2026-08-26 -- Opened as Proposed from CR-0001 Tokimu/Web3D consumer pressure.
