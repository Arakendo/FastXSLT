# QT3 `Axes034`–`Axes037` Descendant-or-Self Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes034`, `Axes035`, `Axes036`, and `Axes037` groups
  (10 cases)
- Environments: `TreeTrunc`, `Tree1Child`, `TreeCompass`, `TreeStack`, and
  `TreeRepeat`
- Forms: `descendant-or-self::*`, named `descendant-or-self::south` and
  `descendant-or-self::center`, and `descendant-or-self::node()`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all ten cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

The typed location-step representation now distinguishes named-element,
any-element, and any-node tests on the descendant-or-self axis. A step examines
the context first, followed by its descendants in document order. The element
wildcard applies the axis's principal element node kind; `node()` retains all
owned context and descendant node kinds. Every examined candidate is charged
once by the traversal and is not charged again while filtering.

`TreeRepeat` supplies nested `center` input contexts. Their
descendant-or-self results overlap, so the step retains the first document-order
occurrence of each XDM identity and omits later occurrences. The repeated
traversal work remains charged even where its result identity has already been
seen.

A focused three-level nested-center control contrasts descendant with
descendant-or-self, verifies three results rather than six overlapping
occurrences, checks their document order, and checks ten exact node visits:
four for the leading descendant search and six across the three overlapping
descendant-or-self traversals. It also rejects the unimplemented
`descendant-or-self::text()` form explicitly.

## Result

| Case | Expected | Actual | Disposition |
| --- | ---: | ---: | --- |
| `Axes034-1` | 1 | 1 | passed |
| `Axes034-2` | 6 | 6 | passed |
| `Axes035-1` | 0 | 0 | passed |
| `Axes035-2` | 0 | 0 | passed |
| `Axes035-3` | 1 | 1 | passed |
| `Axes035-4` | 8 | 8 | passed |
| `Axes036-1` | 1 | 1 | passed |
| `Axes036-2` | 9 | 9 | passed |
| `Axes037-1` | 1 | 1 | passed |
| `Axes037-2` | 22 | 22 | passed |

The admitted `Axes001` through `Axes037` selections now contribute 73 passing
location-path cases through the same metadata-driven direct XPath seam.

Subsequent [selected `Axes041`–`Axes049` evidence](qt3-axes041-049-context-and-absolute-child-execution-2026-08-28.md)
closes attribute/text descendant-or-self context pressure and verifies explicit
and abbreviated absolute child forms.

## Claim boundary

This evidence admits only the listed descendant-or-self step forms and their
unnamespaced name tests. Duplicate elimination is evidenced for overlapping
forward descendant-or-self results; it does not establish generalized path
normalization across arbitrary forward and reverse axis compositions. This
evidence also does not establish `descendant-or-self::text()` or other kind
tests, QName resolution, namespace-sensitive name tests, attributes or
namespace nodes on this axis, other axes, or a general XPath parser.
