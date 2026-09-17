# OASIS XSLT 1.0 Excluded-Prefix Validation -- 2026-09-17

Date: 2026-09-17  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

FastXSLT applied `exclude-result-prefixes` when selecting namespace nodes for a
literal result element, but it did not validate that every named prefix was in
scope at the declaration site. The same omission existed on a conventional
stylesheet root. An unbound token could consequently pass compilation and be
silently ineffective.

The unchanged Microsoft include case
`Include_Include_ParentExplicitlyExcludesChildNamespace` exposes the distinction
between module-local namespace scopes. Its principal stylesheet declares
`exclude-result-prefixes="foo bar"`, but binds only `foo`; the included module's
independent `bar` declaration does not put `bar` in scope on the principal
stylesheet element.

## Repair

The compiler now validates each `exclude-result-prefixes` declaration where it
appears, for both the unqualified stylesheet-root form and the namespaced
literal-result-element form. An invalid or unbound token reports static error
`XTSE0808` at the declaring element. Namespace selection remains a compiled
operation; no runtime lookup, module-scope leakage, or result-shape policy was
introduced.

Focused regressions cover unbound declarations on a stylesheet root and on a
literal result element. The existing namespace-exclusion regression continues
to prove that a bound prefix is omitted from the result while unrelated bound
prefixes remain.

## Corpus result

The unchanged Microsoft include case now reports its required initialization
error. The exact-result lower bound stays at 1,512 because this is an
expected-error correction.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,859 | 1,858 | -1 |
| Initialization failures | 1,276 | 1,277 | +1 |
| Executed successfully | 1,660 | 1,659 | -1 |
| Expected errors observed during initialization | 402 | 403 | +1 |
| Expected errors unexpectedly succeeding | 5 | 4 | -1 |
| Expected-result XML matches | 1,512 | 1,512 | 0 |
| XML comparison mismatches | 118 | 118 | 0 |
| Execution panics | 0 | 0 | 0 |

All four remaining unexpected-success identities are present in the upstream
`doubts.xml` ledger. The two namespace cases and the output case explicitly say
the Recommendation does not require an error; the miscellaneous CDATA case is
doubt-marked without a rationale. FastXSLT therefore keeps them visible rather
than manufacturing errors merely to reduce the counter.

## Verification

The complete sweep conserves all 3,173 identities. It reports 1,858 initialized
cases, 1,659 successful executions, 1,512 exact XML comparisons, 118 visible
XML mismatches, 23 comparator-unsupported outcomes, 403 expected errors during
initialization, 21 expected errors during execution, and four doubt-annotated
expected-error cases that execute successfully.
