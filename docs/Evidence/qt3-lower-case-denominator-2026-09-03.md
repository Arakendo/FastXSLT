# QT3 Lower-Case Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 28-case `fn/lower-case.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

A private source-free evaluator applies Rust's Unicode lowercase iterator to
each scalar value, preserving expanding mappings. The bounded expression slice
composes string constructors, `upper-case`, `count`, `boolean`, `not`,
two-argument `concat`, boolean `and`, and codepoint conversion around the
lowercase operation.

The selected tranche covers:

- empty strings and empty-sequence optional-string conversion;
- ASCII letters, digits, punctuation, and nested case conversion;
- Latin-1 codepoints 160 through 256;
- a title-case character and the expanding mapping from U+0130 to U+0069 plus
  U+0307;
- native boolean, string, integer, and integer-sequence assertion shapes;
- native `XPST0017` invalid-arity diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 27 |
| Profile excluded | 1 |
| Visible default not run | 0 |
| **Total** | **28** |

`fn-lower-case-19` explicitly requires Unicode version 7.0. FastXSLT does not
claim that historical mapping through Rust's toolchain-supplied Unicode tables,
so the case remains a profile exclusion rather than being executed against an
uncontrolled Unicode version.

Across all complete QT3 overlays, the conserved subtotal is now 1,174 cases:
923 passes, 196 profile exclusions, and 55 visible default not-run cases.

## Boundary

This evidence does not select a stable Unicode data source or version, claim
locale-sensitive casing, or admit general XPath function composition. The
evaluator remains a private semantic slice. A future standards profile must
state and verify the Unicode version that underlies public case conversion.
