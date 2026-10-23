# OASIS XSLT 1.0 Literal-Variable Concat AVT

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a literal result attribute evaluate a bounded `concat()` of one string
literal and one atomic variable, including a global variable used in an
`xsl:apply-imports` template, without admitting the general AVT grammar?

## Changes

- The literal AVT compiler recognizes exactly
  `concat('literal',$variable)` between optional static text fragments.
- The compiled representation owns the literal, variable name, and surrounding
  text without retaining stylesheet storage.
- Runtime resolves the variable through the existing invocation/global atomic
  frame and materializes the value through the normal result-attribute owner.
- Other argument kinds, orderings, arities, and multiple dynamic expressions
  retain `FXST1031`.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,478 | 1,479 | +1 |
| Executed successfully | 1,315 | 1,316 | +1 |
| Expected-result XML matches | 1,208 | 1,209 | +1 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |
| `FXST1031` initialization frontier | 20 | 19 | -1 |

The unchanged `Lotus/impincl_impincl24#1` case now produces the expected
`border: solid red` attribute while exercising the already-admitted import and
`apply-imports` lifecycle.

The strict expected-result lower bound is now 1,209 of 2,742
standard-operation cases (44.09%) and 1,209 of all 3,173 catalog cases
(38.10%). This is local compatibility evidence, not a conformance claim.

## Boundaries

This tranche does not admit arbitrary AVT expressions, variable-first or
multi-variable concatenation, path arguments, nested calls, more than two
arguments, constructed-variable conversion, or multiple dynamic expressions
in one literal attribute.

## Verification

- A focused compiler test proves the owned literal and variable name.
- A focused runtime test proves global atomic lookup and exact construction.
- The complete local OASIS measurement proves exactly one initialization
  frontier becomes one exact result with no other disposition change.
- No upstream corpus byte was edited.
