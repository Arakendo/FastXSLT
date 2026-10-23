# OASIS XSLT 1.0 Finite Constant Arithmetic

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the checked constant evaluator emit exact finite decimal results for XSLT
1.0 arithmetic without routing ordinary transformation work through binary
floating point?

## Repair

- Add an XSLT 1.0-only constant-evaluation mode that admits numeric string
  conversion and `number()` around source-free arithmetic.
- Keep intermediate arithmetic as checked exact rationals.
- Emit a decimal lexical only when the reduced denominator terminates in base
  ten; leave recurring decimals outside this bounded fold.
- Add source-free finite comparison over the same evaluator, including nested
  `number()` and parenthesized arithmetic.
- Keep dynamic path conversion and comparison in typed runtime plans rather
  than pretending they are constant.

## Result

Focused tests cover terminating division, decimal multiplication, whitespace-
padded numeric string conversion, nested `number()`, finite comparisons, and
the rejection of recurring or dynamic expressions.

The unchanged Lotus `select37#1` case now initializes, executes, and matches its
expected result. The complete 3,173-case sweep moves to 2,022 initialized cases,
1,922 successful executions, and 1,791 exact XML-semantic matches. The mismatch
count remains 57.

The same slice advances Lotus `math110#1` to its next honest boundary:
comparison of dynamic `number(path)` with a static arithmetic value. That case
remains uncredited.

## Boundary conclusion

This admits exactly representable, source-free finite arithmetic under XSLT
1.0 compatibility. It does not select a general floating-point evaluator,
approximate recurring decimals, or admit dynamic source/variable operands as
constants.
