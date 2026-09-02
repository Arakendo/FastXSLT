# XSLT30 XHTML URI attributes -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0102a` through
`output-0102c` and `output-0103a` through `output-0103c` cases. All six enter
through source-free named templates.

With `escape-uri-attributes="yes"`, `¡` becomes `%C2%A1`, the C1 `#x96`
character becomes `%C2%96`, and an existing ASCII `%C2%96` sequence is not
encoded again. With the property disabled, `¡` remains literal and the C1
character uses the XML-compatible `&#x96;` reference. Attribute markup
delimiters remain XML-escaped in both modes.

## Representation and boundary

Compiled output settings now retain the validated optional boolean rather than
discarding it. The serializer defaults it to enabled for XHTML/HTML and applies
it only to an unnamespaced `href` on a no-namespace or XHTML element. Non-URI
attributes continue through the ordinary attribute escaper.

This slice does not implement NFC normalization, a complete HTML/XHTML URI
attribute table, or defined composition between character maps and URI
escaping. Consequently `output-0101` through `0101b`, whose expected URI first
normalizes a decomposed character, remain visible default not-run cases.

## Denominator movement

The complete output denominator moves from 156 to 162 passes and from 75 to 69
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 354 to 360 passes
and from 123 to 117 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
