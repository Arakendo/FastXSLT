# OASIS XSLT 1.0 Relational Position Predicates

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a location-path step compare its dynamic position with a static integer
using the XPath equality and relational operators while preserving predicate
focus, ordering, and the separate semantics of XSLT match patterns?

## Change

The typed path-position predicate now supports `!=`, `<`, `<=`, `>`, and `>=`
against a checked static integer. Explicit equality retains the existing
`Select`/`Never` representation, and all comparisons are evaluated against the
already established candidate position and size after preceding predicates.

The compiler deliberately rejects general position comparisons in multi-step
template match patterns. Corpus measurement exposed that treating a match
pattern as an ordinary document-rooted selection produced a plausible but
incorrect result for `match16`, whose positional focus is relative to a child
axis. That case remains visibly unsupported until the pattern owner implements
the required semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,429 | 1,434 | +5 |
| Executed successfully | 1,268 | 1,273 | +5 |
| Expected-result XML matches | 1,153 | 1,158 | +5 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position60#1` through
`Lotus/position_position64#1`.

The strict standard-operation lower bound is now
`1,158 / 2,742 = 42.23%`; the deliberately conservative all-catalog ratio is
`1,158 / 3,173 = 36.50%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused path test covers all five operators over one four-node sequence.
- A focused compiler test proves the relational multi-step match pattern stays
  explicitly unsupported.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
