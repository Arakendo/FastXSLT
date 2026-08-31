# XSLT30 mode-1610 through mode-1615 default-mode scope and entry — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1610` through `mode-1615`
exercise `default-mode` on `xsl:if`, explicit instruction-level shadowing, and
the stylesheet element's default initial mode.

## Implemented semantics

- `default-mode` on `xsl:if` contributes to the static context inherited by a
  descendant unmoded `xsl:apply-templates` instruction.
- `default-mode` directly on `xsl:apply-templates` takes precedence over the
  value inherited from a literal result element or stylesheet ancestor.
- A stylesheet-level named `default-mode` becomes the default initial mode when
  the invocation does not supply an explicit initial mode.
- Template mode lists retain `#unnamed` alongside named modes, allowing one
  template rule to be eligible in both the unnamed and named modes.
- The XPath boolean reference path admits location-path node existence. The
  native `//@test` guard is true exactly when its controlled path evaluation
  returns a non-empty node sequence.

Mode names and the default initial mode are resolved during compilation.
Runtime dispatch consumes the compiled identities and the existing mode
selection path; it does not re-read stylesheet attributes.

## Results

All six native XML assertions pass without changing upstream stylesheets,
sources, environments, or expected results:

| Cases | Result |
| --- | --- |
| `mode-1610`–`mode-1612` | Named, unnamed, and qualified defaults inherited through `xsl:if` pass |
| `mode-1613` | Instruction-level `#unnamed` shadows the literal-result default |
| `mode-1614` | Stylesheet default initial mode `a` enters the root rule and instruction default `b` wins |
| `mode-1615` | Stylesheet default initial mode and descendant inherited mode `a` agree |

The conserved 169-case mode denominator advances from 52 to 58 passes, retains
44 profile exclusions, and reduces visible default not-run cases from 73 to 67.
Across the 11 conserved XSLT30 denominators, the visible totals become 252
passes, 3 engine-unsupported cases, 49 profile exclusions, and 227 default
not-run cases.

## Boundaries

This slice does not admit arbitrary XPath effective-boolean-value semantics,
packages, stylesheet-function scopes, attribute-set scopes, or included-module
default-mode composition. Location-path existence remains controlled by the
existing XPath work budget and cancellation points.
