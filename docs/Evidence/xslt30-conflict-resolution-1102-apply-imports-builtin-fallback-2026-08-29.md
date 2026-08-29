# XSLT30 `conflict-resolution-1102` Apply-Imports Built-In Fallback

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1102`
- Stylesheet: `conflict-resolution-1102.xsl`
- Source: inline `conflict-resolution-11` environment

## Representation and execution

The source `top` rule constructs the same invocation-local `<inner/>` temporary
tree used by `1101`, then applies templates to its conceptual document node in
mode `m` with `hi=42`.

Temporary-tree dispatch now retains document and element focus explicitly. The
mode-specific document rule invokes `xsl:apply-imports`; because this admitted
stylesheet contains no imported module or lower import-precedence rule, the
instruction selects the built-in document rule. Built-in descent preserves
mode `m` and the non-tunnel parameter. The `inner` rule consequently overrides
its integer default `21` and emits `<z>42</z>`.

The same instruction path can fall through to a built-in source-node rule. An
invocation without a current matched rule remains a structured `XTDE0560`
failure rather than acquiring an implicit focus.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1102` | `<z>42</z>` | semantically equal | passed |

## Claim boundary

This evidence admits `xsl:apply-imports` only where the compiled stylesheet has
no imported module and therefore the standard target is a built-in rule. It
admits explicit temporary document/element focus, current-mode retention, and
non-tunnel parameter propagation through that fallback.

It does not admit `xsl:import`, import precedence, package precedence,
lower-precedence user rules, tunnel parameters, arbitrary temporary node kinds,
general temporary-tree navigation, or XSLT 1.0 compatibility behavior.
