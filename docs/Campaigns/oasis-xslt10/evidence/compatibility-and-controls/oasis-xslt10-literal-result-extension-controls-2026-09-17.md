# OASIS XSLT 1.0 Literal-Result Extension Controls -- 2026-09-17

Date: 2026-09-17  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

Literal result elements treated every unimplemented attribute in the XSLT
namespace as an unsupported control, even under XSLT 1.0 compatibility. They
also recognized `extension-element-prefixes` only on a conventional stylesheet
root. A declaration on a literal result element therefore failed before its
scope, validation, extension-instruction classification, or namespace-copying
effect could be applied.

The unchanged Lotus and Microsoft cases exercise three separate requirements:

- an unknown XSLT-namespace attribute on an XSLT 1.0 literal result element is
  ignored and does not become a result attribute;
- `extension-element-prefixes` is scoped and validated where it is declared,
  including `#default`;
- a declared extension namespace is excluded from copied namespace nodes unless
  the result element or one of its literal attributes needs that binding.

## Repair

The literal-result compiler now uses the stylesheet's compile-time compatibility
mode to ignore otherwise unknown XSLT-namespace attributes only for XSLT 1.0.
Later versions retain the existing explicit `FXST1007` unsupported boundary.

`extension-element-prefixes` is validated at every supported declaration site.
Malformed or unbound prefixes report static `XTSE1430`. Extension-instruction
classification walks the element's lexical ancestors, so a local declaration
has its specified scope. Namespace selection excludes declared extension
namespaces but preserves a binding required by the result element name or a
literal result attribute.

This slice does not execute extension instructions, admit `xsl:fallback`, add a
runtime version branch, or expose namespace-selection machinery. An actual
declared extension instruction remains explicit unsupported `FXST1059`.

## Corpus result

Three unchanged cases become exact and two additional cases advance through
initialization to their next honest execution boundary. The exact-result lower
bound rises from 1,512 to 1,515. The initial implementation briefly exposed a
missing-prefix serialization regression where an extension namespace was also
required by a literal result attribute; the focused regression and final sweep
confirm that binding is retained.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,858 | 1,861 | +3 |
| Initialization failures | 1,277 | 1,274 | -3 |
| Executed successfully | 1,659 | 1,662 | +3 |
| Execution failures | 199 | 199 | 0 |
| Expected errors observed during initialization | 403 | 403 | 0 |
| Expected errors unexpectedly succeeding | 4 | 4 | 0 |
| Expected-result XML matches | 1,512 | 1,515 | +3 |
| XML comparison mismatches | 118 | 118 | 0 |
| XML comparator unsupported | 23 | 23 | 0 |
| Execution panics | 0 | 0 | 0 |

## Verification

Focused tests cover XSLT 1.0 versus modern unknown-control behavior, invalid
ordinary and default extension prefixes, local declaration scope, extension
namespace exclusion, explicit extension execution rejection, and preservation
of a namespace binding required by a literal result attribute.

The complete sweep conserves all 3,173 identities. It reports 1,861 initialized
cases, 1,662 successful executions, 1,515 exact XML comparisons, 118 visible
XML mismatches, 23 comparator-unsupported outcomes, 403 expected errors during
initialization, 21 expected errors during execution, and four doubt-annotated
expected-error cases that execute successfully.
