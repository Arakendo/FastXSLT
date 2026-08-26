# XSLT30 `path-010` and Complete Path Denominator Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `path`, case `path-010` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed; complete ten-case denominator passes |

## Executed behavior

The native selection and match pattern are identical:

```xpath
doc/element1[(((((2*10)-4)+9) div 5) mod 3)]/child1[last()]
```

The file-backed principal source and stylesheet are imported into bounded
logical resources, handles are closed, and execution consumes the sealed
snapshot. The path representation now attaches an optional positional
predicate to each individual child step. Constant arithmetic selects the second
`element1`; `last()` then selects the last `child1` in that selected parent's
name-matched child sequence.

The same compiled path semantics can serve as a private relative template match
pattern. Matching walks from the candidate to the parent of the pattern's first
step, charging those lineage inspections, evaluates the path from that context,
and tests candidate identity. It is therefore not accidentally anchored only to
the document node.

An independent path test includes an unrelated sibling, proving positional
counts apply to name-matched nodes rather than raw children, and fixes the
small selection at seven XPath node visits.

## Complete denominator

| Disposition | Count |
| --- | ---: |
| Selected and passed | 10 |
| Engine unsupported | 0 |
| Excluded or hidden | 0 |
| Total | 10 |

All native catalog identities, environments, stylesheets, and XML assertions
remain represented. Completing this denominator does not change the disposition
of the paired QT3 `Axes002` group; its four cases remain selected and explicitly
engine-unsupported pending a QT3 executor.

## Claim boundary

The private path pattern supports the child-path and positional forms exercised
here. This evidence does not establish the general XSLT pattern grammar,
default-priority rules, unions, descendant patterns, namespace-qualified tests,
general `last()` expressions, or XSLT/XPath conformance. The path-pattern
priority used by the private dispatcher remains unstabilized.

