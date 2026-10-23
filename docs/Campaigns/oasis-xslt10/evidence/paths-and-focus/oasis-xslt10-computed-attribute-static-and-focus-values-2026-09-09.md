# OASIS XSLT 1.0 computed-attribute static and focus values -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can computed-attribute `xsl:value-of` reuse already-admitted static-string and
sequence-focus semantics without adding a general attribute-content executor?

## Changes

- Literal-only `substring()` expressions use the existing Unicode/codepoint
  XPath folding helper during compilation and retain the resulting immutable
  string in the attribute plan.
- Exact `position()` and `last()` expressions lower to the existing attribute
  focus-position and focus-size operations.
- Runtime focus is neither reconstructed nor approximated: the existing
  computed-attribute materializer supplies the current sequence position and
  size used by other attribute expressions.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,570 | 1,573 | +3 |
| Executed successfully | 1,399 | 1,402 | +3 |
| Expected-result XML matches | 1,279 | 1,282 | +3 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

All three newly initialized cases execute and match exactly. The strict
standard-operation lower bound is now `1,282 / 2,742 = 46.75%`; the
conservative all-catalog ratio is `1,282 / 3,173 = 40.40%`. These are local
compatibility measurements, not an XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit dynamic substring operands, a general value
expression inside computed attributes, or arbitrary attribute-content
sequences. No corpus bytes or expected results changed.

## Verification

- Focused tests cover Unicode-aware static substring reuse and two-item
  position/size focus.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
