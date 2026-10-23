# OASIS XSLT 1.0 computed-attribute context strings -- 2026-09-13

Date: 2026-09-13  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:attribute` obtain the XPath string value of its complete context item
through `xsl:value-of select="."`, rather than supporting that expression only
inside a literal attribute value template?

## Implemented slice

Computed attributes now reuse the existing typed context-string value form.
The runtime obtains the complete string value once when either a literal or
computed attribute requires it, then supplies that value to both attribute
owners. Standalone computed attributes within `xsl:copy` use the same form.

Focused execution covers source and temporary element contexts containing
mixed text and descendant elements. The value is therefore the XPath string
value of the node, not merely its direct text or parser event value. Existing
controlled string-value traversal retains its work-budget and cancellation
charge points.

## Corpus result

All eleven Microsoft Output cases formerly stopped by `FXXP1012` now leave
that initialization frontier. They initialize and reach the existing bounded
HTML-serialization frontier instead.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,681 | 1,692 | +11 |
| Initialization failures | 1,454 | 1,443 | -11 |
| Executed successfully | 1,510 | 1,510 | 0 |
| Execution failures | 171 | 182 | +11 |
| Expected-result XML matches | 1,379 | 1,379 | 0 |
| XML comparison mismatches | 104 | 104 | 0 |

The exact compatibility lower bound remains
`1,379 / 2,742 = 50.29%` for standard-operation cases and
`1,379 / 3,173 = 43.46%` for the complete catalog. No case is credited merely
for advancing to a later boundary.

## Boundaries

This does not broaden HTML serialization, computed attribute names or
namespaces, general XPath value expressions, or result-tree construction.
`select="."` selects only the already-owned context-string operation. The
eleven archival cases remain visibly unsupported until their HTML result
shapes are admitted and verified independently.

## Verification

- A source/temporary-tree parity test covers mixed descendant text through a
  computed attribute on a literal result element.
- A standalone `xsl:attribute` within `xsl:copy` covers the separate copy
  compilation/execution path.
- The complete 3,173-case local measurement shows the eleven-case frontier
  transfer and no new mismatch.
