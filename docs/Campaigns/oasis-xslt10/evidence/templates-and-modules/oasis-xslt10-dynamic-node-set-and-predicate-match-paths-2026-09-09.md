# OASIS XSLT 1.0 dynamic node-set and predicate match paths -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can dynamic node-set equality and nested positional child predicates share one
typed path representation across selection and template matching, rather than
creating a predicate-specific template matcher?

## Decision in the experiment

The private predicate tree now retains:

- existential string-value equality between `following-sibling::*` and
  `descendant::*`;
- a named child at a static position compared with a string literal; and
- a nested form whose optional positioned outer child contains a positioned
  inner child compared with a string literal.

Successfully parsed, unnamespaced, single-step positional-child string-
comparison match patterns now compile as the existing `MatchPattern::Path`.
Template selection therefore uses the same charged path evaluator as ordinary
selection, including the existing default priority and node-membership check.
Other predicate kinds and namespaced/default-XPath-namespace patterns remain
outside this seam.

Sibling, descendant, outer-child, and inner-child visits are charged. Dynamic
string-value pair comparisons are charged as XPath operations.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,622 | 1,624 | +2 |
| Executed successfully | 1,451 | 1,453 | +2 |
| Expected-result XML matches | 1,331 | 1,332 | +1 |
| XML comparison mismatches | 93 | 94 | +1 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate11#1` case proves dynamic node-set
equality and supplies the one new exact match. Predicate match admission is
deliberately restricted to the new positional-child string-comparison kinds;
a broader successfully-parsed-path probe violated an existing unsupported-
attribute-comparison boundary and was removed before final measurement.

`Lotus/predicate_predicate38#1` now executes with the expected selected
templates and values, but remains a comparison mismatch because FastXSLT's
`indent="yes"` serialization inserts two spaces before child elements where
the archival expected XML contains only a line break. This case is not counted
as a pass.

The strict standard-operation lower bound is now
`1,332 / 2,742 = 48.58%`; the conservative all-catalog ratio is
`1,332 / 3,173 = 41.98%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit general node-set comparison, arbitrary nested
predicates, qualified names in these predicate leaves, multi-step predicate
match patterns, or a new match backend. It does not hide the newly exposed
indentation mismatch or change resource authority, cancellation, node
identity, or corpus data.

## Verification

- A focused path test covers sibling/descendant equality, positioned child
  comparison, optional outer positioning, and nested inner positioning.
- A compiler test proves admitted single-step predicate patterns lower to the
  existing path match representation.
- An existing compiler regression proves general attribute-comparison match
  patterns remain unsupported.
- The complete 3,173-case local measurement produced the counters above and a
  targeted trace records the remaining `predicate38` output difference.
