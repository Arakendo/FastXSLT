# OASIS XSLT 1.0 AVT path union -- 2026-09-15

Date: 2026-09-15  
Status: Verified shared semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can one dynamic part of a literal-result attribute value template consume a
location-path union while preserving XPath 1.0 node-set conversion semantics?

## Implemented slice

The private typed multi-part AVT representation now admits a union of at most
eight existing typed location paths as one dynamic part. Compilation uses the
shared top-level union splitter, so union bars inside predicates, parentheses,
or quoted strings are not misclassified.

Execution delegates to the existing controlled location-path union evaluator.
That owner combines alternatives in document order, removes duplicate node
identities, and charges each path traversal before the AVT converts the first
selected node to its string value. Empty unions convert to the empty string.
General XPath expressions and unions containing non-path operands remain
outside this slice.

## Corpus result

The unchanged Microsoft `AVTs__77571` case now initializes, executes, and
matches its expected XML. Its attribute combines `@size` with
`./* | ./text()` and verifies that source punctuation and brace characters are
retained through XPath string conversion and XML serialization.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,765 | 1,766 | +1 |
| Initialization failures | 1,370 | 1,369 | -1 |
| Executed successfully | 1,581 | 1,582 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,443 | 1,444 | +1 |
| XML comparison mismatches | 109 | 109 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,444 / 2,742 = 52.66%` for standard-operation cases and
`1,444 / 3,173 = 45.51%` for the complete catalog.

## Verification

A focused compiler test proves that the union is retained as one typed AVT
part with two bounded alternatives. The complete local corpus sweep establishes
the exact unchanged result, conserved denominators, and absence of new
execution failures or mismatches.
