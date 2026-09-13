# OASIS XSLT 1.0 text contains match pattern -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a literal `contains()` test over the context-node string value extend the
typed node-string match family without admitting general function predicates?

## Implemented slice

The compiler recognizes the exact `node-test[contains(., 'literal')]` shape
and retains the literal in the existing typed node-string pattern. Matching
uses Unicode scalar string containment after the same bounded string-value
construction and XPath-operation charge as the equality family. Source and
temporary-tree execution share the implementation contract.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,670 | 1,671 | +1 |
| Executed successfully | 1,499 | 1,500 | +1 |
| Expected-result XML matches | 1,368 | 1,369 | +1 |
| XML comparison mismatches | 104 | 104 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 18 | 17 | -1 |

`string_string124` now matches its unchanged archival expected XML, including
the parsed entity/quote content in the literal. The strict standard-operation
lower bound is now `1,369 / 2,742 = 49.93%`; the conservative all-catalog ratio
is `1,369 / 3,173 = 43.15%`. These remain local compatibility measurements,
not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit dynamic operands, additional `contains()`
arguments, other functions, boolean composition, general expressions, or a
general match-pattern XPath evaluator.

## Verification

- Compiler coverage proves the typed text-node/literal representation.
- Runtime coverage proves matching and rejection over source and temporary
  text nodes.
- The complete 3,173-case local measurement produced the counters above.
