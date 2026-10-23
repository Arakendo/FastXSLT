# QT3 `Axes031`–`Axes033` Descendant-Axis Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes031`, `Axes032`, and `Axes033` groups (12 cases)
- Environments: `TreeTrunc`, `Tree1Text`, `Tree1Child`, `TreeCompass`, and
  `TreeStack`
- Forms: `descendant::*`, named `descendant::south`, and
  `descendant::node()`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all twelve cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

The typed location-step representation now distinguishes named-element,
any-element, and any-node tests on the descendant axis. The step traverses all
owned child nodes depth first in document order, excluding the context node
and attributes. `descendant::*` applies the axis's principal element node kind;
`descendant::node()` retains element, text, comment, and
processing-instruction descendants. Each traversed candidate is charged once
by the traversal and is not charged again during its name/kind filter.

A focused control uses mixed and nested node kinds to prove document order,
the element principal kind, named filtering, and an exact one-charge-per-node
profile. It also verifies that the unimplemented `descendant::text()` form is
reported as unsupported rather than approximated.

## Result

| Case | Expected | Actual | Disposition |
| --- | ---: | ---: | --- |
| `Axes031-1` | 0 | 0 | passed |
| `Axes031-2` | 0 | 0 | passed |
| `Axes031-3` | 1 | 1 | passed |
| `Axes031-4` | 5 | 5 | passed |
| `Axes032-1` | 0 | 0 | passed |
| `Axes032-2` | 0 | 0 | passed |
| `Axes032-3` | 1 | 1 | passed |
| `Axes032-4` | 8 | 8 | passed |
| `Axes033-1` | 0 | 0 | passed |
| `Axes033-2` | 1 | 1 | passed |
| `Axes033-3` | 1 | 1 | passed |
| `Axes033-4` | 21 | 21 | passed |

The admitted `Axes001` through `Axes033` selections now contribute 63 passing
location-path cases through the same metadata-driven direct XPath seam.

Subsequent [`Axes034`–`Axes037` evidence](qt3-axes034-037-descendant-or-self-execution-2026-08-28.md)
adds the complete descendant-or-self tranche and overlapping-context duplicate
elimination.

## Claim boundary

This evidence admits only the three listed descendant step forms and their
unnamespaced name tests. It does not establish `descendant::text()` or other
kind tests, QName resolution, namespace-sensitive name tests, duplicate
elimination across overlapping input contexts, generalized path normalization,
other forward axes, or a general XPath parser.
