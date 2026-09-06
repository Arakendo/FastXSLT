# OASIS XSLT 1.0 Exact-Rational Path Arithmetic

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `43573da` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; two doubt-annotated expected-result matches promoted |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Shared representation change

The private source-dependent numeric plan now retains exact rational values
through its recursive operator tree. Decimal source lexicals with or without a
leading zero, including `.125`, `.5`, and `.2`, are parsed exactly rather than
through binary floating point. Checked addition, subtraction, multiplication,
and division normalize their rational result; final values serialize only when
they have an exact terminating decimal representation.

This is shared runtime machinery. Compilation still selects XSLT 1.0
first-in-document-order conversion or modern zero-or-one cardinality, and the
runtime contains no version branch. Zero divisors, fractional modulo,
non-terminating decimal output, overflow, empty operands, and unsupported
lexicals remain explicit failures. Every path visit and arithmetic operator
retains its existing work charge.

Focused controls cover exact fractional division, zero division, integral
modulo, leading-dot decimal lexicals, signs, normalization, and the prior path
grammar boundaries.

The same tranche recognizes `div` or `mod` after a closed parenthesized or
predicate operand without requiring intervening whitespace. The keyword still
requires a token boundary: NCNames containing either spelling remain names.

## Corpus result

The complete 3,173-case sweep produced two disposition changes:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,083 | 1,084 | +1 |
| Initialization failures | 2,052 | 2,051 | -1 |
| Executed successfully | 881 | 883 | +2 |
| XML comparison passes | 798 | 800 | +2 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 202 | 201 | -1 |

Unchanged `Lotus/math_math86#1` now produces exact `<out>24,63</out>` from a
ten-factor multiplication whose final three operands are `.125`, `.5`, and
`.2`. Unchanged `Lotus/math_math88#1` now recognizes the parenthesized
multiplication followed immediately by `div` and produces exact `<out>2</out>`.
Both identities occur in the suite's doubts file, so FastXSLT retains that
qualification.

The measured standard-operation ratio becomes 800 of 2,742 cases, or 29.18%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
