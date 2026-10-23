# OASIS XSLT 1.0 Expanded-Axis Wildcard Pattern Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 731 definite unchanged XML passes; 1,013 initialized cases |
| Result | 734 definite unchanged XML passes; 1,016 initialized cases |
| Disposition | Shared template-pattern canonicalization; not a conformance claim |

## Change

The template-pattern compiler now recognizes `attribute::*` and `child::*` as
the expanded-axis spellings of the already-admitted `@*` and `*` wildcard
patterns. They compile to the same `AnyAttribute` and `AnyElement` operations
with the same node-test default priority. Runtime selection is unchanged and no
version branch is introduced.

This does not admit arbitrary axis patterns, named expanded-axis patterns, or a
general pattern grammar.

## Unchanged cases

Three cases move directly from initialization failure to definite XML
comparison passes:

- `Lotus/axes_axes60#1`, matching attributes with `attribute::*`;
- `Lotus/axes_axes61#1`, matching child elements with `child::*`; and
- `Lotus/node_node16#1`, independently matching attributes with
  `attribute::*`.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,013 | 1,016 | +3 |
| Execution succeeded | 812 | 815 | +3 |
| XML comparison passes | 731 | 734 | +3 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 201 | 201 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **734 / 2,742 = 26.77%** of standard-operation
cases and **734 / 3,173 = 23.13%** of the complete catalog.

## Verification

A focused compiler test verifies both spellings lower to the existing typed
wildcard patterns and retain node-test default priority. The complete local
OASIS sweep confirms all three affected identities pass without a new mismatch,
comparator gap, runtime failure, unexpected success, or panic.
