# XSLT30 HTML 5 input value -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0724` case. The source-free
initial template constructs one empty HTML `input` with `type="text"` and an
airplane character in `value`. HTML 5 serialization emits a void start tag with
no closing tag or XML slash and preserves the airplane as ordinary attribute
content.

The executable sentinel also verifies that the non-URI `value` attribute is not
percent-encoded. This keeps URI escaping tied to the serializer's admitted URI
attribute vocabulary rather than to arbitrary non-ASCII attributes.

## Admission boundary

The private validator admits only one no-namespace, binding-free, childless
root `input` whose two attributes are exactly `type="text"` and `value="✈"`.
This does not broaden the ordinary HTML 5 document vocabulary.

## Denominator movement

The complete output denominator moves from 177 to 178 passes and from 54 to 53
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 375 to 376 passes
and from 102 to 101 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
