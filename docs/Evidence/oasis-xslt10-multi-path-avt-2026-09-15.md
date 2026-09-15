# OASIS XSLT 1.0 bounded multi-path AVT -- 2026-09-15

Date: 2026-09-15  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can one literal-result attribute compose several already-admitted XPath
location paths without adding a general expression evaluator or losing brace
characters carried by source data?

## Implemented slice

A private typed AVT expression now retains at most 32 ordered parts. Each part
is either static text or an existing typed location path. At least two dynamic
path parts are required, keeping single-expression AVTs on their existing
specialized paths. Compilation recognizes doubled braces as static braces and
rejects unmatched or nested expression braces.

Execution evaluates each path against the same source focus, applies the XPath
1.0 first-node string-value rule already used by single-path AVTs, preserves
part order, and reuses existing traversal and work-budget accounting. Variables,
arithmetic, general function expressions, and namespace-default rewriting
remain outside this slice.

## Corpus result

Microsoft `AVTs__77582` now initializes and executes. Its two result attributes
contain the unchanged expected values, including literal brace characters from
the source. The archival XML comparator still reports a mismatch because the
expected fragment contains serializer-added whitespace between its two
top-level elements while FastXSLT emits adjacent elements.

The case is therefore **not** counted as an exact pass. This tranche expands
executable breadth and records the remaining serialization/comparison
distinction without weakening the comparator.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,763 | 1,764 | +1 |
| Initialization failures | 1,372 | 1,371 | -1 |
| Executed successfully | 1,579 | 1,580 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,442 | 1,442 | 0 |
| XML comparison mismatches | 108 | 109 | +1 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound remains
`1,442 / 2,742 = 52.59%` for standard-operation cases and
`1,442 / 3,173 = 45.45%` for the complete catalog.

## Verification

A focused compiler test covers ordered paths, static text, and doubled braces.
An end-to-end runtime test uses the archival case's brace-bearing source shape
and checks both complete output attributes. The complete local sweep establishes
the newly executable case, unchanged exact count and execution-failure count,
the visible comparator mismatch, and absence of panics.
