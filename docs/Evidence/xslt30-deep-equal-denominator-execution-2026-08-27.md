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
