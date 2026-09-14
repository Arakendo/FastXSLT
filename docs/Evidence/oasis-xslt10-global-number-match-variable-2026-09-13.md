# OASIS XSLT 1.0 global number match variable -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 match predicate compare the candidate element's numeric string
value with a declared global atomic variable without admitting variables as
name tests or introducing a general match-pattern evaluator?

## Implemented slice

The compiler retains the exact `NCName[. > $NCName]` form as a typed match
pattern containing an expanded element name and global variable name. Source
and temporary-tree dispatch both resolve the variable from the invocation's
prepared global atomic frame, apply XPath numeric conversion to the candidate
string value and variable lexical value, and charge the comparison where it is
performed.

Threading the already immutable global atomic frame through temporary-tree
matching closes source/temporary parity for this form without making local
template state visible to match patterns.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,678 | 1,679 | +1 |
| Executed successfully | 1,507 | 1,508 | +1 |
| Expected-result XML matches | 1,376 | 1,377 | +1 |
| XML comparison mismatches | 104 | 104 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Initialization failures | 1,457 | 1,456 | -1 |

The unchanged Xalan `match14` case agrees exactly with its archival expected
XML. The strict standard-operation lower bound is now
`1,377 / 2,742 = 50.22%`; the conservative all-catalog ratio is
`1,377 / 3,173 = 43.40%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit variables as pattern name tests, local variables,
non-atomic globals, arbitrary comparison operators or operands, general
predicate expressions, `id()`, or `key()`.

## Verification

- Compiler coverage proves the typed element/global-variable representation.
- Runtime coverage proves matching and non-matching numeric values against the
  same global across source and temporary trees.
- The complete 3,173-case local measurement produced the counters above.
