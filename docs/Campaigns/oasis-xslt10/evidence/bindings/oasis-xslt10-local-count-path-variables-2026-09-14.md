# OASIS XSLT 1.0 local count-path variables -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a local variable bind `count(location-path)` through the existing
controlled XPath axis evaluator and typed integer bindings?

## Implemented slice

The compiler recognizes `count(...)` only when its argument is admitted by the
existing bounded location-path parser. It retains the parsed path in a private
context-derived count instruction. Runtime execution evaluates the path from
the current source node, retains the existing per-operation and per-node charge
points, charges the count operation itself, and binds the cardinality as the
existing `xs:integer` atomic representation.

This reuses the already-tested preceding axis rather than introducing a
case-specific `preceding::text()` counter. Retention accounting owns the parsed
path explicitly, and the central accounting dispatcher delegates its size
calculation to a focused helper to remain within the source-unit guardrail.

## Corpus result

All three `count(preceding::text())` cases leave `FXXP1008`:

| Case | New disposition |
| --- | --- |
| `Lotus/position_position79` | Exact expected-result match |
| `Lotus/position_position78` | Later `$this + 1` expression currently misclassified as invalid `FXXP0002` |
| `Microsoft/Miscellaneous_TestOfPrecedingAxis` | Later `$this + 1` expression currently misclassified as invalid `FXXP0002` |

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,697 | 1,698 | +1 |
| Initialization failures | 1,438 | 1,437 | -1 |
| Executed successfully | 1,514 | 1,515 | +1 |
| Execution failures | 183 | 183 | 0 |
| Expected-result XML matches | 1,381 | 1,382 | +1 |
| XML comparison mismatches | 106 | 106 | 0 |
| `FXXP1008` initialization frontier | 10 | 7 | -3 |

The exact compatibility lower bound is now
`1,382 / 2,742 = 50.40%` for standard-operation cases and
`1,382 / 3,173 = 43.56%` for the complete catalog.

## Boundaries

This does not add a general function-call or numeric-expression variable
compiler. Qualified paths, variable-rooted paths, temporary-tree context, and
non-path `count()` arguments remain outside this instruction.

The two later `$this + 1` failures reveal classification debt in value-of
compilation: the expressions are valid XSLT 1.0 XPath but currently enter the
plain-variable parser and receive `FXXP0002`. They are not credited, and their
classification/semantics remain follow-up work independent of count-path
binding.

## Verification

- A focused runtime test counts preceding text nodes from a source element and
  verifies the typed value through ordinary local-variable lookup.
- Existing QT3 axis tests remain the broader oracle for preceding-axis
  selection semantics.
- The complete 3,173-case measurement accounts for all three frontier transfers
  and raises the exact-result lower bound by exactly one without a mismatch.
- Formatting, strict Clippy, the complete workspace suite, documentation, link,
  unsafe-surface, and corpus-inventory checks pass through `scripts/verify.ps1`.
