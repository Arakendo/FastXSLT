# OASIS XSLT 1.0 Stylesheet-Root Control Errors

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Does stylesheet initialization reject a forbidden root `mode` attribute and a
malformed or unbound `extension-element-prefixes` token instead of ignoring
them and reporting success?

## Changes

- `mode` on `xsl:stylesheet` now reports invalid `XTSE0090`.
- Every non-`#default` token in `extension-element-prefixes` must be an NCName
  bound to an in-scope namespace; malformed or unbound tokens report invalid
  `XTSE1430`.
- This validation does not implement extension instructions or change their
  authority model.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Expected errors observed during initialization | 403 | 405 | +2 |
| Expected-error cases with unexpected success | 9 | 7 | -2 |
| Initialized | 1,476 | 1,474 | -2 |
| Executed successfully | 1,313 | 1,311 | -2 |
| Expected-result XML matches | 1,202 | 1,202 | 0 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |

The unchanged `Microsoft/Errors_err017#1` and
`Microsoft/Modes__78322#1` cases now fail during initialization. Their former
executions were not compatibility passes, so the exact-result lower bound
remains 1,202.

## Boundaries

The extension-prefix validation establishes lexical and namespace-binding
correctness only. It does not select extension mechanisms, execute extension
instructions, grant host authority, or reinterpret a valid extension element
as an ordinary literal result element.

## Verification

- A focused compiler test proves both root-control errors and their structured
  invalid categories.
- The complete local OASIS measurement proves both former false successes are
  now observed initialization errors and no exact result regresses.
- No upstream corpus byte was edited.
