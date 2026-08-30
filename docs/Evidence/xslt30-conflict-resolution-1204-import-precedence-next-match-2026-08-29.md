# XSLT30 `conflict-resolution-1204` Import-Precedence Next-Match

Date: 2026-08-29

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Principal stylesheet: `conflict-resolution-1204.xsl`
- Imported stylesheet: `conflict-resolution-1204a.xsl`
- Native dependency: `spec=XSLT20+`
- Expected result: `<out>(5)(4)(3)(2)(25)</out>`

## Harness and execution

The apply-templates corpus adapter now reads every direct stylesheet declaration
under the selected case's `test` metadata, requires exactly one principal, and
admits the principal and secondary bytes under qualified sibling identities in
one sealed snapshot. Existing single-stylesheet cases use the same path. No
filesystem handle survives admission, and compilation has no ambient fallback.

The compiled program contains seven matched rules: six at principal import
precedence and the imported `foo` rule at lower precedence. The imported rule
declares priority `25`, while the applicable principal chain has priorities
`5`, `4`, `3`, and `2`. Execution produces the pinned sequence
`(5)(4)(3)(2)(25)`, proving `xsl:next-match` ranks import precedence before
template priority and enters the imported rule only after the principal
precedence is exhausted.

## Claim boundary

This evidence admits one principal with one sealed relative import and
parameter-free `xsl:next-match` across that boundary. It does not admit a public
module graph, arbitrary dependency counts, packages, next-match parameters
across imports, duplicate named/global declarations, or live resource
acquisition. The complete 50-case apply-templates ledger now records 41 passes
and nine visible not-run dispositions.
