# OASIS XSLT 1.0 ordered position/attribute match patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the bounded matcher compose one exact named-sibling position and one exact
attribute-value predicate without erasing XPath predicate order?

## Implemented slice

The compiler retains `name[N][@a='v']`,
`name[position()=N and @a='v']`, and `name[@a='v'][N]` in one private typed
pattern. The first two count all same-name siblings before applying the
attribute condition. The last counts only same-name siblings that already pass
the attribute predicate. Source and temporary-tree execution share those
semantics and charge every sibling and attribute visit.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,667 | 1,670 | +3 |
| Executed successfully | 1,496 | 1,499 | +3 |
| Expected-result XML matches | 1,365 | 1,368 | +3 |
| XML comparison mismatches | 104 | 104 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 21 | 18 | -3 |

`match_match17`, `match_match18`, and `match_match19` now match their unchanged
archival expected XML. The strict standard-operation lower bound is now
`1,368 / 2,742 = 49.89%`; the conservative all-catalog ratio is
`1,368 / 3,173 = 43.11%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche admits one positive integer position and one unqualified exact
attribute-value test only. It does not admit relational positions, `last()`,
arithmetic, variables, multiple attribute predicates, qualified names, or
general boolean/predicate trees.

## Verification

- Compiler coverage proves all three lexical forms retain their differing
  filter order and path default priority.
- Runtime coverage distinguishes the two orders over both source and temporary
  trees.
- The complete 3,173-case local measurement produced the counters above.
