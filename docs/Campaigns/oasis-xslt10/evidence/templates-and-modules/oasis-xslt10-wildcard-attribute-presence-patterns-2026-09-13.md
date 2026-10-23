# OASIS XSLT 1.0 wildcard attribute-presence patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `*[@name]` template match patterns reuse the bounded attribute-presence
matcher without admitting general predicates?

## Implemented slice

The template compiler now recognizes an unqualified attribute-presence
predicate on the wildcard element node test. It lowers the form to a private
typed match-pattern variant and scans only the candidate element's attributes.
Each inspected attribute consumes an XPath node-visit charge.

Source and temporary result-tree execution implement the same behavior. The
slice does not introduce a general predicate evaluator or a second template
selection path.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,647 | 1,649 | +2 |
| Executed successfully | 1,476 | 1,478 | +2 |
| Expected-result XML matches | 1,346 | 1,348 | +2 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 41 | 39 | -2 |

Both newly executed cases match their unchanged archival expected XML. The
strict standard-operation lower bound is now
`1,348 / 2,742 = 49.16%`; the conservative all-catalog ratio is
`1,348 / 3,173 = 42.48%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit attribute value comparisons on wildcard elements,
namespace-wildcard attributes, multiple predicates, node-test predicates, or
arbitrary boolean expressions. It does not change template ordering, resource
authority, prepared-input ownership, or invocation state.

## Verification

- Compiler coverage proves `*[@test]` lowers to the dedicated typed form with
  path-pattern default priority.
- Runtime coverage proves source and temporary-tree candidates share the same
  behavior.
- The complete 3,173-case local measurement produced the counters above.
