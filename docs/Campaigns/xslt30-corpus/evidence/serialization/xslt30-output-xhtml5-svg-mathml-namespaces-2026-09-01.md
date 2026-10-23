# XSLT30 XHTML 5 SVG and MathML namespaces -- 2026-09-01

## Result

FastXSLT executes three unchanged W3C XSLT30 cases: `output-0224` through
`output-0226`.

The XHTML 5 serializer now emits XHTML, SVG, and MathML element names using the
appropriate default namespace whether the stylesheet authored those names with
default bindings, explicit prefixes, or a mixture. Prefix bindings for these
three special namespaces do not leak into normalized descendants; ordinary
attributes and foreign content remain intact.

## Boundary conservation

Normalization remains private to the admitted XHTML method with version five
and exactly the XHTML, SVG, and MathML namespaces. It does not rewrite arbitrary
foreign namespaces or namespaced attributes, alter XML serialization, or claim
general namespace-fixup semantics. Serialization derives the bindings from the
immutable result tree and retains no cross-invocation state.

## Denominator movement

The complete output denominator moves from 131 to 134 passes and from 101 to 98
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 329 to 332 passes, with 3 engine unsupported cases, 50
profile exclusions, and 146 visible default not-run cases.
