# XSLT30 XHTML 5 empty elements -- 2026-09-01

## Result

FastXSLT executes eight unchanged W3C XSLT30 cases: `output-0216` through
`output-0223`.

The cases exercise the XHTML 5 void-element set in both the XHTML namespace
and no namespace, with and without attributes. They also prove that empty
`title`, `p`, `i`, `u`, `div`, `code`, and `strong` elements retain explicit
end tags rather than being treated as void. The prefixed XHTML control retains
the previously admitted default-namespace normalization.

The tranche exposed two implementation gaps. The serializer's private void
set omitted `embed`, `source`, `track`, and `wbr`; the output compiler also
selected the method before collapsing surrounding whitespace. Both are now
covered by one executable standards matrix.

## Boundary conservation

The no-namespace void rule is active only in the admitted XHTML 5 mode. It does
not change XML serialization, older XHTML serialization, foreign namespaces,
SVG/MathML handling, HTML output, or general empty-element policy. Method-token
normalization preserves the declaration's original source span for structured
diagnostics.

## Denominator movement

The complete output denominator moves from 119 to 127 passes and from 113 to
105 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 317 to 325 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 153 visible default not-run cases.
