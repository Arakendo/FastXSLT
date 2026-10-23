# XSLT30 HTML 5 void elements -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 `output-0601` case. All sixteen
upstream HTML 5 void-element assertions pass: `area`, `base`, `br`, `col`,
`command`, `embed`, `hr`, `img`, `input`, `keygen`, `link`, `meta`, `param`,
`source`, `track`, and `wbr` serialize with start tags and no end tags.

The harness imports the upstream file-backed source into the sealed snapshot
and closes its handle before compilation and execution. The stylesheet's
`xsl:copy-of select="*"` compiles to a distinct child-element copy operation;
child selection is charged and recursive result construction reuses the
existing bounded source-copy path.

## Boundary conservation

The HTML 5 result remains one no-namespace document element with no attributes
or namespace nodes. The admitted vocabulary adds only the standard void names
to the existing `html`, `head`, `title`, `body`, and `p` slice.
`escape-uri-attributes="yes"` is inert because every admitted element is
attribute-free. HTML attributes, URI escaping, raw-text elements, arbitrary
elements, and namespace fixup remain unsupported.

`xsl:copy-of` adds only the exact child-element wildcard `*`; arbitrary XPath
selections remain unsupported.

## Denominator movement

The complete output denominator moves from 141 to 142 passes and from 91 to 90
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 339 to 340 passes, with 3 engine unsupported cases, 50
profile exclusions, and 138 visible default not-run cases.
