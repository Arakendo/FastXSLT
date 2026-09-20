# OASIS XSLT 1.0 Context Translation Prefix

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 conditional compose `translate()` over the current node with
`starts-with()` without admitting a general nested-function evaluator or
weakening the modern static context?

## Implemented slice

Yes. Under XSLT 1.0 static context only, the exact expression
`starts-with(translate(., string-literal, string-literal), string-literal)`
compiles to a private typed boolean plan. Execution obtains the current source
node's string value through controlled XDM traversal, applies the existing
codepoint-correct translation primitive, and tests the translated prefix under
XPath work accounting.

The plan retains exact owned capacities for the search map, replacement map,
and prefix. It does not add runtime version dispatch, admit alternate
translation operands, introduce a second translation implementation, or make
the same nested composition available under the modern static context.

## Corpus result

The unchanged Microsoft `Miscellaneous__84430` case moves from an `FXXP1002`
initialization failure to an exact XML-semantic expected-result match. Against
the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,045 | 2,046 | +1 |
| Initialization failures | 1,090 | 1,089 | -1 |
| Executed successfully | 1,943 | 1,944 | +1 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,814 | 1,815 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |
| `FXXP1002` frontier | 15 | 14 | -1 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

A focused runtime test covers false and true branches plus Unicode-codepoint
translation. A cross-version regression proves that the same nested expression
remains explicitly unsupported under XSLT 3.0 static context. The complete
corpus measurement conserves every catalog identity and adds no mismatch,
execution failure, or panic.
