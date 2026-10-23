# XSLT30 HTML content-type replacement -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0157` and `output-0158`
cases. When enabled content-type injection encounters an existing HTML `META`
whose `http-equiv` value is `Content-Type`, the authored node is omitted from
serialization and exactly one serializer-owned meta is emitted.

The first case replaces `charset=UTF-16`; the second replaces a case-varied
attribute spelling and `version='3.0'` parameter. Both serialize one canonical
`content="text/html; charset=UTF-8"` value using HTML void-element syntax.

## Boundary conservation

The bounded validator permits one no-namespace, case-insensitive `meta` only in
`HEAD`, with exactly the unnamespaced `http-equiv` and `content` attributes, no
namespace bindings, and no children. The content value must begin with
`text/html`. Arbitrary head markup and general meta rewriting remain outside
this slice.

Replacement is a serialization decision; the semantic result tree is not
mutated.

## Denominator movement

The complete output denominator moves from 169 to 171 passes and from 62 to 60
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 367 to 369 passes
and from 110 to 108 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
