# OASIS XSLT 1.0 literal-result AVT scalar values -- 2026-09-15

Date: 2026-09-15  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can literal-result attribute value templates retain bounded context lexical-name
and static string-expression semantics without exposing a general dynamic AVT
evaluator?

## Implemented slice

The private literal-result attribute plan now distinguishes `name()` and
`name(.)` from `local-name()`. Source element and attribute prefixes retained
by prepared XDM are used when materializing the lexical QName. Contexts without
a retained source prefix use the local name, and unnamed nodes produce the
empty string.

A single quoted XPath string expression inside an otherwise static AVT is
folded during stylesheet compilation. Static surrounding text is concatenated
at compile time. Multiple expressions, dynamic operands, computed QNames, and
general AVT expression evaluation remain outside this slice.

## Corpus result

Unchanged Lotus `lre_lre06` becomes exact through the static string-expression
AVT `{'All Done'}`. The newly admitted `{name(.)}` in Lotus `axes_axes129`
correctly advances to its later `count(namespace::*)` frontier; it is not
counted as a pass and does not imply namespace-axis support.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,762 | 1,763 | +1 |
| Initialization failures | 1,373 | 1,372 | -1 |
| Executed successfully | 1,578 | 1,579 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,441 | 1,442 | +1 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,442 / 2,742 = 52.59%` for standard-operation cases and
`1,442 / 3,173 = 45.45%` for the complete catalog.

## Verification

Focused compiler tests cover both scalar AVT forms. An end-to-end runtime test
proves that a namespaced source element produces its retained lexical QName in
a literal result attribute. The complete local sweep establishes the one exact
addition, unchanged mismatch and execution-failure counts, and absence of
panics.
