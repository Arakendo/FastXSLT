# XSLT30 source-free XHTML non-URI attributes -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0102d`, `output-0102f`,
`output-0103d`, and `output-0103f` cases. Each case uses a named initial
template without a principal source. The XHTML serializer preserves `¡` in the
standard non-URI `accesskey` attribute and the unknown `notxhtmlattr` attribute,
both when `escape-uri-attributes` is enabled and when it is disabled.

The output corpus adapter now maps `<initial-template name="...">` to the
engine's existing source-free `InvocationEntry::InitialTemplate` path. It
admits a source resource only when the resolved environment actually supplies
one.

## Boundary conservation

The compiler validates the XSLT-version-specific boolean and admits it for an
explicit XHTML method, but does not yet retain it. This is correct only for the
selected non-URI attributes. The `href` siblings remain visible default
not-run cases; their required normalization and percent encoding must not be
inferred from this inert-property evidence.

## Denominator movement

The complete output denominator moves from 150 to 154 passes and from 81 to 77
visible default not-run cases; its one HTML 4 profile exclusion is unchanged.
Across the eleven conserved XSLT30 denominators, the total moves from 348 to 352
passes and from 129 to 125 visible default not-run cases, with 3 engine
unsupported cases and 51 profile exclusions unchanged.
