# OASIS XSLT 1.0 Recursive Path Arithmetic Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `e5ef053` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; two doubt-annotated expected-result matches promoted and one later boundary exposed |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Owned operator tree

The binary-numeric plan now owns a recursive typed operator tree rather than
exactly two paths. Compilation selects the rightmost top-level operator at each
precedence level, producing left-associative addition/subtraction and
multiplication/division/modulo trees. Leaves remain typed location paths with
their explicit unary sign. Evaluation recursively reuses the same path
selection and work-control machinery and charges every retained operation.

This is not yet a general XPath expression AST. Literal, variable, function,
comparison, and arbitrary atomic leaves remain outside the form. Parentheses
are admitted only insofar as they group the supported path arithmetic.

Focused tests cover chained multiplication, chained division, a parenthesized
multiplicative group, precedence selection, and the earlier negative syntax
sentinels.

## Corpus result

The complete 3,173-case sweep made the later boundary visible:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,080 | 1,083 | +3 |
| Initialization failures | 2,055 | 2,052 | -3 |
| Executed successfully | 879 | 881 | +2 |
| XML comparison passes | 796 | 798 | +2 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 202 | +1 |

The promoted unchanged cases are `Lotus/math_math87#1` and
`Microsoft/XSLTFunctions_RepeatedUseOfDivOperator#1`. Both occur in the
suite's doubts file, so FastXSLT retains that qualification.

`Lotus/math_math86#1` now compiles its repeated multiplication tree. Its first
value executes, while its second reaches `.125`, `.5`, and `.2` path values and
returns `FXRT1022` because the current leaf representation admits integer
lexicals only. It remains uncredited. This identifies exact decimal/rational
intermediates as a concrete representation question rather than permission to
approximate XPath arithmetic with integer truncation.

The measured standard-operation ratio becomes 798 of 2,742 cases, or 29.10%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
