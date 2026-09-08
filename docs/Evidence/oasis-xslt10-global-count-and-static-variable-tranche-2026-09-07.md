# OASIS XSLT 1.0 Global Count and Static Variable Tranche

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can several variable cases reuse the modern compiled/runtime model without
retaining source state in compiled artifacts or weakening XPath 1.0 comparison
semantics?

## Changes

- A global `count(path)` expression retains only its compiled location path and
  evaluates once per invocation against the principal document node.
- Literal `contains()` and exact integral arithmetic in local variables fold to
  typed atomic values during compilation.
- XSLT 1.0 variable-to-string-literal conditions use the existing
  node-set/string comparison implementation. Node-set `=` and `!=` remain
  independently existential rather than treating `!=` as the complement of
  `=`.
- A named-template argument with no matching declared parameter is ignored.
  It does not shadow a same-named global binding.

No source document, dynamic value, or invocation frame is retained by the
compiled stylesheet.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,448 | 1,453 | +5 |
| Executed successfully | 1,287 | 1,292 | +5 |
| Expected-result XML matches | 1,173 | 1,178 | +5 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are:

- `Lotus/variable_variable17#1`
- `Lotus/variable_variable26#1`
- `Lotus/variable_variable37#1`
- `Lotus/variable_variable38#1`
- `Lotus/variable_variable69#1`

The strict standard-operation lower bound becomes
`1,178 / 2,742 = 42.96%`; the conservative all-catalog ratio becomes
`1,178 / 3,173 = 37.13%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- Focused runtime tests cover principal-document global count, static boolean
  and integer variables, undeclared named-call argument behavior, and the case
  where node-set `=` and `!=` are both true.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
