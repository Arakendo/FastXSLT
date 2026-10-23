# XSLT30 output character-map declaration merge -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0305` and exactly
matches its file-backed XML serialization. Two unnamed `xsl:output`
declarations contribute character-map names in declaration order. The later
map therefore supplies the effective replacements for characters also present
in the earlier map.

The observed paragraphs are:

```xml
<p>xx&xx</p>
<p>yy*Byy</p>
<p>zz*&zz</p>
```

## Boundary conservation

The bounded merge accepts repeated `method`, `encoding`, and `indent`
properties only when their compiled values are identical. Character-map lists
are concatenated, and the already-admitted CDATA element lists remain unioned.
Different repeated scalar values still report `FXST1018`; the existing negative
test for conflicting XML and XHTML methods remains green.

This tranche covers declarations at one import precedence. Merging output
declarations across imports, named outputs, result documents, and the remaining
serialization properties are separate work.

## Denominator movement

The complete output denominator moves from 107 to 108 passes and from 125 to
124 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 305 to 306 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 172 visible default not-run cases.
