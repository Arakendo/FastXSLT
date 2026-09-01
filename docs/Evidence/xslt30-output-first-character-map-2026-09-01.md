# XSLT30 first character-map execution -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0201` and exactly
matches its file-backed XML serialization. One named unprefixed character map
maps `$` to the raw replacement string `€`; the unnamed XML output references
that map and the bounded serializer emits `yy€yy` rather than `yy$yy`.

The mapping is compiled into immutable stylesheet-derived output state. It is
applied while serializing text and attribute values, after semantic result-tree
construction, so character-map execution neither mutates nor becomes part of
the transformation result.

## Boundary conservation

The first slice admits one unprefixed character-map name and one output
reference on XML output. It does not claim QName aliases, referenced/composed
maps, imports, precedence, repeated map properties, named outputs,
`xsl:result-document`, CDATA interaction, or non-XML methods. Those shapes
remain explicitly unsupported rather than receiving partial composition.

Replacement bytes are charged through the existing bounded serialization sink.
The compiled-state retention estimate includes the mapping vector and owned
replacement capacity.

## Denominator movement

The complete output denominator moves from 95 to 96 passes and from 137 to 136
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 293 to 294 passes, with 3 engine unsupported cases, 50
profile exclusions, and 184 visible default not-run cases.
