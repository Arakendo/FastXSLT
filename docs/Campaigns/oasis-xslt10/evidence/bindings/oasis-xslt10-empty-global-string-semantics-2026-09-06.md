# OASIS XSLT 1.0 Empty Global String Semantics

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Does an untyped global `xsl:variable` or `xsl:param` with neither `select` nor
content retain the XSLT 1.0 empty-string value instead of being materialized as
an empty temporary document?

## Change

The global-binding compiler now distinguishes a genuinely childless binding
from a binding whose sequence constructor creates text. A childless untyped
binding uses the existing owned string default with value `""`; a non-empty
constructor continues to use the existing temporary-tree owner.

This repairs value-kind identity rather than special-casing `boolean()`. The
empty string therefore composes through existing string conversion, effective
boolean value, comparison, and attribute-value-template paths. Non-empty
result-tree fragments retain their previous semantics.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,333 | 1,333 | 0 |
| Executed successfully | 1,159 | 1,160 | +1 |
| Expected-result XML matches | 1,047 | 1,050 | +3 |
| XML comparison mismatches | 77 | 75 | -2 |
| Execution failures | 174 | 173 | -1 |

Three unchanged archival cases now agree with their expected results:

- `Lotus/boolean_boolean43#1`
- `Lotus/boolean_boolean87#1`
- `Lotus/attribvaltemplate_attribvaltemplate09#1`

The first two were previously misleading executions with the wrong boolean
value. The third previously failed because the empty global parameter existed
under the wrong runtime value kind and was therefore unavailable to the AVT
variable lookup. No upstream corpus byte was edited.

The strict standard-operation lower bound is now
`1,050 / 2,742 = 38.29%`; the deliberately conservative all-catalog ratio is
`1,050 / 3,173 = 33.09%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Verification

- A focused runtime test distinguishes a childless empty-string global from
  non-empty literal and constructed temporary-tree globals.
- The complete local OASIS measurement completed with the counters above.
- The complete local OASIS measurement added no initialization failure,
  comparison mismatch, or execution failure.

