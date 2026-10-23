# XSLT30 `apply-templates-001`–`002` Atomic-Focus Errors

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Cases: `apply-templates-001`, `apply-templates-002`
- Stylesheets: the same-named `.xsl` files
- Entry: initial template `main`; no principal source

## Static semantic check

Both stylesheets establish a statically known integer focus with
`xsl:for-each select="1 to 5"`. The first invokes `xsl:apply-templates` with
default selection; the second explicitly selects the atomic context item with
`select="."`. Neither expression can supply the required node sequence.

FastXSLT recognizes the bounded literal integer-range form and reports
structured `XTTE0510` at the `xsl:apply-templates` source location during
stylesheet compilation. This preserves the invalid-versus-unsupported
diagnostic boundary even though general `xsl:for-each` execution is not yet in
the private instruction slice.

## Results

| Case | Upstream expected code | FastXSLT code | Category | Disposition |
| --- | --- | --- | --- | --- |
| `apply-templates-001` | `XTTE0510` | `XTTE0510` | invalid | passed |
| `apply-templates-002` | `XTTE0510` | `XTTE0510` | invalid | passed |

## Claim boundary

This evidence admits static rejection when a literal signed-integer range is
the `xsl:for-each` focus and the direct child `xsl:apply-templates` uses either
default selection or `select="."`. It does not admit general `xsl:for-each`,
arbitrary static type inference, atomic sequence transformation, dynamic focus
replacement, match pattern `.`, or successful execution of these stylesheets.
