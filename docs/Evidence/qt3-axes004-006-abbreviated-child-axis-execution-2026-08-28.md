# QT3 `Axes004`–`Axes006` Abbreviated Child-Axis Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes004-1` through `Axes006-4`
- Expressions: `fn:count(//center/*)`, `fn:count(//center/south-east)`,
  and `fn:count(//center/node())`
- Environments: `TreeTrunc`, `Tree1Text`, `Tree1Child`, `TreeCompass`, and
  `TreeRepeat`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all eleven cases, environments, source
documents, expressions, and assertions from the pinned test set. Each source is
imported into a bounded sealed snapshot and built into owned XDM before direct
XPath execution through the private `fn:count` seam.

A focused parser control compares the private typed steps produced for each
abbreviated path against its explicit-axis form:

| Abbreviated | Explicit |
| --- | --- |
| `//center/*` | `//center/child::*` |
| `//center/south-east` | `//center/child::south-east` |
| `//center/node()` | `//center/child::node()` |

Each pair produces the same typed child-step sequence. Execution therefore
uses one navigation and work-accounting path rather than separate abbreviated
and explicit evaluators.

## Result

| Group | Selected | Passed | Failed | Unsupported | Harness error |
| --- | ---: | ---: | ---: | ---: | ---: |
| `Axes004` | 3 | 3 | 0 | 0 | 0 |
| `Axes005` | 4 | 4 | 0 | 0 | 0 |
| `Axes006` | 4 | 4 | 0 | 0 | 0 |
| **Total** | **11** | **11** | **0** | **0** | **0** |

Together, complete `Axes001` through `Axes006` contribute 22 passing
child-axis cases through the same metadata-driven direct XPath seam.

## Claim boundary

This evidence establishes only the abbreviated child-axis equivalents of the
already admitted named-element, any-element, and any-child-node tests. It does
not admit attribute abbreviations, attribute axes, additional kind tests,
namespace wildcards, other axes, or a general XPath parser.
