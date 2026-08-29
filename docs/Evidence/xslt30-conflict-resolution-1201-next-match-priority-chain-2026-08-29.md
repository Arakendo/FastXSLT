# XSLT30 `conflict-resolution-1201` Next-Match Priority Chain

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1201`
- Stylesheet: `conflict-resolution-1201.xsl`
- Source: inline `conflict-resolution-12` environment

## Representation and selection

Compilation retains `xsl:next-match` as a typed instruction. A running matched
template frame retains its private compiled-template index in addition to node,
mode, and call depth. The index is execution-plan identity only; it is not node
identity, source order, a cache key, or a public inspection contract.

The next-match selector considers applicable rules in the current mode whose
rank is strictly below the current rule. Within the one-module admitted slice,
lower rank means lower priority or an earlier declaration at equal priority.
It selects the highest remaining rank and invokes the built-in template rule
when no compiled rule remains. Each applicability check retains its existing
work accounting.

The compiler now admits duplicate pattern/mode shapes when their exact compiled
priorities differ. Equal-priority duplicates remain unsupported, so this case
does not silently select an ambiguity policy. The exact `text()` pattern is also
retained as a typed text-node test with node-test default priority.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1201` | `<out>(5)(4)(3)(2)</out>` | semantically equal | passed |

The selected `foo` rule at priority `5` invokes `node()` at `4`, `*` at `3`,
and the second `foo` at `2`, then reaches the built-in fallback.

## Claim boundary

This evidence admits parameter-free `xsl:next-match` across differing
priorities in one stylesheet module, including built-in fallback. It does not
admit next-match parameters, import/package precedence, equal-priority
ambiguity policy, `xsl:fallback` children, temporary-tree next-match, or
`xsl:apply-imports`.
