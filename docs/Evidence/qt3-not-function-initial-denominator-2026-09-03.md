# QT3 `fn:not` Initial Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Complete native test set `fn/not.xml` with 83 cases.
- First-party overlay `corpus/overlays/qt3/not-denominator-v0.toml`.

## Method

The typed QT3 denominator loader validates and conserves the immutable parent
set. The existing source-free boolean parser now folds the standard effective
boolean value of admitted singleton atomic values and the empty sequence before
applying `fn:not`. The atomic value seam is shared with the `fn:deep-equal`,
`fn:empty`, and `fn:exists` evidence rather than introducing a second literal
type system.

The admitted EBV rules cover booleans; strings, untyped atomic values, and
URIs; integers and exact decimals; and floating-point zero and NaN. Multi-item
atomic sequences and atomic types without a defined EBV are not silently
coerced. Single-quoted constructor lexicals are decoded by the same atomic
constructor path so the unchanged NaN cases execute. Boolean composition,
comparison, and string projection continue through the existing work-accounted
safe evaluator. Zero- and multi-argument calls retain native `XPST0017` error
identity.

## Result

| Test set | Native cases | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/not.xml` | 83 | 73 | 3 | 7 |

The selected set includes 39 typed numeric singletons, all 21 source-free
string/composition/projection cases, the NaN and empty-string cases, nine
direct function cases, and the independent prefixed empty-sequence case.
Three cases with native `XQ10+` dependencies are profile-excluded. The seven
visible defaults require source-node focus, mixed node/atomic sequence EBV, or
invocation-clock/predicate/type composition.

The audited QT3 subtotal is now 857 cases: 624 selected passes, 186 profile
exclusions, and 47 visible default not-run cases. The remaining 30,964 QT3
cases stay at catalog inventory only.

## Boundary

This evidence does not establish node-sequence EBV, arbitrary function
composition, general casting, or runtime atomic sequence evaluation. It is a
safe constant-expression reference slice, not a second XPath engine.
