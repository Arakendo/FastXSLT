# OASIS XSLT 1.0 NaN composition -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the XSLT 1.0 compatibility compiler preserve XPath 1.0 NaN propagation
through bounded source-free comparisons, arithmetic, and integral functions
without changing the modern typed numeric rules?

## Implemented slice

The source-free numeric owner now recognizes a failed `number()` conversion of
one string literal as NaN when it participates in one of these XSLT 1.0 forms:

- equality with another admitted numeric operand;
- one binary `+`, `-`, `*`, `div`, or `mod` operation with an admitted finite
  numeric operand;
- `floor()`, `ceiling()`, or `round()`.

The compiler folds these forms to the existing typed boolean or literal result
only while compiling a version 1.0 stylesheet. NaN equality is false and the
listed arithmetic/function compositions produce `NaN`. The equivalent version
3.0 stylesheet remains rejected under the existing decimal semantics. No
runtime version branch, general expression evaluator, or non-source-free NaN
representation is introduced.

## Corpus result

Eight unchanged Lotus cases move from initialization failure to exact expected
results: `math_math21`, `math_math22`, and `math_math89` through `math_math94`.
They cover NaN equality, addition, subtraction, multiplication, division,
modulo, floor, ceiling, and round. Some cases contain more than one expression,
so eight cases exercise eleven folded expressions.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,718 | 1,726 | +8 |
| Initialization failures | 1,417 | 1,409 | -8 |
| Executed successfully | 1,535 | 1,543 | +8 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,402 | 1,410 | +8 |
| XML comparison mismatches | 106 | 106 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,410 / 2,742 = 51.42%` for standard-operation cases and
`1,410 / 3,173 = 44.44%` for the complete catalog.

## Verification

- A focused numeric-owner test proves every admitted grammar form and rejects
  nearby unsupported forms.
- A complete compile/execute test proves the version-sensitive lifecycle and
  the exact serialized values together.
- The complete 3,173-case sweep moves only the intended frontier and adds no
  mismatch, execution failure, or panic.
- The complete `scripts/verify.ps1` gate passes: unsafe-surface policy,
  formatting, strict Clippy, 917 active engine tests, workbench and worker
  tests, documentation, Markdown links, and pinned-corpus integrity.
