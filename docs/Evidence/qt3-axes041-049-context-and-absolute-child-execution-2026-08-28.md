# QT3 `Axes041`–`Axes049` Context and Absolute-Child Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes041-1`, both `Axes043` cases, and complete `Axes044` through
  `Axes049` groups (15 cases)
- Environments: `TreeCompass`, `TreeTrunc`, `Tree1Text`, `TopMany`, and
  `TreeEmpty`
- Context forms: attribute/text followed by `descendant-or-self::node()`
- Absolute forms: explicit and abbreviated child any-element, named-element,
  and any-node tests
- Native assertion: each case's pinned `assert-eq`

The absent case numbers between the selected groups are not part of this
denominator.

## Method

The metadata-driven axis test resolves all fifteen cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

`Axes041` and `Axes043` apply the already typed descendant-or-self node step to
attribute and text contexts. Neither node kind has children, so the axis
retains the supplied node identity as its only result. The empty-text case
first produces no context and therefore no result.

`Axes044` through `Axes049` begin at the owned document node and compare
explicit `child::` forms with their abbreviated equivalents. Any-element tests
select only the document element; named tests preserve the unprefixed expanded
name restriction; any-node tests retain all document children. `TopMany`
supplies seven top-level nodes, proving that document-level comments,
processing instructions, and text remain visible to `node()` but not `*`.

Focused controls verify attribute/text descendant-or-self identity and prove
that `/child::*` equals `/*`, while `/child::node()` equals `/node()`, through
one typed document-origin evaluation path.

## Result

| Group | Cases | Passed | Disposition |
| --- | ---: | ---: | --- |
| `Axes041` | 1 | 1 | passed |
| `Axes043` | 2 | 2 | passed |
| `Axes044` | 2 | 2 | passed |
| `Axes045` | 2 | 2 | passed |
| `Axes046` | 2 | 2 | passed |
| `Axes047` | 2 | 2 | passed |
| `Axes048` | 2 | 2 | passed |
| `Axes049` | 2 | 2 | passed |
| **Selected denominator** | **15** | **15** | **passed** |

The admitted `Axes001` through `Axes049` selections now contribute 88 passing
location-path cases through the same metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the listed context and absolute child forms. It does
not establish all skipped case numbers, generalized absolute expressions,
document-node predicates, QName resolution, namespace-sensitive name tests,
namespace nodes, generalized path normalization across arbitrary axes, or a
general XPath parser.
