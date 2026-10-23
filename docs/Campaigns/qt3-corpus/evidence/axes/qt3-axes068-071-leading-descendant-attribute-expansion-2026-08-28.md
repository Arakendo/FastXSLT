# QT3 `Axes068`–`Axes071` Leading-Descendant Attribute Expansion

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes068` through `Axes071` groups (12 cases)
- Environments: `TreeTrunc`, `TreeEmpty`, and `TreeCompass`
- Explicit forms: `//attribute::*` and `//attribute::mark`
- Abbreviated forms: `//@*` and `//@mark`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all twelve cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

A leading `//` now resets evaluation to the owned document node independently
of the caller's current context. When its first typed step is an attribute
step, evaluation does not search for attributes as descendant nodes. It first
visits the document descendants in document order, then inspects each node's
owned attributes. Namespace declarations remain a separate XDM-owned sequence
and therefore never enter the attribute candidates.

Explicit and abbreviated wildcard forms lower to the same typed attribute
step, as do their named forms. Unprefixed `mark` retains the existing
no-namespace name-test boundary. Traversed descendant nodes and examined
attributes are each charged once; filtering does not add another charge.

A focused source contains two element descendants, three attributes, and a
namespace declaration. Starting from the root element rather than the document
proves the absolute context reset. `//@*` returns three attributes,
`//@plain` returns two, the namespace declaration is excluded, and the exact
work charge is five: two descendant visits plus three attribute visits.

## Result

| Group | Cases | Passed | Disposition |
| --- | ---: | ---: | --- |
| `Axes068` | 3 | 3 | passed |
| `Axes069` | 3 | 3 | passed |
| `Axes070` | 3 | 3 | passed |
| `Axes071` | 3 | 3 | passed |
| **Complete denominator** | **12** | **12** | **passed** |

The admitted `Axes001` through `Axes071` selections now contribute 137 passing
location-path cases through the same metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the listed leading descendant attribute forms. It
does not establish namespace-node axes, QName resolution, namespace-sensitive
attribute name tests, arbitrary abbreviated path composition, predicates,
generalized attribute ordering across arbitrary input sequences, other axes,
or a general XPath parser.

Subsequent [`Axes072`–`Axes073` evidence](qt3-axes072-073-leading-descendant-self-expansion-2026-08-28.md)
adds the axis-distinct leading self expansion without changing this attribute
claim.
