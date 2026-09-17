# OASIS XSLT 1.0 Simplified Stylesheet Frontier -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the principal stylesheet compiler admit XSLT 1.0 simplified stylesheet
syntax without creating a second literal-result execution path or weakening
error accounting?

## Implemented slice

Yes. A non-XSLT document element bearing `xsl:version` is compiled as the
implicit root template of a simplified stylesheet. Its root element uses the
ordinary literal-result-element compiler, so attributes, namespace handling,
child instructions, compatibility context, diagnostics, and runtime execution
remain shared with conventional stylesheets. The pre-existing dependency-module
entry point now reuses that same function.

Missing or invalid `xsl:version`, unsupported child instructions, invalid
computed names, and later execution/serialization boundaries remain explicit.
This change does not treat arbitrary XML documents as stylesheets: the required
XSLT-namespaced version attribute is still the admission marker.

## Corpus result

The complete sweep no longer reports the former blanket `FXST0009` rejection
for the 30 simplified-stylesheet candidates. Twenty-one still stop during
initialization at their more specific semantic boundary, eight initialize and
then report an execution/serialization boundary, and one executes to a visible
XML-semantic mismatch. No case becomes an exact expected-result pass in this
tranche.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,826 | 1,835 | +9 |
| Initialization failures | 1,309 | 1,300 | -9 |
| Executed successfully | 1,636 | 1,637 | +1 |
| Execution failures | 190 | 198 | +8 |
| Expected-result XML matches | 1,491 | 1,491 | 0 |
| XML comparison mismatches | 115 | 116 | +1 |
| Expected errors unexpectedly succeeding | 6 | 6 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound therefore remains
`1,491 / 2,742 = 54.38%` for standard-operation cases and
`1,491 / 3,173 = 46.99%` for the complete catalog. The value of this tranche is
frontier accuracy: simplified syntax is no longer confused with unsupported
stylesheet syntax, and every admitted case now exposes its next real boundary.

## Verification

A focused compiler test proves that a simplified principal stylesheet becomes
one implicit root template through the normal literal-result-element compiler,
including an ordinary literal attribute and a child `xsl:value-of`. The
unchanged complete sweep conserves all 3,173 identities, eliminates the
`FXST0009` frontier, preserves the six known unexpected expected-error
successes, and adds no panic.
