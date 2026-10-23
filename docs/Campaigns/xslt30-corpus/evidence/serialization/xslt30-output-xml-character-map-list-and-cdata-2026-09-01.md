# XSLT30 XML character-map lists and CDATA exclusion -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 cases `output-0309` and
`output-0310` and exactly matches both file-backed XML serializations.

`output-0309` selects three ordered character maps and proves their disjoint
replacements are applied to ordinary XML text. `output-0310` proves the
method-specific exclusion: ordinary `a` element text is mapped, while text
inside elements selected by `cdata-section-elements="b"` is emitted as CDATA
without character-map replacement.

Representative sentinels include:

```xml
<a>AAA</a><b><![CDATA[aaa]]></b>
<b><![CDATA[xx#xx]]></b>
```

## Boundary conservation

Map combination remains a compile-time operation over immutable stylesheet
state. CDATA selection remains a serializer concern and does not modify the
semantic result tree. Mapped and CDATA bytes both pass through the same bounded
output sink.

This evidence does not admit named outputs, result documents, general HTML
serialization, imported output declarations, or arbitrary character-map
dependency chains.

## Denominator movement

The complete output denominator moves from 105 to 107 passes and from 127 to
125 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 303 to 305 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 173 visible default not-run cases.
