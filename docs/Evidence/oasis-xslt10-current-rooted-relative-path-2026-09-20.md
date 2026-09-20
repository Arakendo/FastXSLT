# OASIS XSLT 1.0 Current-Rooted Relative Path

Date: 2026-09-20  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 value expression navigate a supported relative path from
`current()` without adding a second path evaluator or admitting the function in
modern static context?

## Implemented slice

Yes. A compatibility value expression beginning with `current()/` now compiles
the suffix through the existing XSLT 1.0 location-path parser and lowers to the
ordinary first-node string-conversion plan. Execution supplies the
instruction's source focus as the path root, so the explicit function spelling
does not introduce ambient state, a retained current-node slot, or a second
navigation implementation. Existing path work charges, cancellation checks,
node identity, and source provenance remain authoritative.

The modern compiler continues to reject the form. This slice does not admit
`current()` inside a general predicate, arithmetic expression, function
argument, match pattern, or another composed expression.

## Corpus result

The unchanged Lotus `select_select78` case now executes
`current()/@name` inside `xsl:for-each` and matches its expected result exactly.
Against the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,055 | 2,056 | +1 |
| Initialization failures | 1,080 | 1,079 | -1 |
| Executed successfully | 1,953 | 1,954 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,824 | 1,825 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

The focused XSLT 1.0 `current()` regression now reads an attribute through
`current()/@name` while executing within a `for-each`-established focus. The
unchanged corpus case independently proves two iterations retain their own
instruction-entry node and serialize `x = x` followed by `y = y`.

