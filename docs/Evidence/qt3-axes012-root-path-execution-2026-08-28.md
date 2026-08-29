# QT3 `Axes012` Root-Path Execution

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Case: `Axes012-1`
- Environment: `TreeCompass`
- Expression: `fn:count( / )`
- Native `assert-eq`: `1`

## Method

The metadata-driven axis test resolves the case, referenced environment, source
document, expression, and assertion from the pinned QT3 test set. The source is
imported into a bounded sealed snapshot and built into owned XDM before direct
XPath execution through the private `fn:count` seam.

The private location-path representation now has one typed origin with four
states: context item, document node, relative steps, or descendant search. This
replaces the former interacting context/descendant booleans and gives `/` a
document-node meaning independent of the supplied evaluation context.

The count seam removes insignificant surrounding whitespace from its location
path operand. A focused control evaluates `/` from the document element rather
than the document node, verifies that the document node is selected, and
verifies exactly one invocation-local XPath node-visit charge.

## Result

| Case | Expected | Actual | Disposition |
| --- | ---: | ---: | --- |
| `Axes012-1` | 1 | 1 | passed |

Complete `Axes001` through `Axes012` now contribute 38 passing location-path
cases through the same metadata-driven direct XPath seam. Subsequent
[`Axes013`–`Axes019` evidence](qt3-axes013-019-parent-axis-execution-2026-08-28.md)
extends the document origin to exact absolute child paths and typed parent
steps.

## Claim boundary

This evidence admits only the root-only path `/` and surrounding insignificant
whitespace in the private count operand. It does not admit absolute child paths
such as `/far-north`, leading `//` before attribute steps, multiple-document
collections, root selection without an XDM document, or a general XPath parser.
