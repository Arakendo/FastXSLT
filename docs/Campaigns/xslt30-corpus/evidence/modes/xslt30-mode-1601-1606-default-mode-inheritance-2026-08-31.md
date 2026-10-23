# XSLT30 mode-1601 through mode-1606 default-mode inheritance — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1601` through `mode-1606`
exercise the `default-mode` static-context property on literal result elements
and `xsl:template` declarations.

## Implemented semantics

- `xsl:default-mode` on a literal result element supplies the mode for
  descendant `xsl:apply-templates` instructions that omit `mode`.
- Unqualified `default-mode` on `xsl:template` supplies both the template
  rule's implicit mode and the default for unmoded instructions in its sequence
  constructor.
- `#unnamed` lowers to the existing unnamed-mode representation rather than a
  synthetic named mode.
- Prefixed values resolve through the stylesheet namespace context to the same
  expanded mode identity used by template declarations and initial-mode entry.
- An explicit `mode` on `xsl:apply-templates` overrides an inherited default.

The compiler resolves this property once. Runtime dispatch receives the same
existing optional expanded-mode representation; no feature flag or repeated
lexical parsing enters the execution path.

The suite adapter also normalizes the qualified `initial-mode` name in
`mode-1606` from its catalog namespace context. That is harness metadata
interpretation, not an engine shortcut.

## Results

All six native XML assertions pass without changing upstream stylesheets,
sources, or expected results:

| Cases | Result |
| --- | --- |
| `mode-1601`–`mode-1603` | Literal-result-element named, unnamed, and qualified defaults pass |
| `mode-1604`–`mode-1606` | Template-rule named, unnamed, and qualified defaults pass |

The conserved 169-case mode denominator advances from 46 to 52 passes, retains
44 profile exclusions, and reduces visible default not-run cases from 79 to 73.

## Boundaries

This slice does not admit function, attribute-set, package, import-precedence,
warning-event, or general component static-context semantics. It demonstrates
only the default-mode scopes exercised by the six pinned cases, with a focused
compiler control for explicit-mode precedence.
