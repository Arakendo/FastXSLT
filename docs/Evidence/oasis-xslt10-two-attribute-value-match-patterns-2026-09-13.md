# OASIS XSLT 1.0 two-attribute-value match patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can chained and conjunctive exact attribute-value predicates normalize to one
bounded template match operation?

## Implemented slice

The compiler recognizes the equivalent unqualified forms
`name[@a='x'][@b='y']` and `name[@a='x' and @b='y']`. Both lower to one private
typed pattern carrying the element name and two attribute/value requirements.

Selection scans the candidate's attributes once, charges every inspected
attribute as an XPath node visit, and requires both values. Source and temporary
result-tree dispatch implement the same operation.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,657 | 1,660 | +3 |
| Executed successfully | 1,486 | 1,489 | +3 |
| Expected-result XML matches | 1,356 | 1,359 | +3 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 31 | 28 | -3 |

All three newly executed cases match their unchanged archival expected XML.
The strict standard-operation lower bound is now
`1,359 / 2,742 = 49.56%`; the conservative all-catalog ratio is
`1,359 / 3,173 = 42.83%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit more than two predicates, inequality, relational
or numeric comparison, namespace-qualified names, variables, functions, or
arbitrary boolean composition. It does not change template ordering, resource
authority, prepared-input ownership, or invocation state.

## Verification

- Compiler coverage proves the chained and conjunctive spellings normalize to
  the same typed pattern.
- Runtime coverage proves one matching and one nonmatching candidate for both
  source and temporary trees.
- The complete 3,173-case local measurement produced the counters above.
