# XSLT30 `path-006` Parent-Predicate Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `path`, case `path-006` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The unmodified stylesheet expression is:

```xpath
//child1[parent::element1]
```

The native file-backed source and stylesheet are imported into bounded logical
resources and executed from a sealed snapshot. The final existence predicate
now supports a named parent axis using the XDM node's existing parent link. A
present parent is inspected once and charged once to the invocation's XPath
node-visit domain.

An independent semantic test distinguishes the immediate parent from other
ancestors and fixes an exact nine-visit charge profile for its absolute
descendant selection, including the document element, plus two parent checks.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 6 |
| Engine unsupported | 4 |
| Total | 10 |

No membership or exclusion changed.

## Claim boundary

This is a named parent-existence predicate. It does not establish general
parent steps, `..`, arbitrary predicates, namespace-qualified tests, sequence
semantics, or XPath conformance.
