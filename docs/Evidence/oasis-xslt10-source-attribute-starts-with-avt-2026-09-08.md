# OASIS XSLT 1.0 Source-Attribute Starts-With AVT

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a literal result attribute compose static text with the XSLT 1.0 boolean
result of `starts-with()` over two unqualified source attributes without
admitting arbitrary AVT boolean expressions?

## Changes

- The bounded literal AVT compiler recognizes exactly
  `starts-with(@value,@prefix)` between static text fragments.
- The compiled representation retains both expanded source-attribute names and
  the literal prefix/suffix.
- Runtime performs charged source-attribute lookup and one charged XPath
  operation, then emits the canonical XPath `true` or `false` lexical value.
- Missing attributes retain XSLT 1.0 empty node-set string conversion.
- Expressions outside this exact grammar retain `FXST1031`.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,477 | 1,478 | +1 |
| Executed successfully | 1,314 | 1,315 | +1 |
| Expected-result XML matches | 1,207 | 1,208 | +1 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |
| `FXST1031` initialization frontier | 21 | 20 | -1 |

The unchanged `Lotus/attribvaltemplate_attribvaltemplate11#1` case now produces
the exact expected `BeforetrueAfter` attribute value.

The strict expected-result lower bound is now 1,208 of 2,742
standard-operation cases (44.06%) and 1,208 of all 3,173 catalog cases
(38.07%). This is local compatibility evidence, not a conformance claim.

## Boundaries

This tranche does not admit general boolean expressions in AVTs, string or
variable operands, namespaced source attributes, nested function calls,
arbitrary function arity, or multiple dynamic expressions in one literal
attribute.

## Verification

- A focused compiler test proves both typed attribute names and surrounding
  literal text.
- A focused runtime test proves exact boolean conversion and result
  construction.
- The complete local OASIS measurement proves exactly one initialization
  frontier becomes one exact result with no other disposition change.
- No upstream corpus byte was edited.
