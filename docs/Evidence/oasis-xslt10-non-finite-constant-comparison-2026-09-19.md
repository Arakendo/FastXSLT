# OASIS XSLT 1.0 Non-Finite Constant Comparison

Date: 2026-09-19  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XSLT 1.0 constant expressions compare positive infinity, negative infinity,
and NaN without weakening the checked exact-rational evaluator used by ordinary
finite arithmetic?

## Repair

- Preserve the lexical sign of a zero divisor when folding a constant XPath
  1.0 division, so `1 div -0` yields negative infinity while `1 div 0` yields
  positive infinity.
- Add a private constant-number comparison fold for finite rationals, signed
  infinities, and NaN.
- Apply XPath 1.0 comparison behavior: NaN is unequal to every value and all
  ordered comparisons involving NaN are false.
- Select the fold only from an XSLT 1.0 static context. Keep the checked
  exact-rational evaluator and modern expression semantics unchanged.
- Share the fold between value and conditional compilation so the compatibility
  rule has one semantic owner.

## Result

Focused tests cover signed-zero division, infinity ordering/equality, and NaN
equality/ordering. The unchanged Lotus `math75` through `math78` cases now
initialize, execute, and match their expected results.

The complete 3,173-case sweep moves from 2,017 to 2,021 initialized cases, from
1,917 to 1,921 successful executions, and from 1,786 to 1,790 exact
XML-semantic matches. The visible mismatch count remains 57.

## Boundary conclusion

This admits statically decidable comparisons containing a non-finite constant
division under XSLT 1.0 compatibility. It does not introduce floating-point
evaluation into the general XPath engine, admit dynamic non-finite arithmetic,
or change modern XPath numeric semantics.
