# XSLT30 mode-1607 through mode-1609 source for-each default-mode — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1607` through `mode-1609`
exercise named, unnamed, and qualified `default-mode` values on
`xsl:for-each` while the instruction changes the source-node focus.

## Implemented semantics

FastXSLT now has a private source-node `xsl:for-each` instruction distinct from
the earlier temporary-tree reference path. Compilation retains the existing
location-path selection and sequence constructor. Execution:

- evaluates the selection against the current source focus through existing
  XPath work-budget and cancellation charge points;
- executes the body once per selected node in selection order;
- supplies the selected node, one-based position, and total selection size as
  the new focus; and
- preserves the current mode and current template identity across the nested
  sequence constructor.

The existing compile-time `default-mode` inheritance then supplies the mode to
the descendant unmoded `xsl:apply-templates`; no mode lookup or stylesheet
attribute parsing is added to the iteration hot path.

## Results

All three native XML assertions pass without changing upstream stylesheets,
sources, environments, or expected results:

| Case | Result |
| --- | --- |
| `mode-1607` | Named mode `a` is inherited through source-node iteration |
| `mode-1608` | `#unnamed` dispatch is inherited through source-node iteration |
| `mode-1609` | Qualified mode identity is resolved and inherited through iteration |

The conserved 169-case mode denominator advances from 58 to 61 passes, retains
44 profile exclusions, and reduces visible default not-run cases from 67 to 64.
Across the 11 conserved XSLT30 denominators, the visible totals become 255
passes, 3 engine-unsupported cases, 49 profile exclusions, and 224 default
not-run cases.

## Boundaries

This slice does not admit arbitrary sequence item iteration, sorting,
temporary-tree/source mixing, or every XPath selection form. The representation
stays private and reuses the semantic source navigation already owned by the
engine.
