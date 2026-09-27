# OASIS Modern `xsl:value-of` Node-Sequence Conversion

Date: 2026-09-25  
Status: Verified shared-engine evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the modern XSLT core convert a location path selecting multiple nodes using
the effective `xsl:value-of/@separator`, without changing XSLT 1.0's distinct
first-node conversion rule?

## Change

- Modern location-path value expressions now visit every selected node in
  document order and join their string values using the compiled effective
  separator.
- The default separator remains one space; an explicit separator is retained
  and applied between adjacent items only.
- Every selected string value and emitted separator continues through the
  existing charged string-value and result-text paths.
- The XSLT 1.0 compatibility expression remains
  `Xslt10FirstNodeLocationPath`; its established first-node behavior is
  unchanged.

This is shared XSLT 2.0/3.0 sequence behavior, not an archive-specific recovery
rule.

## Corpus result

Unchanged Microsoft `Namespace__78214` now executes instead of failing at the
former `FXRT1001` multi-node conversion boundary. Its simplified stylesheet
declares `xsl:version="2.0"`, so FastXSLT's modern core emits all seven selected
titles separated by spaces. The archival expected result contains only the
first title because the suite targeted an XSLT 1.0 processor applying
forward-compatible behavior. The case therefore remains a visible, uncredited
comparison mismatch rather than being coerced to its historical processor
expectation.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,328 | 2,328 | 0 |
| Executed successfully | 2,280 | 2,281 | +1 |
| Execution failures | 48 | 47 | -1 |
| XML comparison mismatches | 84 | 85 | +1 |
| Exact XML-semantic matches | 2,111 | 2,111 | 0 |

The strict XSLT 1.0 compatibility lower bound remains 2,111 / 3,173 (66.53%).
The new mismatch records newly observable modern behavior and is not a
regression in a previously executing case.

Subsequent corpus-accounting review classified this case as a named
legacy-processor-profile exclusion rather than leaving it among unresolved
engine mismatches. It remains uncredited. See
[OASIS XSLT 1.0 Forward-Version Archival Dispositions](oasis-xslt10-forward-version-archival-dispositions-2026-09-26.md).

## Verification

- A focused end-to-end test selects three source nodes and proves both the
  default space separator and an explicit `|` separator.
- Existing XSLT 1.0 first-node conversion tests remain green.
- The unchanged, hash-verified 3,173-case catalog was rerun.
