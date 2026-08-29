# QT3 `Axes055`–`Axes061` Absolute-Axis Composition

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes055` through `Axes061` groups (19 cases)
- Environments: `TreeEmpty`, `TreeTrunc`, `TreeCompass`, `TreeStack`, and
  `TopMany`
- Forms: absolute `self::node()`, descendant any-element/named/any-node, and
  descendant-or-self any-element/named/any-node steps
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all nineteen cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

Every expression uses the existing typed document-node origin and then the
same self, descendant, or descendant-or-self step used by relative paths. No
absolute-only axis representation or evaluator branch was added.

`/self::node()` retains the document node. A descendant step excludes that
origin and visits its owned descendants in document order. A
descendant-or-self step examines the document node before its descendants;
element name and wildcard tests exclude it because it is not an element, while
`node()` retains it. `TopMany` makes the boundary observable: it has 58
descendant nodes and 59 descendant-or-self nodes including the document.

A focused control starts from an unrelated element context and proves that the
leading slash resets evaluation to the document. It verifies document self,
the equal element results of absolute descendant and descendant-or-self
wildcards, and the document-leading node result of
`/descendant-or-self::node()`.

## Result

| Group | Cases | Passed | Disposition |
| --- | ---: | ---: | --- |
| `Axes055` | 1 | 1 | passed |
| `Axes056` | 3 | 3 | passed |
| `Axes057` | 4 | 4 | passed |
| `Axes058` | 3 | 3 | passed |
| `Axes059` | 2 | 2 | passed |
| `Axes060` | 4 | 4 | passed |
| `Axes061` | 2 | 2 | passed |
| **Complete denominator** | **19** | **19** | **passed** |

The admitted `Axes001` through `Axes061` selections now contribute 107 passing
location-path cases through the same metadata-driven direct XPath seam.

Subsequent [`Axes062`–`Axes067` evidence](qt3-axes062-067-leading-descendant-child-forms-2026-08-28.md)
verifies explicit and abbreviated child steps after the leading descendant
origin.

## Claim boundary

This evidence admits only the listed absolute axis forms. It does not establish
all skipped case numbers, generalized absolute expressions, arbitrary axis
composition, predicates on these absolute steps, QName resolution,
namespace-sensitive name tests, namespace nodes, or a general XPath parser.
