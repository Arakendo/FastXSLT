# OASIS XSLT 1.0 Static Atomic Copy-Of Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 464 definite unchanged XML passes; 696 initialized cases |
| Result | 468 definite unchanged XML passes; 700 initialized cases |
| Disposition | Shared bounded `xsl:copy-of` construction; not a conformance claim |

## Change

The compiler now recognizes context-independent string literals, integer
literals, and `true()`/`false()` in `xsl:copy-of`. It converts each statically
known atomic value to its canonical result text once during compilation and
retains a typed copy-of operation rather than disguising it as literal
stylesheet text. Runtime construction uses the ordinary charged text-result
path, so result-node and UTF-8 byte budgets still apply.

The focused production-path regression combines an escaped string, an integer,
and a boolean in one result. The semantic-inspection surface continues to report
the operation as `CopyOf`, and retained-capacity accounting includes the owned
compiled text.

## Measurement

The complete hash-verified local sweep moves as follows:

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 696 | 700 | +4 |
| Execution succeeded | 522 | 526 | +4 |
| XML comparison passes | 464 | 468 | +4 |
| XML comparison mismatches | 29 | 29 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **468 / 2,742 = 17.07%** of standard-operation
cases and **468 / 3,173 = 14.75%** of the complete catalog. The focused
`FXXP1003` copy-of-selection frontier falls from 29 to 28; the remaining newly
admitted cases previously stopped in broader expression-shape groups.

## Boundaries

This tranche does not admit variables, general atomic expressions, decimal or
floating-point lexical conversion, sequences of atomic values, or temporary
result-tree fragments in `xsl:copy-of`. Those need their real typed runtime
semantics rather than more static string special cases.
