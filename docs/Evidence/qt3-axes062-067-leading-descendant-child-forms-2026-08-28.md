# QT3 `Axes062`–`Axes067` Leading-Descendant Child Forms

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes062` through `Axes067` groups (18 cases)
- Environments: `TreeEmpty`, `TreeTrunc`, `TreeCompass`, `TreeStack`, and
  `TopMany`
- Explicit forms: `//child::*`, `//child::south`, and `//child::node()`
- Abbreviated forms: `//*`, `//south`, and `//node()`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all eighteen cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

The leading `//` form uses the existing typed descendant origin. Explicit
`child::` syntax and its abbreviated form lower to the same typed any-element,
named-element, or any-node step. The origin traverses owned descendants in
document order and charges each visited node once; the step's name/kind filter
does not charge the same candidate again.

`TopMany` proves that `//node()` and `//child::node()` both retain 58
descendants while excluding the document node. Element wildcard forms retain
only elements, and the unprefixed named form does not widen its expanded-name
boundary.

A focused mixed-node control compares the explicit and abbreviated typed
steps, verifies three selected elements and one named nested element, retains
five node kinds in document order, and checks an exact five-visit charge.

## Result

| Group | Cases | Passed | Disposition |
| --- | ---: | ---: | --- |
| `Axes062` | 2 | 2 | passed |
| `Axes063` | 4 | 4 | passed |
| `Axes064` | 3 | 3 | passed |
| `Axes065` | 2 | 2 | passed |
| `Axes066` | 4 | 4 | passed |
| `Axes067` | 3 | 3 | passed |
| **Complete denominator** | **18** | **18** | **passed** |

The admitted `Axes001` through `Axes067` selections now contribute 125 passing
location-path cases through the same metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the listed leading descendant child forms. In
particular, it does not admit the following `Axes068` leading attribute form:
attributes are not descendant nodes, so correct `//attribute::*` semantics
must expand across descendant-or-self element contexts and then apply the
attribute axis. This evidence also does not establish arbitrary abbreviated
path composition, predicates, QName resolution, namespace-sensitive name
tests, namespace nodes, or a general XPath parser.
