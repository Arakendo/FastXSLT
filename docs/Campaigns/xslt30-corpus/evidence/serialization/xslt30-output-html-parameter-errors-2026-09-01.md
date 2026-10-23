# XSLT30 HTML serialization parameter errors -- 2026-09-01

## Result

FastXSLT executes three unchanged W3C XSLT30 negative cases: `output-0184`,
`output-0191`, and `output-0194`.

The compiler now recognizes and retains the standard `html` output method
without treating recognition as general serializer support. Serialization
reports `SESU0007` for an unavailable encoding, `SESU0011` for an unsupported
normalization form, and `SESU0013` for an unsupported explicit HTML version.
Those parameter failures are selected before the private HTML result-shape
capability gate.

## Boundary conservation

This tranche admits error classification, not general HTML serialization.
Successful HTML output remains limited to the previously admitted bounded
character-map result shape. Explicit HTML versions other than the current
version-five spellings remain unsupported, and no HTML4 behavior is inferred.

## Denominator movement

The complete output denominator moves from 135 to 138 passes and from 97 to 94
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 333 to 336 passes, with 3 engine unsupported cases, 50
profile exclusions, and 142 visible default not-run cases.
