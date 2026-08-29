# XSLT30 `conflict-resolution-1205` Next-Match Parameters and Result Attribute

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1205`
- Stylesheet: `conflict-resolution-1205.xsl`
- Source: inline `conflict-resolution-12` environment

## Representation and execution

`xsl:apply-templates` and `xsl:next-match` retain ordered, named arguments in
the compiled plan. This tranche admits only bounded integer literals and
references to an in-scope atomic variable in `xsl:with-param/@select`.
Arguments are evaluated at the call site and become invocation-local parameter
frames; neither the compiled stylesheet nor prepared input retains their
dynamic values.

The initial value `17` enters the priority-`5` rule through
`xsl:apply-templates`, then each `xsl:next-match` evaluates `$p` from its
current frame before binding the next rule. The priority chain remains
`5 → 4 → 3 → 2`.

The final `<e p="{$p}"/>` also adds a deliberately narrow result-tree seam.
Literal result attributes are retained separately from children, charged as
result nodes, and escaped during serialization. This case admits unnamespaced
literal values and whole-value variable attribute value templates only.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1205` | `<out>(5)(4)(3)(2)<e p="17"/></out>` | semantically equal | passed |

## Claim boundary

This evidence admits non-tunnel `xsl:with-param` with integer or atomic-variable
selection on `xsl:apply-templates` and `xsl:next-match`, plus the exact
unnamespaced result-attribute forms described above. It does not admit sequence
constructors in `xsl:with-param`, general XPath argument expressions, tunnel
propagation, required-parameter errors, namespaced result attributes, general
attribute value templates, next-match import precedence, or equal-rank
ambiguity policy.
