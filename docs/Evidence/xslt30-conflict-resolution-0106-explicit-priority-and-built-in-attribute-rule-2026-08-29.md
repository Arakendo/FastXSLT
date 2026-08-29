# XSLT30 `conflict-resolution-0106` Explicit Priority and Built-In Attribute Rule

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0106`
- Stylesheet: `conflict-resolution-0106.xsl`
- Environment: embedded `conflict-resolution-01` principal source
- Native assertion: `<out>true</out>`

## Method

The metadata-driven apply-templates helper resolves the selected case, source,
stylesheet, and expected XML from the pinned test set. Source and stylesheet
bytes are admitted to one bounded sealed snapshot and execute as an identified
batch of one without ambient filesystem access after admission.

The compiler accepts bounded signed priorities and converts explicit values and
default pattern priorities into one exact private comparison domain retained
with each compiled matched template. The initial case admitted integers; the
domain now also accepts up to six fractional digits as separately evidenced by
`conflict-resolution-1701`. Values outside that bound remain structured
unsupported outcomes, and invalid lexicals remain structured invalid input.
The private root-template shortcut rejects explicit priority rather than
silently ignoring it.

The case needs two independent rules to remain correct. Explicit priority `10`
keeps the exact `doc` template ahead of the otherwise competing `node()` rule
with priority `1`. That template selects the `test` attribute. The child-axis
`node()` pattern does not match attributes despite its explicit priority, so the
built-in attribute rule emits the selected attribute's string value, `true`.
A first-party control independently verifies the same built-in attribute result
without a competing matched template.

## Result

| Case | Expected | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0106` | `<out>true</out>` | semantically equal XML | passed |

Focused compiler controls prove retained exact ordering and distinguish a
valid decimal beyond the bounded fixed-point domain (`FXST1025`) from an
invalid lexical priority (`FXST0030`).

## Claim boundary

This evidence establishes signed-integer priority for `0106` and the built-in
string-value rule for attributes. Bounded fractional priority is evidenced
separately by `1701`. Arbitrary precision, explicit priority on the private
root-template shortcut, duplicate-pattern resolution, import/package
precedence, `xsl:mode/@on-multiple-match`, warnings, namespace nodes, and the
complete 50-case denominator remain outside this record.
