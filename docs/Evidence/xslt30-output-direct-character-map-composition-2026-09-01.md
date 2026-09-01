# XSLT30 direct character-map composition -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 cases `output-0202` and
`output-0203` and exactly matches their file-backed XML serializations.

`output-0202` proves that one named map may directly reference another map and
inherit its `$` to `€` replacement. `output-0203` adds local `#` to `A` and
`$` to `*` mappings, proving that the referencing map's local replacement
overrides the inherited replacement for the same character.

The output method may be explicit XML or omitted and inferred as XML from the
semantic result. Both paths use the same immutable compiled mapping and bounded
serializer.

## Boundary conservation

This tranche admits one direct, unprefixed character-map reference. It does not
claim QName aliases, multiple references, longer composition chains, cycles,
imports, precedence between declarations, named outputs, CDATA interaction, or
non-XML methods. Those shapes remain explicitly unsupported rather than
receiving partial composition semantics.

The serializer still charges mapped replacement bytes through its existing
bounded sink. Local override resolution happens at compile time and does not
add invocation mutation or ambient state.

## Denominator movement

The complete output denominator moves from 96 to 98 passes and from 136 to 134
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 294 to 296 passes, with 3 engine unsupported cases, 50
profile exclusions, and 182 visible default not-run cases.
