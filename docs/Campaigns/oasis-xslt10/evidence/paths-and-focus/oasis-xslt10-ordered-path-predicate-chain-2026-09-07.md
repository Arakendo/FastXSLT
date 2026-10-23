# OASIS XSLT 1.0 Ordered Path Predicate Chain

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the controlled location-path evaluator apply an attribute filter before a
positional predicate on the same step without treating predicates as an
unordered collection or widening to a general predicate evaluator?

## Change

Each typed location-path step may now retain one existing bounded axis
predicate followed by one existing bounded positional predicate. Evaluation
applies the attribute presence or literal-equality filter first, constructs the
filtered focus in the axis's direction, and only then applies the positional
predicate.

The reverse form, a positional predicate followed by an axis predicate, remains
unsupported. More than two predicates and general predicate expressions also
remain unsupported. This deliberately preserves lexical predicate order rather
than admitting a representation that could silently commute predicates with
different semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,380 | 1,383 | +3 |
| Executed successfully | 1,219 | 1,222 | +3 |
| Expected-result XML matches | 1,105 | 1,108 | +3 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position38#1`,
`Lotus/position_position39#1`, and `Lotus/position_position40#1`. They exercise
`*[@test][4]`, `*[@test][3]`, and `*[@test][1]` respectively. A trace of the
last case produced `<out>1</out>` and matched the suite result.

The related `Lotus/axes_axes14#1` remains uncredited because its second
expression, `(ancestor-or-self::*)[@att1][1]/@att1`, requires a filter expression
outside the controlled location-path grammar. Its first expression now has a
focused semantic test, but the case as a whole correctly remains unsupported.

The strict standard-operation lower bound is now
`1,108 / 2,742 = 40.41%`; the deliberately conservative all-catalog ratio is
`1,108 / 3,173 = 34.92%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused path coverage proves `ancestor-or-self::*[@att1][1]/@att1` selects
  the nearest attribute-bearing ancestor after filtering in reverse-axis order.
- The focused test proves `[1][@att1]` remains unsupported instead of being
  silently treated as equivalent.
- All 49 controlled path tests pass.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.
