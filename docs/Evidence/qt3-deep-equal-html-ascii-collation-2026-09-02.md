# QT3 deep-equal HTML ASCII Collation -- 2026-09-02

## Result

Unchanged QT3 cases `K-SeqDeepEqualFunc-64` and
`K-SeqDeepEqualFunc-65` now execute through the private atomic-sequence
`deep-equal` path using the standard
`http://www.w3.org/2005/xpath-functions/collation/html-ascii-case-insensitive`
collation.

The equal case preserves two-item sequence order while treating ASCII letter
case as insignificant. The unequal case reaches and rejects the second item.
Both consume one sequence-length decision plus two reached item comparisons,
and neither performs XDM node visits.

## Boundary

The compiled expression retains an explicit private collation choice. String
comparison uses ASCII-only case folding; non-ASCII code points remain exact.
The existing codepoint collation remains the default and explicit baseline.
Unknown URIs and an empty collation argument remain unsupported.

This slice does not admit host-defined collations, collation resolution,
Unicode case folding, locale behavior, the QT3 private caseblind collation,
function items, or collation-aware node comparison.

## Accounting

The `fn/deep-equal.xml` denominator advances from 152 to 154 passes, retains 67
XQuery-profile exclusions, and reduces visible default not-run cases from 44
to 42. Across the two active QT3 denominators, the conserved subtotal is now
343 passes, 179 profile exclusions, and 90 visible default not-run cases out of
612.

