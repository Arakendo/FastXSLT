# OASIS XSLT 1.0 Normalize-Space Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 608 definite unchanged XML passes; 873 initialized cases |
| Result | 609 definite unchanged XML passes; 874 initialized cases |
| Disposition | Shared path-dependent XPath semantics; not a conformance claim |

## Change

`normalize-space()` now accepts an admitted location path as well as the
existing implicit and explicit context-item forms. The runtime evaluates the
path with normal work charging, requires at most one selected node, and streams
that node's complete XDM string value through the existing XML-whitespace
normalizer. An empty selection produces the empty string.

The path is compiled with the effective XPath default namespace. Static
source-free normalization continues through the existing typed function
evaluator; the new route does not replace it or create an XSLT 1.0 evaluator.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 873 | 874 | +1 |
| Execution succeeded | 686 | 687 | +1 |
| XML comparison passes | 608 | 609 | +1 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **609 / 2,742 = 22.21%** of
standard-operation cases and **609 / 3,173 = 19.19%** of the complete catalog.
Unchanged OASIS `Lotus/string_string10#1` reaches a definite XML comparison
pass.

## Boundaries

The shared modern function signature remains zero-or-one: selecting more than
one node reports `XPTY0004`. XPath 1.0's first-node conversion for a multi-node
node-set is not inferred here and remains part of AR-0019's version-aware
compatibility question. Variable, temporary-tree, and general atomic arguments
remain explicit.
