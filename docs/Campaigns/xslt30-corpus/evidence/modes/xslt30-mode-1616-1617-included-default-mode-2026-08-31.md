# XSLT30 mode-1616 and mode-1617 included default-mode — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1616` and `mode-1617` exercise the
`default-mode` static context of an included stylesheet module.

## Resource and compilation path

The suite adapter reads the catalog's principal and secondary stylesheet
references, admits both resources into one bounded sealed snapshot, and closes
the source files before compilation. Relative `xsl:include` resolution uses the
existing snapshot resolver; no filesystem or network authority reaches the
engine.

The included module compiles under its own stylesheet root. Its
`default-mode="a"` therefore assigns its unmoded `match="a"` rule to mode `a`
before the compiled declarations are merged at the principal module's
precedence. The principal module's absence of a default mode neither erases nor
reinterprets that identity.

## Results

Both native XML assertions pass without changing upstream resources or
expected results:

| Case | Result |
| --- | --- |
| `mode-1616` | Explicit principal `mode="a"` selects the included rule compiled under its module default |
| `mode-1617` | Principal literal-result `xsl:default-mode="a"` selects the same included rule |

The conserved 169-case mode denominator advances from 61 to 63 passes, retains
44 profile exclusions, and reduces visible default not-run cases from 64 to 62.
Across the 11 conserved XSLT30 denominators, the visible totals become 257
passes, 3 engine-unsupported cases, 49 profile exclusions, and 222 default
not-run cases.

## Boundaries

This evidence covers one included leaf module and the existing sealed-resource
composition path. It does not establish packages, arbitrary include graphs, or
cross-module function and attribute-set static contexts.
