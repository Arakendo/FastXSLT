# XSLT30 HTML 5 attribute namespaces -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0603a`, `output-0603b`, and
`output-0603c` cases. SVG- and MathML-qualified attributes retain their authored
prefixes and the corresponding local namespace declarations after their
MathML element names normalize to a default namespace. The unrelated
`NamespaceM` binding and `m:zzz` attribute remain qualified on the HTML `p`
element.

Known XHTML/SVG/MathML prefix bindings are removed when they serve only element
names. A prefixed binding survives normalization when a qualified attribute on
that element consumes the binding, so element and attribute namespace rules do
not become conflated.

## Boundary conservation

The admitted namespaced attributes are limited to the exact SVG, MathML, and
`NamespaceM` names in the upstream source. The existing fixed unnamespaced
geometry/presentation attributes remain unchanged. HTML URI attributes,
arbitrary attribute namespaces, generated prefixes, and general namespace
fixup remain unsupported.

## Denominator movement

The complete output denominator moves from 145 to 148 passes and from 87 to 84
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 343 to 346 passes, with 3 engine unsupported cases, 50
profile exclusions, and 132 visible default not-run cases.
