# OASIS XSLT 1.0 Zero-Argument `generate-id()`

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the optional zero-argument `generate-id()` form share FastXSLT's existing
node-identity implementation without introducing a second notion of current
node or identity?

## Implemented slice

Yes. Boolean identity comparisons and value expressions now normalize an
omitted `generate-id()` argument to the explicit current-node path `.` at
compilation. Both forms therefore reuse the existing location-path evaluator,
stable engine-owned node identity, zero-or-one cardinality checks, diagnostic
behavior, and XPath work accounting.

This is ordinary standards behavior rather than an XSLT 1.0-only branch. No
ambient process state, allocation identity, or alternate identity generator
was introduced.

## Corpus result

The unchanged Lotus `idkey07` case now compares the current element against
children, attributes, and text nodes, compiles the corresponding diagnostic
branches, and emits its expected `Success` result exactly. Against the
conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,049 | 2,050 | +1 |
| Initialization failures | 1,086 | 1,085 | -1 |
| Executed successfully | 1,947 | 1,948 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,818 | 1,819 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

The existing stable-identity runtime regression now proves that
`generate-id()` equals `generate-id(.)`, emits the current document-node
identity, and remains distinct from child identities. The unchanged corpus case
adds element, attribute, and text-node coverage. The full measurement conserves
every catalog identity and adds no mismatch, execution failure, or panic.
