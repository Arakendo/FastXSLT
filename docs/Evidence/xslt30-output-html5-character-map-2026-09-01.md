# XSLT30 HTML 5 character map -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0604` case. The compiled
character map replaces `c` in the copied `value="abcde"` attribute with `[C]`
and `x` in the copied `vwxyz` text node with `[X]`. The result therefore
satisfies both upstream serialization assertions through the HTML 5 method.

## Boundary conservation

The HTML serializer admits this result through a separate validator limited to
one no-namespace `doc` element containing one no-namespace `a` element, one
unnamespaced `value` attribute, and one text child. Admission also requires a
non-empty compiled character map. This does not add `doc`, `a`, or `value` to
the ordinary bounded HTML 5 vocabulary or imply general HTML attributes.

## Denominator movement

The complete output denominator moves from 148 to 149 passes and from 84 to 83
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 346 to 347 passes, with 3 engine unsupported cases, 50
profile exclusions, and 131 visible default not-run cases.
