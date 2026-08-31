# ADR-0014: Invocation-Owned Copy-on-Write Atomic Frames

- Status: Accepted
- Date: 2026-08-31
- Related reviews: AR-0013
- Related ADRs: ADR-0002, ADR-0003, ADR-0004, ADR-0009, ADR-0013
- Related evidence: `docs/Evidence/named-template-global-frame-cloning-2026-08-31.md` and adversarial review Finding 12
- Supersedes: None

## Context

Each runtime variable frame previously owned a complete clone of the
invocation's atomic-global `BTreeMap`. An eight-call named-template chain with
256 globals copied 2,048 entries. Against a same-global depth-zero control, the
chain added 8,824 allocation requests, 432,576 allocator-requested bytes, about
419 KiB peak live requested memory, and 598.3 us local median latency.

A safe prototype instead shares the immutable atomic map between frames through
`Arc` and uses `Arc::make_mut` when a frame introduces or replaces a local
binding. A test-only complete-clone path remains the semantic and measurement
oracle. On the same 256-global, eight-call chain, the shared path eliminated all
eight observed complete-map clones, reduced allocation requests from 5,530 to
4,978, requested bytes from 276,889 to 250,053, peak live requested bytes from
273,049 to 246,213, and local median latency from 406.8 us to 336.7 us.

## Decision

Use a private invocation-owned copy-on-write map for atomic variable frames.

The representation must:

- share only within one invocation and never across prepared inputs, resource
  snapshots, workers, engine generations, or concurrent invocations;
- make every local parameter, local variable, or other frame mutation private
  through safe `Arc::make_mut` copy-on-write behavior;
- preserve lexical shadowing, global defaults and host overrides, template
  parameter binding, diagnostics, recursion limits, and deterministic cleanup;
- retain the complete-clone implementation as a test-only differential and
  measurement oracle;
- keep the representation private and expose no map, `Arc`, frame, or lookup
  detail through the facade or host boundaries; and
- use no unsafe code.

The shared map is an execution representation, not compiled or prepared state.
Its values and lifetime remain owned by the invocation that materialized the
globals.

## Non-decisions

This ADR does not admit:

- cross-invocation or cross-generation sharing;
- a persistent parent-chain or general environment representation;
- mutation of compiled stylesheet or prepared-XDM state;
- interning, arenas, custom allocators, unsafe lookup, or a different public
  parameter type;
- a claim that every frame creation is allocation-free; or
- any prepared-XDM representation change.

Prepared-XDM field anatomy remains AR-0013 evidence. Any resulting interning or
layout proposal requires its own comparison and decision.

## Consequences

Read-only calls share one map with constant-size reference-count traffic instead
of copying every global binding. A frame that adds a binding pays a safe map
clone only when its shared backing requires separation, preserving independent
lexical state. This favors the common compile-once/transform-many path without
adding a parent-chain lookup to every variable access.

The representation uses non-atomic `BTreeMap` mutation behind `Arc::make_mut`;
it does not authorize concurrent mutation of one invocation. Concurrent
invocations continue to own independent runtime maps.

## Validation

- Differentially execute the same named-template chain through shared and
  complete-clone frames and compare the complete semantic result.
- Exercise global defaults, host parameter overrides, local bindings, template
  parameters, shadowing, recursion failure, and temporary-tree variables
  through the normal suite.
- Measure allocation count, requested/peak bytes, and latency over the
  0/16/64/256-global matrix.
- Run concurrent invocations over shared compiled/prepared state and verify no
  runtime frame crosses invocation ownership.
- Run unchanged golden, QT3, XSLT30, workbench, and workspace verification
  gates.

Revisit if representative workloads reverse the benefit, copy-on-write
mutation becomes the dominant path, a parent/overlay frame is proposed, or any
sharing beyond one invocation is required.
