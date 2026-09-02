# XSLT30 HTML 5 doctype placement -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0233` case. For the admitted
HTML 5 result shape, serialization emits the automatic `<!DOCTYPE html>` after
root comments and immediately before the document element.

The upstream serialization assertion passes against the native stylesheet,
environment, and expected regular-expression fragment without fixture changes.

## Boundary conservation

The successful HTML slice accepts one no-namespace `html` document element,
root whitespace and miscellaneous nodes, and only the `head`, `title`, `body`,
and `p` element vocabulary without attributes or namespace nodes. Processing
instructions containing `>` still report `SERE0015` before shape selection.

The slice does not claim general HTML serialization, void-element handling,
raw-text elements, URI escaping, arbitrary attributes, namespace fixup, or
other HTML versions. Results outside the admitted shape remain explicitly
unsupported.

## Denominator movement

The complete output denominator moves from 139 to 140 passes and from 93 to 92
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 337 to 338 passes, with 3 engine unsupported cases, 50
profile exclusions, and 140 visible default not-run cases.
