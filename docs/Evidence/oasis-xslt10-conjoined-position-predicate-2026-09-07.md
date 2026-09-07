# OASIS XSLT 1.0 Conjoined Position Predicate

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a typed path predicate compose an already-supported node test with
`position() = N` while retaining the XPath focus in which the conjunction is
evaluated?

## Change

The private axis predicate now retains an optional typed position condition.
For the bounded `node-test and position() = N` form, position and size refer to
the step's named candidate sequence before the node test filters it. Evaluation
short-circuits the left-hand node test before checking position and keeps the
existing charged path evaluator as the sole runtime owner.

This is deliberately not rewritten as `[node-test][N]`: that chained form
numbers the filtered sequence and is observably different when only some
candidates satisfy the node test. Position-first conjunctions, `or`, and general
predicate expressions remain unsupported.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,412 | 1,416 | +4 |
| Executed successfully | 1,251 | 1,255 | +4 |
| Expected-result XML matches | 1,136 | 1,140 | +4 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position03#1`,
`Lotus/position_position32#1`, `Lotus/position_position33#1`, and
`Lotus/position_position34#1`.

The strict standard-operation lower bound is now
`1,140 / 2,742 = 41.58%`; the deliberately conservative all-catalog ratio is
`1,140 / 3,173 = 35.93%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused sparse-attribute source proves that `*[@test and position()=3]`
  selects the third original candidate and that position two remains empty;
  treating the expression as `*[@test][3]` would produce a different result.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
