# XSLT30 `path-004` Attribute-Predicate Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `path`, case `path-004` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The unmodified stylesheet expression is:

```xpath
//child2[attribute::attr1]
```

The shared file-backed corpus runner imports `locationPath004.xml` and the
stylesheet into a bounded resource set, closes the file handles, seals the
snapshot, and then compiles and executes entirely from admitted bytes.

The final existence predicate now has a distinct attribute axis. It examines
the candidate element's separately owned XDM attributes and requires an
unnamespaced attribute with local name `attr1`. Each inspected attribute is
charged to the invocation's XPath node-visit domain.

An independent semantic test proves that the selected element has one child
and one attribute, that the attribute was not inserted into the child list,
and that the complete small evaluation consumes exactly five XPath node visits:
the absolute leading descendant search now includes the document element
before inspecting the selected element's separately owned attribute.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 4 |
| Engine unsupported | 6 |
| Total | 10 |

No membership or exclusion changed.

## Claim boundary

This is a named attribute-existence predicate. It does not establish general
attribute steps, attribute values, wildcards, namespace-qualified tests,
arbitrary predicate expressions, or XPath conformance.
