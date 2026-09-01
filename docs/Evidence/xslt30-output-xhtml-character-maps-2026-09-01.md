# XSLT30 XHTML character maps -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0301`. Its XHTML
serialization satisfies both upstream `all-of` patterns: the result has the
XHTML `html` root and namespace, and its ordered paragraph content is
`xx&xx`, `yy+Ayy`, and `zz%&zz` with the required whitespace separators.

The three selected maps replace `#` with raw `&`, `*` with `+`, and `$` with
`A`. The raw ampersand is intentional character-map output and is not escaped
again by the XHTML serializer.

## Assertion ownership

The focused test owns the bounded assertion shape without introducing a
general regular-expression engine. It checks the exact XHTML root binding,
ordered mapped paragraphs, whitespace-only separators, document closure, and
absence of the original `yy*$yy` content. This directly conserves the two
pinned patterns while leaving unrelated regex syntax outside the harness.

## Boundary conservation

This tranche does not admit the adjacent HTML method case, CDATA interaction,
named outputs, result documents, imported maps, or declaration precedence.
All replacement bytes continue through the bounded serializer sink.

## Denominator movement

The complete output denominator moves from 101 to 102 passes and from 131 to
130 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 299 to 300 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 178 visible default not-run cases.
