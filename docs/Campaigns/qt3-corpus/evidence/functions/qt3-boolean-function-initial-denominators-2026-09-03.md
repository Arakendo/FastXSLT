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

A source-free constant-boolean evaluator now parses prefixed and unprefixed
`true()` and `false()`, `not()`, `and`, `or`, the six value-comparison
operators, their six general-comparison spellings, and the identity
`xs:boolean()` constructor. Evaluation is work-accounted and short-circuits
boolean composition. Nonzero arity is classified separately from unsupported
syntax and compared with the native `XPST0017` expectation.

The QT3 adapter executes each unchanged expression and owns `assert-true`,
`assert-false`, `assert-eq`, `assert-string-value`, the paired `assert-type`
check, and expected-error comparison. A typed scalar projection additionally
handles canonical boolean-to-string conversion, two-value concatenation,
containment, and Unicode-codepoint string length. Function-item invocation
remains under the visible default rather than borrowing a pass from related
function-call functionality.

## Result

| Test set | Native cases | Selected and passed | Profile excluded | Visible default not run |
| --- | ---: | ---: | ---: | ---: |
| `fn/true.xml` | 25 | 24 | 0 | 1 |
| `fn/false.xml` | 25 | 24 | 0 | 1 |
| **Subtotal** | **50** | **48** | **0** | **2** |

The audited QT3 subtotal is now 662 cases: 456 selected passes, 179 profile
exclusions, and 27 visible default not-run cases. The remaining 31,159 QT3
cases stay at catalog inventory only.

## Next pressure

The two function-item/predicate cases remain a separate higher-order-function
boundary. Promote them only through their unchanged expressions and native
assertions; do not treat ordinary named-function calls as evidence for function
items.
