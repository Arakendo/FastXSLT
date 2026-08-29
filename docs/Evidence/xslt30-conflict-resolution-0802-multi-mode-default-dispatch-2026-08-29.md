# XSLT30 `conflict-resolution-0802` Multi-Mode Default Dispatch

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0802`
- Stylesheet: `conflict-resolution-0802.xsl`
- Environment: shared `conflict-resolution-08` principal source

## Method

The metadata-driven helper executes the pinned source and stylesheet through a
bounded sealed snapshot and identified batch of one. One compiled template
retains the mode list `a b #default`. Template selection treats `#default` as
the unnamed mode only; explicit `xsl:apply-templates mode="#default"` therefore
does not become a mode literally named `#default`.

Each invocation calls the same named template, which preserves the named or
unnamed current mode. Its `//bar` expression compiles to a typed descendant
element selection using the stylesheet's default XPath namespace. Runtime
traversal charges every inspected descendant and retains document order before
dispatching the selected `bar` in `#current`.

## Result

| Case | Expected child elements | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0802` | `<a/><b/><default/>` | equal | passed |

The same `foo` template participates in modes `a`, `b`, and the unnamed default
mode, while each selected `bar` reaches the corresponding rule.

## Claim boundary

This evidence admits `#default` in template mode lists and on
`xsl:apply-templates`, plus an exact default-namespaced `//NCName` selection.
It does not admit mode QNames, mode declarations or properties, `#current` in a
template declaration, general default-namespaced paths, predicates on this
descendant form, or arbitrary pattern grammar.
