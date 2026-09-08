# OASIS XSLT 1.0 Source-Attribute Concat AVT

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a literal result attribute compose static text with `concat()` over two
unqualified source attributes without admitting arbitrary AVT expressions or a
second evaluator?

## Changes

- The bounded literal AVT compiler recognizes exactly
  `concat(@left,@right)` between static text fragments.
- The compiled representation retains both expanded source-attribute names and
  the literal prefix/suffix.
- Runtime resolves both attributes from the source focus with existing
  `XPathNodeVisit` charging. A missing attribute contributes the XSLT 1.0 empty
  node-set string value.
- Other function calls, arbitrary argument expressions, more than two
  arguments, and multiple dynamic AVT expressions retain `FXST1031`.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,476 | 1,477 | +1 |
| Executed successfully | 1,313 | 1,314 | +1 |
| Expected-result XML matches | 1,206 | 1,207 | +1 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |
| `FXST1031` initialization frontier | 22 | 21 | -1 |

The unchanged `Lotus/attribvaltemplate_attribvaltemplate10#1` case now produces
the exact expected `BeforeFrontBackAfter` attribute value.

The strict expected-result lower bound is now 1,207 of 2,742
standard-operation cases (44.02%) and 1,207 of all 3,173 catalog cases
(38.04%). This is local compatibility evidence, not a conformance claim.

## Boundaries

This tranche does not admit general XPath function calls in AVTs, string
literals or variables as operands, namespaced source attributes, arbitrary
arity, nested expressions, or multiple dynamic expressions in one literal
attribute.

## Verification

- A focused compiler test proves the two typed attribute names and surrounding
  literal text.
- A focused runtime test proves charged lookup and exact result construction.
- The complete local OASIS measurement proves exactly one initialization
  frontier becomes one exact result with no other disposition change.
- No upstream corpus byte was edited.
