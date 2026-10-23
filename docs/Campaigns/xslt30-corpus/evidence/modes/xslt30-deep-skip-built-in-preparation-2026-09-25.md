# XSLT30 `deep-skip` built-in preparation

Date: 2026-09-25

## Result

FastXSLT now compiles `xsl:mode on-no-match="deep-skip"` into an explicit
built-in policy and executes that policy for both principal-source and
invocation-owned temporary trees.

The safe reference behavior is:

- an unmatched document node applies templates to its children in the active
  mode; and
- every other unmatched node contributes an empty result without descending.

Focused tests cover an unmatched source document, a matched source child with
an unmatched descendant subtree, an invocation-owned temporary document, and
a matched temporary-tree child with unmatched descendants. The temporary-tree
test also exposed and repaired a pre-existing instruction error: no-select
`xsl:apply-templates` on a temporary focus had invoked the built-in rule on the
current node instead of selecting its children.

## Corpus boundary

The unchanged W3C case `mode-1437a` is the nearest direct corpus target. Its
stylesheet declares a static boolean parameter and uses it through the shadow
attribute `_streamable="{$STREAMABLE}"`; the case supplies
`STREAMABLE=false()` as a static test parameter.

FastXSLT still rejects the stylesheet at the unsupported top-level
`xsl:param static` boundary before the new built-in policy can execute. The
case therefore remains a visible default-not-run member of the complete
`attr/mode` denominator. This tranche does not promote it, change the current
88 pass / 48 profile-exclusion / 33 default accounting, or imply streaming
support.

## Scope

This slice does not admit general static parameters, shadow attributes,
streamability analysis, schema-aware processing, or another execution backend.
The mode policy remains compiled stylesheet state; temporary trees and their
focus remain invocation-owned.

## Validation

- `cargo test -p fastxslt deep_skip --all-features`
- `cargo test -p fastxslt temporary_tree_ --all-features`

