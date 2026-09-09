# OASIS XSLT 1.0 path-union count and last-name evidence -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing typed path-union semantics be shared by computed-attribute
`count()` and XSLT 1.0 `name((union)[last()])` without adding an expression-local
navigation implementation?

## Changes

- The XPath path owner now provides one controlled union evaluator that evaluates
  every typed alternative, normalizes the result into document order, and removes
  duplicate node identities.
- Existing `xsl:copy-of` and `xsl:sort` union execution reuse that helper.
- Under XSLT 1.0 compatibility, a computed attribute can count a union only when
  every alternative parses through the typed location-path grammar.
- The value-expression compiler admits the exact
  `name((path | path ...)[last()])` shape as a typed plan. Runtime selects the last
  node from the normalized union and preserves its lexical QName.
- Union operation, navigation, name access, and result construction remain charged
  through the existing invocation control.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,499 | 1,501 | +2 |
| Executed successfully | 1,334 | 1,336 | +2 |
| Expected-result XML matches | 1,226 | 1,228 | +2 |
| XML comparison mismatches | 82 | 82 | 0 |
| Execution failures | 165 | 165 | 0 |

Both unchanged Lotus `position83` identities now pass. They count a union of
`ancestor::section`, `ancestor::simplesect`, and `ancestor::article`, then report
the lexical name of the last node in that normalized union. The line-ending
difference in the archival expected bytes is correctly ignored by the XML infoset
comparison.

The strict standard-operation lower bound becomes
`1,228 / 2,742 = 44.78%`; the conservative all-catalog ratio becomes
`1,228 / 3,173 = 38.70%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit a general union expression type, arbitrary filters
over parenthesized expressions, completion-order semantics, union variables,
temporary-tree unions, or general function composition. No corpus bytes or
expected results were changed.

## Verification

- Existing path-union regressions continue to cover document-order normalization
  and deduplication through the shared helper. A focused runtime test covers the
  new computed-attribute union count and lexical-name selection of the last union
  member.
- Both unchanged `position83` identities pass their expected XML comparison.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
