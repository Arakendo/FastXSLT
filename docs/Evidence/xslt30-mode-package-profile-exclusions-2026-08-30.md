# XSLT30 Mode Package Profile Exclusions

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Profile authority: ADR-0007 excludes XSLT packages from the current staged
  profile
- Native test shape: a principal `<package>` artifact under `<test>`

The pinned catalog contains exactly 18 mode cases with that test shape:

- `mode-1701`, `mode-1701a`, `mode-1702`, `mode-1702a`, `mode-1703`,
  `mode-1704`, `mode-1705`, `mode-1705a`, and `mode-1705b`;
- `mode-1706`, `mode-1707`, `mode-1708`, `mode-1709`, `mode-1710`,
  `mode-1711`, `mode-1712`, `mode-1713`, and `mode-1714err`.

## Classification behavior

Every case now has explicit `excluded-by-profile` selection and `not-run`
execution dispositions in the mode denominator overlay. The executable
inventory verifies each identity against the pinned test set and requires the
native `<test><package .../></test>` artifact before accepting the exclusion.

The classification does not infer package use from a filename, case-number
range, description, or stylesheet syntax. Non-package cases remain selected or
visibly not run according to their existing dispositions. No upstream package,
expected result, dependency, or gitlink changed.

## Result

The complete mode denominator now records 36 passes, 44 profile exclusions,
and 89 visible default not-run cases out of 169. The exclusions comprise 26
native streaming-dependent cases and 18 native package cases. Across the 11
conserved XSLT30 denominators, the total remains 230 passes and 3 engine-
unsupported cases, while profile exclusions rise to 49 and visible default
not-run cases fall to 249 out of 531.

## Claim boundary

This is corpus classification, not package support or execution evidence.
FastXSLT does not compile `xsl:package`, resolve package dependencies, implement
component visibility/override rules, expose package artifacts, or satisfy error
assertions inside an excluded package case. Reconsideration requires a focused
review, an ADR-0007 profile revision, and dedicated package conformance
evidence.
