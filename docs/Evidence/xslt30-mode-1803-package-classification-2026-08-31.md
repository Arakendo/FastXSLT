# XSLT30 mode-1803 package classification — 2026-08-31

## Scope

The conserved XSLT30 `attr/mode` denominator previously left `mode-1803` in
the default not-run population. The native catalog represents its principal
artifact with a `stylesheet` entry, rather than the `package` entry used by the
other package cases in this test set.

## Finding

The referenced `mode-1803.xsl` document has `xsl:package` as its document
element. The catalog artifact label therefore does not change the semantic
capability required to compile the case.

ADR-0007 excludes XSLT packages from the current profile. The first-party
overlay now classifies `mode-1803` as `excluded-by-profile`, and the inventory
test preserves the exceptional native catalog shape explicitly instead of
assuming every package case uses a `package` entry.

## Accounting result

The conserved 169-case mode denominator retains 65 passes, advances from 44 to
45 profile exclusions, and reduces visible default not-run cases from 60 to
59. Across the 11 conserved XSLT30 denominators, the visible totals become 259
passes, 3 engine-unsupported cases, 50 profile exclusions, and 219 default
not-run cases.

No upstream file, expected result, or test-set metadata was changed.
