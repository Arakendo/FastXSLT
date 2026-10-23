# OASIS XSLT 1.0 contains context-number NaN -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing typed context-number NaN predicate preserve the common XPath
1.0 `contains(number(.), 'NaN')` validation idiom without admitting general
implicit function conversion?

## Implemented slice

For version 1.0 stylesheets only, boolean compilation recognizes the exact
two-argument expression `contains(number(.), 'NaN')`. XPath 1.0 converts the
numeric first argument to a string before `contains`; finite numbers cannot
contain `NaN`, while a failed context conversion has the exact lexical result
`NaN`. The expression therefore lowers to the already verified private
context-number NaN plan.

Execution continues to use controlled context string-value access, one charged
numeric conversion, and the shared XPath 1.0 numeric lexical rules. General
numeric operands, other search strings, and arbitrary nested function
composition remain unsupported.

## Corpus result

Unchanged Lotus `math_math104` becomes an exact expected-result match across
twelve valid and invalid numeric lexical forms, including empty content,
leading or trailing decimal points, negative zero, and malformed strings.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,729 | 1,730 | +1 |
| Initialization failures | 1,406 | 1,405 | -1 |
| Executed successfully | 1,546 | 1,547 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,411 | 1,412 | +1 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,412 / 2,742 = 51.50%` for standard-operation cases and
`1,412 / 3,173 = 44.50%` for the complete catalog.

## Verification

The focused context-number lifecycle test now executes both the explicit
`string(number(.)) = 'NaN'` spelling and the implicit `contains` spelling over
valid and malformed context values. The complete local corpus sweep confirms
the one-case exact movement, unchanged mismatch and execution-failure counts,
and absence of panics.
