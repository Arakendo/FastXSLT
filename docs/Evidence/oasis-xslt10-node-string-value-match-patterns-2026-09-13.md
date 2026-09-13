# OASIS XSLT 1.0 node string-value match patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can exact context-node string comparisons in template match predicates be
admitted across element, text, comment, and processing-instruction node tests
without admitting a general match-predicate evaluator?

## Implemented slice

The compiler recognizes the exact `node-test[.='literal']` shape for an
unqualified element name, `text()`, `comment()`, `processing-instruction()`, or
a statically named processing instruction. It lowers the form to one private
typed pattern carrying the node test and literal.

Selection first checks the typed node test, then compares the node string value.
Source and temporary result-tree dispatch implement the same semantics. The
comparison consumes an XPath-operation charge; temporary string-value traversal
retains its existing per-node XDM charges.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,649 | 1,654 | +5 |
| Executed successfully | 1,478 | 1,483 | +5 |
| Expected-result XML matches | 1,348 | 1,353 | +5 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 39 | 34 | -5 |

All five newly executed cases match their unchanged archival expected XML. The
strict standard-operation lower bound is now
`1,353 / 2,742 = 49.34%`; the conservative all-catalog ratio is
`1,353 / 3,173 = 42.64%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit numeric comparison, inequality, `contains()`,
boolean composition, variables, multiple predicates, qualified element names,
or arbitrary predicate expressions. It does not alter match priority, resource
authority, prepared-input ownership, or invocation state.

## Verification

- Compiler coverage proves every admitted node-test form lowers to the typed
  representation with path-pattern default priority.
- Runtime coverage proves source node kinds and temporary element nodes use the
  same exact string-value predicate semantics.
- The complete 3,173-case local measurement produced the counters above.
