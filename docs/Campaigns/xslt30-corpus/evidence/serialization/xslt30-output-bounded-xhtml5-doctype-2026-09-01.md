# XSLT30 bounded XHTML 5 doctype -- 2026-09-01

## Result

FastXSLT executes seven unchanged W3C XSLT30 cases: `output-0208` through
`output-0210` and `output-0212` through `output-0215`.

The first four retain `html-version` values numerically equal to five and emit
an automatic doctype only for an XHTML-namespace `html` document element. The
doctype preserves the element's authored `html`, `HTML`, or `HtMl` casing.
`output-0212` also preserves its top-level whitespace, and `output-0208` retains
the already-admitted XHTML content-type metadata insertion.

The remaining three are negative structural controls. An XHTML `body` root and
two alien-namespace `html` roots receive neither an automatic doctype nor XHTML
content-type metadata. The prefixed alien root keeps its `z` binding and name.

## Boundary conservation

Only decimal spellings equal to five enter this serializer path. Other valid
positive versions remain `FXST1049`; invalid values remain `XTSE0020`.

The normalized version marker is immutable compiled output state, participates
in same-precedence output merging, semantic inspection, and known-capacity
accounting. Prefix normalization, general HTML serialization, broader XHTML 5
empty-element rules, imported output declarations, named outputs, and result
documents remain outside this tranche.

## Denominator movement

The complete output denominator moves from 111 to 118 passes and from 121 to
114 visible default not-run cases. Across the eleven conserved XSLT30
denominators, the total moves from 309 to 316 passes, with 3 engine unsupported
cases, 50 profile exclusions, and 162 visible default not-run cases.
