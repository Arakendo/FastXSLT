# OASIS XSLT 1.0 Reverse-Axis Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 662 definite unchanged XML passes; 937 initialized cases |
| Result | 686 definite unchanged XML passes; 961 initialized cases |
| Disposition | Shared typed reverse-axis mechanics; not a conformance claim |

## Change

The shared typed path model now admits unqualified named-element, any-element,
and any-node tests on the `preceding-sibling` and `preceding` axes. Candidate
sequences are constructed in reverse axis order so positional predicates see
the XPath proximity positions. Surviving nodes then enter the existing
document-order normalization and identity-deduplication path.

`preceding-sibling` considers only earlier children of the same parent.
`preceding` traverses the document while excluding ancestors, attributes, and
the document node. Traversal is charged through the existing invocation-owned
XPath work budget. Namespace-qualified reverse-axis tests, arbitrary
predicates, and the `following` axis remain outside this tranche.

## Unchanged cases

All 24 newly initialized cases execute and reach definite XML comparison
passes:

- 22 Lotus axis cases: `axes08`, `axes10`, `axes30`, `axes31`, `axes34`,
  `axes35`, `axes69` through `axes78`, `axes81`, `axes89`, `axes101`,
  `axes125`, `axes126`, and `axes128`;
- `position107`, covering `preceding-sibling::*[last()]`;
- `position109`, covering `preceding::*[last()]`.

The broader cases that combine these axes with unsupported predicates,
functions, whitespace declarations, or other stylesheet features remain at
their existing initialization boundaries and are not credited.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 937 | 961 | +24 |
| Execution succeeded | 743 | 767 | +24 |
| XML comparison passes | 662 | 686 | +24 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 194 | 194 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **686 / 2,742 = 25.02%** of standard-operation
cases and **686 / 3,173 = 21.62%** of the complete catalog.

## Verification

A first-party typed-path test distinguishes nearest and farthest positions on
both reverse axes and proves that ancestors are excluded from `preceding`.
The complete local OASIS sweep confirms all 24 intended cases pass without a
new mismatch, comparator gap, runtime failure, unexpected success, or panic.
