# OASIS XSLT 1.0 Qualified Sort Paths

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an `xsl:sort` key reuse the same namespace-aware qualified
child/attribute path owner as ordinary value and apply selections?

## Change

The ordinary location-path branch of sort-key compilation now falls back to
the existing qualified child/attribute path parser when the expression uses an
explicit prefix. Prefixes resolve against the `xsl:sort` element's in-scope
stylesheet namespaces, and the existing sort runtime continues to evaluate the
resulting typed path.

The fallback does not admit namespace wildcards, predicates, explicit axes,
functions, or a separate sorting evaluator.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,402 | 1,406 | +4 |
| Executed successfully | 1,241 | 1,245 | +4 |
| Expected-result XML matches | 1,127 | 1,130 | +3 |
| XML comparison mismatches | 79 | 80 | +1 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact cases are `Microsoft/Sorting__77532#1`,
`Microsoft/Sorting__84466#1`, and
`Microsoft/Sorting_SortOnAttributeWithNamespacePrefix#1`. They cover qualified
element and attribute sort keys. `Microsoft/Sorting__77539#1` now executes but
retains a visible whitespace/result comparison mismatch and receives no pass
credit.

The strict standard-operation lower bound is now
`1,130 / 2,742 = 41.21%`; the deliberately conservative all-catalog ratio is
`1,130 / 3,173 = 35.61%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused lifecycle test sorts source elements numerically through a
  qualified attribute key and verifies exact output order.
- Traced corpus executions identify all three newly exact cases and preserve
  the newly exposed mismatch as an uncredited disposition.
- The complete local OASIS measurement completed without editing upstream
  corpus bytes.
