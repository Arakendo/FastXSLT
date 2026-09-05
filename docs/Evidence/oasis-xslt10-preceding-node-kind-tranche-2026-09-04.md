# OASIS XSLT 1.0 Preceding Node-Kind Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 721 definite unchanged XML passes; 999 initialized cases |
| Result | 724 definite unchanged XML passes; 1,002 initialized cases |
| Disposition | Shared typed reverse-axis node-kind mechanics; not a conformance claim |

## Change

The typed `preceding` axis now admits explicit `text()`, `comment()`, and
`processing-instruction()` tests. It reuses the existing work-charged traversal
that excludes ancestors, attributes, and the document node. Candidate filtering
and positional predicates occur in reverse-axis order before surviving nodes
enter ordinary document-order normalization.

This does not add named processing-instruction tests on `preceding`, generalize
the kind tests to sibling axes, or widen predicate composition.

## Unchanged cases

Three Lotus cases move directly to definite XML comparison passes:

- `axes104`, selecting preceding comments through a chained attribute,
  ancestor, child-position, and attribute path;
- `axes105`, applying the same composition to preceding text nodes;
- `axes106`, applying it to preceding processing instructions.

These cases also exercise the prior ancestor-axis and leading-descendant work in
a substantially more composed path than the focused first-party tests.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 999 | 1,002 | +3 |
| Execution succeeded | 802 | 805 | +3 |
| XML comparison passes | 721 | 724 | +3 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 197 | 197 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **724 / 2,742 = 26.40%** of standard-operation
cases and **724 / 3,173 = 22.82%** of the complete catalog.

## Verification

A first-party typed-path test covers text, comment, and processing-instruction
selection plus reverse-axis nearest/farthest positional behavior. The complete
local OASIS sweep confirms all three intended composed cases pass without a new
mismatch, comparator gap, runtime failure, unexpected success, or panic.
