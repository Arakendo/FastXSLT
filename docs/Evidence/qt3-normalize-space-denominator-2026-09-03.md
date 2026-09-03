# QT3 Normalize-Space Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 39-case `fn/normalize-space.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The private source-free string evaluator trims and collapses only the four XML
whitespace characters: space, tab, carriage return, and line feed. It does not
use a broader Unicode-whitespace shortcut. The optional-string empty sequence
becomes the empty string, while a zero-argument call without supplied focus
reports `XPDY0002`.

The selected tranche covers:

- leading, trailing, repeated, and mixed XML whitespace;
- empty strings, empty sequences, and whitespace-only strings;
- nested `fn:string` and `fn:normalize-space` composition;
- equality assertions and a lazily unselected missing-context branch;
- native `XPST0017` invalid-arity and `XPDY0002` missing-context diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 33 |
| Profile excluded | 4 |
| Visible default not run | 2 |
| **Total** | **39** |

Cases 23 through 26 require schema-aware typed nodes and are profile-excluded.
The zero-argument focus case and the clock/sequence composition case remain
visibly unexecuted rather than receiving inferred results.

Across all complete QT3 overlays, the conserved subtotal is now 1,278 cases:
1,007 passes, 207 profile exclusions, and 64 visible default not-run cases.

## Boundary

This evidence does not claim the zero-argument form with a supplied focus,
schema-aware atomization, invocation clocks, or general predicate/sequence
composition. The evaluator remains a private semantic slice.
