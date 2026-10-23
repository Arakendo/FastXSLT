# OASIS XSLT 1.0 sequential focus match predicates -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the remaining Xalan `match20` through `match26` patterns preserve XSLT
predicate ordering and focus without routing match patterns through a general
XPath evaluator?

## Implemented slice

The compiler retains a bounded typed list of sequential predicates for one
named element test. The admitted predicates cover integer position tests,
`position() mod N`, `position()` relations, `last()`, numeric attribute
relations, and numeric attribute modulo. Runtime matching first constructs the
named sibling sequence and then filters it from left to right, recomputing
position and size for every predicate exactly as required by XPath predicate
semantics.

The scalar predicate evaluator is shared by source and temporary trees. Each
tree owner retains its own charged navigation and attribute access, and the
compiled representation participates in prepared-engine retention accounting.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,671 | 1,678 | +7 |
| Executed successfully | 1,500 | 1,507 | +7 |
| Expected-result XML matches | 1,369 | 1,376 | +7 |
| XML comparison mismatches | 104 | 104 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Initialization failures | 1,464 | 1,457 | -7 |

All seven unchanged `match20` through `match26` cases agree exactly with their
archival expected XML. The strict standard-operation lower bound is now
`1,376 / 2,742 = 50.18%`; the conservative all-catalog ratio is
`1,376 / 3,173 = 43.37%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit general match-pattern expressions, variables,
arbitrary arithmetic, boolean composition, qualified attribute operands, or
predicates over multi-step patterns. It does not infer that the same bounded
grammar is appropriate for ordinary XPath selection.

## Verification

- Parser tests cover every admitted predicate form and reject single/general
  expression shapes.
- A focused runtime test proves left-to-right focus behavior and equivalent
  source/temporary-tree dispatch.
- The complete 3,173-case local measurement produced the counters above.
