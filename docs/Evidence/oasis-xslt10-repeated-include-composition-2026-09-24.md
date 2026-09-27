# OASIS XSLT 1.0 Repeated Include Composition

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related reviews: AR-0014, AR-0019

## Pressure

The sealed stylesheet-resource compiler admitted only one or two sibling
dependencies even when the host supplied a larger bounded module budget.
Microsoft `BVTs_bvt077` requires five ordered `xsl:include` declarations and
references the same admitted module four times. Treating resource identity as
declaration identity, or merely removing the two-dependency check, would lose
observable declaration order and weaken resource accounting.

## Implemented slice

Same-precedence include-only graphs now compose an ordered vector of private
compiled modules. Every include declaration retains its own occurrence even
when multiple declarations resolve to the same logical resource identity. The
compiler merges those occurrences at the declaration locations in principal
source order, preserving template conflict order and existing cross-module
validation.

Acquisition remains sealed and memory-resident. Each occurrence is still
resolved, parsed, and charged against the host-supplied depth, module,
aggregate-byte, and resolution-attempt ceilings. This tranche does not add
deduplication, cross-occurrence caching, live resolution, general mixed
include/import assembly, or broader import-precedence semantics.

## Verification

A focused regression admits two module identities through three declarations
in the order `first`, `second`, `first`. The compiled template order retains all
three occurrences. Re-running the same graph with a three-module ceiling fails
with `FXRS0006`, proving that the repeated identity does not evade the
occurrence budget.

The unchanged OASIS sweep moves from 2,302 to 2,306 initialized cases and from
2,250 to 2,254 successful executions. Two include-graph cases become exact,
raising exact expected-result matches from 2,089 to **2,091 / 3,173 (65.90%)**.
`Microsoft/BVTs_bvt077#1` now compiles and executes, but remains a visible XML
comparison mismatch because FastXSLT omits literal stylesheet whitespace and
serializes the old-namespace empty element with explicit start/end tags. That
serializer/result-construction difference is not counted as include success.

The full sweep has no initialization or execution panics. Remaining nested
mixed dependency shapes continue to report their explicit later frontiers.

## Reproduction

```powershell
cargo test -p fastxslt --all-features repeated_includes_preserve_every_bounded_declaration_occurrence_in_source_order
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'BVTs_bvt077#1'
./scripts/verify.ps1
```
