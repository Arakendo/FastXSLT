# OASIS XSLT 1.0 following-sibling relational predicates -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the typed predicate path implement XSLT 1.0 existential node-set/numeric
comparison for `following-sibling::*` across equality, inequality, and ordered
operators without introducing a general predicate interpreter?

## Decision in the experiment

The existing following-sibling numeric leaf now retains one typed comparison
operator. It accepts `=`, `!=`, `<`, `<=`, `>`, and `>=` with a static integer
on either side. When the integer is on the left, compilation reverses the
ordered operator so runtime always compares each sibling's numeric string value
against the retained scalar.

Evaluation remains existential: a predicate succeeds when any following
element sibling satisfies the comparison. Each sibling visit is charged. XPath
NaN behavior is preserved: equality and ordering fail, while inequality
succeeds.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,612 | 1,622 | +10 |
| Executed successfully | 1,441 | 1,451 | +10 |
| Expected-result XML matches | 1,321 | 1,331 | +10 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate09#1` and
`Lotus/predicate_predicate27#1` through
`Lotus/predicate_predicate35#1` cases now match exactly. The strict
standard-operation lower bound is now `1,331 / 2,742 = 48.54%`; the
conservative all-catalog ratio is `1,331 / 3,173 = 41.95%`. These remain local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit path-to-path comparison, non-static scalar
operands, other axes, arbitrary numeric expressions, or chained comparisons.
It does not change result order, node identity, diagnostics, cancellation,
resource authority, or corpus data.

## Verification

- A focused path test covers every admitted operator, both operand directions,
  existential behavior, and the empty-suffix outcome.
- Ten unchanged corpus cases execute through the production compiler, runtime,
  and serializer and match their expected XML.
- The complete 3,173-case local measurement produced the counters above.
