# XSLT30 HTML ins/del whitespace -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0160` case. With HTML output
and `indent="no"`, the result preserves the newline, spaces, and tab characters
inside `del`, preserves the authored spaces inside `ins`, and emits the two
elements adjacently because the whitespace-only stylesheet node between the
result instructions does not construct a text node.

The executable sentinel checks the significant substring from `is ` through
` pieces`; the W3C assertion permits platform line-ending variation.

## Admission boundary

The HTML validator admits only one attribute-free, no-namespace
`html/head/body/p/del/ins` hierarchy. `head`, `del`, and `ins` may contain only
text, `body` contains the one paragraph, and the paragraph contains exactly one
`del` followed by one `ins` among its text nodes. This is not admission of a
general HTML result vocabulary or indentation implementation.

## Denominator movement

The complete output denominator moves from 172 to 173 passes and from 59 to 58
visible default not-run cases; its one profile exclusion is unchanged. Across
the eleven conserved XSLT30 denominators, the total moves from 370 to 371 passes
and from 107 to 106 visible default not-run cases, with 3 engine-unsupported
cases and 51 profile exclusions unchanged.
