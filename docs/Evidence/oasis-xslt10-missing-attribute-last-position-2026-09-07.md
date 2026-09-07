# OASIS XSLT 1.0 Missing-Attribute and Last-Position Predicates

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the typed path evaluator admit the bounded `not(@name)` node test and the
equality-equivalent `last()=position()` spelling without introducing a general
boolean predicate evaluator?

## Change

The private predicate vocabulary now has an explicit missing-attribute kind.
It reuses the charged attribute scan and negates only the bounded presence
result. Chained positional predicates continue to run over the sequence that
survives the missing-attribute filter.

The position parser also recognizes `last()=position()` as the same typed last
position already used for `last()` and `position()=last()`. General `not()`,
qualified attribute names, and other comparisons remain unsupported.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,416 | 1,418 | +2 |
| Executed successfully | 1,255 | 1,257 | +2 |
| Expected-result XML matches | 1,140 | 1,142 | +2 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/position_position58#1` and
`Lotus/position_position59#1`. `Lotus/predicate_predicate58#1` remains visibly
uncredited at its earlier, distinct `string-length(@ex)=0` predicate boundary.

The strict standard-operation lower bound is now
`1,142 / 2,742 = 41.65%`; the deliberately conservative all-catalog ratio is
`1,142 / 3,173 = 35.99%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused source with interleaved present and absent attributes proves that
  missing-attribute filtering happens before the chained last-position test.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
