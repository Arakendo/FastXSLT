# XSLT30 imported empty doctype override -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0312` case. The principal
stylesheet's explicit empty `doctype-system` and `doctype-public` values
override the lower-precedence imported values, so the exact serialized result
is `<a><b/></a>` with no XML declaration or doctype.

The compiler retains the set of explicitly declared unnamed output properties
as private compiled metadata. An imported output declaration is admitted only
when every one of its properties is explicitly shadowed by the principal
declaration. This proves the complete-override rule without silently treating
an absent principal property as an override.

## Boundary conservation

Imported output properties that require inheritance or partial merging still
report `FXST1024`. Named output declarations and `xsl:result-document` remain
outside this slice, so sibling `output-0313` is not promoted.

Explicit empty doctype identifiers remain distinguishable in compiled metadata
from absent properties, but serialize no doctype. XML empty elements use their
empty-element tag form in this doctype-parameter lane.

## Denominator movement

The complete output denominator moves from 140 to 141 passes and from 92 to 91
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 338 to 339 passes, with 3 engine unsupported cases, 50
profile exclusions, and 139 visible default not-run cases.
