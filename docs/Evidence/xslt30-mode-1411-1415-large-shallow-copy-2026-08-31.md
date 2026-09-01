# XSLT30 mode-1411 and mode-1415 large shallow-copy — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1411` and `mode-1415` exercise
`on-no-match="shallow-copy"` over the suite's complete external `mode-14.xml`
source and compare against their native file-backed XML results.

`mode-1411` enters named mode `s`, retains the explicit `typed="no"` control,
and relies entirely on shallow-copy built-in rules. `mode-1415` uses the
unnamed mode and composes those rules with explicit text-node upper-casing and
empty overrides for `v` elements and `chapter/text()` nodes.

## Verification boundary

- The source, stylesheets, and expected results are read from the pinned
  immutable XSLT30 submodule without modification.
- Both cases use the existing sealed in-memory resource snapshot and ordinary
  compile/transform lifecycle.
- The existing 16 KiB case-specific result ceiling covers each roughly 9 KiB
  output without weakening the engine-wide bounded serialization contract.
- Parsed-XML comparison checks the complete result trees, including copied
  attributes and the explicit template overrides in `mode-1415`.
- The existing smaller `mode-1445` and `mode-1446` controls continue to cover
  both accepted false lexicals for `typed`.

## Result

Both unchanged cases pass. The conserved 169-case mode denominator advances
from 68 to 70 passes, retains 45 profile exclusions, and reduces visible
default not-run cases from 56 to 54. Across the 11 conserved XSLT30
denominators, the totals become 264 passes, 3 engine-unsupported cases, 50
profile exclusions, and 214 visible default not-run cases.

## Boundary retained

The intervening non-streaming `mode-1413` case remains visible. It requires an
explicit attribute template to intercept shallow-copy's attribute processing
and construct a replacement numeric attribute. This tranche does not infer
that behavior merely because complete shallow-copy succeeds when attributes
are copied unchanged.
