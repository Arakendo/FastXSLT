# QT3 Deep-Equal Production Path

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 263-case `fn/deep-equal.xml` test set.
- The existing first-party typed denominator overlay and 184 selected-case
  ledger records.

## Method

All 184 selected expressions now execute twice against the same immutable QT3
source text. The focused evaluator remains a safe oracle for exact
XPath-operation and zero-node-visit accounting. The production comparison
compiles the expression inside `xsl:value-of`, executes the ordinary XSLT
runtime, serializes the boolean result, and compares it with the native QT3
assertion.

The five invalid-arity cases and two collation-error cases compile through the
same production path and retain their native `XPST0017`, `FOCH0002`, or
`XPTY0004` code. This exercise found and repaired production's previous
flattening of the shared parser's `XPST0017` arity identity to private
`FXXP0005`. A literal-sequence sentinel reaches the same expression through the
host-facing workbench engine.

The selected set covers typed and mixed atomic values, exact decimal and
integer-subtype comparisons, float/double promotion and NaN, QNames, binary
values, ranges, ordered sequences, literal index lookup, arrays, maps,
composite updates, admitted collations, and boolean composition.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 184 |
| Profile excluded | 67 |
| Visible default not run | 12 |
| **Total** | **263** |

The conserved QT3 aggregate remains 1,441 cases: 1,170 selected passes, 207
profile exclusions, and 64 visible default not-run cases. This change raises
production-path confidence for existing passes; it does not add cases to the
conserved denominator.

## Boundary

This evidence admits only the selected private deep-equality grammar and value
model. It does not claim the twelve visible unsupported cases, the excluded
XQuery/schema-aware cases, arbitrary function composition, a public XDM value
representation, or complete XPath 3.1 deep-equality conformance.
