# QT3 deep-equal literal arrays -- 2026-09-02

## Result

FastXSLT now executes unchanged QT3 `fn-deep-equal-arrays-1` through
`fn-deep-equal-arrays-7`, `-11`, `-12`, and `-14` through `-17` from the pinned
`fn/deep-equal.xml` test set. A safe
private reference representation retains an array as one XDM item whose members
are sequences; it therefore distinguishes all of the pressure in this tranche:

- an empty array from an array containing one empty sequence member;
- an array item from an atomic integer item;
- member order and integer equality; and
- nested arrays without flattening either item or member boundaries.

The second tranche adds codepoint string members and top-level sequences of
arrays. It proves equality across nested string arrays, early string mismatch,
top-level length mismatch, a late integer mismatch, and nested member-count
mismatch without evaluating later sequence items.

Every sequence-length, item, and array-member decision is charged in the XPath
operation domain. The adapter validates native identity, expression, boolean
assertion, exact result, exact operation count, and zero node visits.

## Bounds and ownership

Literal parsing accepts only integers, strings, empty sequences, square-array
constructors, and nesting required by these cases. Nesting is capped at 64 and
each array at 1,024 members before admission. Evaluation uses safe Rust and
invocation-owned work accounting. The representation remains private and does
not select a public XDM array type or an optimized layout under AR-0013.

## Conservation and boundary

The 263-case deep-equal denominator now records 169 passes, 67 profile
exclusions, and 27 visible defaults. The combined 612-case QT3 subtotal is 373
passes, 179 exclusions, and 60 visible defaults.

This does not admit maps, node-valued members, array update functions, brace
constructors, general function-item equality, or non-codepoint collation inside
arrays. Those cases remain excluded or visibly not run according to their
existing metadata.
