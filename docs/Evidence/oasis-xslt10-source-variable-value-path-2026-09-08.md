# OASIS XSLT 1.0 Source-Variable Value Path

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:value-of` navigate a relative location path from an invocation-local
source-node variable while preserving the existing source document, path,
conversion, and work-accounting owners?

## Changes

- Under XSLT 1.0 static context, the value compiler can retain an unprefixed
  variable root followed by the existing typed relative location path.
- Runtime evaluation starts the shared controlled path evaluator from every
  source node bound to that variable, then normalizes the combined result in
  document order and removes duplicates.
- `xsl:value-of` applies the existing XSLT 1.0 first-node string conversion to
  the normalized result. An empty result contributes no text.
- Namespace-defaulting for child name tests uses the same static-context rule
  as ordinary value paths. Attribute and other admitted path steps retain their
  existing semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,470 | 1,471 | +1 |
| Executed successfully | 1,308 | 1,309 | +1 |
| Expected-result XML matches | 1,194 | 1,195 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 162 | 162 | 0 |

The unchanged `Lotus/select_select77#1` case now navigates
`$hotelnode/location/@country`; the missing attribute correctly produces no
text. The strict standard-operation lower bound becomes
`1,195 / 2,742 = 43.58%`; the conservative all-catalog ratio becomes
`1,195 / 3,173 = 37.66%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche is limited to source-node sequences already owned by the current
invocation or its compiled globals. It does not add XPath 1.0 result-tree
fragment conversion, extension node-set conversion, variable-rooted descendant
abbreviation, predicate expansion beyond the shared path grammar, or modern
compatibility coercion. A variable bound to an atomic or temporary-tree value
does not masquerade as a source-node path.

## Verification

- A focused runtime test navigates two source nodes, normalizes their selected
  attributes, and applies first-node string conversion.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
