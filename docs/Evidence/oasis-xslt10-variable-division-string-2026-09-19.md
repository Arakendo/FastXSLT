# OASIS XSLT 1.0 Variable Division String Conversion

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `string($numerator div $denominator)` preserve XPath 1.0 IEEE-style
zero-divisor behavior without weakening FastXSLT's exact-rational arithmetic
boundary?

## Finding

The existing exact-rational evaluator deliberately rejects zero divisors rather
than manufacturing an infinite rational. XPath 1.0 numeric division, however,
requires double behavior and its string conversion spells the positive result
`Infinity`. Routing this compatibility expression into the exact evaluator
would conflate two intentionally different numeric contracts.

## Repair

- Add one typed XSLT 1.0 plan for `string($variable div $variable)`.
- Resolve both operands through the existing charged XSLT 1.0 numeric variable
  conversion.
- Perform the selected compatibility operation as an IEEE double and emit the
  XPath lexical values `Infinity`, `-Infinity`, `NaN`, or the ordinary numeric
  spelling.
- Preserve the exact-rational evaluator's zero-divisor rejection for its modern
  and source-dependent plans.
- Account for both owned variable names in the compiled representation.
- Run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

A focused regression divides the atomic value `123` by `0` and observes
`<out>Infinity</out>`.

The unchanged Microsoft `Variables__84637#1` and `Variables__84712#1` cases
now initialize and execute, and their generated HTML contains the required
`string(123/0) = Infinity`. Initialization rises from 2,014 to 2,016 and
successful execution rises from 1,914 to 1,916. Both cases move into the
existing HTML-comparator-unsupported bucket, which rises from 65 to 67; the
exact-result lower bound therefore remains 1,625.

## Boundary conclusion

This is an explicit XSLT 1.0 compatibility plan, not a change to FastXSLT's
general arithmetic representation. It does not admit arbitrary nested numeric
expressions, change modern division errors, or turn HTML text comparison into
an exact conformance assertion.
