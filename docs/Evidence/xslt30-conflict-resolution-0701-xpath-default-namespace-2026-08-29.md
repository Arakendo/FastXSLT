# XSLT30 `conflict-resolution-0701` XPath Default Namespace

Date: 2026-08-29

## Inputs

- Suite: W3C XSLT 3.0 Test Suite
- Pinned revision: `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`
- Test set: `tests/insn/apply-templates/_apply-templates-test-set.xml`
- Case: `conflict-resolution-0701`
- Stylesheet: `conflict-resolution-0701.xsl`
- Environment: shared `conflict-resolution-07` principal source

## Method

The metadata-driven helper resolves the shared namespaced source, stylesheet,
and asserted XML, then executes the admitted bytes through a bounded sealed
snapshot and identified batch of one.

The compiler derives the effective `xpath-default-namespace` by walking the
stylesheet ancestry of each expression-bearing element. For this bounded slice,
an unprefixed simple element pattern lowers directly to an expanded name, and
an unprefixed simple `xsl:apply-templates` child selection lowers to a dedicated
expanded-name selection. Runtime dispatch and selection compare expanded names;
they do not consult prefixes or lexical namespace attributes.

The separately prefixed `u:foo` pattern resolves to the same namespace and
confirms prefix identity is not part of source node identity. Focused controls
reject default-namespaced multi-step patterns and selections with `FXST1027`
instead of allowing the older local-name-only path representation to produce a
plausible but incorrect result.

## Result

| Case | Expected result string | Actual | Disposition |
| --- | --- | --- | --- |
| `conflict-resolution-0701` | `Match-of-qualified-name` | equal | passed |

The template-local default namespace resolves `doc` and the child selection
`foo` to `http://some.uri/`; the prefixed exact `u:foo` rule then wins over the
wildcard and node-test fallbacks.

## Claim boundary

This evidence admits inherited non-empty `xpath-default-namespace` only for
simple unprefixed element patterns and simple child-element selections. It does
not yet admit default namespaces in multi-step paths, predicates, arbitrary
XPath, attributes, `0702`'s namespaced attribute placement on a literal result
element, stylesheet-wide `0703`, or current-mode cases. Empty/default namespace
reset and URI validation also remain outside this slice.
