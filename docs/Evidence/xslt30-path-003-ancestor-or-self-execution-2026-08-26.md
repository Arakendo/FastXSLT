# XSLT30 `path-003` Ancestor-or-Self Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `path`, case `path-003` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The unmodified stylesheet expression is:

```xpath
//child2[ancestor-or-self::element2]
```

The case reuses the bounded file-backed environment importer established for
`path-002`: source and stylesheet handles are closed after import, the bytes are
admitted under logical identities, and execution consumes the sealed snapshot.

The final existence predicate now has a distinct ancestor-or-self axis. It
tests the candidate first and then walks its parent chain, charging each node
examined. An independent semantic test selects an `element2` candidate through
its self position; this prevents the native case from passing through a mere
alias to the ancestor-only implementation.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 3 |
| Engine unsupported | 7 |
| Total | 10 |

No case membership or exclusion changed.

## Claim boundary

This remains a single final named existence predicate over the private path
representation. It does not establish general axis steps, arbitrary predicates,
namespace-qualified tests, effective boolean value rules, or XPath conformance.

