# OASIS XSLT 1.0 source node-set comparison -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XSLT 1.0 boolean and value expressions compare two source node sets using
the standard independently existential string-value rule while preserving
typed compilation, bounded work accounting, and the modern semantic core?

## Changes

- The XSLT 1.0 expression compiler recognizes `=` and `!=` only when the
  operator is outside predicates, parentheses, and string literals and both
  operands independently parse as supported location paths.
- The compiled plan retains both typed paths and the selected equality
  operator. Modern expressions do not select this compatibility plan.
- Runtime evaluates each path through the existing controlled navigator, then
  compares candidate node string values existentially. Equality and inequality
  are independent existential tests; `A != B` is not implemented as
  `not(A = B)`.
- Candidate-pair comparison and both node string-value walks remain charged.
  No source nodes or comparison results are retained beyond the invocation.
- The same evaluator now serves `xsl:if`/`xsl:when`, `xsl:value-of`, and the
  previously admitted template-argument comparison path.

The first complete-corpus run exposed an over-broad recognizer that diverted
non-path scalar equalities. Admission was tightened to require two successfully
parsed paths before the specialized plan is selected; the complete denominator
then recovered every prior result and added only the intended semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,490 | 1,498 | +8 |
| Executed successfully | 1,325 | 1,333 | +8 |
| Expected-result XML matches | 1,217 | 1,225 | +8 |
| XML comparison mismatches | 82 | 82 | 0 |
| Execution failures | 165 | 165 | 0 |

The unchanged Lotus `boolean70` through `boolean76` cases cover equal and
unequal pairs across overlapping and disjoint filtered node sets. The unchanged
Lotus `position31` case composes the same comparison with the
`preceding-sibling` axis inside a conditional. All eight now match their
expected XML.

The strict standard-operation lower bound becomes
`1,225 / 2,742 = 44.68%`; the conservative all-catalog ratio becomes
`1,225 / 3,173 = 38.61%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit relational node-set comparisons, node-set versus
number or boolean coercion, variables as either operand, arbitrary XPath
expressions, or a general expression AST. Unsupported forms retain their prior
structured boundary. No corpus bytes or expected results were changed.

## Verification

- A focused runtime test covers both conditional and value construction,
  overlapping node sets, disjoint node sets, and the fact that equality and
  inequality may both be true for different candidate pairs.
- The complete 3,173-case local measurement produced the counters above with no
  panic, mismatch increase, or execution-failure increase.
- The ordinary workspace verification gate passes.
