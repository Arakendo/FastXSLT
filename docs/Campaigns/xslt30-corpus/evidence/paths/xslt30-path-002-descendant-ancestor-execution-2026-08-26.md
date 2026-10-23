# XSLT30 `path-002` Descendant and Ancestor Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `path`, case `path-002` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The harness resolves `locationPath002.xml` from the pinned test-set environment,
reads and closes the import handle, admits the bytes under a logical resource
identity, and seals the snapshot before compilation or transformation. Engine
execution consumes only the sealed in-memory source.

The unmodified stylesheet expression is:

```xpath
//child2[ancestor::element2]
```

The private evaluator now supports leading descendant navigation followed by a
final unprefixed name test and one explicit named ancestor existence predicate.
Descendants are visited in document order. Descendant visits and each examined
ancestor are charged to the invocation's XPath node-visit work domain.

The result selects only the `child2` beneath `element2` and matches the native
XML assertion by result element name and string value.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 2 |
| Engine unsupported | 8 |
| Total | 10 |

No membership or exclusion changed. The four paired QT3 `Axes002` cases remain
selected and engine-unsupported because the private harness does not yet
execute their environment, `fn:count`, and assertion combination.

## Claim boundary

This evidence does not establish a general axis grammar, `descendant::`, an
arbitrary predicate language, namespace-qualified node tests, ordering across
general XPath sequences, or XPath conformance. Leading `//` and the final
ancestor predicate are private syntax admitted for this semantic slice. The
ASCII-only bytes in the ISO-8859-1-declared stylesheet do not settle general
encoding admission.

