# OASIS XSLT 1.0 Homogeneous Literal Equality Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 553 definite unchanged XML passes; 818 initialized cases |
| Result | 560 definite unchanged XML passes; 825 initialized cases |
| Disposition | Shared XPath atomic semantics; not a conformance claim |

## Change

Source-free equality and inequality now recognize homogeneous boolean and
string literals, including symbolic operators without surrounding whitespace.
Boolean values compare as booleans and string values compare by exact codepoint
content. The operation reuses the bounded, work-charged scalar result path and
the end-to-end literal-comparison sentinel now covers numeric, boolean, and
string values through an XSLT 1.0 stylesheet.

The compiler keeps mixed-type comparisons out of this shared slice. In
particular, `1 = '001'` remains unsupported because its XSLT/XPath 1.0 coercion
differs from the modern typed-language rules and requires explicit static
version context rather than an accidental global compatibility behavior.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 818 | 825 | +7 |
| Execution succeeded | 631 | 638 | +7 |
| XML comparison passes | 553 | 560 | +7 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **560 / 2,742 = 20.42%** of standard-operation
cases and **560 / 3,173 = 17.65%** of the complete catalog. Every newly
initialized case reaches a definite unchanged XML comparison pass.

## Boundaries

This tranche does not add mixed-type compatibility coercion, node-set general
comparison, collation selection, or ordered string comparison. Those remain
explicit until their typed and edition-sensitive semantics are designed.
