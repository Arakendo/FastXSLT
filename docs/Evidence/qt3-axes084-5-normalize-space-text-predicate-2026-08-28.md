# QT3 `Axes084-5` Normalize-Space Text Predicate

Date: 2026-08-28

## Inputs

- Suite: W3C QT3
- Pinned revision: `83993587711dbd5c18ed846385ec37d079d6e492`
- Test set: `prod/AxisStep.xml`
- Case: `Axes084-5`
- Environment: `nw_Customers`
- Source: `prod/AxisStep/nw_Customers.xml` (37,212 bytes)
- Expression: `fn:count(//text()[normalize-space()])`
- Native assertion: `assert-eq` 827

## Method

The metadata-driven axis harness resolves the case, environment, source,
expression, and assertion from the pinned QT3 test set. The larger source is
admitted into a sealed snapshot under a case-specific 64 KiB entry and
aggregate byte ceiling, then parsed under a 16,384-event and 64-depth ceiling.
Other axis cases retain their smaller 4 KiB and 2,048-event limits.

The private parser admits only a final `text()[normalize-space()]` form and
lowers it to a typed context predicate. Evaluation applies the zero-argument
function to each already-selected owned text-node value and uses the resulting
string's effective boolean value. Only XML whitespace characters—space, tab,
carriage return, and line feed—are collapsed for the emptiness decision.

A focused source contains whitespace-only and non-whitespace text nodes. The
predicate retains exactly the two meaningful values and rejects the whitespace
nodes. All seven descendant candidates are charged once; inspecting the
already-owned text value adds no hidden navigation.

## Result

| Case | Expected | Actual | Disposition |
| --- | ---: | ---: | --- |
| `Axes084-5` | 827 | 827 | passed |

The complete `Axes084` group now passes. The admitted `Axes001` through
`Axes084` selections contribute 182 passing location-path cases through the
same metadata-driven direct XPath seam.

## Claim boundary

This evidence admits only the listed zero-argument `normalize-space()`
predicate over a final text-node step. It does not establish the general
function, argument forms, normalization result materialization, predicates on
other node kinds or atomic values, arbitrary effective-boolean-value semantics,
predicate composition, XQuery constructors, or a general XPath parser.
