# XSLT30 `conflict-resolution-1101` Built-In Parameter Propagation

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1101`
- Stylesheet: `conflict-resolution-1101.xsl`
- Source: inline `conflict-resolution-11` environment

## Representation and execution

The `top` template constructs `<inner/>` in a local variable. Compilation
retains that construction as a typed temporary-tree variable instruction.
Execution materializes the tree into the current invocation's variable frame;
it is not stored in the compiled stylesheet, prepared source, global runtime
state, or another request.

Applying templates to `$x` enters the temporary tree through its conceptual
document root. The built-in document rule descends to `inner` while preserving
the non-tunnel parameter `hi=42`. The matched `inner` template therefore
overrides its compiled integer default `21` with the supplied value and emits
`<z>42</z>`.

This tranche also corrects ordinary source built-in traversal: document and
element built-in rules now forward the caller's parameter map instead of
manufacturing an empty map at each child.

## Result

| Case | Expected XML | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1101` | `<z>42</z>` | semantically equal | passed |

## Claim boundary

This evidence admits local attribute-free literal temporary trees, applying
templates to their root, bounded integer template-parameter defaults, and
non-tunnel parameter propagation through built-in document/element traversal.
It does not admit arbitrary local sequence constructors, temporary text or
attribute nodes, general temporary-tree navigation, same-tree context-dependent
instructions, tunnel parameters through this path, XSLT 1.0 compatibility
behavior, `xsl:apply-imports`, or the mode/import behavior exercised by `1102`.
