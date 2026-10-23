# OASIS XSLT 1.0 Count-Path Conditions

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can instruction-level conditions compare the node count of an admitted
location path with a static nonnegative integer without adding a separate path
or sequence evaluator?

## Change

The boolean-expression compiler now lowers equality between `count(path)` and
a static nonnegative integer, in either operand order, to one typed operation.
Runtime evaluation uses the shared controlled location-path evaluator, charges
the comparison, and compares the resulting sequence length without retaining a
second sequence representation.

This bounded slice does not admit general count arithmetic, relational
operators, dynamic comparison operands, or a compatibility-only evaluator.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,426 | 1,429 | +3 |
| Executed successfully | 1,265 | 1,268 | +3 |
| Expected-result XML matches | 1,150 | 1,153 | +3 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position37#1`,
`Lotus/position_position73#1`, and `Lotus/position_position74#1`.

The strict standard-operation lower bound is now
`1,153 / 2,742 = 42.05%`; the deliberately conservative all-catalog ratio is
`1,153 / 3,173 = 36.34%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused lifecycle test selects leaf and non-leaf branches from the count
  of the current node's element children.
- All three unchanged corpus cases execute through ordinary `xsl:choose`,
  source focus, controlled path evaluation, and result serialization.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
