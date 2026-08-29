# QT3 `Axes080`–`Axes084` Internal-Descendant Attribute Composition

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes080` through `Axes083` groups and `Axes084-1` through
  `Axes084-4` (17 cases)
- Environments: `TreeTrunc`, `Tree1Child`, `TreeCompass`, and `TreeRepeat`
- Forms: explicit and abbreviated wildcard, named, and node attribute tests
  following an internal `//`
- Native assertion: each selected case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all 17 selected cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

No production mechanism was added for this tranche. The typed internal
`descendant-or-self::node()` expansion composes with the existing typed
attribute step. Explicit and abbreviated wildcard and named forms share the
same steps; `attribute::node()` retains the attribute principal node kind.
Owned namespace declarations remain separate and are not attribute candidates.

A focused source expands nested `center` contexts and selects attributes from
the contexts and their descendants. Four owned attributes are returned in
document order. The exact 15-node charge includes the leading search, repeated
nested descendant-or-self traversal, and four attribute inspections; result
deduplication does not erase performed work.

## Result

| Group | Selected | Passed | Disposition |
| --- | ---: | ---: | --- |
| `Axes080` | 3 | 3 | passed |
| `Axes081` | 4 | 4 | passed |
| `Axes082` | 3 | 3 | passed |
| `Axes083` | 3 | 3 | passed |
| `Axes084-1`–`Axes084-4` | 4 | 4 | passed |
| **Selected denominator** | **17** | **17** | **passed** |

The admitted `Axes001` through selected `Axes084` cases now contribute 181
passing location-path cases through the same metadata-driven direct XPath seam.

`Axes084-5` is not selected or counted as passed. It requires the distinct
`//text()[normalize-space()]` boolean-predicate and function semantics and
remains a visible subsequent boundary.

## Claim boundary

This evidence admits only the listed attribute-axis forms after one internal
descendant abbreviation. It does not establish `Axes084-5`, general predicate
effective-boolean-value semantics, `normalize-space`, namespace-sensitive name
tests, generalized attribute ordering from arbitrary input sequences, other
following axes, or a general XPath parser.
