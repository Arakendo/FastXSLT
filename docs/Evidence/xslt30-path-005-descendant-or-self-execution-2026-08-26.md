# XSLT30 `path-005` Descendant-or-Self Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `path`, case `path-005` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The unmodified stylesheet expression is:

```xpath
element1[descendant-or-self::child2]
```

The native file-backed source and stylesheet are imported into bounded logical
resources and executed from a sealed snapshot. The final existence predicate
now supports a distinct descendant-or-self axis: it tests the candidate first,
then visits descendants in document order until a matching unnamespaced element
name is found. Every inspected node consumes the invocation's XPath node-visit
budget.

An independent test covers both a descendant match and a candidate self-match.
It also fixes the exact charge count for the small descendant case, preventing
the self portion or descendant traversal from becoming unaccounted work.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 5 |
| Engine unsupported | 5 |
| Total | 10 |

No membership or exclusion changed.

## Claim boundary

This is a final named existence predicate. It does not establish general
descendant-or-self steps, arbitrary predicate expressions, namespace-qualified
tests, sequence semantics, or XPath conformance.

