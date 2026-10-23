# OASIS XSLT 1.0 Focus Equality Across Values and Conditions

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can ordinary value expressions and instruction-level conditions compare
`position()` or `last()` with a static nonnegative integer or each other while
retaining the invocation's existing sequence focus and missing-focus behavior?

## Change

The instruction compiler now shares one bounded focus-equality parser between
value expressions and boolean conditions. It accepts two typed operands—focus
position, focus size, or a static nonnegative integer—provided at least one is
focus-dependent. Value and conditional execution both use the already
established `SequenceFocus` and charge one XPath operation; value expressions
serialize the result through the existing boolean result path.

This slice does not admit general focus arithmetic, relational operators, or a
second focus model. A focusless invocation still fails with located `XPDY0002`.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,419 | 1,426 | +7 |
| Executed successfully | 1,258 | 1,265 | +7 |
| Expected-result XML matches | 1,143 | 1,150 | +7 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position01#1`,
`Lotus/position_position12#1`, `Lotus/position_position13#1`,
`Lotus/position_position14#1`, `Lotus/position_position24#1`,
`Lotus/position_position11#1`, and
`Microsoft/Variables_PositionAndLastComparisonShouldBeEqual#1`.

The strict standard-operation lower bound is now
`1,150 / 2,742 = 41.94%`; the deliberately conservative all-catalog ratio is
`1,150 / 3,173 = 36.24%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused lifecycle tests cover `apply-templates` and `for-each` focus,
  position and size comparisons, focus-to-focus equality, and symmetric
  operand order.
- A focusless initial-template control preserves located `XPDY0002` through
  the new value-expression operation.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
