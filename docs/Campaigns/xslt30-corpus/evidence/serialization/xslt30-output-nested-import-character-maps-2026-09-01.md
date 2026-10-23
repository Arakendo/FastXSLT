# XSLT30 nested-import character maps -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0306` and exactly
matches its file-backed XML serialization. The sealed stylesheet graph is a
three-level import chain: the principal module imports `output-0306y.xsl`,
which imports `output-0306z.xsl`.

The case proves that character-map declarations retain import precedence
through nested program composition. The principal `format1` declaration wins
over both imported definitions, while `format2` is inherited from the
intermediate imported module. Two principal `xsl:output` declarations then
compose those map names in declaration order.

## Bounded graph support

The resource compiler now recognizes exactly one principal import containing
exactly one leaf import. It first compiles the imported pair, then rebases that
complete program beneath the principal module. The same private declaration
merge and final character-map resolution used by direct imports remains the
only semantic path.

Arbitrary-depth import graphs, mixed nested include/import graphs, cycles,
imported output declarations, named outputs, and result documents remain
outside this tranche. Compilation and execution use only immutable resources
already admitted to the sealed snapshot.

## Denominator movement

The complete output denominator moves from 108 to 109 passes and from 124 to
123 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 306 to 307 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 171 visible default not-run cases.
