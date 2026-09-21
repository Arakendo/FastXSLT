# OASIS XSLT 1.0 Grouped Reverse-Axis Position

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can FastXSLT preserve the XPath distinction between a positional predicate
inside a reverse-axis step and the same predicate applied outside a grouped
reverse-axis node-set?

## Method

- Normalize redundant enclosing parentheses only after recording whether the
  numeric predicate is lexically inside the reverse-axis step or outside a
  parenthesized axis result.
- Retain reverse proximity order for `ancestor::name[1]` and
  `(ancestor::name[1])`; use document order for `(ancestor::name)[1]` and
  `((ancestor::name))[1]`.
- Feed the normalized typed path through the existing charged reverse-axis,
  position-filter, and attribute-step evaluator.
- Add a focused nested-parentheses regression that distinguishes the nearest
  and document-first ancestors, then run unchanged Lotus `position_position85`
  from the hash-verified local OASIS XSLT 1.0 CD04 archive.

## Result

The focused regression preserves both orderings. The unchanged corpus case's
five grouped variations produce the expected `a, a, c, c, c` result and match
exactly.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,118 | 2,119 | +1 |
| Initialization failures | 1,017 | 1,016 | -1 |
| Executed successfully | 2,029 | 2,030 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,898 | 1,899 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,899 / 3,173 = 59.85%`.
The generic `FXXP1001` initialization frontier falls from 28 to 27 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- The admitted group contains one supported reverse-axis named step, one
  positive integer position, and a following location-path suffix.
- Parentheses may be redundant, but grouping remains semantically observable
  when the predicate is outside the axis expression.
- General filter expressions, arbitrary grouped paths, dynamic positions, and
  a general parenthesized XPath grammar remain unsupported.

## Reproduction

```powershell
cargo test -p fastxslt --all-features chained_axis_then_position_predicates_preserve_lexical_filter_order
./scripts/measure-oasis-xslt10.ps1 -TraceCase position_position85
./scripts/verify.ps1
```
