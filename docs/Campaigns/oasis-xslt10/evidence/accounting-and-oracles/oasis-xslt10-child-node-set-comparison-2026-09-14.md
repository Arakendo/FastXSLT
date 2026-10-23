# OASIS XSLT 1.0 child node-set comparison -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can predicates compare two relative child node-set paths using XPath 1.0
general-comparison semantics without adding a general predicate evaluator?

## Implemented slice

The typed path predicate plan now retains two relative, unprefixed child paths
of at most four steps. Runtime evaluation selects both node sets under work
control and returns true when any left/right pair has equal string values. This
preserves XPath 1.0 node-set equality rather than comparing only the first node
or requiring singleton operands.

The parser's boolean-predicate gate now recognizes spaced equality. Besides the
new child-path comparison, that allows the existing typed context-string
predicate to accept `. = 'literal'` with ordinary XPath whitespace.

Qualified steps, non-child axes, variables, arithmetic operands, and operators
other than equality remain outside this slice.

## Corpus result

Four unchanged Microsoft namespace cases using `DOCUMENT[TAG1 = TAG2]` and
`DOCUMENT[TAG3 = TAG4/TAG5]` become exact. Unchanged Lotus `output25` also
becomes exact through its spaced context-string comparison. Microsoft
`Miscellaneous__84427` passes the same XPath expressions but reaches the
existing explicit bounded-HTML-serialization rejection, so it is not credited
as an exact result.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,751 | 1,757 | +6 |
| Initialization failures | 1,384 | 1,378 | -6 |
| Executed successfully | 1,568 | 1,573 | +5 |
| Execution failures | 183 | 184 | +1 later frontier |
| Expected-result XML matches | 1,431 | 1,436 | +5 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,436 / 2,742 = 52.37%` for standard-operation cases and
`1,436 / 3,173 = 45.26%` for the complete catalog.

## Verification

A focused lifecycle test covers multiple nodes on one side, nested child paths,
one matching document, and one nonmatching document. The complete local sweep
establishes the six-case initialization movement, five exact additions, one
honest later unsupported frontier, unchanged mismatch count, and absence of
panics.
