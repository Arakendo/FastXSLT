# XSLT30 Path Denominator and Child-Axis Predicate Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| XSLT suite | W3C XSLT 3.0 test suite |
| XSLT revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| XSLT test set | `tests/expr/path/_path-test-set.xml` |
| QT3 revision | `83993587711dbd5c18ed846385ec37d079d6e492` |
| QT3 pressure group | `Axes002-1` through `Axes002-4` in `prod/AxisStep.xml` |

## Denominator

The first-party XSLT30 overlay now retains all ten cases in the complete pinned
`path` test set before further implementation. The private integration test
resolves every native case identity and verifies that each retains an
environment, stylesheet, and XML assertion.

| XSLT disposition | Count |
| --- | ---: |
| Selected and passed | 1 |
| Engine unsupported | 9 |
| Total | 10 |

No case is excluded or hidden as a harness gap. The unsupported cases preserve
pressure for ancestor, ancestor-or-self, attribute, descendant-or-self, and
parent axes; descendant navigation; positional arithmetic; functions in
predicates; and complex match patterns.

The QT3 overlay also retains the complete four-case `Axes002` named-child-axis
group. All four remain engine-unsupported because their native expressions
also require QT3 environment resolution, descendant navigation, and
`fn:count`. Recording them is expression pressure, not an execution claim.

## Executed behavior

Unmodified XSLT30 `path-001` now executes from its pinned catalog, inline
principal source, stylesheet, and `assert-xml`. Its expression is:

```xpath
child1[child::child2]
```

The private XPath path representation can attach one explicit unprefixed
`child::name` existence predicate to its final relative child step. Evaluation
preserves document order and requires both names to have no namespace. It
charges the invocation's XPath node-visit budget for both outer candidates and
children examined by the predicate.

## Claim boundary

This is one deliberately narrow predicate form. It does not establish a general
predicate grammar, arbitrary axis steps, namespace-qualified names, effective
boolean value rules, positions, functions, descendant abbreviation, or XPath
conformance. The upstream XML declaration names ISO-8859-1, but this case's
stylesheet bytes are ASCII; execution therefore does not settle broader input
encoding admission under AR-0008.

