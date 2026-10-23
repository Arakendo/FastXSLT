# OASIS XSLT 1.0 `key()` Copy

## Question

Can `xsl:copy-of` consume the existing charged key node selection without
introducing a second lookup path or widening the admitted key grammar?

## Method

- Compile a static XSLT 1.0 `key()` selection into a private typed copy plan.
- Reuse the shared complete key scan to obtain source nodes.
- Copy each selected node through the existing charged source deep-copy owner.
- Extend exact prepared-capacity accounting and semantic inspection without
  exposing the private lookup representation.
- Add `copy-of` to the focused cross-consumer key test and run the unchanged,
  hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case deep-copies three ordered source elements selected across two
same-name key declarations. The unchanged `Lotus/copy_copy30#1` case now
initializes, executes, and compares exactly.

The strict exact-result lower bound rises from 1,591 to 1,592. Initialized
cases rise from 1,979 to 1,980, successfully executed cases rise from 1,875 to
1,876, and initialization failures fall from 1,156 to 1,155. All other
top-level disposition counts remain unchanged.

## Boundary conclusion

This tranche adds one typed consumer of the existing reference selection. It
does not introduce a key index, dynamic key arguments, alternate copy
semantics, cross-invocation retention, or a public node-set representation.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
