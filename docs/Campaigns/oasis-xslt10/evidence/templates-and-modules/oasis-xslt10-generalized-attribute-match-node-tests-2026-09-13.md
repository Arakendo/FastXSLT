# OASIS XSLT 1.0 generalized attribute match node tests -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can wildcard-element exact attribute-value predicates and `node()` attribute-
presence predicates reuse the bounded attribute matcher without admitting a
general predicate evaluator?

## Implemented slice

The compiler now admits `*[@name='literal']` as a private typed wildcard-element
attribute-value pattern. It also normalizes `node()[@name]` to the existing
wildcard-element attribute-presence pattern: among the principal node kinds
matched by `node()`, only elements can have an attribute axis, so the two forms
have the same selected-node set.

Both operations scan only the candidate element's attributes and charge every
inspected attribute as an XPath node visit. Source and temporary result-tree
dispatch implement the same semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,660 | 1,662 | +2 |
| Executed successfully | 1,489 | 1,491 | +2 |
| Expected-result XML matches | 1,359 | 1,361 | +2 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 28 | 26 | -2 |

Both newly executed cases match their unchanged archival expected XML. The
strict standard-operation lower bound is now
`1,361 / 2,742 = 49.64%`; the conservative all-catalog ratio is
`1,361 / 3,173 = 42.89%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit namespace-qualified attributes, numeric or
relational comparison, multiple wildcard predicates, arbitrary node-test
predicates, variables, functions, or general boolean composition. It does not
change template ordering, resource authority, prepared-input ownership, or
invocation state.

## Verification

- Compiler coverage proves both generalized forms lower to the intended typed
  patterns with path-pattern default priority.
- Runtime coverage proves matching and nonmatching source and temporary-tree
  candidates.
- The complete 3,173-case local measurement produced the counters above.
