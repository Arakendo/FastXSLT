# XSLT30 repeated character-map references -- 2026-09-01

## Result

FastXSLT executes the unchanged W3C XSLT30 case `output-0206` and exactly
matches its file-backed XML serialization. One character map references the
same expanded map identity three times using two lexical prefixes. Compilation
resolves the complete reference list and combines the repeated mapping
idempotently, producing `yy€yy` exactly once.

## Boundary conservation

Character-map reference lists may now contain multiple QName entries and retain
their declared order. Each directly referenced map must still be terminal:
longer composition chains and cycles remain explicitly unsupported. Imported
maps, declaration precedence, named outputs, and result documents also remain
outside this tranche.

Repeated references do not duplicate runtime lookup state because compilation
merges by mapped character before producing immutable output settings.

## Denominator movement

The complete output denominator moves from 100 to 101 passes and from 132 to
131 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 298 to 299 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 179 visible default not-run cases.
