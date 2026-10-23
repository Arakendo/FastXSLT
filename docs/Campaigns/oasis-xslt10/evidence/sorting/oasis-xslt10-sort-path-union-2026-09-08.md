# OASIS XSLT 1.0 Sort Path Union

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 sort key select from a bounded union of location paths while
preserving node-set normalization and first-node string conversion?

## Changes

- `xsl:sort` compilation now retains a bounded union of supported location
  paths as one typed private sort-key plan.
- Runtime evaluation uses the shared controlled source-path union evaluator,
  restoring document order and removing duplicate node identity before taking
  the first node's string value as the XSLT 1.0 sort key.
- Every alternative retains the ordinary path charge points. Stable sorting,
  numeric conversion, language handling, and case-order behavior are unchanged.
- Prepared-engine retention accounting includes the union alternatives and
  their owned path capacity.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,478 | 1,481 | +3 |
| Executed successfully | 1,315 | 1,318 | +3 |
| Expected-result XML matches | 1,201 | 1,202 | +1 |
| XML comparison mismatches | 79 | 81 | +2 |
| Execution failures | 163 | 163 | 0 |

The unchanged `Lotus/sort_sort26#1` case now matches its expected result. Two
additional cases also cross the initialization and execution frontier but
remain visible comparison mismatches; this record does not count those as
compatibility passes. The strict standard-operation lower bound becomes
`1,202 / 2,742 = 43.84%`; the conservative all-catalog ratio becomes
`1,202 / 3,173 = 37.88%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche admits only unions whose operands are already supported source
location paths. It does not admit arbitrary union operands, atomic sequences,
temporary-tree operands, cross-document ordering, or new collation behavior.
Input-order sort stability and the complete ordinary path evaluator remain the
semantic reference behavior.

## Verification

- A focused runtime test proves ascending sorting by the first document-order
  node contributed by either union alternative.
- The unchanged `Lotus/sort_sort26#1` corpus case matches exactly.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
