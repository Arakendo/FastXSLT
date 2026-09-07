# OASIS XSLT 1.0 Path Substring Tranche

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XPath 1.0 `substring(path, start[, length])` reuse the shared path and
constant-string semantics without introducing a general expression evaluator?

## Change

The XSLT 1.0 compiler now retains a namespace-aware location path plus finite
static start and optional length operands. Runtime evaluation converts the
first selected node in document order to its controlled string value and then
uses the same Unicode-codepoint and XPath-rounding implementation as constant
substring folding. An empty node set supplies the empty string.

The retained floating-point operands are stored as exact bit patterns so the
compiled representation remains equality-testable without changing their
selected values. Navigation, string-value traversal, and the substring
operation remain charged through existing work-control domains.

Non-finite arithmetic operands, dynamic numeric operands, nested calls, and
modern typed substring semantics remain outside this bounded compatibility
plan.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,311 | 1,318 | +7 |
| Executed successfully | 1,138 | 1,145 | +7 |
| Expected-result XML matches | 1,026 | 1,033 | +7 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 173 | 173 | 0 |

All seven newly executed cases agree with their unchanged archival expected
results. No upstream corpus byte was edited.

The strict standard-operation lower bound is now
`1,033 / 2,742 = 37.67%`; the deliberately conservative all-catalog ratio is
`1,033 / 3,173 = 32.56%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Verification

- Focused integer, fractional-rounding, multi-node, and empty-path sentinel
  passed.
- Strict crate Clippy passed.
- The complete local OASIS measurement completed with the counters above.
- Full workspace verification is recorded after the surrounding campaign
  tranche is closed.
