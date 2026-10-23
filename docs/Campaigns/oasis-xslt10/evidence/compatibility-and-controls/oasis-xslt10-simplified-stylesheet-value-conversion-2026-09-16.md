# OASIS XSLT 1.0 Simplified Stylesheet Value Conversion -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Does a simplified stylesheet's `xsl:version="1.0"` establish the same XSLT 1.0
static compatibility context as a conventional `xsl:stylesheet version="1.0"`
root?

## Implemented slice

Yes. Value-expression compilation now recognizes the XSLT-namespaced version
attribute on a literal-result ancestor. An `xsl:value-of` location path in that
context therefore reuses the existing XSLT 1.0 first-node-in-document-order
conversion plan. The runtime implementation and its modern-version cardinality
oracle are unchanged.

The compiler does not treat every non-`1.0` simplified version as XSLT 1.0.
Forward-compatible processing for a higher version remains a separate semantic
boundary rather than being approximated by this tranche.

## Corpus result

Unchanged Microsoft `Namespace__78215` now produces its exact XML-semantic
expected result. Its higher-version companion `Namespace__78214` remains an
explicit `FXRT1001` execution boundary.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,835 | 1,835 | 0 |
| Initialization failures | 1,300 | 1,300 | 0 |
| Executed successfully | 1,637 | 1,638 | +1 |
| Execution failures | 198 | 197 | -1 |
| Expected-result XML matches | 1,491 | 1,492 | +1 |
| XML comparison mismatches | 116 | 116 | 0 |
| Expected errors unexpectedly succeeding | 6 | 6 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,492 / 2,742 = 54.41%` for standard-operation cases and
`1,492 / 3,173 = 47.02%` for the complete catalog.

## Verification

The focused simplified-stylesheet compiler test requires its child
`xsl:value-of` to contain the typed `Xslt10FirstNodeLocationPath` plan. The
existing conventional-stylesheet runtime pair still proves that XSLT 1.0 uses
the first selected node while a modern stylesheet retains the explicit
multi-node cardinality boundary. The complete sweep conserves all 3,173
identities and adds no mismatch, unexpected success, or panic.
