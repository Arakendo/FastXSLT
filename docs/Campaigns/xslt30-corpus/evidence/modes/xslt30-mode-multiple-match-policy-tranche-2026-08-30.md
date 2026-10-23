# XSLT30 Mode Multiple-Match Policy Tranche

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Shared stylesheet: `tests/attr/mode/mode-0801.xsl`
- Selected cases: `mode-0801a`, `mode-0801b`, `mode-0801c`, `mode-0803`,
  `mode-0805`, and `mode-0806`

The adapter reads each native dependency before selecting execution behavior.
`mode-0801a` requests XSLT 1.0/2.0 recovery, `mode-0801b` requests the native
error-on-multiple-match policy, and `mode-0801c` relies on XSLT 3.0 default
recovery. The upstream bytes and expected result remain unchanged.

## Executable behavior

All three cases compile the same four competing path rules. The stylesheet's
absolute `/sss//*` pattern is represented by the existing typed location path
and evaluated from the source document node. Relative `sss//*` matching keeps
its existing candidate-relative behavior. Both forms retain the standard
non-simple-pattern default priority.

The recovery cases select the later rule among equal highest semantic ranks
and match the native `assert-xml` result. The error case reaches the existing
private transform-set error policy and reports concrete dynamic error
`XTDE0540`, structured `Invalid` category, request identity `mode-0801b`, and a
stylesheet location. The concrete code satisfies the suite's `XTRE0540`
pattern without rewriting the upstream assertion.

The subsequent declaration-validation slice admits the `xsl:mode`
`warning-on-multiple-match` property only when it is absent or disabled.
`mode-0803` validates `no`; `mode-0805` validates the XSLT 3.0 `false` and `0`
forms. These declarations add no runtime state and both cases retain the native
recovery result. `mode-0806` preserves the invalid mixed-case `Yes` lexical and
matches its native static `XTSE0020` assertion. Warning-enabled values remain
structured `Unsupported` outcomes because FastXSLT does not yet own warning
event delivery.

## Result

All six native cases pass. The complete mode denominator now records 31
selected/passed cases and 138 visible default not-run cases out of 169. Across
the 11 conserved XSLT30 denominators, the total is 225 passes, 3 engine-
unsupported cases, 5 profile exclusions, and 298 visible default not-run cases
out of 531.

## Claim boundary

This is private reference-path and corpus evidence. It does not expose a public
or host-configurable multiple-match policy, select a general XSLT 1.0/2.0
compatibility profile, implement warning-enabled
`xsl:mode/@warning-on-multiple-match` semantics, admit warning delivery, other
`xsl:mode` properties, or arbitrary leading-descendant
patterns such as `//name`. Only validation of the warning-disabled and invalid
boolean lexicals is admitted here.
