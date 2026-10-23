# OASIS XSLT 1.0 context lexical-name predicates -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared typed path-predicate evaluator admit the bounded XSLT 1.0 forms
`starts-with(name(.), literal)` and `string-length(name(.)) = integer` without
confusing lexical QName semantics with expanded or local names?

## Decision in the experiment

The private predicate tree now retains typed leaves for a context node's
lexical QName prefix test and lexical QName length. Evaluation constructs the
lexical name from the prepared document's retained prefix and expanded local
name, so a prefixed `p:fizz` node is not treated as if `name(.)` returned only
`fizz`.

Each context-name operation is charged through the XPath operation domain;
path traversal retains its existing node-visit accounting. The implementation
does not add a general nested-function expression tree.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,608 | 1,610 | +2 |
| Executed successfully | 1,437 | 1,439 | +2 |
| Expected-result XML matches | 1,317 | 1,319 | +2 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate18#1` and
`Lotus/predicate_predicate19#1` cases now match exactly. The strict
standard-operation lower bound is now `1,319 / 2,742 = 48.10%`; the
conservative all-catalog ratio is `1,319 / 3,173 = 41.57%`. These remain local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary `name()` calls, general string
functions, dynamic second operands, relational name-length comparisons, or a
second predicate evaluator. It does not change node identity, result order,
diagnostics, cancellation, resource authority, or corpus data.

## Verification

- A focused path test proves prefix-sensitive `name(.)` behavior using
  unprefixed `foo`/`bar` nodes and a prefixed `p:fizz` node.
- The two unchanged corpus cases execute through the production compiler,
  runtime, and serializer and match their expected XML.
- The complete 3,173-case local measurement produced the counters above.
