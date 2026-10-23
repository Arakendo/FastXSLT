# XSLT30 XHTML C1 attributes -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0102e` and `output-0103e`
cases. Both enter through source-free named templates and construct a C1
`#x96` character in the non-URI XHTML `accesskey` attribute. The serializer
emits `&#x96;` whether `escape-uri-attributes` is enabled or disabled.

## Boundary conservation

C1 numeric-reference emission belongs to the common XML-compatible attribute
escaper. The selected attribute is not URI-valued, so the compiled
`escape-uri-attributes` property remains inert. The adjacent `href` cases stay
visible default not-run until FastXSLT retains that property and implements the
required normalization and percent encoding.

## Denominator movement

The complete output denominator moves from 154 to 156 passes and from 77 to 75
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 352 to 354 passes
and from 125 to 123 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
