# OASIS XSLT 1.0 leading descendant attribute and position patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can leading descendant name patterns reuse the existing typed path predicate
machinery for one unqualified attribute test or one static sibling position
without admitting general predicate patterns?

## Implemented slice

For a single unqualified named step after leading `//`, the template compiler
now admits either:

- one unqualified attribute presence or literal-value predicate; or
- one positive static position predicate.

The path remains document-rooted and uses the existing charged evaluator and
bounded invocation-owned membership cache. Static positions retain child-axis
predicate semantics: the selected node is at that position among matching
siblings of its parent, not at a global position in the document-wide result.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,642 | 1,645 | +3 |
| Executed successfully | 1,471 | 1,474 | +3 |
| Expected-result XML matches | 1,341 | 1,344 | +3 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 47 | 43 | -4 |

All three newly executed cases match their unchanged archival expected XML.
The strict standard-operation lower bound is now
`1,344 / 2,742 = 49.02%`; the conservative all-catalog ratio is
`1,344 / 3,173 = 42.36%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit child-value predicates, boolean composition,
relational positions, dynamic positions, predicates on multi-step leading
descendant patterns, namespace-qualified names, or a second match backend. A
compiler regression keeps `//foo[bar='1']` unsupported.

## Verification

- Compiler coverage proves the two admitted predicate families retain
  `MatchPattern::Path` and child-value predicates remain unsupported.
- Runtime coverage proves attribute-value matching and per-parent sibling
  positioning against two distinct parents.
- The complete 3,173-case local measurement produced the counters above.
