# OASIS XSLT 1.0 Static `number()` Position Predicate

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an ordinary location-path predicate reuse the existing checked static
`number()` conversion when its result is an exact positive integer, without
adding a compatibility-only numeric evaluator?

## Change

The typed position-predicate parser now asks the shared constant-numeric owner
to fold an admitted source-free `number()` conversion before applying the
existing checked integer-position rules. Non-integral, non-finite, dynamic, or
otherwise unsupported numeric predicates remain outside this narrow slice.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,437 | 1,438 | +1 |
| Executed successfully | 1,276 | 1,277 | +1 |
| Expected-result XML matches | 1,161 | 1,162 | +1 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Lotus/position_position67#1`, whose
`a[number('3')]` selection produces the expected third node.

The strict standard-operation lower bound is now
`1,162 / 2,742 = 42.38%`; the deliberately conservative all-catalog ratio is
`1,162 / 3,173 = 36.62%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- The focused position-predicate test includes the static `number('3')` form.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
