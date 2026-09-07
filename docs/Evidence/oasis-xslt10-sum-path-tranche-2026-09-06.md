# OASIS XSLT 1.0 Sum Path Tranche

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XPath 1.0 `sum(path)` reuse the shared controlled path evaluator while
retaining compatibility conversion and bounded work accounting?

## Change

The XSLT 1.0 compiler now retains a namespace-aware location path for the
bounded `sum(path)` form. Execution evaluates the path through the existing
controlled navigation owner, converts every selected node's string value with
the admitted XPath 1.0 decimal lexical rules, and charges each conversion.
Empty node sets produce `0`; an invalid numeric lexical produces `NaN`.

Variable operands, nested expressions, and the broader modern typed `sum()`
surface remain explicit. The plan is selected only in XSLT 1.0 compatibility
mode and does not weaken modern sequence/type behavior.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,306 | 1,311 | +5 |
| Executed successfully | 1,133 | 1,138 | +5 |
| Expected-result XML matches | 1,021 | 1,026 | +5 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 173 | 173 | 0 |

All five newly executed cases agree with their unchanged archival expected
results. No upstream corpus byte was edited.

The strict standard-operation lower bound is now
`1,026 / 2,742 = 37.42%`; the deliberately conservative all-catalog ratio is
`1,026 / 3,173 = 32.34%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Verification

- Focused multi-node, attribute, and empty-node-set runtime sentinel passed.
- Strict crate Clippy passed.
- The complete local OASIS measurement completed with the counters above.
- Full workspace verification is recorded after the surrounding campaign
  tranche is closed.
