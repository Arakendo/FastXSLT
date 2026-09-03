# QT3 `fn:boolean` Effective-Value Initial Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- Complete native test set `fn/boolean.xml` with 143 cases.
- First-party overlay
  `corpus/overlays/qt3/boolean-denominator-v0.toml`.

## Method

The typed QT3 denominator loader conserves all immutable upstream case
identities. A private source-free boolean evaluator reuses the safe atomic
value parser and exact integer/decimal representations already exercised by
three earlier function denominators. It applies XPath effective-boolean-value
rules to empty sequences and singleton booleans, strings, untyped atomic
values, URIs, integers, decimals, floats, and doubles. Zero and NaN are false;
nonzero numeric values are true. Both quote styles used by native constructor
cases are decoded without changing the upstream expressions.

The same parser handles prefixed and unprefixed `boolean`, `not`, and `empty`
composition and preserves `XPST0017` for zero- and multi-argument function
calls. Every evaluated expression is work-accounted. Unsupported EBV domains
are not replaced by case-specific results.

## Result

| Test set | Native cases | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/boolean.xml` | 143 | 114 | 5 | 24 |

The selected set comprises 39 typed numeric singleton cases, all 49 direct
mixed atomic/empty cases, and 26 direct function cases. Five native `XQ10+`
cases are excluded from the XPath-in-XSLT profile. The 24 visible defaults
retain node and mixed node/atomic EBV, `FORG0006` delivery, function-item,
map/array, invocation-clock, and broader composition pressure.

This denominator raises the audited QT3 subtotal to exactly 1,000 cases: 738
selected passes, 191 profile exclusions, and 71 visible default not-run cases.
The remaining 30,821 QT3 cases stay at catalog inventory only.

## Boundary

The implementation is a safe constant-expression reference slice. It does not
establish general runtime EBV, node-sequence EBV, function/map/array EBV, or
complete dynamic error semantics.
