# OASIS XSLT 1.0 Composed Context-Step Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 728 definite unchanged XML passes; 1,009 initialized cases |
| Result | 729 definite unchanged XML passes; 1,010 initialized cases |
| Disposition | Shared XPath abbreviated-axis semantics; not a conformance claim |

## Change

The typed location-path parser now recognizes `.` as an abbreviated
`self::node()` step when it occurs inside a path. It reuses the existing typed
self-axis evaluator, per-candidate XPath work charging, and ordinary
document-order normalization. The existing complete-expression `.` form still
uses the direct context-item origin.

This is a modern XPath path equivalence, not an XSLT 1.0 compatibility
exception. It does not admit arbitrary filter expressions, parenthesized path
composition, or a general expression grammar.

## Unchanged case

`Lotus/axes_axes98#1` moves directly from initialization failure to a definite
XML comparison pass. Its `@*/.` path selects each attribute and then preserves
that same node through the abbreviated self step.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,009 | 1,010 | +1 |
| Execution succeeded | 809 | 810 | +1 |
| XML comparison passes | 728 | 729 | +1 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 200 | 200 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **729 / 2,742 = 26.59%** of standard-operation
cases and **729 / 3,173 = 22.98%** of the complete catalog.

## Verification

A focused typed-path test composes an attribute wildcard with `.`, verifies
that both original attribute identities survive, and checks the exact four
XPath node-visit charges. The complete local OASIS sweep confirms the intended
case passes without a new mismatch, comparator gap, runtime failure, unexpected
success, or panic.
