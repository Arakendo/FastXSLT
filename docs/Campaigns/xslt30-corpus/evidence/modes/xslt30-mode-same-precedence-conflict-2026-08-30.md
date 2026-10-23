# XSLT30 Mode Same-Precedence Declaration Conflict

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Selected case: `mode-1502`
- Principal stylesheet: `tests/attr/mode/mode-1502.xsl`
- Native assertion: static error `XTSE0545`

The unmodified stylesheet declares expanded mode `c` twice in one principal
module and therefore at one import precedence. The first declaration specifies
`on-no-match="shallow-copy"`; the second specifies
`on-no-match="text-only-copy"`.

## Executable behavior

Before compiling top-level instructions, the private stylesheet compiler gives
the dedicated mode-declaration owner the complete non-excluded top-level
sequence. That owner resolves each named declaration through the existing
expanded-mode-name logic, validates the closed `on-no-match` lexical set, and
compares explicit values for the same expanded mode at that one precedence.

Different explicit values produce structured static error `XTSE0545` at the
later declaration. This occurs before the compiler reaches the stylesheet's
unrelated initial template or classifies runtime `on-no-match` execution as
unsupported. A focused unit control verifies the same behavior independently
of the corpus adapter.

## Result

The native case passes its exact error assertion. The complete mode denominator
now records 37 passes, 44 profile exclusions, and 88 visible default not-run
cases out of 169. Across the 11 conserved XSLT30 denominators, the total is 231
passes, 3 engine-unsupported cases, 49 profile exclusions, and 248 visible
default not-run cases out of 531.

## Claim boundary

This slice owns only conflicts between explicit `on-no-match` values on
declarations in the same compiled module and import precedence. It does not
retain or execute `on-no-match`, merge declarations across includes, resolve
conflicts through imports, compare other mode properties, implement
`on-multiple-match="fail"`, or admit the positive and multi-module `mode-1503`
through `mode-1506` cases. Those require a broader mode-property composition
model rather than extrapolation from this static check.
