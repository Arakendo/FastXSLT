# XSLT30 Nonempty Mode Declaration Static Error

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Selected case: `mode-1108`
- Principal stylesheet: `tests/attr/mode/mode-1108.xsl`
- Native assertion: `XTSE0010` or `XTSE0260`

The unmodified stylesheet contains several accumulator declarations and a
nonempty `xsl:mode name="Q"` whose sequence constructor contains
`xsl:apply-templates`. The native assertion permits the general stylesheet
structure error or the specific mode-declaration error.

## Executable behavior

The compiler's existing stylesheet-wide mode-declaration prepass now validates
that every top-level `xsl:mode` is empty before ordinary top-level compilation.
Meaningful content produces structured static error `XTSE0260` at the mode
declaration. This precedence is important for the selected case: FastXSLT
reports the stylesheet's concrete standards error before encountering the
unimplemented accumulator declarations that appear earlier in source order.

A focused compiler control preserves this ordering independently of the corpus
adapter, and the corpus test verifies that `XTSE0260` is one of the exact native
alternatives rather than accepting an arbitrary failure.

## Result

The complete mode denominator now records 39 passes, 44 profile exclusions,
and 86 visible default not-run cases out of 169. Across the 11 conserved
XSLT30 denominators, the total is 233 passes, 3 engine-unsupported cases, 49
profile exclusions, and 246 visible default not-run cases out of 531.

## Claim boundary

This slice owns only the required emptiness of top-level `xsl:mode` and the
specific static diagnostic. It does not admit accumulators, static parameters,
streaming, `use-accumulators`, or any executable mode-property behavior found
elsewhere in `mode-1106` and `mode-1107`.
