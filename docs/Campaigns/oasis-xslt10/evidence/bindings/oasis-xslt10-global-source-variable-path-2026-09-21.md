# OASIS XSLT 1.0 Global Source-Variable Path

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can a global XSLT 1.0 binding derive source nodes from an earlier global
source-node variable, as in `$authors/last-name`, without retaining invocation
state in the compiled stylesheet?

## Method

- Compile the existing typed variable-rooted relative path into a new global
  binding default.
- Record the referenced variable in the existing global dependency graph so
  forward and backward declarations use the same topological ordering.
- During invocation initialization, evaluate the existing charged location
  path separately from each source-node root, then restore document order and
  remove duplicate node identities.
- Store only the variable name and compiled relative path in the reusable
  stylesheet; source nodes remain invocation-owned global state.
- Add a complete transform with `$authors` followed by
  `$last-names := $authors/last-name`, then recheck unchanged Microsoft
  `BVTs_bvt092`.

## Result

The golden transform produces two derived names in document order and passes
the full source-to-result lifecycle. The unchanged corpus case advances past
its former invalid-variable-reference failure and now stops explicitly at the
next unsupported expression:

```text
book[$composed = ./author/last-name]
```

That next expression needs general XPath 1.0 node-set comparison against a
dynamic variable inside a path predicate. This tranche deliberately does not
approximate that behavior, so the conserved corpus counters remain at 1,906
exact matches out of 3,173.

## Boundaries

- Only XSLT 1.0 global source-node variables and already admitted relative
  location paths are accepted.
- Atomic and temporary-tree variables retain their existing type-specific
  behavior.
- No source node enters compiled state, and no cross-invocation or
  cross-generation sharing is introduced.
- This does not admit general dynamic-variable predicates or node-set
  comparisons.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_source_variable_paths_follow_dependency_order
./scripts/measure-oasis-xslt10.ps1 -TraceCase bvt092
./scripts/verify.ps1
```
