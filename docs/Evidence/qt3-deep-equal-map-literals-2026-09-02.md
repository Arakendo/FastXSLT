# QT3 deep-equal literal maps -- 2026-09-02

## Result

FastXSLT now executes unchanged QT3 `fn-deep-equal-maps-1` through `-4` and
`fn-deep-equal-arrays-8` through `-9` from pinned `fn/deep-equal.xml`.
The safe private composite-value reference representation supports empty maps
and bounded integer-keyed maps with integer, boolean, empty-sequence, or
already-admitted composite values.

Map equality compares entry counts, locates corresponding keys independently
of constructor order, and recursively compares each value sequence. The two
array cases prove that a map remains one item when retained as an array member.
Duplicate literal keys are rejected before admission.

Every sequence, item, map-size, key-lookup, and recursive value decision is
charged in the XPath operation domain. The adapter validates unchanged native
identity, expression, assertion, exact result, exact operation count, and zero
node visits.

## Conservation and boundary

The 263-case deep-equal denominator now records 175 passes, 67 profile
exclusions, and 21 visible defaults. The combined 612-case QT3 subtotal is 379
passes, 179 exclusions, and 54 visible defaults.

This slice does not select numeric cross-type key equivalence, NaN keys,
string keys, collations, node values, map update functions, general map
constructors, hashing strategy, or a public XDM map type. The representation is
safe, bounded, and private; it supplies a semantic oracle without settling the
prepared-layout questions retained by AR-0013.
