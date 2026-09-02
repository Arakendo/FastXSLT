# XSLT30 Apply-Imports Atomic-Focus Denominator

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set
  `tests/insn/apply-imports/_apply-imports-test-set.xml`.
- Native case `apply-imports-001`, its inline source, three stylesheet modules,
  initial-template entry, and XPath result assertion.

## Method and result

A first-party overlay conserves the sole native case as
`harness-unsupported/not-run`. The adapter validates its exact identity,
XSLT30 dependency, principal/secondary stylesheet roles, and assertion text.
It imports the inline source and all three unchanged stylesheet modules into a
bounded sealed snapshot, proving that the complete native environment can be
acquired without ambient filesystem access during future execution.

The case is not compiled or executed. It applies templates to the atomic
sequence `1 to 5` and uses priority plus `xsl:apply-imports` across atomic
matches. FastXSLT's current apply-imports implementation owns source and
temporary-tree node focus; accepting this result would require atomic template
matching and built-in behavior, not merely another module-loader branch.

This raises conserved XSLT30 accounting to 578 cases: 414 passed comparisons,
3 engine-unsupported cases, 54 profile exclusions, and 107 visible default
not-run cases across 15 complete test-set denominators.

## Limitation

This record is denominator and acquisition evidence only. It does not claim
that the current compiler accepts the stylesheet modules or that source-node
`xsl:apply-imports` establishes atomic-item semantics. The native expected
answer remains visible for the future vertical slice.
