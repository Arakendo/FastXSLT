# XSLT30 Initial and All-Mode Tranche

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/attr/mode/_mode-test-set.xml` |
| Added cases | `mode-1101` through `mode-1104`, `mode-1201` through `mode-1204` |
| Result | 24 selected native passes; 145 visible default not-run cases |

## Initial-mode evidence

The mode adapter now honors the native `<initial-mode name="X"/>` entry instead
of silently running those cases as ordinary principal-source transforms.
Cases 1101 through 1103 preserve explicit X mode through matched and named
templates, whitespace-normalized multi-mode declarations, `#current`, and
`#all`.

Case 1104 additionally imports its exact suite-declared quoted string parameter
as invocation-local state. The supplied value overrides the compiled global
default without mutating the reusable stylesheet. The adapter admits only this
observed scalar form; it is not a general QT3/XSLT30 parameter-expression
evaluator.

Case 1105 is intentionally not selected. Its inline environment requests
`select="/doc"`, making the document element rather than the document node the
initial context. The current private initial-mode entry accepts an admitted
resource and establishes document focus. The harness does not manufacture an
element-selection convention that would settle a future engine lifecycle
boundary.

## `#all` evidence

Cases 1201 through 1204 execute the same `#all` rule in two explicit modes.
Mode-specific and `#all` rules compete through import precedence, explicit
priority, and declaration order. `xsl:next-match` from a winning `#all` rule
retains the active explicit mode and reaches only that mode's lower-ranked rule.

This tranche does not admit `xsl:mode` declarations, warning policy, element
initial-context entry, accumulators, streaming, packages, or mode visibility.
