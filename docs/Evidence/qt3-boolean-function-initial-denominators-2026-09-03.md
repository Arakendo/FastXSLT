# QT3 Boolean Function Initial Denominators

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Complete native test sets `fn/true.xml` and `fn/false.xml`.
- First-party overlays `corpus/overlays/qt3/true-denominator-v0.toml` and
  `corpus/overlays/qt3/false-denominator-v0.toml`.

## Method

The typed QT3 denominator loader parses each immutable upstream test set,
rejects a suite or revision mismatch, requires the exact native case count and
unique identities, and applies one explicit default selection and execution
disposition to every case not selected by the private ledger. Unlike the two
earlier QT3 denominators, these sets need no dependency exclusion rule; the
loader now permits an empty rule list while retaining strict validation for
every rule that is present.

No expression was executed in this tranche. The two sets deliberately begin
with `harness-unsupported/not-run` defaults because they combine boolean
constants with value and general comparisons, boolean and string functions,
function items, predicates, arity errors, and several assertion families. The
existing implementation of some component semantics is not evidence that an
unchanged QT3 case passed through a suitable adapter and comparator.

## Result

| Test set | Native cases | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/true.xml` | 25 | 0 | 0 | 25 |
| `fn/false.xml` | 25 | 0 | 0 | 25 |
| **Subtotal** | **50** | **0** | **0** | **50** |

The audited QT3 subtotal is now 662 cases: 408 selected passes, 179 profile
exclusions, and 75 visible default not-run cases. The remaining 31,159 QT3
cases stay at catalog inventory only.

## Next pressure

Use these denominators to build one genuine constant-boolean XPath slice and a
QT3-owned assertion adapter. Promote cases only after their unchanged
expressions and native assertions execute; do not replace them with
case-specific expected answers or infer a pass from a compiler constant folder
used through another XSLT path.
