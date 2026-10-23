# XSLT30 unused named output declaration -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0134` case. The stylesheet's
unused named `text` output declaration is parsed and validated separately from
the unnamed principal `xhtml` declaration. The implicit final result therefore
uses the principal format and satisfies both native serialization patterns.

## Boundary

This slice admits a uniquely named, otherwise supported output declaration only
when no implemented instruction consumes it. It does not retain a public named
format table, admit `xsl:result-document`, merge duplicate named declarations,
or resolve character-map references for an unused named declaration. Those
features remain explicit future work rather than being inferred from principal
result support.

## Denominator movement

The complete `decl/output` denominator moves from 192 to 193 passes and from 39
to 38 visible default not-run cases; its one profile exclusion is unchanged.
Across the eleven conserved XSLT30 denominators, the total moves from 390 to
391 passes and from 87 to 86 visible default not-run cases, with three
engine-unsupported cases and 51 profile exclusions unchanged.
