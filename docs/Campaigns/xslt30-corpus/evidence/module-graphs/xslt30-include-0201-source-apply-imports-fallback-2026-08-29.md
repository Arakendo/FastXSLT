# XSLT30 `include-0201` Source Apply-Imports Fallback

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/decl/include/_include-test-set.xml`
- Case: `include-0201`
- Stylesheet: `include-0201.xsl`
- Source: inline `include-02` environment

## Execution

The stylesheet contains no `xsl:include`, `xsl:import`, secondary module, or
lower-precedence user rule. Its `doc` template constructs `out` and invokes
`xsl:apply-imports` on the source element.

FastXSLT retains the current source node and selects the built-in element rule
as the no-import fallback. Built-in element descent then reaches the source
text rule and produces the asserted `<out>This text should be output</out>`.
This independently exercises the source-tree counterpart to the temporary-tree
fallback admitted by `conflict-resolution-1102`.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `include-0201` | `<out>This text should be output</out>` | semantically equal | passed |

The conserved 16-case include denominator now has 2 explicit passes and 14
visible default not-run dispositions.

## Claim boundary

This evidence admits `xsl:apply-imports` from a matched source element when no
imported module or lower-precedence user template exists. It does not add
module acquisition, `xsl:include`, `xsl:import`, import precedence, parameter
propagation, or repeated apply-imports semantics beyond previously admitted
paths.
