# XSLT30 imported character maps -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 cases `output-0204` and
`output-0207` and exactly matches both file-backed XML serializations.

`output-0204` proves that a principal output declaration can reference a
character map declared only in a sealed imported stylesheet. `output-0207`
proves that a principal declaration with the same expanded name replaces the
lower-precedence imported declaration, producing `$` to `*` rather than the
imported `$` to `€` mapping.

## Compilation ownership

Character-map declarations and unresolved output-map names now survive private
single-document compilation. Module composition merges declarations using the
existing import-precedence order, after which one finalization step resolves
the package's immutable serializer mapping. Execution still receives only
compiled, memory-resident state and performs no resource acquisition.

The output corpus harness admits each catalog-declared secondary stylesheet
under the RFC 3986 identity obtained from the principal stylesheet base. The
engine therefore exercises the ordinary sealed snapshot resolver rather than a
test-only filesystem shortcut.

## Boundary conservation

No cross-generation or global character-map cache was introduced. Declaration
and unresolved-reference retention is owned by one compiled stylesheet package
and included in its known-capacity estimate. Imported output declarations,
arbitrary composition chains, cycles, named outputs, and result documents
remain outside this tranche.

## Denominator movement

The complete output denominator moves from 102 to 104 passes and from 130 to
128 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 300 to 302 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 176 visible default not-run cases.
