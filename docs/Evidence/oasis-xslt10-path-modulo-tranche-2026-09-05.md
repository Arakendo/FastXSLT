# OASIS XSLT 1.0 Path Modulo Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `c3c5592` plus the measured implementation |
| Corpus | OASIS XSLT/XPath 1.0 Committee Draft 04 local archive |
| Corpus SHA-256 | `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5` |
| Status | Complete; four doubt-annotated expected-result matches promoted |
| Governing review | [AR-0019](../Architectural%20Reviews/AR-0019-xslt10-compatibility-profile-on-modern-core.md) |

## Final bounded binary operator

The private binary-numeric path plan now admits token-delimited `mod` over its
existing checked integer operands. The recognizer distinguishes the operator
from elements literally named `mod` or `div`; a zero divisor produces the same
structured unsupported boundary used by source-dependent division rather than
panicking or manufacturing a value.

This completes the useful single-operator integer-path cluster. The immediately
following corpus cases contain chained and nested arithmetic, mixed literals,
variables, functions, and precedence. They require an owned expression parser
and typed tree rather than further character-level recognition, so they remain
explicitly unsupported after this tranche.

## Corpus result

The complete 3,173-case sweep changed only the intended frontier:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,076 | 1,080 | +4 |
| Initialization failures | 2,059 | 2,055 | -4 |
| Executed successfully | 875 | 879 | +4 |
| XML comparison passes | 792 | 796 | +4 |
| XML comparison mismatches | 50 | 50 | 0 |
| Execution failures | 201 | 201 | 0 |

The promoted unchanged cases are `Lotus/math_math79#1` through
`Lotus/math_math82#1`. All four occur in the suite's doubts file, so FastXSLT
retains that qualification.

The measured standard-operation ratio becomes 796 of 2,742 cases, or 29.03%.
This remains compatibility evidence rather than an XSLT 1.0 conformance claim.
