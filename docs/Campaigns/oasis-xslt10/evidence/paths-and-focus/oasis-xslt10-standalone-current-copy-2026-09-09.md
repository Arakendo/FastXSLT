# OASIS XSLT 1.0 standalone current-node copy -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can exact standalone `xsl:copy-of select="current()"` reuse the existing
current-item copy plan without admitting composed `current()` expressions?

## Decision in the experiment

For an entire select expression consisting only of `current()`, there is no
nested predicate or expression context in which XSLT's current item can differ
from the evaluation context item. The compiler therefore lowers this exact
form to the same typed `CopyOfCurrent` operation used by `select="."`.

Copy execution, deep-copy behavior, namespaces, result budgets, cancellation,
and source provenance remain owned by the existing runtime path.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,573 | 1,574 | +1 |
| Executed successfully | 1,402 | 1,403 | +1 |
| Expected-result XML matches | 1,282 | 1,283 | +1 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The newly initialized case matches exactly. The strict standard-operation
lower bound is now `1,283 / 2,742 = 46.79%`; the conservative all-catalog
ratio is `1,283 / 3,173 = 40.43%`. These are local compatibility measurements,
not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit `current()` inside predicates, arithmetic,
functions, unions, or longer paths. Those forms can observe XSLT's distinction
between the current item and a changing inner context and require their own
typed semantics. No corpus bytes or expected results changed.

## Verification

- A focused runtime test copies the current document node through the exact
  standalone function form.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
