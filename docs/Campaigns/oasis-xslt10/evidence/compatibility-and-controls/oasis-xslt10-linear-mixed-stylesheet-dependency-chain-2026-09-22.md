# OASIS XSLT 1.0 Linear Mixed Stylesheet Dependency Chain

Date: 2026-09-22  
Status: Local implementation and compatibility evidence

## Question

Can the existing include/import program-composition machinery compile a nested
stylesheet dependency chain when each module has exactly one dependency, without
admitting a general branching graph or changing resource authority?

## Method

- Keep acquisition separate: every module is already present in the sealed
  resource snapshot before compilation.
- Recursively compile a dependency graph only while every visited module has at
  most one child.
- Compile the leaf through the ordinary single-stylesheet compiler, then compose
  each parent edge through the existing include or import program-composition
  path according to the edge kind.
- Preserve the existing explicit `FXST1027` boundary if a nested module branches.
- Add a focused sealed-resource regression for a principal stylesheet that
  includes a module which imports a leaf module, then rerun the unchanged
  3,173-case OASIS catalog.

## Result

The focused production-path regression produces `<out>IMPORTED</out>` through
the mixed include/import chain. Six additional OASIS cases initialize and
execute: five compare exactly and one reaches a visible expected-result mismatch.

`Microsoft/BVTs_bvt039#1` also progresses past the former nested-module
composition boundary. It now reports the later explicit `XTSE0810` conflict
between namespace aliases at the same import precedence; that case is not
counted as a pass and its XSLT 1.0 compatibility/recovery semantics remain open.

The complete sweep initializes 2,164 cases, reports 971 initialization failures,
executes 2,108 successfully, and reports 56 execution failures. Exact
XML-semantic matches reach 1,962, with 64 mismatches. The strict compatibility
lower bound is `1,962 / 3,173 = 61.83%`.

## Boundaries

- This is a linear dependency-chain implementation, not a general nested or
  branching stylesheet-graph implementation.
- It reuses established include/import composition and precedence behavior; it
  does not introduce a second compiler path.
- Resource discovery, identity, authority, module/depth/byte limits, cycle
  detection, and snapshot sealing are unchanged.
- The engine performs no filesystem or network I/O during compilation or
  execution.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features linear_include_then_import_chain_reuses_module_composition
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/BVTs_bvt039#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
