# QT3 `Axes007`–`Axes011` Attribute-Axis Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes007-1` through `Axes011-3`
- Environments: `TreeTrunc`, `Tree1Child`, and `TreeCompass`
- Forms: `attribute::*`, `attribute::west-attr-2`, `attribute::node()`,
  `@*`, and `@west-attr-2`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all 15 cases, environments, sources,
expressions, and assertions from the pinned QT3 test set. Each source is
imported into a bounded sealed snapshot and converted to owned XDM before
direct XPath execution through the private `fn:count` seam.

The private path representation now records child and attribute choices as
typed steps. Attribute steps obtain candidates only from the XDM attribute
sequence; attributes remain outside the child sequence, and namespace
declarations remain outside the attribute sequence. Named tests require an
unnamespaced attribute with the requested local name. On the attribute axis,
both `*` and `node()` select the axis's principal attribute node kind.

Explicit and abbreviated wildcard/name forms lower to equal typed steps. Every
examined attribute is charged once to the invocation-local XPath node-visit
domain. A focused control also rejects leading `//@*` because the private slice
does not yet implement that abbreviation's descendant-or-self expansion.

## Result

| Group | Meaning | Selected | Passed | Failed | Unsupported | Harness error |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `Axes007` | explicit attribute wildcard | 3 | 3 | 0 | 0 | 0 |
| `Axes008` | explicit named attribute | 3 | 3 | 0 | 0 | 0 |
| `Axes009` | explicit attribute `node()` | 3 | 3 | 0 | 0 | 0 |
| `Axes010` | abbreviated attribute wildcard | 3 | 3 | 0 | 0 | 0 |
| `Axes011` | abbreviated named attribute | 3 | 3 | 0 | 0 | 0 |
| **Total** |  | **15** | **15** | **0** | **0** | **0** |

Complete `Axes001` through `Axes011` now contribute 37 passing child- and
attribute-axis cases through one metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the listed attribute-axis name tests and their
abbreviations. It does not admit namespace-node selection, namespace
wildcards, leading `//` before an attribute step, other axes, additional kind
tests, general QName resolution, or a general XPath parser.
