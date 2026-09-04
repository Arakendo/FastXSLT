# OASIS XSLT 1.0 Static Binary String Functions Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 567 definite unchanged XML passes; 832 initialized cases |
| Result | 592 definite unchanged XML passes; 857 initialized cases |
| Disposition | Shared bounded XPath constant folding; not a conformance claim |

## Change

Two-literal-argument `contains()`, `starts-with()`, `substring-before()`, and
`substring-after()` expressions now fold during compilation. Both unprefixed
XPath 1.0 spellings and explicit `fn:` spellings use the same private parser.
Quoted commas and nested syntax remain protected by the bounded argument
splitter introduced for static `concat()`.

The fold retains boolean results as typed booleans in the existing
work-charged scalar path and string results as bounded literal strings. Focused
and end-to-end tests cover empty-string behavior, absent search strings, and
Unicode substring boundaries.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 832 | 857 | +25 |
| Execution succeeded | 645 | 670 | +25 |
| XML comparison passes | 567 | 592 | +25 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **592 / 2,742 = 21.59%** of standard-operation
cases and **592 / 3,173 = 18.66%** of the complete catalog. Every newly
initialized case reaches a definite unchanged XML comparison pass.

## Boundaries

This tranche does not add source-node, variable, collation, or general atomic
conversion operands; dynamic string functions remain explicit. It also does
not infer the three-argument collation forms from these codepoint-based static
cases.
