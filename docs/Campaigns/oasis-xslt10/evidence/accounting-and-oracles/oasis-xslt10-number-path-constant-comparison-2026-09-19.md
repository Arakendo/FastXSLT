# OASIS XSLT 1.0 Number-Path/Constant Comparison

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 expression compare `number(path)` with source-free constant
arithmetic without reevaluating the static side or widening the general XPath
runtime?

## Repair

- Compile the source path through the existing namespace-aware `number(path)`
  path compiler.
- Fold the source-free operand once through the checked exact-rational constant
  evaluator and retain its canonical finite-decimal lexical in compiled state.
- Evaluate and charge only the path selection, source string-value conversion,
  numeric conversion, and comparison during each invocation.
- Preserve XPath 1.0 NaN equality behavior and account for the retained path and
  decimal lexical in compiled-state capacity.
- Keep the operation behind XSLT 1.0 static-context selection.

## Result

A focused runtime regression covers equality, inequality, and an empty path
whose numeric conversion is NaN. The unchanged Lotus `math110#1` case now
initializes, executes, and matches its expected result.

The complete 3,173-case sweep moves to 2,023 initialized cases, 1,923 successful
executions, and 1,792 exact XML-semantic matches. The mismatch count remains 57.

## Boundary conclusion

This admits equality and inequality between one typed `number(path)` operand
and one exactly folded, terminating finite constant. It does not admit general
dynamic numeric expressions, ordered path-number comparison, approximate
constant folding, or a separate XSLT 1.0 evaluator.
