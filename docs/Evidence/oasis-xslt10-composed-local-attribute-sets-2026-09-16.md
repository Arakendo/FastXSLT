# OASIS XSLT 1.0 Composed Local Attribute Sets -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can multiple same-name `xsl:attribute-set` declarations in one stylesheet
module compose without adding runtime lookup state or prematurely admitting
include/import precedence semantics?

## Implemented slice

Yes. The compile-time local attribute-set graph now retains every same-name
declaration in document order. Each declaration expands its referenced sets
before its own static attributes; later same-name declarations replace earlier
same-name values; and attributes on the consuming literal result,
`xsl:element`, or `xsl:copy` instruction retain their existing higher
precedence.

Graph validation visits the complete declaration group, so an undefined
reference or cycle in any member remains a structured `XTSE0710` or `XTSE0720`
failure. Same-name declarations contributed by separate include/import modules
remain explicitly unsupported; this slice does not infer import precedence or
cross-module composition.

## Corpus result

Eight unchanged Lotus cases become exact XML-semantic expected-result passes:
`attribset10`, `attribset27`, `attribset29`, `attribset31`, `attribset32`,
`attribset41`, `attribset42`, and `attribset43`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,809 | 1,817 | +8 |
| Initialization failures | 1,326 | 1,318 | -8 |
| Executed successfully | 1,619 | 1,627 | +8 |
| Execution failures | 190 | 190 | 0 |
| Expected-result XML matches | 1,474 | 1,482 | +8 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,482 / 2,742 = 54.05%` for standard-operation cases and
`1,482 / 3,173 = 46.71%` for the complete catalog.

## Verification

A focused production-lifecycle test composes two declarations that reference
different sets, verifies document-order replacement, and verifies that an
explicit consuming attribute still wins. Existing focused tests retain
undefined-reference and direct/indirect-cycle diagnostics, and the existing
module-assembly test continues to reject same-name cross-module composition.
The unchanged complete sweep conserves all 3,173 identities and adds no XML
mismatch, execution failure, or panic.
