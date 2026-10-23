# OASIS XSLT 1.0 Following-Axis Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 686 definite unchanged XML passes; 961 initialized cases |
| Result | 694 definite unchanged XML passes; 969 initialized cases |
| Disposition | Shared typed forward-axis mechanics; not a conformance claim |

## Change

The shared typed path model now admits unqualified named-element, any-element,
and any-node tests on the `following` axis. The evaluator traverses the bounded
prepared document in document order and excludes the context node's
descendants, attributes, and the document node. Positional predicates therefore
observe forward-axis proximity positions before survivors enter the ordinary
document-order normalization and identity-deduplication path.

Both the context-subtree exclusion scan and document scan are charged through
the invocation-owned XPath work budget. Their temporary sets remain bounded by
the prepared document's already admitted node count. Namespace-qualified axis
tests and predicates outside the existing typed positional/existence subset
remain unsupported.

## Unchanged cases

All eight newly initialized cases execute and reach definite XML comparison
passes:

- `Lotus/axes_axes07#1`;
- `Lotus/axes_axes28#1` and `axes29#1`;
- `Lotus/axes_axes79#1` and `axes80#1`;
- `Lotus/axes_axes88#1`;
- `Lotus/axes_axes109#1`;
- `Lotus/axes_axes127#1`.

Other catalog cases containing `following::` remain at their existing
unsupported predicate, function, whitespace, key, or stylesheet boundaries.
Parser recognition changes some unsupported-frontier labels, but no non-pass
case is credited.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 961 | 969 | +8 |
| Execution succeeded | 767 | 775 | +8 |
| XML comparison passes | 686 | 694 | +8 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 194 | 194 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **694 / 2,742 = 25.31%** of standard-operation
cases and **694 / 3,173 = 21.87%** of the complete catalog.

## Verification

A first-party typed-path test proves that the axis excludes a context
descendant, includes descendants of later nodes, preserves document order, and
applies first/last positions correctly. The complete local OASIS sweep confirms
all eight intended cases pass without a new mismatch, comparator gap, runtime
failure, unexpected success, or panic.
