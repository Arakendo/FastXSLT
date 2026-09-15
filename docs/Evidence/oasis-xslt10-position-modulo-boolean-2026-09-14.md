# OASIS XSLT 1.0 position modulo boolean -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the common XPath focus predicate `position() mod N = M` compose with the
existing runtime focus and numbering semantics without admitting general
numeric boolean expressions?

## Implemented slice

Instruction boolean compilation now recognizes equality between a static
nonnegative integer and `position() mod N`, in either operand order. The
divisor must be positive and the remainder must be smaller than it. The
compiled plan retains only the divisor, remainder, and source location.

Runtime evaluation requires a dynamic focus, charges one XPath operation, and
compares the one-based focus position's remainder. Missing focus preserves the
structured `XPDY0002` diagnostic. General arithmetic operands, negative values,
and noncanonical remainders remain outside this bounded plan.

## Corpus result

Unchanged Lotus `numbering_numbering45` becomes an exact expected-result match.
The stylesheet uses the new predicate to select alternating notes and the
existing `xsl:number` pattern support to maintain independent odd and even
counters across 25 nodes.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,730 | 1,731 | +1 |
| Initialization failures | 1,405 | 1,404 | -1 |
| Executed successfully | 1,547 | 1,548 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,412 | 1,413 | +1 |
| XML comparison mismatches | 108 | 108 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound is now
`1,413 / 2,742 = 51.53%` for standard-operation cases and
`1,413 / 3,173 = 44.53%` for the complete catalog.

## Verification

A focused end-to-end test exercises both operand orders over a four-node focus
and checks the odd/even result. The unchanged corpus case verifies composition
with numbering. The complete local sweep confirms the one-case exact movement,
unchanged mismatch and execution-failure counts, and absence of panics.
