# OASIS XSLT 1.0 Static Translate Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 592 definite unchanged XML passes; 857 initialized cases |
| Result | 598 definite unchanged XML passes; 863 initialized cases |
| Disposition | Shared bounded XPath constant folding; not a conformance claim |

## Change

Three-literal-argument `translate()` expressions now fold during compilation.
The private implementation operates on Unicode codepoints, applies only the
first occurrence of a duplicated search character, removes characters whose
search position has no replacement, and never retranslates produced
characters. Unprefixed XPath 1.0 and explicit `fn:` spellings share the same
path.

Focused and end-to-end tests cover replacement, removal, duplicate search
characters, non-recursive translation, and non-ASCII codepoints.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 857 | 863 | +6 |
| Execution succeeded | 670 | 676 | +6 |
| XML comparison passes | 592 | 598 | +6 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **598 / 2,742 = 21.81%** of standard-operation
cases and **598 / 3,173 = 18.85%** of the complete catalog. Every newly
initialized case reaches a definite unchanged XML comparison pass.

## Boundaries

This tranche does not add source-node, variable, or general atomic conversion
operands. Dynamic `translate()` remains explicitly unsupported rather than
silently using compile-time behavior.
