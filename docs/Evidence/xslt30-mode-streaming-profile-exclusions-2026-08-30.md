# XSLT30 Mode Streaming Profile Exclusions

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Profile authority: ADR-0007 excludes XSLT streaming conformance from the
  current staged profile
- Native dependency: `<feature value="streaming"/>`

The pinned test-set catalog contains exactly 26 mode cases with that dependency:

- `mode-0002`, `mode-0004`, `mode-0006`, `mode-0008`, `mode-0010`,
  `mode-0012`, and `mode-0014`;
- `mode-1406`, `mode-1408`, `mode-1410`, `mode-1412`, `mode-1414`,
  `mode-1416`, `mode-1418`, `mode-1420`, `mode-1422`, `mode-1424`,
  `mode-1426`, `mode-1428`, `mode-1430`, `mode-1432`, `mode-1436`,
  `mode-1437`, and `mode-1438`; and
- `mode-1506` and `mode-1903`.

## Classification behavior

The mode denominator overlay now gives every native streaming-dependent case
the explicit selection disposition `excluded-by-profile` and execution
disposition `not-run`. The executable inventory verifies all 26 identities,
checks each case's native dependency directly in the pinned catalog, and
asserts that the overlay has 36 selected and 26 profile-excluded overrides.

This is dependency-driven classification, not an inference from stylesheet
syntax or filenames. `mode-0014`, for example, contains an invalid mixed-case
streamability lexical and expects `XTSE0020`, but the native case still declares
the streaming feature. FastXSLT does not count that assertion as a pass while
the required feature is outside its selected profile.

No upstream bytes, dependencies, results, or gitlinks changed. Cases without
the exact native streaming dependency retain their prior selected or visible
default not-run dispositions.

## Result

The complete mode denominator now records 36 passes, 26 profile exclusions,
and 107 visible default not-run cases out of 169. Across the 11 conserved
XSLT30 denominators, the total remains 230 passes and 3 engine-unsupported
cases, while profile exclusions rise from 5 to 31 and visible default not-run
cases fall from 293 to 267 out of 531.

## Claim boundary

Classification does not execute the cases, implement a streaming evaluator,
admit streamability analysis, or claim XSLT streaming conformance. Event-fed
XML parsing, ordinary tree execution of a similar stylesheet, or future
non-streaming sibling cases cannot substitute for the suite's explicit feature
dependency. Reconsidering these exclusions requires a deliberate profile
revision and the dedicated conformance evidence required by ADR-0007 and
AR-0007.
