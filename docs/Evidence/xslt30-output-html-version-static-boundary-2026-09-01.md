# XSLT30 `html-version` static boundary -- 2026-09-01

## Result

FastXSLT admits the unchanged W3C XSLT30 case `output-0230` through its native
`XTSE0020` result. The stylesheet supplies `html-version="five"`; compilation
now recognizes the standard attribute and rejects the non-decimal value as
invalid with retained stylesheet location.

## Boundary conservation

This tranche does not implement XHTML 5 serialization. Positive decimal
lexicals—including `5`, `5.0`, whitespace-surrounded `5.00`, and `+4.1`—are
recognized as valid syntax and then report private unsupported diagnostic
`FXST1049`. Zero, negative, empty, multiple-point, exponent, and nonnumeric
lexicals do not pass the positive-decimal boundary.

No `html-version` field is retained in `OutputSettings`, and no doctype,
namespace normalization, void-element, or content-type behavior is inferred
from it. A future positive-version tranche must admit those serialization
semantics explicitly.

## Denominator movement

The complete output denominator moves from 110 to 111 passes and from 122 to
121 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 308 to 309 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 169 visible default not-run cases.
