# OASIS XSLT 1.0 Position-Then-Name Predicate

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a typed location-path step apply a positional filter and then test the
lexical name of the surviving context node in the required predicate order?

## Change

The private path representation now retains one narrowly admitted trailing
`name()='literal'` predicate after one or more positional predicates. Evaluation
first applies each position predicate with its recomputed focus, then compares
the surviving node's lexical QName without allocating a temporary name.

This does not admit arbitrary predicate reordering or a general predicate AST.
Unsupported arrangements still fail explicitly.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,439 | 1,440 | +1 |
| Executed successfully | 1,278 | 1,279 | +1 |
| Expected-result XML matches | 1,163 | 1,164 | +1 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Lotus/position_position82#1`. Its
`alpha/*[last()][name()='z']` path counts six parent-local last children whose
lexical name is `z`.

The strict standard-operation lower bound is now
`1,164 / 2,742 = 42.45%`; the deliberately conservative all-catalog ratio is
`1,164 / 3,173 = 36.68%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused path test proves that the name test observes the position-filtered
  focus rather than running before it.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
