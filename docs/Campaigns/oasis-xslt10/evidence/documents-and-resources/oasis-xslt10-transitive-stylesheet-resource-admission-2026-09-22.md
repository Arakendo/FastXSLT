# OASIS XSLT 1.0 Transitive Stylesheet Resource Admission

Date: 2026-09-22  
Status: Local harness evidence

## Question

Can the local OASIS acquisition adapter seal relative stylesheet dependencies
referenced by an already admitted module when the catalog lists only the first
module in that dependency chain?

## Method

- Keep the engine's snapshot-only compilation contract unchanged.
- Starting from the catalog-authorized principal and supplemental stylesheets,
  inspect only top-level XSLT `xsl:include` and `xsl:import` declarations.
- Acquire only relative references whose canonical physical path remains below
  the current case directory; do not map absolute URI references or grant
  filesystem/network authority to the engine.
- Bound discovery to 64 modules and 8 MiB, normalize each logical identity
  through the ordinary URI resolver, and seal all discovered bytes before
  compilation begins.
- Add a focused top-level/XSLT-namespace discovery regression and rerun the
  unchanged 3,173-case catalog.

## Result

The missing-resource frontier falls from 40 to 21 cases. Nineteen cases now
reach their actual engine or fixture disposition: fifteen reach later explicit
parser, dependency-depth, module-composition, or limit boundaries; three compare
exactly; and `Microsoft/Output__78180#1` reaches a visible whitespace mismatch.

For example, `Microsoft/BVTs_bvt039#1` now seals the relative import referenced
by its admitted include and reaches the existing `FXST1027` nested-module
composition boundary instead of incorrectly reporting missing authority.

The complete sweep initializes 2,158 cases, reports 977 initialization
failures, executes 2,102 successfully, and reports 56 execution failures.
Exact XML-semantic matches reach 1,957, with 63 mismatches. The strict
compatibility lower bound is `1,957 / 3,173 = 61.68%`.

## Boundaries

- Discovery belongs only to this explicitly authorized local corpus importer;
  compilation and execution remain memory-resident and perform no ambient I/O.
- Absolute `file:`, HTTP, and HTTPS references are not mapped to local files.
- Canonical paths must remain beneath the case directory, preventing relative
  traversal from widening acquisition authority.
- Discovery does not relax the engine's independent dependency depth, module,
  byte, cycle, or compilation-shape limits.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features oasis_stylesheet_dependency_discovery_reads_only_top_level_xslt_references
./scripts/measure-oasis-xslt10.ps1 -TraceCase Microsoft/BVTs_bvt039#1
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier missing-resource/FXRS0002/FXRS0002
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
