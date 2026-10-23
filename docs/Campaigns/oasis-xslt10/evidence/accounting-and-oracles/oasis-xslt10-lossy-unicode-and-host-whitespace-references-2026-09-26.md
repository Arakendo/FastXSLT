# OASIS XSLT 1.0 Lossy Unicode and Host-Whitespace References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do the remaining Microsoft `BVTs_bvt085` and `Variables__84439` comparison
failures demonstrate FastXSLT semantic defects, or do their immutable archival
references require behavior that a conforming, host-neutral processor should
not reproduce?

## Findings

### `Microsoft/BVTs_bvt085`

The stylesheet constructs U+186A0 one hundred times, reuses that temporary value
ten times, and reports a string length of 1,000. FastXSLT emits the requested
1,000 supplementary Unicode characters and the expected length.

The 2,022-byte archival result instead contains 1,000 literal ASCII question
marks followed by the same length report. The replacement is lossy: the
reference no longer identifies the requested characters and cannot distinguish
correct supplementary-character handling from arbitrary replacement. The case
therefore receives an exact, bounded `unusable-reference-result-excluded`
disposition and no pass credit.

### `Microsoft/Variables__84439`

The source contains whitespace-only text nodes between the element children of
the final `CCC`. The stylesheet declares neither `xsl:strip-space` nor a DTD
whose element-content declarations authorize parser stripping. Its global
`select="//AAA/CCC/text()"` therefore selects those source text nodes as well as
the `C2` and `C3` nodes.

FastXSLT preserves and emits the selected source whitespace. The archival
reference keeps only `C2 C3 `, matching a historical host-parser configuration
that discarded whitespace-only source nodes before transformation. Reproducing
that output in the engine would introduce ambient parser policy and contradict
the immutable XDM/stylesheet-owned whitespace model. The case therefore
receives the existing bounded `host-parser-policy-excluded` disposition and no
pass credit.

## Measurement

The all-catalog denominator and exact lower bound are unchanged:

| Measurement | Before | After |
| --- | ---: | ---: |
| Catalog cases | 3,173 | 3,173 |
| Exact expected-result matches | 2,125 | 2,125 |
| Visible XML mismatches | 51 | 49 |
| Host-parser-policy exclusions | 8 | 9 |
| Unusable-reference-result exclusions | 21 | 22 |

No corpus byte changed. Both cases remain visible in the denominator under
named non-pass dispositions.

## Architectural note

This classification preserves two existing boundaries: the engine retains
Unicode scalar values rather than emulating a lossy legacy output code page,
and source whitespace is governed by parsed XML plus stylesheet semantics, not
an undeclared host-parser switch.
