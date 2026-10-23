# XSLT30 `mode-1901` imported-policy override -- 2026-09-02

## Result

FastXSLT executes the unchanged W3C XSLT30 `mode-1901` case and reports its
expected `XTDE0555` dynamic error. The principal stylesheet imports
`mode-0001.xsl`, whose named mode `s` requests `text-only-copy`, then replaces
that mode policy at higher import precedence with `on-no-match="fail"`. The
imported named initial template reaches its parentless temporary comment and
the effective principal policy rejects the unmatched node.

## Boundary and authority

The corpus adapter explicitly admits the principal and its exact imported
stylesheet into one bounded sealed snapshot. Relative dependency resolution
occurs inside FastXSLT without filesystem fallback; the engine does not reopen
either source path during compilation or execution.

This case establishes higher-precedence replacement for the admitted
`on-no-match` policy and reuse of an imported named template. It does not admit
package-level mode overriding, general mode-component merging, or broader
visibility and accumulator semantics.

## Conservation

The complete 169-case `attr/mode` denominator moves from 85 to 86 passes and
from 39 to 38 visible default not-run cases; its 45 profile exclusions remain
unchanged. Across the eleven conserved XSLT30 denominators, the total moves
from 405 to 406 passes and from 72 to 71 visible default not-run cases, with
three engine-unsupported cases and 51 profile exclusions unchanged.
