# OASIS XSLT 1.0 following-sibling numeric predicate -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a path predicate compare the string values of a
`following-sibling::*` node-set with a static integer under XSLT 1.0
conversion rules, without admitting general numeric predicate expressions?

## Decision in the experiment

The private path-predicate tree now has one bounded
following-sibling-element/integer equality leaf. Evaluation visits following
siblings in axis order, charges every inspected node through the existing XPath
node-visit domain, converts element string values through the shared XSLT 1.0
numeric lexical path, and succeeds existentially when one converted value equals
the compiled integer. Both operand orders compile to the same typed operation.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,605 | 1,607 | +2 |
| Executed successfully | 1,434 | 1,436 | +2 |
| Expected-result XML matches | 1,314 | 1,316 | +2 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate05#1` and
`Lotus/predicate_predicate26#1` cases now match exactly. The strict
standard-operation lower bound is now `1,316 / 2,742 = 47.99%`; the
conservative all-catalog ratio is `1,316 / 3,173 = 41.47%`. These remain local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit decimal operands, relational operators, arbitrary
axes, node-set-to-node-set numeric comparison, or a general numeric predicate
grammar. It does not change result order, node identity, diagnostics,
cancellation, resource authority, or corpus data.

## Verification

- A focused path test proves existential selection of the two nodes preceding a
  sibling whose string value converts to `3`.
- The complete 3,173-case local measurement produced the counters above.
- Strict workspace Clippy passes before the full verification gate.
