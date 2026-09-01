# XSLT30 `html-version` static boundary -- 2026-09-01

## Result

FastXSLT admits the unchanged W3C XSLT30 case `output-0230` through its native
`XTSE0020` result. The stylesheet supplies `html-version="five"`; compilation
now recognizes the standard attribute and rejects the non-decimal value as
invalid with retained stylesheet location.

## Boundary conservation

This tranche separates positive decimal lexicals from invalid syntax. The
subsequent bounded XHTML 5 tranche admits values numerically equal to five;
other positive versions, including `+4.1`, report private unsupported
diagnostic `FXST1049`. Zero, negative, empty, multiple-point, exponent, and
nonnumeric lexicals do not pass the positive-decimal boundary.

At this checkpoint no `html-version` field was retained in `OutputSettings` and
no behavior was inferred from it. The later XHTML 5 evidence records the
deliberate representation and serializer admission separately.

## Denominator movement

The complete output denominator moves from 110 to 111 passes and from 122 to
121 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 308 to 309 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 169 visible default not-run cases.
