# QT3 deep-equal standard collation errors -- 2026-09-02

## Result

FastXSLT now executes the unchanged QT3 `K-SeqDeepEqualFunc-4` and
`K-SeqDeepEqualFunc-5` cases from the pinned `fn/deep-equal.xml` test set.
Their native `any-of` assertions permit either a successful equal result or a
specific standard error; the private evaluator deliberately selects and
preserves the error outcomes:

| Case | Input boundary | Standard outcome |
| --- | --- | --- |
| `K-SeqDeepEqualFunc-4` | Unknown literal collation URI | `FOCH0002` |
| `K-SeqDeepEqualFunc-5` | Empty third argument | `XPTY0004` |

The adapter validates the unchanged case identity, expression, native
`any-of/error` branch, exact standard code, and expression source location.
Stylesheet compilation carries these two standard codes as invalid diagnostics
instead of reducing them to the private `FXXP1010` unsupported category.

## Conservation

The complete QT3 deep-equal denominator remains 263 cases. Its disposition is
now 156 selected passes, 67 XQuery-profile exclusions, and 40 visible
`harness-unsupported/not-run` cases. Together with AxisStep, the active
612-case QT3 subtotal is 345 passes, 179 exclusions, and 88 visible defaults.

## Boundary

This slice does not admit a resolver for host-defined collations, dynamic
collation expressions, the QT3 private caseblind URI, or successful evaluation
of an empty collation operand. It classifies only the two exact standards
failures exercised by the unchanged cases and leaves every other unimplemented
collation form explicitly unsupported.
