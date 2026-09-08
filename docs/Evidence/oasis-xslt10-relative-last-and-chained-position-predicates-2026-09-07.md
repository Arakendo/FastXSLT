# OASIS XSLT 1.0 Relative-Last and Chained Position Predicates

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an ordinary location-path step select relative to its dynamic focus size
with `last()-N`, then apply additional positional predicates in lexical order
with a freshly computed focus after every filter?

## Change

The private typed path owner now represents a bounded sequence of positional
predicates for each step. It evaluates each predicate in lexical order and
recomputes position and size from the surviving sequence before evaluating the
next predicate. This preserves the semantic difference between
`item[last()-1][1]` and a boolean conjunction over the original focus.

The same owner now recognizes `last()-N` for a checked static nonnegative
integer. Underflow produces an empty selection. Non-simple positional
predicates remain explicitly unsupported in multi-step template match patterns
because match-pattern focus semantics are not ordinary path-selection
semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,434 | 1,437 | +3 |
| Executed successfully | 1,273 | 1,276 | +3 |
| Expected-result XML matches | 1,158 | 1,161 | +3 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position103#1`,
`Lotus/position_position104#1`, and `Lotus/position_position105#1`.

The strict standard-operation lower bound is now
`1,161 / 2,742 = 42.34%`; the deliberately conservative all-catalog ratio is
`1,161 / 3,173 = 36.59%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused path test covers last-relative selection, underflow, and chained
  position filters whose focus is recomputed after each filter.
- Existing axis-then-position tests continue to prove lexical predicate order.
- A focused compiler test keeps last-relative multi-step match patterns
  explicitly unsupported.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
