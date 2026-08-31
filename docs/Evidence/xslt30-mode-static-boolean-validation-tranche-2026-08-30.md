# XSLT30 Mode Static Boolean Validation Tranche

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Selected cases: `mode-1444` and `mode-1447`
- Principal stylesheets: `mode-1444.xsl` and `mode-1447.xsl`

Both cases retain their immutable upstream stylesheet bytes and native
`XTSE0020` assertions. `mode-1444` supplies invalid mixed-case
`warning-on-no-match="Yes"`; `mode-1447` supplies invalid mixed-case
`typed="No"`.

## Executable behavior

The dedicated private `xsl:mode` declaration compiler now validates the XSLT
3.0 boolean lexical space consistently for `warning-on-multiple-match`,
`warning-on-no-match`, and `typed`. The accepted values remain case-sensitive:
`yes`, `no`, `true`, `false`, `1`, and `0`, with XSLT whitespace permitted for
the XSLT 3.0 forms.

Static lexical validation occurs before unsupported execution semantics are
classified. Consequently, `mode-1444` reports native static error `XTSE0020`
without requiring `on-no-match="text-only-copy"`, warning delivery, or
`xsl:message` execution. `mode-1447` reports the same native static code without
requiring `on-no-match="shallow-copy"`, schema-aware typed modes, or a source
document. Both failures retain structured `Invalid` category and stylesheet
location.

Valid `typed="false"` is admitted as an inert declaration property. Valid
`typed="true"` remains structured `Unsupported` because schema-aware source
semantics are excluded from the current profile. Warning-enabled values remain
unsupported until AR-0004 owns warning delivery. Explicit `on-no-match`
execution remains unsupported outside this static-validation tranche.

## Result

Both native cases pass their exact error assertions. The complete mode
denominator now records 33 selected/passed cases and 136 visible default
not-run cases out of 169. Across the 11 conserved XSLT30 denominators, the total
is 227 passes, 3 engine-unsupported cases, 5 profile exclusions, and 296 visible
default not-run cases out of 531.

## Claim boundary

This is static-validation evidence only. It does not implement
`warning-on-no-match`, warning or message delivery, `on-no-match` execution,
schema-aware typed modes, shallow copying, or the positive sibling cases in the
`mode-14` family. It also does not alter AR-0016: `mode-1301` remains visibly
not run pending a complete stylesheet-dependent whitespace view.
