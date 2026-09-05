# OASIS XSLT 1.0 Binary Numeric Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `5c27087` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; five doubt-annotated expected-result matches promoted |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Shared typed plan

The value-expression compiler now recognizes one bounded source-dependent
binary numeric form: two already supported typed location paths joined by `+`
or `*`. The compiled plan owns both paths, the operator, and the operand
cardinality policy selected from stylesheet static context.

For an exact XSLT 1.0 stylesheet, each non-empty node sequence contributes the
string value of its first node in document order. The modern plan instead
retains the existing zero-or-one cardinality rule and reports `XPTY0004` for a
multi-node operand. Runtime therefore contains no stylesheet-version branch.
Both paths execute through the shared work-charged evaluator, and arithmetic is
checked over the currently admitted integer lexical form.

Empty operands, non-integer lexicals, overflow, subtraction, division, chained
operators, general atomization, numeric promotion, and floating-point behavior
remain explicit boundaries. This is not a general XPath arithmetic evaluator.

A focused lifecycle test compiles the same multi-node source twice: the XSLT
1.0 plan produces the first-node result while the modern plan returns
`XPTY0004`. It also covers paired attribute paths and multiplication.

## Corpus result

The complete 3,173-case sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,061 | 1,066 | +5 |
| Initialization failures | 2,074 | 2,069 | -5 |
| Executed successfully | 860 | 865 | +5 |
| XML comparison passes | 777 | 782 | +5 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged cases are `Lotus/math_math55#1` through
`Lotus/math_math59#1`. They cover element and attribute string values with
addition and multiplication, including parenthesized operands. All five occur
in the suite's doubts file, so the qualification is retained.

The measured standard-operation ratio becomes 782 of 2,742 cases, or 28.52%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
