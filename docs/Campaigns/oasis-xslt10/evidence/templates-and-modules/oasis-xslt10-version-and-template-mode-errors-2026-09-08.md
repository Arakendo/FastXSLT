# OASIS XSLT 1.0 Version and Template-Mode Errors

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Does stylesheet initialization reject invalid version lexicals and a `mode`
attribute on a named-only template instead of executing those stylesheets?

## Changes

- Principal, included, imported, and simplified stylesheet roots now require a
  positive decimal version lexical. Unknown positive versions remain eligible
  for the existing forwards-compatible behavior; malformed, empty, zero, and
  negative values report invalid `XTSE0110`.
- A top-level named template without `match` now rejects an explicit `mode`
  attribute as invalid `XTSE0500`. Named templates that also have a match rule
  retain their existing admitted behavior.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Expected errors observed during initialization | 400 | 403 | +3 |
| Expected-error cases with unexpected success | 12 | 9 | -3 |
| Initialized | 1,479 | 1,476 | -3 |
| Executed successfully | 1,316 | 1,313 | -3 |
| Expected-result XML matches | 1,202 | 1,202 | 0 |
| XML comparison mismatches | 81 | 81 | 0 |
| Execution failures | 163 | 163 | 0 |

The unchanged `Microsoft/Errors_err062#1`,
`Microsoft/Errors_err108#1`, and `Microsoft/Namespace__78027#1` cases now fail
during initialization. Their former executions were not compatibility passes,
so the exact-result lower bound remains 1,202.

## Boundaries

This tranche validates only stylesheet-version lexical shape and the
match/mode relationship on a template. It does not select one supported future
version, change forwards-compatible feature handling, or expand named-template
mode semantics.

## Verification

- A focused compiler test proves malformed version and named-only mode errors
  are structured invalid outcomes.
- Simplified stylesheet compilation routes through the same version validator.
- The complete local OASIS measurement proves all three former false successes
  are now observed initialization errors and no exact result regresses.
- No upstream corpus byte was edited.
