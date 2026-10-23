# OASIS XSLT 1.0 Variable-Position Sort Keys

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 numeric variable select one child by position for an
`xsl:sort` key through the same compatibility semantics already used by value
expressions?

## Implemented slice

The typed sort plan now admits one single-step child path followed by either
of these XSLT 1.0 predicates:

```xpath
*[$index]
*[position() = $index]
```

The compiler reuses the existing bounded variable-position path parser. For
each sort candidate, execution evaluates the child step with ordinary charged
path traversal, resolves the local or global variable through XSLT 1.0 value
conversion, and uses the selected node's string value as the ordinary text or
numeric sort key. Non-finite, fractional, zero, negative, and out-of-range
numeric values select no node. Direct temporary-tree predicates retain their
existing XSLT 1.0 effective-boolean-value behavior.

The plan remains limited to one child step. It does not approximate
multi-step predicate focus, add general predicate expressions to sort keys, or
change modern static-context admission.

## Corpus result

The unchanged exact cases are:

- `Lotus/sort_sort29#1`
- `Lotus/sort_sort30#1`
- `Lotus/sort_sort31#1`

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,056 | 2,059 | +3 |
| Initialization failures | 1,079 | 1,076 | -3 |
| Executed successfully | 1,954 | 1,957 | +3 |
| Exact XML-semantic matches | 1,825 | 1,828 | +3 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound is now `1,828 / 3,173 = 57.61%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- A focused runtime regression covers a local direct-variable key and a global
  explicit `position()` comparison across both `xsl:for-each` and
  `xsl:apply-templates` sorting.
- The complete 3,173-case measurement moved exactly the three targeted cases
  from initialization failure to exact output without changing mismatch or
  execution-failure totals.
- No upstream corpus byte was edited.
