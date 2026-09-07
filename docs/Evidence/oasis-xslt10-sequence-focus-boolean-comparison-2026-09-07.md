# OASIS XSLT 1.0 Sequence-Focus Boolean Comparison

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can instruction-local boolean evaluation consume the same dynamic sequence
focus already used by value expressions, so `position() != last()` does not
need a parallel focus model or an XSLT 1.0-only runtime?

## Change

The private boolean plan now recognizes only the symmetric comparison
`position() != last()` (or `last() != position()`). The compiled operation
retains its source location. Runtime boolean evaluation receives the existing
sequence focus, charges one XPath operation, compares its position and size,
and reports located `XPDY0002` when no dynamic focus exists.

This is intentionally not a general numeric-comparison parser. Other focus
comparisons, arithmetic operands, relational operators, and predicate forms
remain outside this tranche.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,340 | 1,343 | +3 |
| Executed successfully | 1,179 | 1,182 | +3 |
| Expected-result XML matches | 1,067 | 1,070 | +3 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged `Lotus/position_position68#1`,
`Lotus/position_position69#1`, and `Lotus/position_position77#1` cases now
execute and match their expected XML. No newly executing case stopped at a
later boundary.

The strict standard-operation lower bound is now
`1,070 / 2,742 = 39.02%`; the deliberately conservative all-catalog ratio is
`1,070 / 3,173 = 33.72%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test proves the final member of a three-node sequence is
  distinguished without changing the sequence's context position or size.
- A focusless initial-template test proves the operation retains its source
  location and reports `XPDY0002` rather than inventing a singleton focus.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
