# OASIS XSLT 1.0 ordered literal path predicates -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XSLT 1.0 ordered comparisons between literal booleans, numbers, and strings
be folded consistently for every path-consuming instruction?

## Decision in the experiment

The existing XSLT 1.0 ordered-literal fold now converts boolean literals to
their XPath 1.0 numeric values. Static comparison predicates moved from the
`xsl:value-of` compiler into the XSLT 1.0 typed-path entry point, so
`xsl:for-each`, template selection, and value extraction share one compile-time
compatibility rule. True predicates retain the parsed path and false predicates
produce the typed empty path. No runtime compatibility branch was added.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,586 | 1,588 | +2 |
| Executed successfully | 1,415 | 1,417 | +2 |
| Expected-result XML matches | 1,295 | 1,297 | +2 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate07#1` and
`Lotus/predicate_predicate08#1` cases now match exactly. The strict
standard-operation lower bound is now `1,297 / 2,742 = 47.30%`; the
conservative all-catalog ratio is `1,297 / 3,173 = 40.88%`. These are local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche covers only source-free literal predicates. Dynamic comparisons,
node-set relational comparison, and general predicate expression trees remain
outside the admitted slice. Modern XPath semantics are unchanged. No corpus
bytes or expected results changed.

## Verification

- Focused constant-fold tests cover boolean-to-number conversion.
- The focused runtime test proves the folded predicate through normal transform
  execution.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
