# XSLT30 `conflict-resolution-0601` Variable Pattern and Source Copy

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0601`
- Stylesheet: `conflict-resolution-0601.xsl`
- Source: inline principal source

## Representation and execution

The global parameter `$p` retains its integer default `2` in compiled static
state and is materialized into invocation-local global values. The match
pattern `*[@id=$p]` retains a typed variable reference rather than substituting
the default during compilation, so a future supplied global parameter can
affect applicability without recompiling the stylesheet.

Template selection charges each inspected source attribute and applies the
XPath general-comparison behavior needed by this case: an untyped source
attribute is converted to the integer type of `$p` before comparison. Only the
nested `a` carrying `id="2"` selects the higher-priority predicate rule.

`xsl:copy` retains one static `xsl:attribute` constructor and its remaining
body separately. At execution it copies the context element's expanded name
and in-scope namespace declarations, does not implicitly copy source
attributes or children, constructs `special="yes"`, and then executes the
recursive `xsl:apply-templates` body.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0601` | `<doc><a><a special="yes"/></a><a><b/></a></doc>` | semantically equal | passed |

## Claim boundary

This evidence admits integer-literal global defaults, an unnamespaced
any-element attribute-equals-global-variable pattern, and element `xsl:copy`
with leading static unnamespaced text-valued `xsl:attribute` constructors. It
does not admit general expressions in patterns, arbitrary general comparison,
computed or namespaced attribute names, dynamic attribute values, copying
non-element nodes, `copy-namespaces` controls, or general namespace fixup.
