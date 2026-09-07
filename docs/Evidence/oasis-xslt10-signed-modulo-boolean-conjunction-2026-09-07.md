# OASIS XSLT 1.0 Signed-Modulo Boolean Conjunction

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing source-free boolean tree compare exact signed-modulo
expressions without adding a second instruction-local arithmetic evaluator?

## Change

Literal comparisons inside the source-free boolean owner now delegate to the
checked exact-rational arithmetic evaluator before falling back to the existing
literal coercion rules. The same evaluator now permits a nonzero negative
integral modulo divisor and uses remainder semantics whose sign follows the
dividend, covering all four sign combinations exercised by the corpus.

The already-existing typed `and` tree retains short-circuit evaluation and work
charging. Dynamic arithmetic operands and nonintegral modulo remain unsupported.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,418 | 1,419 | +1 |
| Executed successfully | 1,257 | 1,258 | +1 |
| Expected-result XML matches | 1,142 | 1,143 | +1 |
| XML comparison mismatches | 80 | 80 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Lotus/math_math83#1`.

The strict standard-operation lower bound is now
`1,143 / 2,742 = 41.68%`; the deliberately conservative all-catalog ratio is
`1,143 / 3,173 = 36.02%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused exact-numeric tests cover positive and negative dividends and
  divisors.
- A focused source-free boolean test evaluates the unchanged four-comparison
  conjunction to `true` through the production scalar evaluator.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
