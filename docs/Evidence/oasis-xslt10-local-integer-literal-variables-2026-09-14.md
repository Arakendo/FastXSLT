# OASIS XSLT 1.0 local integer-literal variables -- 2026-09-14

Date: 2026-09-14  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a local `xsl:variable` with a plain signed 64-bit integer literal in its
`select` attribute reuse the typed atomic binding machinery already used by
global variables and folded local arithmetic?

## Implemented slice

The static local-variable compiler now recognizes a plain `i64` lexical value
and stores it as the existing `xs:integer` atomic representation. Runtime
lookup, value conversion, equality, scoping, and work accounting continue to
use the existing typed local-binding path.

This is deliberately narrower than general numeric XPath. It does not admit
decimals, doubles, arithmetic expressions, paths, functions, node sequences,
or content-built variables.

## Corpus result

Eight cases leave the `FXXP1008` local-variable-select frontier. Their new
dispositions are conserved individually:

| Cases | New disposition |
| --- | --- |
| `Lotus/variable_variable09`, `Microsoft/Variables__78306` | Exact expected-result match |
| `Lotus/sort_sort29`, `Lotus/sort_sort30` | Later unsupported variable predicate path, `*[$index]` |
| `Microsoft/Variables__84634`, `Microsoft/Variables__84710` | Invalid duplicate local binding, `FXST0017` |
| `Microsoft/Variables__78154` | Execution failure at the existing non-atomic local-alias boundary, `FXRT0002` |
| `Microsoft/Variables_SelectAttributeAndWhitespaceBetweenElement` | Visible expected-result mismatch |

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,692 | 1,696 | +4 |
| Initialization failures | 1,443 | 1,439 | -4 |
| Executed successfully | 1,510 | 1,513 | +3 |
| Execution failures | 182 | 183 | +1 |
| Expected-result XML matches | 1,379 | 1,381 | +2 |
| XML comparison mismatches | 104 | 105 | +1 |
| `FXXP1008` initialization frontier | 21 | 13 | -8 |

The exact compatibility lower bound is now
`1,381 / 2,742 = 50.36%` for standard-operation cases and
`1,381 / 3,173 = 43.52%` for the complete catalog. Advancing to another
unsupported, invalid, failed, or mismatching disposition is not counted as a
pass.

## Boundaries

The remaining thirteen `FXXP1008` cases require heterogeneous expressions such
as `name()`, `count(preceding::text())`, multiplication, `document()`,
variable-rooted paths, `current()` predicates, and decimals outside this
integer representation. This tranche does not approximate any of them.

The two duplicate-binding diagnostics and the later alias failure are recorded
as observed behavior, not as conformance credit. The whitespace-sensitive
mismatch remains visible for independent investigation.

## Verification

- A focused runtime test proves that two local integer literals retain typed
  value and equality behavior through the ordinary local-binding path.
- The complete 3,173-case measurement accounts for all eight moved cases and
  shows the exact two-pass increase plus one new visible mismatch.
- Formatting, strict Clippy, the complete workspace suite, documentation, link,
  unsafe-surface, and corpus-inventory checks pass through `scripts/verify.ps1`.
