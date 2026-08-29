# XSLT30 `conflict-resolution-1801` Element Kind-Test Priority

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-1801`
- Stylesheet: `conflict-resolution-1801.xsl`
- Source: inline `conflict-resolution-18` environment

## Method

The pattern compiler lowers the exact `element()` kind test to the existing
typed any-element pattern. It therefore shares the exact `-0.5` default
priority with `*` without adding a second runtime applicability path. Explicit
priorities remain in the fixed-point domain established by
`conflict-resolution-1701`, and declaration order remains the final tie-break.

The case also requires `name(.)`. The private value-expression representation
retains that exact operation and charges its context-node inspection. Current
XDM names retain expanded identity but not the source lexical prefix, so this
slice returns the local name only for unnamespaced context nodes. A namespaced
context fails as structured unsupported `FXRT1008` instead of fabricating a
QName.

The inline source and upstream stylesheet are copied into a bounded sealed
snapshot before compilation and execution.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-1801` | `Match-booMatch-of-element-no-name:cooMatch-of-element-no-name:foo` | equal | passed |

The explicit `boo` priority `-0.4` outranks `element()`; `element()` at `-0.5`
matches `coo` and outranks the explicit `foo` priority `-0.6`.

## Claim boundary

This evidence admits the exact `element()` pattern and `name(.)` for an
unnamespaced named context node. It does not admit typed element tests,
schema-aware patterns, wildcard attributes, general function calls, lexical
QName reconstruction for namespaced nodes, import or package precedence, or
broader pattern grammar. The admitted document-element kind tests are evidenced
separately by `conflict-resolution-1602`–`1603`.
