# QT3 `Axes072`–`Axes073` Leading-Descendant Self Expansion

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Cases: complete `Axes072` and `Axes073` groups (4 cases)
- Environments: `TreeEmpty`, `TreeCompass`, and `TopMany`
- Expressions: `//self::*` and `//self::node()`
- Native assertion: each case's pinned `assert-eq`

## Method

The metadata-driven axis test resolves all four cases, their referenced
environments and sources, expressions, and assertions from the pinned QT3 test
set. Each source is imported into a bounded sealed snapshot and built into
owned XDM before direct XPath execution through the private `fn:count` seam.

Leading `//` represents a descendant-or-self expansion followed by the written
step. That expansion cannot be reduced to one undifferentiated descendant scan
for every following axis. Child and attribute steps retain their existing
axis-specific expansions. A first self step now receives the document node and
every descendant in document order. Its node test retains all those contexts,
including the document; its element-principal wildcard filters the document
out.

A focused source begins evaluation from its root element to prove the absolute
context reset. `//self::node()` returns the document plus five descendants,
while `//self::*` returns only the three descendant elements. The six actual
node visits are charged exactly once and filtering adds no charge.

## Result

| Group | Cases | Passed | Disposition |
| --- | ---: | ---: | --- |
| `Axes072` | 2 | 2 | passed |
| `Axes073` | 2 | 2 | passed |
| **Complete denominator** | **4** | **4** | **passed** |

The admitted `Axes001` through `Axes073` selections now contribute 141 passing
location-path cases through the same metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the listed leading descendant self forms. It does
not establish arbitrary `//` composition after an element, reverse-axis
expansion, namespace-sensitive name tests, predicates, other omitted axis
groups, or a general XPath parser.
