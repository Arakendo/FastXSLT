# OASIS XSLT 1.0 Explicit Position Equality

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the controlled location-path parser recognize `position() = N` as the same
typed positional operation already used by the numeric predicate `[N]`, without
admitting general comparisons into path predicates?

## Change

The private positional-predicate parser now accepts `position()` followed by
an equality sign and an existing bounded constant-integer expression. It lowers
that form to the same `Select`/`Never` representation as a numeric predicate.
Whitespace around the equality sign is accepted.

Non-equality comparisons, a position comparison against a dynamic expression,
and general predicate expressions remain unsupported. Evaluation, axis focus,
document-order normalization, duplicate removal, work charging, and diagnostics
continue through the existing typed location-path owner.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,383 | 1,393 | +10 |
| Executed successfully | 1,222 | 1,232 | +10 |
| Expected-result XML matches | 1,108 | 1,118 | +10 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position02#1`,
`Lotus/position_position04#1`, `Lotus/position_position09#1`,
`Lotus/position_position15#1`, `Lotus/position_position16#1`,
`Lotus/position_position17#1`, `Lotus/position_position25#1`,
`Lotus/position_position26#1`, `Lotus/position_position35#1`, and
`Lotus/position_position36#1`. The cases cover both a single positional
predicate and attribute-filter-then-position chains.

The strict standard-operation lower bound is now
`1,118 / 2,742 = 40.77%`; the deliberately conservative all-catalog ratio is
`1,118 / 3,173 = 35.23%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused coverage proves `*[@test][position() = 2]/num` filters the candidate
  sequence before applying the explicit positional equality.
- The same focused test proves `position() > 1` remains unsupported.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.
