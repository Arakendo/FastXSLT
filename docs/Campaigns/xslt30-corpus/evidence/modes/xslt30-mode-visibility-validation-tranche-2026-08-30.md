# XSLT30 Mode Visibility Validation Tranche

Date: 2026-08-30

## Inputs

- Suite revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/attr/mode/_mode-test-set.xml`
- Selected cases: `mode-1507`, `mode-1508`, and `mode-1509`
- Principal stylesheets: `mode-1507.xsl`, `mode-1508.xsl`, and
  `mode-1509.xsl`

The upstream cases assert native static error `XTSE0020` for three invalid
relationships: an unnamed mode cannot be public, an unnamed mode cannot be
final, and a named mode cannot be abstract. Their unrelated template bodies
contain XPath outside the selected evaluator surface so that processors must
reject the declaration without needing to execute them.

## Executable behavior

The dedicated private `xsl:mode` declaration compiler now recognizes the
closed `visibility` lexical set and validates its relationship to presence of a
mode name before the implementation classifies otherwise valid visibility
semantics as unsupported. Each native case therefore reports `XTSE0020`,
structured `Invalid` category, and stylesheet location during compilation.

The check does not make mode visibility operational. Valid visibility values
remain structured `Unsupported`, and a valid unnamed declaration remains
outside the private named-mode slice. This ordering prevents unrelated
unsupported template expressions from hiding a required static stylesheet
error.

## Result

All three native error cases pass. The complete mode denominator now records 36
selected/passed cases and 133 visible default not-run cases out of 169. Across
the 11 conserved XSLT30 denominators, the total is 230 passes, 3 engine-
unsupported cases, 5 profile exclusions, and 293 visible default not-run cases
out of 531.

## Claim boundary

This evidence validates only the three native name/visibility constraints and
invalid visibility lexicals in the private compiler. It does not expose mode
visibility, packages, abstract components, overrides, finality, unnamed-mode
declarations, or the surrounding template expressions as supported behavior.
