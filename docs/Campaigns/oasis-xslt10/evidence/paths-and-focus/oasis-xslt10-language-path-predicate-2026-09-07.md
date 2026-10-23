# OASIS XSLT 1.0 Language Path Predicate

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a location-path predicate reuse the existing charged context-language
operation rather than introducing predicate-local `xml:lang` traversal?

## Change

The typed path predicate parser now recognizes a literal `lang()` call and
retains its requested language in the existing predicate slot. Evaluation
delegates to the same context-language operation used by ordinary boolean and
value expressions. Nearest inherited `xml:lang`, ASCII case-insensitive
matching, sublanguage matching, work charging, and cancellation therefore stay
in one owner.

A second bounded tranche composes already-supported predicate atoms with a
top-level `and`. The parser respects quoted text and parenthesis depth, and the
runtime short-circuits the right-hand atom. A successfully typed predicate path
also takes precedence over the source-free scalar recognizer, so an `and`
inside `xsl:value-of select` is not misclassified as a standalone boolean
expression.

Variable `lang()` arguments, `or`, and general predicate expressions remain
unsupported.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,407 | 1,412 | +5 |
| Executed successfully | 1,246 | 1,251 | +5 |
| Expected-result XML matches | 1,131 | 1,136 | +5 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Lotus/expression_expression01#1`,
`Lotus/expression_expression03#1`, `Lotus/expression_expression04#1`,
`Lotus/expression_expression05#1`, and `Lotus/predicate_predicate48#1`.
`Lotus/expression_expression06#1` crosses its first `lang()` predicate and then
stops at the distinct unsupported qualified-attribute predicate
`ancestor-or-self::*[@xml:lang]/@xml:lang`; it remains visible and uncredited.

The strict standard-operation lower bound is now
`1,136 / 2,742 = 41.43%`; the deliberately conservative all-catalog ratio is
`1,136 / 3,173 = 35.80%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused path and lifecycle tests prove inherited, case-insensitive sublanguage
  matching, rejection after a nearer conflicting `xml:lang` value, top-level
  conjunction, short-circuit evaluation, and compiler dispatch through an
  ordinary instruction expression.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
