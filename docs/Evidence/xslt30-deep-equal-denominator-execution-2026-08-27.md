# XSLT30 `fn/deep-equal` Denominator Execution

| Field | Value |
| --- | --- |
| Date | 2026-08-27 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Test set | `tests/fn/deep-equal/_deep-equal-test-set.xml` |
| Cases | `deep-equal-001`, `deep-equal-002` |
| Dependency | `XSLT20+` |
| Outcome | Two selected native passes; no exclusions or hidden cases |

## Executed behavior

FastXSLT reads the complete pinned two-case test-set metadata, retains the
shared inline source environment and each native XML assertion, imports the
referenced stylesheet bytes into a bounded sealed snapshot, and executes both
cases through the ordinary principal-source lifecycle.

The admitted expression slice provides:

- positioned descendant selection for unprefixed elements and comments;
- attribute selection from the positioned element without treating attributes
  as children;
- document-order traversal with each inspected node charged to the XPath-node
  visit budget;
- pairwise sequence-length and item comparison;
- attribute equality by node kind, expanded name, and value; and
- comment equality by node kind and value.

A focused control compares separately owned attribute nodes with equal names
and values. They are deeply equal even though their XDM node identities are
distinct. Another comparison retains equal lexical values under different
attribute names and returns false, preventing string equality from standing in
for node deep equality.

## Claim boundary

This evidence establishes the two native suite cases, not general
`fn:deep-equal`. The parser deliberately accepts only the positioned descendant
attribute and comment selector forms used by this denominator. It does not yet
claim element/document deep equality, namespace-node handling, typed values,
maps, arrays, functions, collations, NaN rules, arbitrary sequences, or the
complete XPath grammar.

The implementation remains a private XPath semantic path. It does not publish
its selector representation, make a conformance percentage claim, or alter the
host resource and invocation contracts.

## QT3 typed-integer extension

The same private function parser now also executes the complete five-case QT3
`fn-deep-equalint2args` group from `fn/deep-equal.xml` at pinned QT3 revision
`83993587711dbd5c18ed846385ec37d079d6e492`. The group covers equal lower-bound
`xs:int` values and unequal lower-, middle-, and upper-bound combinations in
both argument orders. All five native boolean assertions pass.

These source-free expressions parse each `xs:int` constructor into its checked
32-bit numeric value and charge one XPath operation for the comparison; they do
not perform node visits. This extends the evidence from node comparison to one
typed atomic family without claiming arbitrary atomic sequences, numeric type
promotion, cross-type comparison, float/NaN behavior, or the other 258 cases in
the QT3 `fn-deep-equal` test set.

The adjacent complete five-case `fn-deep-equalintg2args` group also passes.
Those `xs:integer` operands include positive and negative 18-digit values that
do not fit in `xs:int`; FastXSLT parses them through a checked `i128` path and
again preserves equal and unequal results in both argument orders. This is an
admitted implementation subset of XPath's arbitrary-precision `xs:integer`,
not evidence that values outside `i128` are supported.

## QT3 exact-decimal extension

The complete five-case `fn-deep-equaldec2args` group from the same pinned QT3
test set also passes. FastXSLT parses each admitted `xs:decimal` lexical value
into a checked `i128` coefficient and a decimal scale, removes insignificant
trailing coefficient zeros, and compares the normalized values exactly. No
binary floating-point conversion participates in these results.

This evidence covers the group's equal negative lower-bound values and its
unequal lower-, middle-, and upper-bound combinations in both argument orders.
It does not establish arbitrary-precision decimal support, values whose
coefficient exceeds `i128`, numeric type promotion, cross-type comparison,
float or double semantics, NaN behavior, or the remainder of the QT3
`fn-deep-equal` test set.

## QT3 bounded-long extension

The adjacent complete five-case `fn-deep-equallng2args` group also passes.
Each admitted `xs:long` constructor is range-checked by parsing through Rust's
signed 64-bit integer representation before comparison. A focused boundary
test accepts `9223372036854775807` and rejects the next positive integer rather
than silently widening an invalid `xs:long` lexical value.

This establishes the five native assertions and the constructor boundary used
by them. It does not add numeric promotion or general constructor-function
semantics, and it does not broaden the bounded `xs:integer` claim above.

## QT3 unsigned-short extension

The complete five-case `fn-deep-equalusht2args` group also passes through an
`xs:unsignedShort` constructor backed by a checked `u16` parse. The group covers
equal lower-bound values plus unequal lower-, middle-, and upper-bound
combinations in both argument orders. Focused controls accept `65535` while
rejecting `-1` and `65536`, so the derived type's value boundary is executable
rather than inferred from the selected fixtures.

This evidence is limited to this constructor and these scalar comparisons. It
does not establish the remaining XML Schema derived-integer families, lexical
whitespace normalization, cross-type promotion, or general sequence equality.

## QT3 negative-integer extension

The adjacent complete five-case `fn-deep-equalnint2args` group also passes.
FastXSLT parses each `xs:negativeInteger` constructor through the checked
`i128` integer path and then enforces the derived type's strictly-less-than-zero
value space. The group covers an equal negative pair and unequal negative
lower-, middle-, and upper-magnitude combinations in both argument orders.

Focused controls accept `-1` while rejecting both `0` and `1`; the constructor
therefore cannot silently collapse into the broader admitted `xs:integer`
subset. This evidence does not establish arbitrary-precision integers, values
outside `i128`, the other derived-integer families, lexical whitespace
normalization, cross-type promotion, or general sequence equality.

## QT3 positive-integer extension

The adjacent complete five-case `fn-deep-equalpint2args` group also passes.
FastXSLT parses each `xs:positiveInteger` constructor through the checked
`i128` integer path and enforces the derived type's strictly-greater-than-zero
value space. The group covers an equal lower-bound pair and unequal lower-,
middle-, and upper-magnitude combinations in both argument orders.

Focused controls accept `1` while rejecting both `0` and `-1`; the constructor
cannot silently collapse into the broader admitted `xs:integer` subset. This
evidence does not establish arbitrary-precision integers, values outside
`i128`, the other derived-integer families, lexical whitespace normalization,
cross-type promotion, or general sequence equality.

## QT3 unsigned-long extension

The adjacent complete five-case `fn-deep-equalulng2args` group also passes.
Each admitted `xs:unsignedLong` constructor is parsed through checked `u64`
before conversion to the evaluator's `i128` comparison representation. The
group covers equal zero values and unequal lower-, middle-, and suite-described
upper-fixture combinations in both argument orders.

The suite's upper fixture is not the actual XML Schema upper boundary, so a
focused control separately accepts `18446744073709551615` and rejects both
`-1` and `18446744073709551616`. This establishes the fixed unsigned 64-bit
value boundary for this private constructor path, but does not establish
lexical whitespace normalization, cross-type promotion, or general sequence
equality.

## QT3 non-positive-integer extension

The adjacent complete five-case `fn-deep-equalnpi2args` group also passes.
FastXSLT parses each `xs:nonPositiveInteger` constructor through checked `i128`
and enforces the derived type's less-than-or-equal-to-zero value space. The
group covers an equal negative pair and unequal negative lower-, middle-, and
zero-boundary combinations in both argument orders.

Focused controls accept both `-1` and `0` while rejecting `1`, distinguishing
this inclusive boundary from both `xs:negativeInteger` and unrestricted
`xs:integer`. This is still a bounded subset of the arbitrary-precision type;
values outside `i128`, lexical whitespace normalization, cross-type promotion,
and general sequence equality remain unclaimed.
