# QT3 Upper-Case Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 29-case `fn/upper-case.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

Every selected unchanged expression now compiles through generated XSLT into an
owned production case-conversion expression and executes through the ordinary
runtime, result tree, and text serializer. A workbench sentinel reaches the same
semantic path. The shared evaluator applies Rust's Unicode uppercase iterator
to each scalar value and preserves expanding mappings. It uses the same bounded
expression path as the complementary lowercase denominator.

The selected tranche covers:

- empty strings and empty-sequence optional-string conversion;
- ASCII letters, digits, punctuation, and nested case conversion;
- Latin-1 codepoints 160 through 256;
- a title-case character, sharp-s expansion to `SS`, and the expanding
  Armenian ligature mapping;
- native boolean, string, integer, and integer-sequence assertion shapes;
- native `XPST0017` invalid-arity diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 28 |
| Profile excluded | 1 |
| Visible default not run | 0 |
| **Total** | **29** |

`fn-upper-case-19` explicitly requires Unicode version 7.0. FastXSLT does not
claim that historical mapping through Rust's toolchain-supplied Unicode tables,
so the case remains a profile exclusion.

Across all complete QT3 overlays, the conserved subtotal is now 1,203 cases:
951 passes, 197 profile exclusions, and 55 visible default not-run cases.

## Boundary

This evidence does not select a stable Unicode data source or version, claim
locale-sensitive casing, or admit general XPath function composition. The
production expression remains a private semantic slice. A future standards profile must
state and verify the Unicode version underlying public case conversion.
