# XSLT30 mode-1407 and mode-1409 upper-case context — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1407` and `mode-1409` combine
`on-no-match="text-only-copy"` with explicit templates whose value expression
is exactly `upper-case(.)`.

## Implemented expression slice

The private value-expression plan now represents `upper-case(.)` directly. At
execution it visits the context node's complete XDM string value, applies
locale-independent Unicode uppercase mapping, charges each input scalar as an
XPath operation, and sends the transformed fragments through the existing
bounded result-text path.

This does not admit general function-call syntax, arbitrary `upper-case`
arguments, function items, collation behavior, or a second expression backend.
The expression owns no retained dynamic state and adds no prepared-engine
capacity beyond its enum discriminator.

## Results

Both native semantic assertions pass without modifying upstream artifacts:

| Case | Evidence |
| --- | --- |
| `mode-1407` | Named mode `s` uses built-in text descent and upper-cases the explicit `chtitle` rule |
| `mode-1409` | The unnamed mode upper-cases every explicitly matched text node |

The conserved 169-case mode denominator advances from 66 to 68 passes, retains
45 profile exclusions, and reduces visible default not-run cases from 58 to
56. Across the 11 conserved XSLT30 denominators, the visible totals become 262
passes, 3 engine-unsupported cases, 50 profile exclusions, and 216 default
not-run cases.
