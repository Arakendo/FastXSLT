# OASIS XSLT 1.0 Static Finite Number-Conversion Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 747 definite unchanged XML passes; 1,031 initialized cases |
| Result | 749 definite unchanged XML passes; 1,033 initialized cases |
| Disposition | Shared finite constant conversion; not a conformance claim |

## Change

The value-expression compiler now folds `number()` around one source-free,
finite decimal literal or one quoted finite decimal lexical value. The compiled
plan retains the canonical decimal string, so repeated transformation performs
neither numeric parsing nor version dispatch.

This deliberately small slice does not admit context or path conversion,
compound arithmetic arguments, NaN, infinity, exponent notation, or general
XPath number formatting. Negative zero is canonicalized to `0`; no negative-
zero identity or formatting claim is made by this value-only operation.

## Unchanged cases

Two standard-operation cases move directly to definite XML comparison passes:

- `Lotus/math_math12#1` (`number(2)`); and
- `Lotus/math_math13#1` (`number('3')`).

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,031 | 1,033 | +2 |
| Execution succeeded | 830 | 832 | +2 |
| XML comparison passes | 747 | 749 | +2 |
| XML comparison mismatches | 50 | 50 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 201 | 201 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **749 / 2,742 = 27.32%** of standard-operation
cases and **749 / 3,173 = 23.61%** of the complete catalog.

## Verification

Focused parser tests cover numeric and quoted decimal arguments,
canonicalization, negative zero, and refusal of NaN, path, compound-arithmetic,
and lookalike-function forms. A runtime test drives representative finite
conversions through normal stylesheet compilation and transformation. The
complete local OASIS sweep confirms both affected identities pass without a
new mismatch, comparator gap, runtime failure, unexpected success, or panic.
