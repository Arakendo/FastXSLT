# XSLT30 HTML content-type meta -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0123`, `output-0124`,
`output-0124a`, `output-0124b`, `output-0125`, `output-0125a`, and
`output-0125b` cases.

The default and explicitly enabled variants inject
`<meta http-equiv="Content-Type" content="application/xhtml-xml; charset=UTF-8">`
as the first serializer-produced content in `HEAD`. The XSLT 2.0 `yes` and
XSLT 3.0 `true`/`1` lexicals reach the same behavior. The `no`, `false`, and `0`
variants produce no content-type meta.

## Boundary conservation

Content-type injection is shared with the existing XHTML policy, but HTML uses
void-element syntax without the XHTML closing slash. Admission is limited to a
single unnamespaced `HTML` root containing unnamespaced, attribute-free `HEAD`
and `BODY` elements with text children. The serializer does not mutate the
semantic result tree.

Existing-meta replacement, arbitrary HTML head content, and general HTML
serialization remain separate evidence.

## Denominator movement

The complete output denominator moves from 162 to 169 passes and from 69 to 62
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 360 to 367 passes
and from 117 to 110 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
