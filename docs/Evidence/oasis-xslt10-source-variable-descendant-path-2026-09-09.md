# OASIS XSLT 1.0 source-variable descendant path -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a path rooted at a source-node variable use the descendant separator and
kind tests through the shared typed path evaluator while retaining XSLT 1.0's
distinct result-tree-fragment behavior?

## Decision in the experiment

For an XSLT 1.0 expression shaped as `$variable//path`, the compiler retains
the suffix as a context-descendant `LocationPath` and the existing
`SourceVariablePath` operation. At execution, that path is evaluated from each
bound source node, then the combined result is normalized to document order
and deduplicated.

The existing path evaluator owns descendant traversal, kind testing, work
charging, cancellation, ordering, and deduplication. A variable holding a
constructed XSLT 1.0 result-tree fragment does not acquire node-set navigation;
the same expression fails with `XPTY0004` rather than silently importing modern
temporary-tree semantics into compatibility mode.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,575 | 1,576 | +1 |
| Executed successfully | 1,404 | 1,405 | +1 |
| Expected-result XML matches | 1,284 | 1,285 | +1 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/variable_variable50#1` case now matches exactly. The
strict standard-operation lower bound is now `1,285 / 2,742 = 46.86%`; the
conservative all-catalog ratio is `1,285 / 3,173 = 40.50%`. These are local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche admits only the already-typed path grammar after a source-node
variable in XSLT 1.0 compatibility mode. It does not select general static
variable typing, result-tree-fragment conversion extensions, or broader
temporary-tree navigation. No corpus bytes or expected results changed.

## Verification

- A focused runtime test evaluates `$nodes//text()` from two source-node roots
  and verifies document-order text focus.
- A second focused test proves that the same navigation over a constructed
  XSLT 1.0 result-tree fragment fails with `XPTY0004`.
- The complete 3,173-case local measurement produced the counters above.
