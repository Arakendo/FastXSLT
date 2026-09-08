# OASIS XSLT 1.0 Subtraction Token Boundaries

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the private checked arithmetic tree distinguish subtraction from hyphens in
XML names when XSLT 1.0 expressions omit otherwise convenient whitespace?

## Changes

- A minus preceded by whitespace is recognized as binary subtraction only when
  the preceding non-whitespace character is not another arithmetic operator.
  This admits `@div -5` and `n1 -n3` while preserving unary negation in
  `n-2 - -n-1`.
- A minus following a decimal digit is recognized as subtraction when the next
  token can begin a number, variable, or name. An XML name cannot begin with a
  decimal digit, so this resolves forms such as `100-n6`, `15-$anum`, and
  `16-div` without reclassifying the valid name `n-2`.
- The compiler still constructs the same checked exact-rational tree. This is
  lexical recognition only; it adds no evaluator, conversion, authority, or
  public API.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,465 | 1,469 | +4 |
| Executed successfully | 1,303 | 1,307 | +4 |
| Expected-result XML matches | 1,189 | 1,193 | +4 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 162 | 162 | 0 |

The unchanged exact cases are:

- `Lotus/math_math98#1`
- `Lotus/math_math99#1`
- `Lotus/select_select24#1`
- `Lotus/select_select33#1`

The strict standard-operation lower bound becomes
`1,193 / 2,742 = 43.51%`; the conservative all-catalog ratio becomes
`1,193 / 3,173 = 37.60%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Conservation and rejected broadening

An initial rule treated whitespace on either side of `-` as sufficient. The
full corpus immediately regressed two previously initialized and passing cases,
so that rule was removed. The retained rule is directional and operator-aware.
Focused parser tests preserve hyphenated names, signed paths, adjacent numeric
subtraction, parenthesized operands, and the newly admitted corpus expressions.

## Verification

- The focused binary-numeric/compiler/runtime tests pass.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
