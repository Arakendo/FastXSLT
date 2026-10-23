# XSLT30 XHTML 5 explicit doctypes -- 2026-09-01

## Result

FastXSLT executes four unchanged W3C XSLT30 cases: `output-0227` through
`output-0229` and `output-0231`.

The cases cover paired public/system identifiers, a system-only identifier,
and the XHTML 5 public-only rule that emits the automatic short `html` doctype.
They also exercise the admitted `5`, `5.0`, and `+5.0` positive-decimal
spellings through the same normalized compiled version marker.

## Boundary conservation

This tranche does not widen doctype admission beyond the XHTML `html` document
element, change public/system identifier quoting, or claim other XHTML 5
serialization rules. `output-0230` remains the independently admitted native
`XTSE0020` control for an invalid nondecimal version value.

## Denominator movement

The complete output denominator moves from 127 to 131 passes and from 105 to
101 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 325 to 329 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 149 visible default not-run cases.
