# XSLT30 character-map static boundary -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0501` and reports
native static error `XTSE0010` because its `xsl:character-map` omits the
required `name` attribute. The structured invalid outcome preserves the
stylesheet resource location.

## Boundary conservation

This validation does not admit character-map execution. A named
`xsl:character-map` remains explicitly unsupported with private code
`FXST1047`; its output-character declarations, QName identity, import
precedence, composition order, duplicate mappings, output-property references,
and serializer replacement rules are not approximated.

The compiler validates the standard-required declaration shape before applying
that broader capability boundary, preventing an unsupported-family diagnostic
from concealing malformed stylesheet input.

## Denominator movement

The complete output denominator moves from 94 to 95 passes and from 138 to 137
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 292 to 293 passes, with 3 engine unsupported cases, 50
profile exclusions, and 185 visible default not-run cases.
