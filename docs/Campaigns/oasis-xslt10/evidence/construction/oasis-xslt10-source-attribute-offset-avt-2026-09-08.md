# OASIS XSLT 1.0 Source-Attribute Offset AVT

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a literal result attribute compose static text with one checked integer
offset over an unqualified source attribute, reusing existing result and work-
control ownership rather than introducing a general AVT arithmetic evaluator?

## Changes

- The bounded literal AVT compiler recognizes one expression of the form
  `@name + integer` or `@name - integer` between static text fragments.
- The compiled representation retains the expanded source-attribute name,
  checked signed offset, and literal prefix/suffix.
- Runtime scans source attributes with `XPathNodeVisit` charging, performs
  checked integer conversion and addition, then appends the result through the
  existing result-attribute owner.
- Expressions outside this exact grammar retain `FXST1031` rather than being
  approximated.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,475 | 1,476 | +1 |
| Executed successfully | 1,312 | 1,313 | +1 |
| Expected-result XML matches | 1,205 | 1,206 | +1 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |
| `FXST1031` initialization frontier | 23 | 22 | -1 |

The unchanged `Lotus/attribvaltemplate_attribvaltemplate06#1` case now produces
the exact expected `before0after` attribute value.

The strict expected-result lower bound is now 1,206 of 2,742 standard-operation
cases (43.98%) and 1,206 of all 3,173 catalog cases (38.01%). This is local
compatibility evidence, not a conformance claim.

## Boundaries

This tranche does not admit decimal or floating-point arithmetic, variables,
multiple AVT expressions, namespaced attributes, general numeric expressions,
or XSLT 1.0 `NaN` conversion for missing/non-numeric nodes. Those shapes remain
explicitly outside the admitted grammar.

## Verification

- A focused compiler test proves the typed name and signed offset.
- A focused runtime test proves charged source lookup and static-text
  composition.
- The complete local OASIS measurement proves exactly one initialization
  frontier becomes one exact result with no other disposition change.
- No upstream corpus byte was edited.
