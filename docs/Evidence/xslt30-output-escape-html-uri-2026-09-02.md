# XSLT30 literal escape-html-uri -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0141`, `output-0141a`, and
`output-0141b` cases. The three accepted boolean lexicals disable serializer
URI-attribute escaping, leaving the literal IRI unchanged, while the independent
literal `fn:escape-html-uri()` call produces the required uppercase UTF-8
percent escapes in a computed `href` attribute.

The function does not perform Unicode normalization: the decomposed `a` plus
combining ring remains `a%CC%8A`, distinct from the serializer-owned URI path
that deliberately normalizes to NFC before escaping.

## Boundary

The admitted function form is a compile-time call with one single-quoted string
literal inside the existing leading computed-attribute slice. Variable
arguments, general XPath function dispatch, doubled-quote string escaping, and
direct `xsl:value-of` use remain unsupported. The fold covers the complete
XPath printable-ASCII preservation rule and percent-escapes other characters by
their UTF-8 bytes; it does not reuse the serializer's XML escaping or NFC step.

## Denominator movement

The complete `decl/output` denominator moves from 193 to 196 passes and from 38
to 35 visible default not-run cases; its one profile exclusion is unchanged.
Across the eleven conserved XSLT30 denominators, the total moves from 391 to
394 passes and from 86 to 83 visible default not-run cases, with three
engine-unsupported cases and 51 profile exclusions unchanged.
