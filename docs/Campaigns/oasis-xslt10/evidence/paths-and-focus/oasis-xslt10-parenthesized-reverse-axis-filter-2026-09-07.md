# OASIS XSLT 1.0 Parenthesized Reverse-Axis Filter

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the controlled path evaluator preserve the semantic distinction between a
predicate attached directly to a reverse-axis step and the same predicate
attached to a parenthesized reverse-axis result?

## Change

The private location-path plan now retains one boolean semantic marker for the
bounded shape `(reverse-axis::name-test)[predicate]...`. Before applying that
first step's admitted predicates, evaluation normalizes its candidates into
document order. The ordinary `reverse-axis::name-test[predicate]` form continues
to apply predicates in reverse-axis proximity order and normalizes only after
predicate selection.

The admitted parenthesized inner expression is exactly one `ancestor`,
`ancestor-or-self`, `preceding`, or `preceding-sibling` step. Nested
parentheses, multi-step inner paths, and general filter expressions remain
unsupported.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,395 | 1,398 | +3 |
| Executed successfully | 1,234 | 1,237 | +3 |
| Expected-result XML matches | 1,120 | 1,123 | +3 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/axes_axes14#1`,
`Lotus/axes_axes114#1`, and `Lotus/axes_axes115#1`. Each contrasts direct
reverse-axis predicate order with parenthesized filter order and produces the
suite's expected `a, c` result. A more deeply nested expression in
`Lotus/position_position85#1` remains explicitly unsupported and uncredited.

The strict standard-operation lower bound is now
`1,123 / 2,742 = 40.96%`; the deliberately conservative all-catalog ratio is
`1,123 / 3,173 = 35.39%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused coverage proves `ancestor-or-self::*[@att1][1]/@att1` selects the
  nearest matching ancestor while `(ancestor-or-self::*)[@att1][1]/@att1`
  selects the first matching node in document order.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.
