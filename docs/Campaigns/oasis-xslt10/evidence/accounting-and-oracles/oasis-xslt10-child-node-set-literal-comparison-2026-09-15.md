# OASIS XSLT 1.0 child node-set/literal comparison -- 2026-09-15

Date: 2026-09-15  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can bounded location-path predicates compare a relative child node set with a
string literal using XPath 1.0 general-comparison semantics without weakening
the existing typed path model?

## Implemented slice

The private path-predicate plan now retains a relative unprefixed child path of
at most four steps, a string literal, and equality or inequality. A second
bounded form permits an unprefixed attribute as the final step after at least
one child step. Evaluation selects the node set under invocation work control
and applies the XPath 1.0 existential comparison rule to every selected node's
string value.

Inequality is deliberately not implemented as the negation of equality: for a
multi-node set, `nodes != 'x'` is true when any selected node differs from the
literal. Empty node sets make both comparisons false. Either operand order is
accepted.

Qualified names, non-child axes, attributes before the final step, numeric or
boolean conversion, and dynamic operands remain outside this slice.

## Corpus result

Two unchanged XSLT-Result-Tree cases become exact. Representative Lotus
`select_select51` now evaluates both `author[name='Mary Brady']` and
`author[name/@real='no']` inside its union selection and produces the unchanged
expected result.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,760 | 1,762 | +2 |
| Initialization failures | 1,375 | 1,373 | -2 |
| Executed successfully | 1,576 | 1,578 | +2 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,439 | 1,441 | +2 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,441 / 2,742 = 52.55%` for standard-operation cases and
`1,441 / 3,173 = 45.41%` for the complete catalog.

## Verification

A focused path test covers child equality, reversed operands, multi-node
equality, true existential inequality, an empty-node-set outcome, and a final
attribute step. The complete local sweep establishes the two exact additions,
unchanged mismatch and execution-failure counts, and absence of panics.
