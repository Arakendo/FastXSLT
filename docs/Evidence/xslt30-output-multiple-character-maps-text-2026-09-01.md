# XSLT30 multiple character maps for text output -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0303` and exactly
matches its file-backed text serialization. The unnamed output declaration
selects three unprefixed character maps. Their disjoint `$` to `A`, `#` to
`&`, and `*` to `+` replacements are combined at compile time and applied to
the text value of the semantic result tree.

The result is exactly:

```text
xx&xx
yy+Ayy
zz%&zz
```

Mapped text is written through the same bounded output sink as ordinary text;
the replacement strings are not XML-escaped because the selected method is
text.

## Boundary conservation

This tranche admits a whitespace-separated list of unprefixed map names on
text output. QName map identity, duplicate references, imported maps,
declaration precedence, named outputs, result documents, and composition
chains remain outside the admitted corpus slice. The adjacent XHTML and HTML
cases are not counted by this evidence; their upstream assertions and
method-specific serializer behavior require independent admission.

## Denominator movement

The complete output denominator moves from 98 to 99 passes and from 134 to 133
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 296 to 297 passes, with 3 engine unsupported cases, 50
profile exclusions, and 181 visible default not-run cases.
