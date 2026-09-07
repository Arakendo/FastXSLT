# OASIS XSLT 1.0 Path Translate Tranche

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XPath 1.0 `translate(path, search, replacement)` reuse shared controlled
path conversion and the tested Unicode translation primitive?

## Change

The XSLT 1.0 compiler now retains a namespace-aware location path and two
static string operands for the bounded `translate()` form. Runtime evaluation
converts the first selected node in document order to its controlled string
value, or uses the empty string for an empty node set, then calls the same
Unicode-codepoint translation helper used by constant folding.

The helper preserves XPath translation behavior: the first occurrence of a
search character owns its mapping, and search characters without a
corresponding replacement character are removed. Navigation, string-value
traversal, and translation remain charged through existing work-control
owners.

Dynamic search/replacement operands, nested calls, and modern typed semantics
remain outside this compatibility-only plan.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,318 | 1,326 | +8 |
| Executed successfully | 1,145 | 1,153 | +8 |
| Expected-result XML matches | 1,033 | 1,041 | +8 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 173 | 173 | 0 |

All eight newly executed cases agree with their unchanged archival expected
results. No upstream corpus byte was edited.

The strict standard-operation lower bound is now
`1,041 / 2,742 = 37.96%`; the deliberately conservative all-catalog ratio is
`1,041 / 3,173 = 32.81%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Verification

- Focused replacement and deletion sentinel passed alongside the neighboring
  path-string cases.
- The complete local OASIS measurement completed with the counters above.
- Full workspace verification is recorded after the surrounding campaign
  tranche is closed.
