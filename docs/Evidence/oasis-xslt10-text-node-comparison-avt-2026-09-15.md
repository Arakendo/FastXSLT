# OASIS XSLT 1.0 text-node comparison AVT -- 2026-09-15

Date: 2026-09-15  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can source text-node sets participate in XPath 1.0 string comparisons, and can
a bounded path-to-string comparison supply the lexical boolean value of a
literal-result attribute value template?

## Implemented slice

Relative child paths used by the shared predicate comparison owner now retain
typed element and terminal text steps. This admits comparisons such as
`text()='Mary'` without representing the kind test as an element name. The
existing existential node-set `=` and `!=` behavior, traversal charging, and
comparison charging remain unchanged.

A single-expression literal-result AVT may now retain one typed location path,
one quoted string literal, and an equality or inequality operator. Execution
applies XPath 1.0 existential node-set comparison and emits the required
`true` or `false` lexical value. General boolean expressions and converted
non-string operands remain outside this slice.

## Corpus result

The unchanged Microsoft `AVTs__77564` case now initializes and executes. Its
seven result elements contain the expected first-name values and boolean
last-name values. The archival XML comparator reports a mismatch because the
expected fragment contains serializer-added whitespace between its top-level
elements while FastXSLT emits adjacent elements.

The case is therefore **not** counted as an exact pass. This tranche expands
executable breadth while retaining the strict fragment comparison boundary.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,766 | 1,767 | +1 |
| Initialization failures | 1,369 | 1,368 | -1 |
| Executed successfully | 1,582 | 1,583 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,444 | 1,444 | 0 |
| XML comparison mismatches | 109 | 110 | +1 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound remains
`1,444 / 2,742 = 52.66%` for standard-operation cases and
`1,444 / 3,173 = 45.51%` for the complete catalog.

## Verification

Focused tests cover text-child node-set comparison and the typed AVT
comparison representation. The complete local sweep establishes the newly
executable case, unchanged exact count and execution-failure count, visible
fragment-format mismatch, and absence of panics.
