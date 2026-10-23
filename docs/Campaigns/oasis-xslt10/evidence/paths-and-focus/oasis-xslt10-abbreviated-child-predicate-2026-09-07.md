# OASIS XSLT 1.0 Abbreviated Child Predicate

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an unprefixed QName used as a predicate, such as `[near-south]`, reuse the
existing typed `child::near-south` effective-boolean-value operation without
admitting general predicate expressions?

## Change

The private axis-predicate parser now recognizes one ASCII NCName as the XPath
abbreviation for a named child-axis test. Evaluation uses the existing bounded,
charged named-child predicate owner. Explicit `child::name` behavior is
unchanged.

Nested predicates, qualified child names, boolean composition, and arbitrary
predicate expressions remain unsupported by this slice.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,393 | 1,395 | +2 |
| Executed successfully | 1,232 | 1,234 | +2 |
| Expected-result XML matches | 1,118 | 1,120 | +2 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/axes_axes63#1`, which selects
`self::*[near-south]`, and `Microsoft/Sorting__77985#1`, which uses named-child
predicates in typed sort paths. Other newly recognized expressions expose later
stylesheet, resource, or construction boundaries and remain uncredited.

The strict standard-operation lower bound is now
`1,120 / 2,742 = 40.85%`; the deliberately conservative all-catalog ratio is
`1,120 / 3,173 = 35.30%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused coverage proves `self::*[near-south]` retains an item containing the
  named child and rejects a sibling containing a differently named child.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.
