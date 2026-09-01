# XSLT30 default-mode variants -- 2026-08-31

## Result

FastXSLT executes the unchanged W3C XSLT30 cases `mode-1433`, `mode-1434`,
and `mode-1435` from the pinned mode test set.

The cases conserve three related forms of default-mode behavior:

- `mode-1433` uses named mode `dm` as both the stylesheet's default initial
  mode and the inherited mode of unqualified template rules and invocations;
- `mode-1434` explicitly selects `#unnamed` and retains it through
  `#current` descent;
- `mode-1435` proves whitespace normalization and expanded-QName identity by
  spelling the same namespace-qualified mode with three lexical forms.

All three execute over the unchanged external `mode-14.xml` source and match
their native inline XML assertions.

## Compiler conservation

The shared stylesheet rule `v | chapter/text()` revealed that a union cannot
be assigned one synthetic default priority. FastXSLT now lowers only
provably-disjoint admitted union alternatives into separate private rules:
`v` retains exact-name priority while `chapter/text()` retains path priority.
Potentially overlapping unions remain explicitly unsupported.

The same tranche normalizes whitespace in `default-mode`, maps an explicit
`xsl:apply-templates mode="#unnamed"` to unnamed dispatch, and admits the
standard `exclude-result-prefixes` attribute on `xsl:template`. Focused tests
conserve the individual priorities, unnamed invocation, normalized default,
and explicit rejection of an overlapping union.

## Denominator movement

The complete mode denominator moves from 73 to 76 passes, with 45 profile
exclusions and 48 visible default not-run cases. Across the eleven conserved
XSLT30 denominators, the total moves from 267 to 270 passes, with 3 engine
unsupported cases, 50 profile exclusions, and 208 visible default not-run
cases.
