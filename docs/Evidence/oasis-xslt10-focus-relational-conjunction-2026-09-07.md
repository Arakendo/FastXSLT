# OASIS XSLT 1.0 Focus-Relational Conjunction

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can instruction conditions compose focus-relative relational comparisons with
short-circuit `and` semantics without introducing a separate XSLT 1.0 boolean
evaluator?

## Change

The typed instruction boolean compiler now admits `position()` and `last()`
relational comparisons against checked static integers, then composes those
atoms through the existing recursive boolean tree. Runtime evaluation uses the
current sequence focus, charges the XPath operation where it executes, and
evaluates the right side of `and` only when the left side is true.

The change is deliberately separate from location-path predicates: both reuse
the same focus concepts, but instruction conditions and path filtering retain
their own typed owners and evaluation boundaries.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,438 | 1,439 | +1 |
| Executed successfully | 1,277 | 1,278 | +1 |
| Expected-result XML matches | 1,162 | 1,163 | +1 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Lotus/position_position41#1`, whose
`position() >= 2 and position() <= 6` condition selects the expected bounded
middle range.

The strict standard-operation lower bound is now
`1,163 / 2,742 = 42.41%`; the deliberately conservative all-catalog ratio is
`1,163 / 3,173 = 36.65%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test exercises both relational atoms and their
  short-circuit conjunction over a seven-node focus.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
