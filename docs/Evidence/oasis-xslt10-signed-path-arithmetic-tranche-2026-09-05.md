# OASIS XSLT 1.0 Signed Path Arithmetic Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `935063b` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; five doubt-annotated expected-result matches promoted |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Typed unary operands

Each operand in the private binary-numeric path plan now retains an explicit
unary-negation bit. Compilation separates that sign from the already typed
location path, including a parenthesized path, so NCName hyphens remain part of
the name. Execution applies checked negation after the existing path selection,
string-value traversal, work charging, and integer lexical conversion.

The operator recognizer also distinguishes a whitespace-led binary minus
immediately followed by unary minus. This admits `a --b` without treating the
hyphens in names such as `n-2` as operators. Repeated signs, unary arithmetic
over general expressions, floating-point values, and chained expression trees
remain outside this bounded form.

A focused lifecycle test executes signed element and attribute paths, both
directly and inside parentheses. Parser tests conserve name/sign boundaries and
the existing QT3 invalid-axis sentinel remains green.

## Corpus result

The complete 3,173-case sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,071 | 1,076 | +5 |
| Initialization failures | 2,064 | 2,059 | -5 |
| Executed successfully | 870 | 875 | +5 |
| XML comparison passes | 787 | 792 | +5 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged cases are `Lotus/math_math63#1`,
`Lotus/math_math65#1`, and `Lotus/math_math67#1` through
`Lotus/math_math69#1`. All five occur in the suite's doubts file, so FastXSLT
retains that qualification.

The measured standard-operation ratio becomes 792 of 2,742 cases, or 28.88%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
