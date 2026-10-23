# OASIS XSLT 1.0 nested static conversion -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a bounded source-free XPath 1.0 number/string conversion composition reuse
the modern compiler without admitting general nested expressions?

## Implemented slice

Under an XSLT 1.0 stylesheet only, equality between static finite decimals now
accepts `number(string(static-decimal))` as either operand. Both the inner
string conversion and outer number conversion canonicalize through the existing
finite-decimal rules, and the result lowers to the ordinary constant-boolean
plan.

Dynamic operands, non-finite operands, inequality, broader nesting, and modern
stylesheets remain outside this fold.

## Corpus result

Unchanged Lotus `math_math16` leaves the function-shaped path frontier and
exactly matches its expected result.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,757 | 1,758 | +1 |
| Initialization failures | 1,378 | 1,377 | -1 |
| Executed successfully | 1,573 | 1,574 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,436 | 1,437 | +1 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,437 / 2,742 = 52.41%` for standard-operation cases and
`1,437 / 3,173 = 45.29%` for the complete catalog.

## Verification

Focused folding tests cover true, false, and dynamic rejection. A lifecycle
test proves compatibility is selected from stylesheet version and that the
modern spelling remains outside the private slice. The complete local sweep
establishes one exact addition with unchanged mismatch/execution-failure counts
and no panics.
