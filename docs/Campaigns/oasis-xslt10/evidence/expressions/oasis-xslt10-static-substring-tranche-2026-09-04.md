# OASIS XSLT 1.0 Static Substring Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 598 definite unchanged XML passes; 863 initialized cases |
| Result | 606 definite unchanged XML passes; 871 initialized cases |
| Disposition | Shared bounded XPath constant folding; not a conformance claim |

## Change

Two- and three-argument `substring()` expressions with a literal string and
finite literal numeric positions now fold during compilation. The shared
implementation uses XPath's one-based positions and rounds numeric arguments
toward positive infinity at a half-unit boundary. Selection operates on
Unicode codepoints rather than UTF-8 bytes. Unprefixed XPath 1.0 and explicit
`fn:` spellings share the same path.

Focused and end-to-end tests cover omitted length, fractional positions,
negative positions, zero positions, empty results, and non-ASCII codepoints.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 863 | 871 | +8 |
| Execution succeeded | 676 | 684 | +8 |
| XML comparison passes | 598 | 606 | +8 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **606 / 2,742 = 22.10%** of standard-operation
cases and **606 / 3,173 = 19.10%** of the complete catalog. Every newly
initialized case reaches a definite unchanged XML comparison pass.

## Boundaries

This tranche does not add node, variable, or general atomic-conversion
operands. It deliberately does not fold NaN or positive/negative infinity
expressions: their legacy-version behavior must not be allowed to settle the
modern semantic core before AR-0019 selects a version-mode contract.
