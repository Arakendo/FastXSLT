# OASIS XSLT 1.0 Standalone Exact Numeric Literal

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a standalone decimal literal use the existing checked numeric plan rather
than falling through to the location-path parser, including when its magnitude
exceeds the earlier narrow integer-folding range?

## Changes

- The numeric-expression compiler now retains a standalone exact decimal as a
  checked numeric plan in addition to retaining arithmetic operations.
- A bare path and a bare variable are still rejected by this compiler and keep
  their existing typed owners. The change therefore does not turn source paths
  into numeric expressions or apply XSLT 1.0 variable conversion to modern
  stylesheets.
- Runtime execution and canonical decimal formatting reuse the same safe
  exact-rational evaluator used by composed path arithmetic.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,469 | 1,470 | +1 |
| Executed successfully | 1,307 | 1,308 | +1 |
| Expected-result XML matches | 1,193 | 1,194 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 162 | 162 | 0 |

The unchanged `Lotus/math_math105#1` case now produces the exact expected
`9876543210` value. The strict standard-operation lower bound becomes
`1,194 / 2,742 = 43.54%`; the conservative all-catalog ratio becomes
`1,194 / 3,173 = 37.63%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not add exponential numeric syntax, non-finite values,
general `number(expression)` composition, numeric comparisons, arbitrary-size
integers, or a general XPath parser. Values outside the checked `i128`
exact-rational representation remain outside this private slice.

## Verification

- Focused compiler and runtime tests prove the large standalone numeric path.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
