# OASIS XSLT 1.0 Path Subtraction and Exact Division Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `0bf0e8a` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; five doubt-annotated expected-result matches promoted |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Bounded extension

The private binary-numeric path plan now also admits whitespace-delimited
subtraction and `div`. Requiring whitespace around subtraction distinguishes
the operator from hyphens inside valid NCNames such as `n-2`; `div` is
recognized only as a separate top-level token, so element names `div` and
`mod` remain valid operands.

Division is deliberately limited to checked, nonzero, exactly integral
quotients over the existing integer lexical form. Fractional results and
division by zero remain structured unsupported outcomes because this tranche
does not implement XPath's general floating-point or decimal division
semantics. The compiled XSLT 1.0 versus modern operand-cardinality selection,
path work charging, and runtime-version neutrality remain unchanged.

Focused tests cover subtraction between hyphenated element names, division
between elements literally named `div` and `mod`, refusal of fractional and
zero divisors, and the QT3 `/*/` invalid-axis sentinel that initially exposed
an over-broad multiplication recognizer.

## Corpus result

The complete 3,173-case sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,066 | 1,071 | +5 |
| Initialization failures | 2,069 | 2,064 | -5 |
| Executed successfully | 865 | 870 | +5 |
| XML comparison passes | 782 | 787 | +5 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged cases are `Lotus/math_math61#1` and
`Lotus/math_math71#1` through `Lotus/math_math74#1`. All five occur in the
suite's doubts file, so FastXSLT retains that qualification.

The measured standard-operation ratio becomes 787 of 2,742 cases, or 28.70%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
