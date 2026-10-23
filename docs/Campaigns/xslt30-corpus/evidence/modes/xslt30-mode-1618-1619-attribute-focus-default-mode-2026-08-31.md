# XSLT30 mode-1618 and mode-1619 attribute-focus default-mode — 2026-08-31

## Scope

The unchanged XSLT30 `attr/mode` cases `mode-1618` and `mode-1619` exercise
nested `default-mode` scopes while `xsl:apply-templates` changes focus from an
element to one of its attributes.

## Implemented semantics

- The template pattern compiler now recognizes wildcard attribute tests `@*`
  and `attribute()` as one private semantic pattern.
- Wildcard attribute rules match every source attribute and no other node kind,
  with the standard node-test default priority.
- Named attribute selection continues through the existing controlled XPath
  selection path and supplies attribute focus to template dispatch.
- A template-level default mode is inherited by an unmoded descendant
  instruction; an explicit `default-mode` on that instruction overrides it.

No attribute nodes are copied into a separate representation. Dispatch retains
the prepared source node identity, source location, name, and string value.

## Results

Both native XML assertions pass without changing upstream stylesheets, sources,
environments, or expected results:

| Case | Result |
| --- | --- |
| `mode-1618` | Element rule and selected attribute both dispatch in inherited mode `a` |
| `mode-1619` | Element rule dispatches in mode `a`; instruction override dispatches the attribute in mode `b` |

The conserved 169-case mode denominator advances from 63 to 65 passes, retains
44 profile exclusions, and reduces visible default not-run cases from 62 to 60.
Across the 11 conserved XSLT30 denominators, the visible totals become 259
passes, 3 engine-unsupported cases, 49 profile exclusions, and 220 default
not-run cases.

## Boundaries

This slice does not admit namespace-wildcard attribute patterns, schema-typed
attribute tests, or arbitrary attribute-axis expressions. Temporary trees do
not currently construct attributes, so the new pattern applies only to source
XDM attributes.
