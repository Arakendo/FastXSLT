# OASIS XSLT 1.0 Extension Attributes -- 2026-09-17

Date: 2026-09-17  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

FastXSLT rejected every attribute outside an instruction's admitted unqualified
attribute set. XSLT 1.0 instead permits attributes with expanded names in a
non-null, non-XSLT namespace on XSLT elements and requires the processor to
ignore them when it does not implement their extension semantics.

The unchanged Lotus namespace case places a foreign attribute on
`xsl:template` and `xsl:copy-of`. The unchanged Microsoft element case places
one on `xsl:output`. Neither attribute changes transformation semantics.

## Repair

The common instruction-attribute validator and the specialized `xsl:output`
validator now ignore foreign namespaced attributes only when the containing
stylesheet uses XSLT 1.0 compatibility. Unqualified attributes, attributes in
the XSLT namespace, and attributes in the reserved XML namespace retain their
existing validation and semantic paths. The modern-version behavior remains
explicit unsupported rather than being changed by this compatibility slice.

The rule is compile-time only. It adds no extension callback, runtime lookup,
resource authority, or result namespace behavior.

## Corpus result

Both unchanged cases become exact, raising the exact-result lower bound from
1,515 to 1,517.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,861 | 1,863 | +2 |
| Initialization failures | 1,274 | 1,272 | -2 |
| Executed successfully | 1,662 | 1,664 | +2 |
| Execution failures | 199 | 199 | 0 |
| Expected errors observed during initialization | 403 | 403 | 0 |
| Expected errors unexpectedly succeeding | 4 | 4 | 0 |
| Expected-result XML matches | 1,515 | 1,517 | +2 |
| XML comparison mismatches | 118 | 118 | 0 |
| XML comparator unsupported | 23 | 23 | 0 |
| Execution panics | 0 | 0 | 0 |

## Verification

Focused tests cover foreign attributes on ordinary XSLT instructions and
`xsl:output`, and prove that the current modern-version unsupported boundary is
unchanged.

The complete sweep conserves all 3,173 identities. It reports 1,863 initialized
cases, 1,664 successful executions, 1,517 exact XML comparisons, 118 visible
XML mismatches, 23 comparator-unsupported outcomes, 403 expected errors during
initialization, 21 expected errors during execution, and four doubt-annotated
expected-error cases that execute successfully.
