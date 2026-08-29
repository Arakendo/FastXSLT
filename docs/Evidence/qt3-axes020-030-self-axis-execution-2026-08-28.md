# QT3 `Axes020`–`Axes030` Selected Self-Axis Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: `Axes020-1`, `Axes021-1`, `Axes023-1`, `Axes027-1`, `Axes030-1`,
  and `Axes030-2`
- Environment: `TreeCompass`
- Forms: `self::*`, named `self::` tests, and `self::node()`
- Context kinds: element, attribute, and text
- Native assertion: each case's pinned `assert-eq`

The gaps in case numbering are not part of this selected denominator.

## Method

The metadata-driven axis test resolves the six selected cases, their
environment and source, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

The typed location-step representation now distinguishes named-element,
any-element, and any-node tests on the self axis. A self step examines exactly
the current context node and charges that candidate once. The element wildcard
applies the self axis's principal element node kind, while `node()` retains an
element, attribute, or text context. A typed child `text()` step supplies the
text-node contexts required by `Axes030` without treating text as an element or
widening kind-test support on other axes.

A focused control executes all three context kinds, proves that `self::*`
rejects an attribute while `self::node()` retains it, and checks an exact
single node-visit charge. It also verifies that unimplemented `text()` kind
tests on the attribute, parent, and self axes report an unsupported path at
lowering rather than silently returning an empty sequence.

## Result

| Case | Expected | Actual | Disposition |
| --- | ---: | ---: | --- |
| `Axes020-1` | 1 | 1 | passed |
| `Axes021-1` | 1 | 1 | passed |
| `Axes023-1` | 1 | 1 | passed |
| `Axes027-1` | 1 | 1 | passed |
| `Axes030-1` | 0 | 0 | passed |
| `Axes030-2` | 1 | 1 | passed |

The admitted `Axes001` through `Axes030` selections now contribute 51 passing
location-path cases through the same metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the six listed cases, unprefixed named and wildcard
self element tests, `self::node()`, and the child `text()` kind test needed by
`Axes030`. It does not establish `self::text()`, other kind tests, QName
resolution, namespace-sensitive name tests, general sequence type checking,
the unselected numbering gaps, broader axis ordering, or a general XPath
parser.
