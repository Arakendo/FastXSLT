# OASIS XSLT 1.0 context-string predicate condition -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an instruction condition evaluate a predicate path such as
`b[not(. = '')]` through the shared typed path evaluator and effective boolean
value, rather than a separate `xsl:if` implementation?

## Decision in the experiment

The private predicate tree now supports context-node string equality with a
literal and composes it with the existing `not` branch. Instruction-local
boolean compilation now recognizes any successfully parsed typed predicate
path as the existing node-existence boolean operation. Failed path recognition
continues to the ordinary scalar classifier, so this routing change does not
turn unsupported predicate syntax into a new error category.

Context string-value comparison is charged through the XPath operation domain;
path traversal retains its existing node-visit accounting.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,607 | 1,608 | +1 |
| Executed successfully | 1,436 | 1,437 | +1 |
| Expected-result XML matches | 1,316 | 1,317 | +1 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

The unchanged `Lotus/predicate_predicate37#1` case now matches exactly. The
strict standard-operation lower bound is now `1,317 / 2,742 = 48.03%`; the
conservative all-catalog ratio is `1,317 / 3,173 = 41.51%`. These remain local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary context-value comparisons, general
predicate expressions, or a second conditional evaluator. It does not change
result order, node identity, diagnostics, cancellation, resource authority, or
corpus data.

## Verification

- A focused path test proves that only the non-empty child survives
  `not(. = '')`.
- A traced unchanged corpus case proves the path is compiled through
  instruction-local effective boolean value and matches its expected result.
- The complete 3,173-case local measurement produced the counters above.
