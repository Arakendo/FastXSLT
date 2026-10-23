# XSLT30 `xml:space` output control -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0285`. The stylesheet
compiler recognizes `xml:space` on `xsl:output` as an XML-namespaced control
attribute rather than an unknown unqualified serialization parameter, ignores
it for output-property construction, and serializes the unchanged empty `ok`
result.

## Boundary conservation

The admission is local to `xsl:output` and only the expanded name
`{http://www.w3.org/XML/1998/namespace}space`. It does not ignore arbitrary
unqualified attributes, foreign namespaced attributes, or unknown attributes
on other XSLT instructions. Existing `FXST1009` diagnostics therefore remain
the default for attributes outside each instruction's admitted vocabulary.

The control attribute is not retained as serialization metadata because it
does not select serializer behavior in this case.

## Denominator movement

The complete output denominator moves from 93 to 94 passes and from 139 to 138
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 291 to 292 passes, with 3 engine unsupported cases, 50
profile exclusions, and 186 visible default not-run cases.
