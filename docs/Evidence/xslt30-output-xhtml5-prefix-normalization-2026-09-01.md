# XSLT30 XHTML 5 prefix normalization -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0211` case. Under the
admitted XHTML method with `html-version="5"`, the serializer emits the
automatic `html` doctype and serializes XHTML-namespace elements through the
default XHTML namespace rather than retaining the authored `h` prefix.

The executable regression asserts that the result contains the default XHTML
namespace, contains unprefixed `html`, `head`, and `body` elements, and contains
neither an `xmlns:h` declaration nor an `h:` lexical name.

## Boundary conservation

This is a bounded XHTML 5 element-name rule. It does not establish general
namespace normalization, namespace fixup for copied trees, XHTML treatment of
SVG or MathML descendants, foreign-namespace attribute rewriting, or the
remaining XHTML 5 serialization rules. The normalization is derived during
serialization from immutable result-tree namespace bindings and introduces no
retained or cross-invocation state.

## Denominator movement

The complete output denominator moves from 118 to 119 passes and from 114 to
113 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 316 to 317 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 161 visible default not-run cases.
