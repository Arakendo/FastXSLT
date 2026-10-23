# OASIS XSLT 1.0 Ceiling-Half Focus

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a condition compare `position()` with the rounded-up midpoint of its
current sequence without introducing a general compatibility-only arithmetic
evaluator?

## Change

The existing typed focus operand now represents the exact
`ceiling(last() div 2)` composition. Compilation retains that operation rather
than folding a dynamic size, and runtime evaluation derives the midpoint from
the invocation's current sequence focus using overflow-safe integer arithmetic.
The same operand is available to the existing equality owners for instruction
conditions and ordinary values.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,440 | 1,441 | +1 |
| Executed successfully | 1,279 | 1,280 | +1 |
| Expected-result XML matches | 1,164 | 1,165 | +1 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Lotus/position_position27#1`. Its seven-member
focus identifies position four as the rounded-up midpoint while retaining the
first/last branches around it.

The strict standard-operation lower bound becomes
`1,165 / 2,742 = 42.49%`; the conservative all-catalog ratio becomes
`1,165 / 3,173 = 36.72%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test exercises the midpoint operand over a seven-node
  sequence.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
