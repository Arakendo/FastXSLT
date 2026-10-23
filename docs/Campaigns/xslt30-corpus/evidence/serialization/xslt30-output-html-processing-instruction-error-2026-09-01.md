# XSLT30 HTML processing-instruction delimiter error -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0196` case. When a result
processing instruction contains `>` in its data and the selected method is
HTML, serialization reports native `SERE0015` with invalid-input classification
and request identity.

The validation traverses the bounded semantic result tree before general HTML
shape selection, so the standards-defined error is not hidden behind a private
capability diagnostic.

## Boundary conservation

The rule is active only for the recognized HTML output method. XML and XHTML
processing-instruction serialization remains unchanged, and this negative case
does not admit general HTML result serialization.

## Denominator movement

The complete output denominator moves from 138 to 139 passes and from 94 to 93
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 336 to 337 passes, with 3 engine unsupported cases, 50
profile exclusions, and 141 visible default not-run cases.
