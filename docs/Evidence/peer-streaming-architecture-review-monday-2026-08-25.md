# Peer Streaming Architecture Review: Monday

| Field       | Value                                                              |
| ----------- | ------------------------------------------------------------------ |
| Received    | 2026-08-25                                                         |
| Reviewer    | Monday                                                             |
| Scope       | Planning for future streaming without committing to implementation |
| Disposition | Accepted as incubation evidence for AR-0007                        |

## Summary

The reviewer recommended planning for future streaming by preventing a
fully-materialized random-access tree from becoming an unnecessary assumption
across every engine layer. The recommendation explicitly rejected detailed
streaming design or implementation at this stage.

The central principle is that FastXSLT semantics may require capabilities a
forward-only source cannot provide. Engine layers should depend on semantic
capabilities rather than treating one physical tree representation as permanent.

## Useful capability model

The review offered this illustrative progression:

```text
forward-only event access
    -> bounded subtree materialization
    -> retained ancestor and context access
    -> arbitrary document navigation
    -> full-document materialization
```

This is a reasoning aid, not an accepted Rust trait hierarchy or formal
standards classification. Child-oriented traversal may need less retention than
reverse axes, repeated global access, keys, or other operations whose semantics
can require wider navigation.

## Recommendations accepted for incubation

- Keep XDM semantics independent of one physical storage representation.
- Avoid passing a concrete document arena through layers whose actual need is a
  narrower semantic operation, while also avoiding premature abstraction.
- Preserve a compilation seam where expressions and templates could eventually
  carry required navigation, retention, or evaluation capabilities.
- Keep semantic compilation distinct from a current tree execution strategy so
  a later strategy does not require a second semantic compiler.
- Treat any selective materialization as explicit bounded memory, consistent
  with ADR-0002, never as hidden disk spill.
- Let a sealed resource snapshot own immutable bytes and identity without
  requiring every batch source to be pre-parsed and retained as a tree.
- Benchmark abstraction cost before introducing generalized interfaces.

## Explicitly not accepted

This review does not authorize or claim:

- a streaming executor, scheduler, event algebra, or hybrid runtime;
- XSLT streaming conformance or formal streamability analysis;
- multiple execution backends without semantic parity evidence;
- a generic node-provider trait family before another implementation or a
  measured boundary needs it; or
- unbounded buffering, silent materialization, spill files, or ambient resource
  resolution.

## Result

AR-0007 incubates architectural optionality. The first evaluator may use a
fully materialized tree. The evidence obligation is to keep tree-specific
assumptions owned and visible, not to pay speculative abstraction or streaming
complexity before representative standards cases and workloads exist.
