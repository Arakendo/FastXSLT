# XSLT30 Data-Manipulation Denominator and Conditional Execution

Date: 2026-08-26

## Candidate selection

The remaining XSLT30 expression families were compared at pinned revision
`6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`, including inherited test-set
dependencies. The superficially smaller `treat-as`, `type-expr`, and
`type-functions` families are wholly schema-aware and therefore excluded by
ADR-0007. `data-manipulation` is the smallest unclaimed expression family with
cases inside the accepted initial profile.

## Conserved denominator

- Test set: `tests/expr/data-manipulation/_data-manipulation-test-set.xml`
- Cases: `data-manipulation-001` through `data-manipulation-028`
- Dependency: all 28 declare `XSLT10+`
- Entry: all 28 use a referenced principal-source environment
- Assertion: all 28 use `assert-xml`
- Selection: all 28 selected; no profile exclusions

The first-party overlay contains exactly one disposition for every upstream
case. The admission test resolves all four referenced environments, including
inline and file-backed sources, closes import handles, admits each logical
source and stylesheet into a bounded sealed snapshot, and verifies every inline
or file-backed expected result.

## Executed slice

Cases `data-manipulation-001` through `data-manipulation-006` execute through
the native compiler and runtime against their upstream XML assertions. Together
they establish:

- `xsl:if` with true and false constant numeric predicates;
- ordered `xsl:choose`, first-true-branch selection, optional
  `xsl:otherwise`, and nested `xsl:if`;
- checked addition, subtraction, multiplication, `div`, and positive-integer
  `mod` within the admitted constant predicate grammar;
- exact rational comparison for chained division, so 5 divided by 2 remains
  2.5 without binary floating-point approximation;
- instruction work charging only for the chosen sequence constructor; and
- built-in document/element dispatch into the exact `doc` template.

The exact-rational evaluator is separate from the constant-integer evaluator
used by positional paths. A fractional division result therefore does not
silently become a valid integer position.

## Current disposition

| Cases | Selection | Execution | Principal pressure |
| --- | --- | --- | --- |
| `001`–`006` | selected | passed | conditional instructions and exact constant numeric predicates |
| `007`–`008` | selected | engine unsupported | decimal literals and `round()` predicates |
| `009`–`019` | selected | engine unsupported | general formatting plus global variable/parameter state |
| `020`–`028` | selected | engine unsupported | node-valued global bindings, dependencies, and broader paths |

The denominator is 28 discovered and selected: six pass, 22 are explicit
engine gaps, and none are excluded, harness-unsupported, failed, or lost.

## Claim boundary

This evidence does not establish general XPath numeric semantics, decimal
literals, IEEE floating-point behavior, arbitrary comparisons, effective
boolean value over general sequences, dynamic predicates, general
`format-number`, or global variable/parameter behavior. Constant folding is an
admitted compilation strategy for source-independent expressions; it is not a
claim that every predicate is evaluated statically.

The next adjacent pressure is `data-manipulation-007/008`, but admitting
`round(3.7)` should use an explicit decimal numeric representation rather than
route through binary floating point merely to advance two cases.
