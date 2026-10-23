# OASIS XSLT 1.0 lexical context comparison -- 2026-09-15

Date: 2026-09-15  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing typed lexical-name and context-string plans preserve XSLT 1.0
equality and inequality in both location-path predicates and instruction
conditions without introducing a general expression evaluator?

## Implemented slice

The private location-path predicate plan now represents `name()` and `name(.)`
comparison with a string literal for both `=` and `!=`. Evaluation uses the
context node's lexical QName, including its source prefix, and charges the
comparison as XPath work. Either operand order is accepted.

Instruction-local boolean compilation now recognizes the same lexical-name
comparisons. It also recognizes context string-value equality and inequality
before the broader path-comparison parser; inequality reuses the existing typed
equality plan through boolean negation. The semantic runtime remains shared and
no new public expression or representation type is exposed.

QName-to-QName comparison, dynamic operands, namespace-axis nodes, and general
XPath comparison conversion remain outside this bounded slice.

## Corpus result

Unchanged Lotus `sort_sort37` becomes exact when
`./*[name(.) = 'never']` correctly selects an empty sort key for every item.
Unchanged Lotus `node_node15` becomes exact after its `name(.)!=''` and `.!=''`
instruction conditions compile through the same typed comparison semantics.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,758 | 1,760 | +2 |
| Initialization failures | 1,377 | 1,375 | -2 |
| Executed successfully | 1,574 | 1,576 | +2 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,437 | 1,439 | +2 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,439 / 2,742 = 52.48%` for standard-operation cases and
`1,439 / 3,173 = 45.35%` for the complete catalog.

## Verification

Focused path evaluation covers equality, reversed inequality, and a prefixed
lexical QName. The existing context-name lifecycle test now also covers
instruction-local lexical-name and context-string inequality. The complete
local sweep establishes the two exact additions, unchanged mismatch and
execution-failure counts, and absence of panics.
