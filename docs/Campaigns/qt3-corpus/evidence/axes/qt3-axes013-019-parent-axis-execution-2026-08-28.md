# QT3 `Axes013`–`Axes019` Parent-Axis Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes013-1` through `Axes019-1`
- Environment: `TreeCompass`
- Forms: `parent::*`, named `parent::` tests, `parent::node()`, and `..`
- Absolute-path pressure: `/far-north/parent::*` and
  `/far-north/parent::node()`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all seven cases, the referenced
environment and source, expressions, and assertions from the pinned QT3 test
set. The source is imported into a bounded sealed snapshot and built into owned
XDM before direct XPath execution through the private `fn:count` seam.

The typed location-step representation now distinguishes named-element,
any-element, and any-node tests on the parent axis. A parent step obtains at
most the context node's owned XDM parent and charges that examined candidate
once. The element wildcard applies the parent axis's principal element node
kind; `node()` accepts the document node as well. The `..` abbreviation lowers
to the same typed step as `parent::node()`.

A document-node origin with following steps also admits the two exact absolute
paths. A focused control starts from an unrelated element context, verifies
that `/root` begins at the document node, verifies the element/document parent
distinction, compares explicit and abbreviated parent steps, and checks one
exact parent-visit charge.

## Result

| Case | Expected | Actual | Disposition |
| --- | ---: | ---: | --- |
| `Axes013-1` | 1 | 1 | passed |
| `Axes014-1` | 0 | 0 | passed |
| `Axes015-1` | 1 | 1 | passed |
| `Axes016-1` | 0 | 0 | passed |
| `Axes017-1` | 1 | 1 | passed |
| `Axes018-1` | 1 | 1 | passed |
| `Axes019-1` | 1 | 1 | passed |

Complete `Axes001` through `Axes019` now contribute 45 passing location-path
cases through the same metadata-driven direct XPath seam.

Subsequent [`Axes020`–`Axes030` evidence](qt3-axes020-030-self-axis-execution-2026-08-28.md)
adds the six admitted self-axis cases, including attribute and text-node
contexts.

## Claim boundary

This evidence admits only the listed parent steps, their unprefixed name tests,
the `..` abbreviation, and absolute child paths needed by these cases. It does
not establish general reverse-axis ordering, duplicate elimination across
multiple shared parents, parent-axis predicates or positions, QName resolution,
other reverse axes, or a general XPath parser.
