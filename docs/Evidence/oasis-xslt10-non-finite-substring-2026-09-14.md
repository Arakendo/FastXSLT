# OASIS XSLT 1.0 non-finite substring -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can static XSLT 1.0 `substring()` calls preserve the specification's NaN and
infinity position rules without weakening modern numeric error behavior or
adding a general expression evaluator?

## Implemented slice

The existing static substring folder now has a version-selected XPath 1.0
entry point. Its string operand remains a literal, while start and optional
length accept either a numeric literal or one bounded literal `div` operation.
IEEE division supplies NaN and positive or negative infinity; the existing
XPath position-selection algorithm then applies those values directly.

The ordinary modern entry point continues to require finite numeric literals.
Dynamic operands, composed arithmetic, and general nested numeric expressions
remain outside this fold.

## Corpus result

Unchanged Lotus `string18`, `string19`, `string20`, `string21`, and `string113`
leave the function-shaped path frontier and exactly match their expected
results.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,746 | 1,751 | +5 |
| Initialization failures | 1,389 | 1,384 | -5 |
| Executed successfully | 1,563 | 1,568 | +5 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,426 | 1,431 | +5 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,431 / 2,742 = 52.19%` for standard-operation cases and
`1,431 / 3,173 = 45.10%` for the complete catalog.

## Verification

Focused folding tests cover NaN, positive infinity, the negative-infinity plus
positive-infinity boundary, and rejection of dynamic input. A lifecycle test
executes all four distinct result shapes through stylesheet compilation and
serialization. The complete local sweep establishes five exact additions,
unchanged mismatch/execution-failure counts, and absence of panics.
