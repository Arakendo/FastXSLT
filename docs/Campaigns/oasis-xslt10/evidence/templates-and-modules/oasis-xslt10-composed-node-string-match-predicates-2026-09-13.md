# OASIS XSLT 1.0 composed node-string match predicates -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing exact context-node string match predicate safely cover the
bounded XSLT 1.0 forms `not(.='x')`, `.!='x'`, and `.='x' or .='y'`?

## Implemented slice

The private node-string match representation now carries a typed predicate:
exact equality, exact inequality, or equality against either of two literals.
The compiler admits only those lexical forms over the already-supported node
tests. `not()` around exact equality and direct inequality lower to the same
typed inequality operation.

Source and temporary result-tree selection evaluate the same typed predicate.
The existing XPath-operation and temporary string-value traversal charges are
preserved.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,654 | 1,657 | +3 |
| Executed successfully | 1,483 | 1,486 | +3 |
| Expected-result XML matches | 1,353 | 1,356 | +3 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 34 | 31 | -3 |

All three newly executed cases match their unchanged archival expected XML.
The strict standard-operation lower bound is now
`1,356 / 2,742 = 49.45%`; the conservative all-catalog ratio is
`1,356 / 3,173 = 42.74%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary `not()` operands, longer boolean trees,
numeric or relational comparisons, functions, variables, multiple predicates,
or qualified element names. It does not change template ordering, resource
authority, prepared-input ownership, or invocation state.

## Verification

- Compiler coverage proves the three admitted lexical forms lower to the typed
  predicate operations and retain path-pattern default priority.
- Runtime coverage proves equality-or and both inequality spellings select the
  expected nodes alongside source/temporary exact-equality coverage.
- The complete 3,173-case local measurement produced the counters above.
