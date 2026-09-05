# OASIS XSLT 1.0 Ancestor-Axis Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 697 definite unchanged XML passes; 974 initialized cases |
| Result | 718 definite unchanged XML passes; 996 initialized cases |
| Disposition | Shared typed axis and leading-descendant mechanics; not a conformance claim |

## Change

The typed path model now admits unqualified named-element, any-element, and
any-node tests on `ancestor` and `ancestor-or-self`. Candidate sequences run
from the context toward the document root so predicates observe reverse-axis
proximity positions. Surviving nodes then enter the existing document-order
normalization and identity-deduplication path.

The tranche also repairs the general leading `//` abbreviation exposed by
`axes83`. Leading `//` now expands document `descendant-or-self::node()`
contexts before evaluating the following step independently for each context.
It no longer substitutes a global descendant scan for arbitrary axes or applies
a first-step positional predicate to one globally combined sequence. Both the
context expansion and actual step evaluation remain work charged.

Namespace-qualified ancestor tests, non-node principal kinds, and general
predicate composition remain outside this tranche.

## Unchanged cases

Twenty-one newly initialized cases execute and reach definite XML comparison
passes:

- 12 Lotus axis cases: `axes01`, `axes02`, `axes16`, `axes17`, `axes83`,
  `axes87`, `axes97`, `axes100`, `axes103`, `axes107`, `axes111`, and
  `axes119`;
- `node21`;
- six position identities: `position98`, `position99`, `position100`, both
  catalog identities for `position101`, and `position106`;
- `predicate40` and `predicate41`.

`position72` now initializes but stops at the retained explicit
`FXRT1001` multi-node `xsl:value-of` conversion boundary. It is not credited.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 974 | 996 | +22 |
| Execution succeeded | 778 | 799 | +21 |
| XML comparison passes | 697 | 718 | +21 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 196 | 197 | +1 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **718 / 2,742 = 26.19%** of standard-operation
cases and **718 / 3,173 = 22.63%** of the complete catalog.

## Verification

First-party tests distinguish nearest and farthest positions on both ancestor
axes, prove that leading `//ancestor::*` evaluates the requested axis rather
than a substituted descendant scan, and prove that `//a[1]` applies the
predicate per expanded context. The complete local OASIS sweep confirms all 21
intended cases pass without a new mismatch, comparator gap, unexpected success,
or panic.
