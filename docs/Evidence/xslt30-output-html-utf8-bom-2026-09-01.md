# XSLT30 HTML UTF-8 byte-order mark -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0162` and `output-0163`
cases through its byte-result lane. With `byte-order-mark="yes"`, the result is
the exact prefix `EF BB BF` followed by `<html><body>Hello</body></html>`. With
`byte-order-mark="no"`, the same HTML bytes begin immediately without a mark.

The BOM bytes participate in the existing serialized-byte limit and work
accounting. The string-result lane remains unable to represent a requested BOM;
the corpus adapter deliberately selects bytes for this serialization property.

## Denominator movement

The complete output denominator moves from 173 to 175 passes and from 58 to 56
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 371 to 373 passes
and from 106 to 104 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
