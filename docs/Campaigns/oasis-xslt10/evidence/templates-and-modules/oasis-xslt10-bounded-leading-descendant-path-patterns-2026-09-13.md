# OASIS XSLT 1.0 bounded leading descendant path patterns -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the exact `match="//name"` slice extend to the directly adjacent
`//ancestor/child` and `//ancestor//descendant` forms without admitting general
match-pattern grammar?

## Implemented slice

The existing typed leading-descendant match representation now admits at most
two unqualified named steps, separated by either one child step or the
descendant abbreviation. All steps remain predicate-free. Evaluation remains
document-rooted and uses the bounded invocation-owned membership cache from
ADR-0013, with the complete charged path evaluator retained as fallback and
differential oracle.

The path form and its path-pattern default priority are preserved. No special
template matcher or source index was introduced.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,632 | 1,642 | +10 |
| Executed successfully | 1,461 | 1,471 | +10 |
| Expected-result XML matches | 1,337 | 1,341 | +4 |
| XML comparison mismatches | 97 | 103 | +6 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |
| Generic unsupported match-pattern frontier | 59 | 47 | -12 |

The six newly visible mismatches are
`Microsoft/Whitespaces__91221#1`, `91222#1`, `91224#1`, `91225#1`,
`91226#1`, and `91228#1`. They exercise named whitespace stripping and
preservation outside ADR-0012's accepted exact `elements="*"` visibility
profile. They remain uncredited and do not establish broader whitespace
semantics.

The strict standard-operation lower bound is now
`1,341 / 2,742 = 48.91%`; the conservative all-catalog ratio is
`1,341 / 3,173 = 42.26%`. These remain local compatibility measurements, not
an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit more than two named steps, predicates,
namespace-qualified names, a default XPath namespace, arbitrary axes, unions,
or a second match backend. The previously explicit rejection of
`//foo[@id='1']` remains a compiler regression.

## Verification

- Compiler coverage proves all three admitted leading-descendant forms retain
  `MatchPattern::Path` and that predicate-bearing input remains unsupported.
- Runtime coverage distinguishes a direct child from a deeper descendant and
  proves exact selection without duplicates.
- The complete 3,173-case local measurement produced the counters above.
