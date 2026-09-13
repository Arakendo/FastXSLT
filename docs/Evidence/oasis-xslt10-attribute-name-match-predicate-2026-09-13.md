# OASIS XSLT 1.0 attribute name match predicate -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the exact `@*[name()='NCName']` pattern retain predicate priority without
mistaking lexical `name()` comparison for an expanded-name test?

## Implemented slice

The compiler retains this one bounded predicate as a private typed match rather
than normalizing it to `@name`. It receives the XSLT predicate default priority
of `0.5`, charges the XPath name operation, and matches only an unprefixed,
no-namespace attribute with the requested NCName. Source and temporary-tree
execution share that bounded meaning.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,666 | 1,667 | +1 |
| Executed successfully | 1,495 | 1,496 | +1 |
| Expected-result XML matches | 1,364 | 1,365 | +1 |
| XML comparison mismatches | 104 | 104 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 22 | 21 | -1 |

`conflictres_conflictres32` now matches its unchanged archival expected XML and
proves the predicate-pattern priority wins over the later exact `@x1` pattern.
The strict standard-operation lower bound is now
`1,365 / 2,742 = 49.78%`; the conservative all-catalog ratio is
`1,365 / 3,173 = 43.02%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit prefixed lexical names, dynamic strings, other name
functions, boolean composition, arbitrary attribute predicates, or general
XPath in match patterns.

## Verification

- Compiler coverage proves the typed operand and `0.5` default priority.
- Runtime coverage proves it outranks a later exact-name pattern.
- The complete 3,173-case local measurement produced the counters above.
