# XSLT30 character-map QName identity -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0205` and exactly
matches its file-backed XML serialization. The declaration names a character
map as `one:format1`; a referencing map uses `two:format1`. Both prefixes are
bound to `http://mycharmap.example.org`, so compilation resolves them to the
same expanded name and inherits the `$` to `€` mapping.

This is semantic QName identity rather than lexical string matching. Prefixes
remain stylesheet syntax and are not retained as character-map identity in the
compiled output state.

## Boundary conservation

QName expansion is performed at each declaration or reference using that
element's in-scope namespace bindings. An invalid QName is a static error and
an unbound prefix is reported separately from an unknown expanded map name.

This tranche does not admit imported character maps, multiple references from
one map, repeated references, longer composition chains, cycles, declaration
precedence, named outputs, or result documents.

## Denominator movement

The complete output denominator moves from 99 to 100 passes and from 133 to 132
visible default not-run cases. Across the eleven conserved XSLT30 denominators,
the total moves from 297 to 298 passes, with 3 engine unsupported cases, 50
profile exclusions, and 180 visible default not-run cases.
