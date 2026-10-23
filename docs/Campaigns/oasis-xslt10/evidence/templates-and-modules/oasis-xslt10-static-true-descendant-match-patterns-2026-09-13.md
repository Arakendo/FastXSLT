# OASIS XSLT 1.0 static-true descendant match patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the exact `//name[true()]` match-pattern shape be admitted without adding a
general function-predicate matcher?

## Implemented slice

The template compiler recognizes only an unqualified leading-descendant name
followed by the exact static predicate `[true()]`. It removes the statically
true predicate and compiles the remaining leading-descendant name through the
existing typed path representation.

The representation continues to receive path-pattern default priority and to
use charged, document-rooted membership. This is compile-time normalization;
there is no runtime function call or alternate match backend.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,645 | 1,647 | +2 |
| Executed successfully | 1,474 | 1,476 | +2 |
| Expected-result XML matches | 1,344 | 1,346 | +2 |
| XML comparison mismatches | 103 | 103 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 43 | 41 | -2 |

Both newly executed cases match their unchanged archival expected XML. The
strict standard-operation lower bound is now
`1,346 / 2,742 = 49.09%`; the conservative all-catalog ratio is
`1,346 / 3,173 = 42.42%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit `false()`, dynamic functions, boolean composition,
qualified names, or function predicates on multi-step paths. It does not
change predicate evaluation, template ordering, resource authority, or runtime
state.

## Verification

- Compiler coverage proves the exact form lowers to `MatchPattern::Path`.
- Runtime coverage proves nested matching descendants are selected exactly
  once.
- The complete 3,173-case local measurement produced the counters above.
