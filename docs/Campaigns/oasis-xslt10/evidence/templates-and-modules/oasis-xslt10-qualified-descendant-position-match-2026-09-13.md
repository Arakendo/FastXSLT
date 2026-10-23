# OASIS XSLT 1.0 qualified descendant-position match -- 2026-09-13

Date: 2026-09-13  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the shared match engine retain namespace-aware semantics for the bounded
XSLT 1.0 pattern `//n:book/n:chapter[2]/foo` without admitting a general
match-pattern evaluator?

## Implemented slice

The compiler now retains the three expanded element names and the positive
middle-step position in a private typed match plan. Runtime selection checks a
candidate leaf by walking to its positioned parent and named ancestor, then
computes the parent's position among same-named element siblings in document
order. Every inspected relationship is charged through invocation work
control.

The source-tree and temporary-tree executors implement the same bounded
semantics. Focused coverage proves that only the second namespace-qualified
`chapter` matches in both representations and that an unbound prefix remains
an invalid static error.

## Corpus result

The unchanged Microsoft
`Namespace_CheckXmlnsResetOnResultTree` case now compiles this match pattern and
reaches its next independent boundary: the valid but unsupported XPath
namespace axis in `xsl:for-each select="namespace::*"`. The selection compiler
now reports that boundary as unsupported `FXXP1001`, rather than incorrectly
reclassifying the axis token as an invalid QName.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,678 | 1,678 | 0 |
| Executed successfully | 1,507 | 1,507 | 0 |
| Expected-result XML matches | 1,376 | 1,376 | 0 |
| Initialization failures | 1,457 | 1,457 | 0 |

The aggregate counters do not change because the same case still stops during
initialization. No pass is claimed. The evidence advances and corrects the
visible capability frontier while keeping the exact lower bound at
`1,376 / 2,742 = 50.18%`; the conservative all-catalog ratio remains
`1,376 / 3,173 = 43.37%`.

## Boundaries

This does not admit a general pattern evaluator, arbitrary predicates,
arbitrary path depth, namespace-axis evaluation, namespace-node XDM support,
or the case's expected result. The typed plan is intentionally limited to one
leading descendant step, one positioned named child step, and one named leaf.

## Verification

- Focused compiler coverage checks expanded names, priority, unbound-prefix
  rejection, and the unsupported namespace-axis classification.
- Focused runtime coverage checks source/temporary parity and sibling position.
- The complete 3,173-case local measurement produced the counters above and
  traced the next boundary as unsupported `FXXP1001`.
