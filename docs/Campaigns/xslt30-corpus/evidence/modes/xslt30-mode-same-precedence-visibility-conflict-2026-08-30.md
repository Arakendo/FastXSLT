# XSLT30 Same-Precedence Mode Visibility Conflict

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Selected case: `mode-1904`
- Principal stylesheet: `tests/attr/mode/mode-1904.xsl`
- Native assertion: static error `XTSE0545`

The unmodified stylesheet declares expanded mode `X` twice in one principal
module. One declaration specifies `visibility="final"`; the other specifies
`visibility="private"`.

## Executable behavior

The stylesheet-wide mode-declaration prepass resolves both names through the
existing expanded-mode-name logic and compares explicitly supplied visibility
values at the shared import precedence. Different values produce structured
static error `XTSE0545` at the later declaration before the compiler considers
whether either valid visibility value has executable meaning in the private
engine.

A focused compiler control verifies the same property independently of the
corpus adapter. The native case passes its exact static-error assertion.

## Result

The complete mode denominator now records 40 passes, 44 profile exclusions,
and 85 visible default not-run cases out of 169. Across the 11 conserved
XSLT30 denominators, the total is 234 passes, 3 engine-unsupported cases, 49
profile exclusions, and 245 visible default not-run cases out of 531.

## Claim boundary

This slice owns only conflicts between explicit visibility values for one
expanded mode at one import precedence. It does not retain or execute mode
visibility, merge declarations across includes, resolve higher-precedence
overrides, or admit `mode-1902` and `mode-1905`.
