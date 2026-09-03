# XSLT30 Apply-Imports Atomic-Focus Denominator

Date: 2026-09-02

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete native test set
  `tests/insn/apply-imports/_apply-imports-test-set.xml`.
- Native case `apply-imports-001`, its inline source, three stylesheet modules,
  initial-template entry, and XPath result assertion.

## Method

A first-party overlay conserves the sole native case as
`selected/passed`. The adapter validates its exact identity,
XSLT30 dependency, principal/secondary stylesheet roles, and assertion text.
It imports the inline source and all three unchanged stylesheet modules into a
bounded sealed snapshot before compilation and execution.

The private compiler lowers `1 to 5` to a bounded integer selection and the
three exact `.[. ge N]` patterns to integer-threshold rules. Runtime dispatch
retains position/size and current template identity for every atomic item,
charges each candidate inspection, applies ordinary priority/import precedence,
and lets `xsl:apply-imports` inspect only imported descendants of the current
stylesheet level. A leaf import therefore reaches the built-in atomic rule; it
does not cross into a sibling import merely because that sibling has lower
numeric precedence.

The case also establishes two XSLT 3.0 compilation corrections. `xsl:import`
may occur after other top-level declarations, and duplicate named templates at
different import precedence are resolved in favor of the higher-precedence
declaration rather than rejected as an unsupported collision.

## Result

- Complete conserved denominator: 1 case.
- Selected and passed: 1.
- Visible default not run: 0.
- Native assertion `/out = "R1R2BQ3BQ4AP5"` matches the serialized result.

Current conserved XSLT30 accounting is 578 cases: 422 passed comparisons, 3
engine-unsupported cases, 54 profile exclusions, and 99 visible default not-run
cases across 15 complete test-set denominators.

## Limitation

This evidence admits only a static integer range, integer greater-or-equal
patterns, and the exact principal-with-two-sibling-leaf-import topology. It
does not establish general atomic pattern grammar, mixed-item sequences,
arbitrary expressions, `xsl:next-match` over atomic values, or a general
compiled representation of import ancestry.
