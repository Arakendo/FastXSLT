# OASIS XSLT 1.0 Following Node-Kind Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 694 definite unchanged XML passes; 969 initialized cases |
| Result | 697 definite unchanged XML passes; 974 initialized cases |
| Disposition | Shared typed axis node-kind mechanics; not a conformance claim |

## Change

The typed `following` axis now admits explicit `text()`, `comment()`, and
`processing-instruction()` tests. These variants reuse the same charged,
context-descendant-excluding traversal as the element and `node()` forms and
filter by retained XDM node kind before ordinary path normalization.

The change does not generalize kind tests to other axes, add named processing
instruction tests on `following`, or broaden predicates. Those remain separate
semantic work.

## Unchanged cases

Three Lotus cases move directly to definite XML comparison passes:

- `axes108` selects following comments from an attribute context;
- `axes110` selects following processing instructions;
- `axes112` selects following text nodes.

Two Microsoft whitespace cases now initialize and execute their
`count(.../following::text())` expressions, then report the existing
`FXRT1014` boundary because their source combines `xsl:strip-space` with
`xml:space`. They remain visibly uncredited; this path work does not widen the
accepted whitespace profile.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 969 | 974 | +5 |
| Execution succeeded | 775 | 778 | +3 |
| XML comparison passes | 694 | 697 | +3 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 194 | 196 | +2 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **697 / 2,742 = 25.42%** of standard-operation
cases and **697 / 3,173 = 21.97%** of the complete catalog.

## Verification

A first-party typed-path test independently selects a following comment,
processing instruction, and text node. The complete local OASIS sweep confirms
the three intended passes, preserves both later whitespace boundaries, and
introduces no mismatch, comparator gap, unexpected success, or panic.
